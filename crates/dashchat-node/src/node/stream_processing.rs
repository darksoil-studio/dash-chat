use futures::StreamExt;
use p2panda_core::Operation;
use p2panda_spaces::{
    group::GroupError,
    manager::ManagerError,
    space::SpaceError,
    traits::{AuthoredMessage, MessageStore},
    types::AuthGroupError,
};
use serde::{Deserialize, Serialize};
use tokio_stream::Stream;

use crate::{
    payload::InboxPayload,
    spaces::ArgType,
    testing::AliasedId,
    topic::{LogId, TopicKind},
};

use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub header: Header<Extensions>,
    pub payload: Payload,
}

impl Node {
    /// Internal function to start the necessary tasks for processing group chat
    /// network activity.
    ///
    /// This must be called:
    /// - when creating a new group chat
    /// - when initializing the node, for each existing group chat
    pub(super) async fn initialize_topic<K: TopicKind>(
        &self,
        topic: Topic<K>,
        is_author: bool,
    ) -> anyhow::Result<()> {
        if self.gossip.read().await.contains_key(&topic.into()) {
            return Ok(());
        }

        if is_author {
            self.author_store
                .add_author(topic.into(), self.public_key())
                .await;
        }

        let (network_tx, network_rx, _gossip_ready) = self.network.subscribe(topic.into()).await?;
        tracing::info!(?topic, "subscribed to topic");

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

        let _pubkey = self.public_key();
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
                    Err(err) => {
                        tracing::warn!(?err, "ingest operation error");
                        None
                    }
                }
            });

        let author_store = self.author_store.clone();
        self.spawn_stream_process_loop(stream, author_store);

        self.gossip.write().await.insert(topic.into(), network_tx);

        Ok(())
    }

    fn spawn_stream_process_loop(
        &self,
        stream: impl Stream<Item = Operation<Extensions>> + Send + 'static,
        author_store: AuthorStore<LogId>,
    ) {
        let node = self.clone();
        let mut stream = Box::pin(stream);

        task::spawn(
            async move {
                let node = node.clone();
                tracing::debug!("stream process loop started");
                while let Some(operation) = stream.next().await {
                    let hash = operation.hash;
                    let log_id = operation.header.extensions.log_id;

                    if let Err(err) = node.process_ordering(operation).await {
                        tracing::error!(?err, "process ordering error");
                        continue;
                    }

                    // FIXME: this can include operations for other logs!
                    // need to refactor to have only a single stream, merged from each topic subscription
                    let reordered = node
                        .next_ordering()
                        .await
                        .map_err(|err| {
                            tracing::error!(?err, "next ordering error");
                        })
                        .unwrap_or_default();
                    // dbg!(&reordered.iter().map(|o| o.hash.alias()).collect::<Vec<_>>());

                    // let reordered = vec![operation];

                    for operation in reordered {
                        match node
                            .process_operation(operation, author_store.clone(), false)
                            .await
                        {
                            Ok(()) => (),
                            Err(err) => {
                                tracing::error!(
                                    ?log_id,
                                    hash = hash.alias(),
                                    ?err,
                                    "process operation error"
                                )
                            }
                        }
                    }
                }
                tracing::warn!("stream process loop ended");
            }
            .instrument(tracing::info_span!("stream_process_loop")),
        );
    }

    pub async fn process_ordering(&self, operation: Operation<Extensions>) -> anyhow::Result<()> {
        self.ordering.write().await.process(operation).await?;
        Ok(())
    }

    pub async fn next_ordering(&self) -> anyhow::Result<Vec<Operation<Extensions>>> {
        let mut ordering = self.ordering.write().await;
        let mut next = vec![];
        while let Some(op) = ordering.next().await? {
            next.push(op);
        }
        Ok(next)
    }

    pub async fn process_operation(
        &self,
        // topic: Topic<K>,
        operation: Operation<Extensions>,
        author_store: AuthorStore<LogId>,
        is_author: bool,
    ) -> anyhow::Result<()> {
        let Operation { header, body, hash } = operation;

        let log_id = header.extensions.log_id;

        // NOTE: this is very much needed!!
        // TODO: this eventually needs to be more selective than just adding any old author
        author_store.add_author(log_id, header.public_key).await;
        tracing::debug!(?log_id, "adding author");

        let payload = body.map(|body| Payload::try_from_body(&body)).transpose()?;

        match payload.as_ref() {
            Some(Payload::Chat(ChatPayload::Space(msgs))) => {
                let mut sd = self.space_dependencies.write().await;
                for msg in msgs {
                    sd.insert(msg.id(), hash.clone());
                }
            }
            _ => {}
        }

        tracing::trace!(?payload, "RECEIVED PAYLOAD");

        if let Err(err) = self
            .process_payload(&header, payload.as_ref(), is_author)
            .await
        {
            tracing::error!(?payload, ?err, "process operation error");
        }

        tracing::info!(hash = hash.alias(), "processed operation");

        if let Some(payload) = payload.as_ref() {
            self.notify_payload(&header, payload).await?;
        }

        self.op_store.mark_op_processed(log_id, &hash);

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
                    header: header.clone(),
                    payload: payload.clone(),
                })
                .await
                .unwrap_or_else(|_| tracing::warn!("notification channel closed"));
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, fields(me=?self.public_key()))]
    pub async fn process_payload(
        &self,
        // topic: Topic<K>,
        header: &Header<Extensions>,
        payload: Option<&Payload>,
        is_author: bool,
    ) -> anyhow::Result<()> {
        let log_id = header.extensions.log_id;
        // TODO: maybe have different loops for the different kinds of topics and the different payloads in each
        match &payload {
            Some(Payload::Chat(ChatPayload::Space(msgs))) => {
                let chat_id = header.extensions.topic().recast::<kind::Chat>();
                let mut chats = self.nodestate.chats.write().await;
                let chat = chats.entry(chat_id).or_insert(Chat::new(chat_id));
                tracing::info!(
                    hash = header.hash().alias(),
                    messages = ?msgs.iter().map(|m| m.id().alias()).collect::<Vec<_>>(),
                    "processing space msgs"
                );
                for msg in msgs.iter() {
                    // While authoring, all message types other than Application
                    // are already processed
                    if is_author && msg.arg_type() != ArgType::Application {
                        continue;
                    }
                    tracing::debug!(opid = msg.id().alias(), "processing space msg");
                    self.spaces_store.set_message(&msg.id(), &msg).await?;
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
                                opid = op.alias(),
                                "duplicate space control msg"
                            );
                        }

                        Err(ManagerError::UnexpectedMessage(op)) => {
                            tracing::error!(
                                header = header.hash().alias(),
                                op = op.alias(),
                                "space manager unexpected operation"
                            );
                        }

                        Err(ManagerError::MissingAuthMessage(op, auth_op)) => {
                            tracing::error!(
                                op = op.alias(),
                                auth_op = auth_op.alias(),
                                "space manager missing auth message"
                            );
                        }

                        Err(err) => {
                            tracing::error!(?err, "space manager process error");
                        }
                    }
                }
            }

            Some(Payload::Chat(ChatPayload::JoinGroup(chat_id))) => {
                // XXX: The group should not be auto-joined!
                // TODO: for testing, pull this out into a notification handler, which simulates UI
                //       behavior like accepting group invitations.
                self.join_group(*chat_id).await?;
                // TODO: maybe close down the chat tasks if we are kicked out?
            }

            Some(Payload::Inbox(invitation)) => {
                let active_topics = self.local_data.active_inbox_topics.read().await;
                if !active_topics.iter().any(|it| *it.topic == *log_id) {
                    // not for me, ignore
                    return Ok(());
                }
                tracing::info!(
                    ?invitation,
                    from = header.public_key.alias(),
                    "received invitation message"
                );
                match invitation {
                    InboxPayload::Friend(_) => {
                        // Nothing to do.
                    }
                }
            }

            Some(Payload::Announcements(_)) => {
                // Nothing to do.
            }

            Some(Payload::DeviceGroup(_)) => {
                // Nothing to do.
            }

            None => {
                tracing::error!(?log_id, "no payload");
            }
        }
        Ok(())
    }

    async fn process_chat_event(
        &self,
        chat: &mut Chat,
        event: Event<ChatId, ()>,
    ) -> anyhow::Result<()> {
        match event {
            Event::Space(space_event) => {
                match space_event {
                    p2panda_spaces::event::SpaceEvent::Ejected { .. } => {
                        tracing::warn!(?chat.id, "removed from chat");
                        chat.removed = true;
                    }
                    _ => {
                        // Handle other space events if needed
                    }
                }
            }
            _ => {
                // nothing to do
            }
        }
        Ok(())
    }
}
