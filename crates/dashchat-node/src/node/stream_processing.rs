use futures::StreamExt;
use p2panda_core::Operation;
use p2panda_spaces::{
    group::GroupError, manager::ManagerError, message::AuthoredMessage, space::SpaceError,
    types::AuthGroupError,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use tokio_stream::Stream;

use crate::{ShortId, operation::InvitationMessage, spaces::ArgType};

use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub payload: Payload,
    pub author: PK,
    pub timestamp: u64,
}

impl Node {
    pub(super) async fn initialize_inbox(
        &self,
        pubkey: PK,
    ) -> anyhow::Result<tokio::sync::mpsc::Sender<ToNetwork>> {
        if let Some(friend) = self.friends.read().await.get(&pubkey) {
            return Ok(friend.network_tx.clone());
        }

        let (network_tx, _gossip_ready) = self.initialize_topic(pubkey.into()).await?;
        Ok(network_tx)
    }

    pub(super) async fn initialize_group(&self, chat_id: ChatId) -> anyhow::Result<Chat> {
        if let Some(chat_network) = self.chats.read().await.get(&chat_id) {
            return Ok(chat_network.clone());
        }

        self.author_store
            .add_author(chat_id.into(), self.public_key())
            .await;

        let (network_tx, _gossip_ready) = self.initialize_topic(chat_id.into()).await?;

        let chat = Chat::new(chat_id, network_tx);
        self.chats.write().await.insert(chat_id, chat.clone());

        Ok(chat)
    }

    /// Internal function to start the necessary tasks for processing group chat
    /// network activity.
    ///
    /// This must be called:
    /// - when created a new group chat
    /// - when initializing the node, for each existing group chat
    pub(super) async fn initialize_topic(
        &self,
        topic: Topic,
    ) -> anyhow::Result<(Sender<ToNetwork>, tokio::sync::oneshot::Receiver<()>)> {
        let (network_tx, network_rx, gossip_ready) = self.network.subscribe(topic.clone()).await?;
        tracing::debug!(?topic, "subscribed to topic");

        let stream = ReceiverStream::new(network_rx);
        let stream = stream.filter_map(|event| async {
            match event {
                FromNetwork::GossipMessage { bytes, .. } => match decode_gossip_message(&bytes) {
                    Ok(result) => Some(result),
                    Err(err) => {
                        tracing::warn!(?err, "decode gossip message error");
                        None
                    }
                },
                FromNetwork::SyncMessage {
                    header, payload, ..
                } => Some((header, payload)),
            }
        });

        // Decode and ingest the p2panda operations.
        let stream = stream
            .decode()
            .filter_map(|result| async {
                match result {
                    Ok(operation) => Some(operation),
                    Err(err) => {
                        tracing::warn!(?err, "decode operation error");
                        None
                    }
                }
            })
            .ingest(self.op_store.clone(), 128)
            .filter_map(|result| async {
                match result {
                    Ok(operation) => Some(operation),
                    Err(err) => match err {
                        // IngestError::Duplicate(hash) => {
                        //     tracing::warn!(hash = hash.short(), "ingest: operation already exists");
                        //     None
                        // }
                        err => {
                            tracing::warn!(?err, "ingest operation error");
                            None
                        }
                    },
                }
            });

        let author_store = self.author_store.clone();
        self.spawn_stream_process_loop(stream, author_store, topic.clone());

        Ok((network_tx, gossip_ready))
    }

    fn spawn_stream_process_loop(
        &self,
        stream: impl Stream<Item = Operation<Extensions>> + Send + 'static,
        author_store: AuthorStore<Topic>,
        topic: Topic,
    ) {
        let node = self.clone();
        let mut stream = Box::pin(stream);
        task::spawn(
            async move {
                let node = node.clone();
                tracing::debug!("stream process loop started");
                while let Some(operation) = stream.next().await {
                    let hash = operation.hash;
                    match node
                        .process_operation(topic, operation, author_store.clone(), false)
                        .await
                    {
                        Ok(()) => (),
                        Err(err) => {
                            tracing::error!(
                                ?topic,
                                hash = hash.short(),
                                ?err,
                                "process operation error"
                            )
                        }
                    }
                }
                tracing::warn!("stream process loop ended");
            }
            .instrument(tracing::info_span!(
                "stream_process_loop",
                topic = format!("{:?}", topic)
            )),
        );
    }

    // async fn enforce_ordering(
    //     &self,
    //     operation: Operation<Extensions>,
    // ) -> Vec<Operation<Extensions>> {
    //     //
    //     // XXX: this is a temporary hack because ordering is not implemented for `previous` deps
    //     //
    //     let mut deps = vec![];
    //     for hash in operation.header.previous {
    //         if !self.op_store.has_operation(hash).await.unwrap_or(false) {
    //             self.ooo_buffer.write().await.push(operation);
    //             return vec![];
    //         }
    //     }
    //     let mut ready = vec![];
    // }

    pub async fn process_operation(
        &self,
        topic: Topic,
        operation: Operation<Extensions>,
        author_store: AuthorStore<Topic>,
        is_author: bool,
    ) -> anyhow::Result<()> {
        let Operation { header, body, hash } = operation;

        // NOTE: this is very much needed!!
        // TODO: this eventually needs to be more selective than just adding any old author
        author_store.add_author(topic, header.public_key).await;
        tracing::debug!(?topic, "adding author");

        let payload = body.map(|body| Payload::try_from_body(body)).transpose()?;

        match payload.as_ref() {
            Some(Payload::SpaceControl(msgs)) => {
                let mut sd = self.space_dependencies.write().await;
                for msg in msgs {
                    sd.insert(msg.id(), hash.clone());
                }
            }
            _ => {}
        }

        tracing::trace!(?payload, "RECEIVED OPERATION");

        if let Err(err) = self
            .process_payload(topic, &header, payload.as_ref(), is_author)
            .await
        {
            tracing::error!(?payload, ?err, "process operation error");
        }

        tracing::debug!(hash = hash.short(), "processed operation");

        if let Some(payload) = payload.as_ref() {
            self.notify_payload(&header, payload).await?;
        }

        anyhow::Ok(())
    }

    pub async fn notify_payload(
        &self,
        header: &Header<Extensions>,
        payload: &Payload,
    ) -> anyhow::Result<()> {
        if let Some((notification_tx, payload)) = self.notification_tx.clone().zip(Some(payload)) {
            notification_tx
                .send(Notification {
                    payload: payload.clone(),
                    author: header.public_key.into(),
                    timestamp: header.timestamp,
                })
                .await
                .unwrap_or_else(|_| tracing::warn!("notification channel closed"));
        }
        Ok(())
    }

    pub async fn process_payload(
        &self,
        topic: Topic,
        header: &Header<Extensions>,
        payload: Option<&Payload>,
        is_author: bool,
    ) -> anyhow::Result<()> {
        // TODO: maybe have different loops for the different kinds of topics and the different payloads in each
        match (topic, &payload) {
            (Topic::Chat(chat_id), Some(Payload::SpaceControl(msgs))) => {
                let mut chats = self.chats.write().await;
                let chat = chats.get_mut(&chat_id).unwrap();
                let types: Vec<_> = msgs.iter().map(|m| m.arg_type()).collect();
                tracing::debug!(?types, "processing space msgs");
                for msg in msgs {
                    // While authoring, all message types other than Application
                    // are already processed
                    if is_author && msg.arg_type() != ArgType::Application {
                        continue;
                    }
                    tracing::debug!(
                        argtype = ?msg.arg_type(),
                        opid = msg.id().short(),
                        batch = ?msgs.iter().map(|m| m.id().short()).collect::<Vec<_>>(),
                        "processing space msg"
                    );
                    match self.manager.process(msg).await {
                        Ok(events) => {
                            for (i, event) in events.into_iter().enumerate() {
                                // TODO: need to get this from the space message, not the header!
                                // because the welcome message could be passed from a differetn author
                                self.process_chat_event(chat, event)
                                    .instrument(tracing::info_span!("chat event loop", ?i))
                                    .await?;
                            }
                        }
                        Err(ManagerError::Space(SpaceError::AuthGroup(
                            AuthGroupError::DuplicateOperation(op, _id),
                        )))
                        | Err(ManagerError::Group(GroupError::AuthGroup(
                            AuthGroupError::DuplicateOperation(op, _id),
                        ))) => {
                            // assert_eq!(op, msg.id());
                            tracing::error!(
                                argtype = ?msg.arg_type(),
                                opid = op.short(),
                                "duplicate space control msg"
                            );
                        }

                        Err(ManagerError::UnexpectedMessage(op)) => {
                            tracing::error!(op = op.short(), "space manager unexpected operation");
                        }

                        Err(err) => {
                            tracing::error!(?err, "space manager process error");
                        }
                    }
                }
            }
            (Topic::Inbox(public_key), Some(Payload::Invitation(invitation))) => {
                if public_key != self.public_key() {
                    // not for me, ignore
                    return Ok(());
                }
                tracing::debug!(?invitation, "received invitation message");
                match invitation {
                    InvitationMessage::JoinGroup(chat_id) => {
                        self.join_group(*chat_id).await?;
                        // TODO: maybe close down the chat tasks if we are kicked out?
                    }
                    InvitationMessage::Friend => {
                        tracing::debug!("received friend invitation from: {:?}", header.public_key);
                    }
                }
            }
            (topic, payload) => {
                tracing::error!(?topic, ?payload, "unhandled topic/payload");
            }
        }
        Ok(())
    }

    async fn process_chat_event(
        &self,
        chat: &mut Chat,
        event: Event<ChatId>,
    ) -> anyhow::Result<()> {
        match event {
            Event::Application { data, .. } => {
                let message = ChatMessage::from_bytes(&data)?;
                chat.messages.insert(message);
            }
            Event::Removed { .. } => {
                tracing::warn!(?chat.id, "removed from chat");
                chat.removed = true;
            }
        }
        Ok(())
    }
}
