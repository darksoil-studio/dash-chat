use std::pin::Pin;

use futures::Stream;
use futures::StreamExt;
use futures::stream::SelectAll;
use p2panda_core::Operation;
use p2panda_spaces::{
    group::GroupError,
    manager::ManagerError,
    space::SpaceError,
    traits::{AuthoredMessage, MessageStore},
    types::AuthGroupError,
};
use serde::{Deserialize, Serialize};
use tokio::task;
use tracing::Instrument;

use crate::mailbox::MailboxSubscription;
use crate::spaces::SpaceOperation;
use crate::{
    payload::InboxPayload,
    spaces::ArgType,
    topic::{LogId, TopicKind},
};

use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub header: Header,
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
        // TODO: this has a race condition
        if self
            .initialized_topics
            .read()
            .await
            .contains_key(&topic.into())
        {
            return Ok(());
        }

        if is_author {
            self.author_store
                .add_author(topic.into(), self.public_key())
                .await;
        }

        {
            if let Some(mailbox_rx) = self.mailbox.subscribe(topic.into()).await? {
                let stream = ReceiverStream::new(mailbox_rx).filter_map(async |op| {
                    let hash = op.hash;
                    if hash == op.header.hash() {
                        let header_bytes = op.header.to_bytes();
                        Some((op.header, op.body, header_bytes))
                    } else {
                        tracing::error!(hash = ?hash.renamed(), "hash mismatch from mailbox server");
                        None
                    }
                }).ingest(self.op_store.clone(), 128) .filter_map(|result| async {
                    match result {
                        Ok(operation) => Some(operation),
                        Err(err) => {
                            tracing::warn!(?err, "ingest operation error");
                            None
                        }
                    }
                });

                self.stream_tx
                    .send(Pin::from(Box::new(stream)))
                    .await
                    .map_err(|_| anyhow::anyhow!("stream channel closed"))?;
            }
        }

        #[cfg(feature = "p2p")]
        {
            let (network_tx, network_rx, _gossip_ready) =
                self.network.subscribe(topic.into()).await?;
            tracing::info!(?topic, "SUB: subscribed to topic");

            let stream = ReceiverStream::new(network_rx);
            let stream = stream.filter_map(|event| async {
                match event {
                    FromNetwork::GossipMessage { bytes, .. } => match decode_gossip_message(&bytes)
                    {
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
                        Err(err) => {
                            tracing::warn!(?err, "ingest operation error");
                            None
                        }
                    }
                });

            self.stream_tx
                .send(Pin::from(Box::new(stream)))
                .await
                .map_err(|_| anyhow::anyhow!("stream channel closed"))?;

            self.initialized_topics
                .write()
                .await
                .insert(topic.into(), network_tx);
        }

        Ok(())
    }

    pub fn spawn_stream_process_loop(
        &self,
        mut stream_rx: mpsc::Receiver<
            Pin<Box<dyn Stream<Item = Operation<Extensions>> + Send + 'static>>,
        >,
        _author_store: AuthorStore<LogId>,
    ) {
        let node = self.clone();

        task::spawn(
            async move {
                let node = node.clone();
                let mut streams = SelectAll::new();

                loop {
                    tokio::select! {
                        Some(stream) = stream_rx.recv() => {
                            tracing::info!("received new STREAM");
                            // Stream is already Pin<Box<...>> from the channel, push directly
                            streams.push(stream);
                        }

                        Some(op) = streams.next() => {
                            tracing::info!(op = ?op.hash.renamed(), topic = ?op.header.extensions.log_id.renamed(), "processing stream item");
                            // Process the FromNetwork item here
                            if let Err(err) = node.process_stream_item(op).await {
                                tracing::error!(?err, "process stream item error");
                            }
                        }

                        else => {
                            // Both stream_rx is closed and streams is exhausted
                            break;
                        }
                    }
                }
            }
            .instrument(tracing::info_span!("stream_process_loop")),
        );
    }

    async fn process_stream_item(&self, operation: Operation<Extensions>) -> anyhow::Result<()> {
        let hash = operation.hash;
        let log_id = operation.header.extensions.log_id;

        if let Err(err) = self.op_store.process_ordering(operation).await {
            tracing::error!(?err, "process ordering error");
        }

        let reordered = self
            .op_store
            .next_ordering()
            .await
            .map_err(|err| {
                tracing::error!(?err, "next ordering error");
            })
            .unwrap_or_default();

        // let reordered = vec![operation];

        for operation in reordered {
            match self
                .process_operation(operation, self.author_store.clone(), false, false)
                .await
            {
                Ok(()) => (),
                Err(err) => {
                    tracing::error!(
                        ?log_id,
                        hash = ?hash.renamed(),
                        ?err,
                        "process operation error"
                    )
                }
            }
        }
        Ok(())
    }

    pub async fn process_operation(
        &self,
        // topic: Topic<K>,
        operation: Operation<Extensions>,
        author_store: AuthorStore<LogId>,
        is_author: bool,
        _is_repair: bool,
    ) -> anyhow::Result<()> {
        let Operation { header, body, hash } = operation;

        let log_id = header.extensions.log_id;

        // XXX: this eventually needs to be more selective than just adding any old author
        author_store.add_author(log_id, header.public_key).await;
        tracing::debug!(?log_id, "adding author");

        tracing::info!(log_id = ?log_id.renamed(), hash = ?hash.renamed(), "PROC: processing operation");

        let payload = body.map(|body| Payload::try_from_body(&body)).transpose()?;

        tracing::trace!(?payload, "RECEIVED PAYLOAD");

        // if !is_repair {
        if let Err(err) = self
            .process_payload(&header, payload.as_ref(), is_author)
            .await
        {
            tracing::error!(
                hash = ?header.hash().renamed(),
                ?payload,
                ?err,
                "process operation error"
            );
            return Err(err);
        }

        tracing::info!(hash = ?hash.renamed(), "processed operation");

        if let Some(payload) = payload.as_ref() {
            self.notify_payload(&header, payload).await?;
        }

        // XXX: don't repair this often.
        // Box::pin(self.repair_spaces_and_publish()).await?;

        self.op_store.mark_op_processed(log_id, &hash);

        anyhow::Ok(())
    }

    pub async fn repair_spaces_and_publish(&self) -> anyhow::Result<()> {
        let repair_required = self.manager.spaces_repair_required().await?;
        if !repair_required.is_empty() {
            tracing::warn!(missing = ?repair_required, "spaces repair required");
            for space_id in repair_required {
                let (msgs, _) = self.manager.repair_spaces(&vec![space_id]).await?;
                self.process_authored_ingested_space_messages(msgs).await?;
            }
        }
        Ok(())
    }

    pub async fn notify_payload(&self, header: &Header, payload: &Payload) -> anyhow::Result<()> {
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

    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, fields(me=?self.public_key().renamed())))]
    pub async fn process_payload(
        &self,
        // topic: Topic<K>,
        header: &Header,
        payload: Option<&Payload>,
        is_author: bool,
    ) -> anyhow::Result<()> {
        let log_id = header.extensions.log_id;
        // TODO: maybe have different loops for the different kinds of topics and the different payloads in each
        match &payload {
            Some(Payload::Space(args)) => {
                let space_op = SpaceOperation::new(header.clone(), args.clone());
                let opid = space_op.id();
                let chat_id = header.extensions.topic().recast::<kind::Chat>();
                let mut chats = self.nodestate.chats.write().await;
                let chat = chats.entry(chat_id).or_insert(Chat::new(chat_id));
                // Skip already processed messages
                if self.spaces_store.message(&opid).await?.is_some() {
                    return Ok(());
                }

                self.spaces_store.set_message(&opid, &space_op).await?;

                // While authoring, all message types other than Application
                // are already processed
                if is_author && space_op.arg_type() != ArgType::Application {
                    return Ok(());
                }

                tracing::info!(opid = ?opid.renamed(), "SM: processing space msg");
                match self.manager.process(&space_op).await {
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
                            argtype = ?space_op.arg_type(),
                            opid = ?op.renamed(),
                            "duplicate space control msg"
                        );
                    }

                    Err(ManagerError::UnexpectedMessage(op)) => {
                        tracing::error!(
                            header = ?header.hash().renamed(),
                            op = ?op.renamed(),
                            "space manager unexpected operation"
                        );
                    }

                    Err(ManagerError::MissingAuthMessage(op, auth_op)) => {
                        tracing::error!(
                            op = ?op.renamed(),
                            auth_op = ?auth_op.renamed(),
                            "space manager missing auth message"
                        );
                    }

                    Err(err) => {
                        tracing::error!(
                            hash = ?header.hash().renamed(),
                            opid = ?opid.renamed(),
                            ?err,
                            "space manager process error"
                        );
                    }
                }
            }

            Some(Payload::Chat(ChatPayload::JoinGroup(chat_id))) => {
                // TODO: maybe close down the chat tasks if we are kicked out?
            }

            Some(Payload::Inbox(invitation)) => {
                let active_topics = self.local_data.active_inbox_topics.read().await;
                if !active_topics.iter().any(|it| **it.topic == *log_id) {
                    // not for me, ignore
                    return Ok(());
                }
                tracing::info!(
                    ?invitation,
                    from = ?header.public_key.renamed(),
                    "received invitation message"
                );
                match invitation {
                    InboxPayload::Contact(_) => {
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
