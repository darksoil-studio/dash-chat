pub(crate) mod author_operation;
mod stream_processing;

use std::collections::HashSet;
use std::pin::Pin;

use anyhow::Result;
use chrono::{Duration, Utc};
use futures::Stream;
use named_id::Rename;
use named_id::*;
use p2panda_core::Body;
use p2panda_net::ResyncConfiguration;
use p2panda_spaces::ActorId;
use p2panda_store::{LogStore, MemoryStore};
use p2panda_stream::IngestExt;
use p2panda_stream::partial::operations::PartialOrder;
use tokio::sync::mpsc;

use mailbox_client::manager::{Mailboxes, MailboxesConfig};

use crate::chat::{ChatMessage, ChatMessageContent};
use crate::contact::{InboxTopic, QrCode, ShareIntent};
use crate::local_store::NodeData;
use crate::mailbox::MailboxOperation;
use crate::payload::{
    AnnouncementsPayload, ChatPayload, Extensions, InboxPayload, Payload, Profile,
};
use crate::stores::OpStore;
use crate::topic::{Topic, TopicId};
use crate::{
    AgentId, AsBody, ChatId, DeviceGroupId, DeviceGroupPayload, DeviceId, DirectChatId, Header,
    Operation,
};

pub use crate::local_store::LocalStore;
pub use stream_processing::Notification;

#[derive(Clone, Debug)]
pub struct NodeConfig {
    pub resync: ResyncConfiguration,
    pub contact_code_expiry: Duration,
    pub mailboxes_config: MailboxesConfig,
}

impl NodeConfig {
    #[cfg(feature = "testing")]
    pub fn testing() -> Self {
        let mut mailboxes_config = MailboxesConfig::default();
        mailboxes_config.success_interval = std::time::Duration::from_millis(1000);
        mailboxes_config.error_interval = std::time::Duration::from_millis(1000);
        Self {
            resync: ResyncConfiguration::new().interval(3).poll_interval(1),
            contact_code_expiry: Duration::days(7),
            mailboxes_config,
        }
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        let resync = ResyncConfiguration::new().interval(3).poll_interval(1);
        Self {
            resync,
            contact_code_expiry: Duration::days(7),
            mailboxes_config: MailboxesConfig::default(),
        }
    }
}

pub type Orderer<S> =
    PartialOrder<TopicId, Extensions, S, p2panda_stream::partial::MemoryStore<p2panda_core::Hash>>;

pub type NodeOpStore = OpStore<MemoryStore<TopicId, Extensions>>;

#[derive(Clone)]
pub struct Node {
    pub op_store: NodeOpStore,

    pub mailboxes: Mailboxes<MailboxOperation, NodeOpStore>,

    // groups: p2panda_auth::group::Groups,
    config: NodeConfig,
    notification_tx: Option<mpsc::Sender<Notification>>,

    /// Add new subscription streams
    stream_tx: mpsc::Sender<Pin<Box<dyn Stream<Item = Operation> + Send + 'static>>>,

    local_store: LocalStore,
    node_data: NodeData,
}

impl Node {
    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, fields(me = ?local_store.device_id().expect("can't load private key").renamed())))]
    pub async fn new(
        local_store: LocalStore,
        config: NodeConfig,
        notification_tx: Option<mpsc::Sender<Notification>>,
    ) -> Result<Self> {
        let node_data = local_store.node_data()?;

        let op_store = OpStore::new_memory();
        // let op_store = OpStore::new_sqlite().await?;

        let (stream_tx, stream_rx) = mpsc::channel(100);

        let mailboxes = Mailboxes::spawn(op_store.clone(), config.mailboxes_config.clone()).await?;

        let node = Self {
            op_store: op_store.clone(),
            mailboxes,
            config,
            local_store: local_store.clone(),
            node_data,
            notification_tx,
            stream_tx,
        };

        node.spawn_stream_process_loop(stream_rx);

        node.initialize_topic(
            Topic::announcements(node.agent_id())
                .with_name(&format!("announce({})", node.agent_id().renamed())),
            true,
        )
        .await?;

        for topic in local_store.get_active_inbox_topics()?.iter() {
            node.initialize_topic(
                topic
                    .topic
                    .clone()
                    .with_name(&format!("inbox({})", node.device_id().renamed())),
                false,
            )
            .await?;
        }

        // TODO: locally store list of groups and initialize them when the node starts

        Ok(node)
    }

    pub async fn get_interleaved_logs(
        &self,
        topic_id: TopicId,
        authors: Vec<DeviceId>,
    ) -> anyhow::Result<Vec<(Header, Option<Payload>)>> {
        let mut logs = Vec::new();
        for author in authors {
            for (h, b) in self.get_log(topic_id, author).await? {
                if let Some(body) = b {
                    if let Ok(payload) = Payload::try_from_body(&body) {
                        logs.push((h, Some(payload)));
                    } else {
                        tracing::error!("Failed to decode payload: {body:?}");
                    }
                } else {
                    logs.push((h, None));
                }
            }
        }
        logs.sort_by_key(|(h, _)| h.timestamp);
        Ok(logs)
    }

    pub async fn get_log(
        &self,
        topic: TopicId,
        author: DeviceId,
    ) -> anyhow::Result<Vec<(Header, Option<Body>)>> {
        let _heights = self.op_store.get_log_heights(&topic).await?;
        match self.op_store.get_log(&author, &topic, None).await? {
            Some(log) => Ok(log),
            None => {
                let author = *author;
                tracing::warn!("No log found for topic {topic:?} and author {author:?}");
                Ok(vec![])
            }
        }
    }

    pub async fn get_authors(&self, topic_id: TopicId) -> anyhow::Result<HashSet<DeviceId>> {
        let authors = self
            .op_store
            .get_log_heights(&topic_id)
            .await?
            .into_iter()
            .map(|(pk, _)| DeviceId::from(pk))
            .collect::<HashSet<_>>();
        Ok(authors)
    }

    /// Create a new contact QR code with configured expiry time,
    /// subscribe to the inbox topic for it, and register the topic as active.
    pub async fn new_qr_code(
        &self,
        share_intent: ShareIntent,
        inbox: bool,
    ) -> anyhow::Result<QrCode> {
        let inbox_topic = if inbox {
            let inbox_topic = InboxTopic {
                topic: Topic::inbox().with_name(&format!("inbox({})", self.device_id().renamed())),
                expires_at: Utc::now() + self.config.contact_code_expiry,
            };
            self.initialize_topic(inbox_topic.topic, false).await?;
            self.local_store
                .add_active_inbox_topic(inbox_topic.clone())?;
            Some(inbox_topic)
        } else {
            None
        };

        Ok(QrCode {
            device_pubkey: self.device_id(),
            inbox_topic,
            agent_id: self.node_data.agent_id,
            share_intent,
        })
    }

    pub fn agent_id(&self) -> AgentId {
        self.node_data.agent_id
    }

    /// Get the topic for a direct chat between two public keys.
    ///
    /// The topic is the hashed sorted public keys.
    /// Anyone who knows the two public keys can derive the same topic.
    // TODO: is this a problem? Should we use a random topic instead?
    pub fn direct_chat_topic(&self, other: AgentId) -> DirectChatId {
        let me = self.agent_id();
        // TODO: use two secrets from each party to construct the topic
        let topic = Topic::direct_chat([me, other]);
        if me > other {
            topic.with_name(&format!("direct({},{})", other.renamed(), me.renamed()))
        } else {
            topic.with_name(&format!("direct({},{})", me.renamed(), other.renamed()))
        }
    }

    /// Create a new direct chat Space.
    /// Note that only one node should create the space!
    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, fields(me = ?self.device_id().renamed())))]
    pub async fn create_direct_chat_space(&self, other: AgentId) -> anyhow::Result<()> {
        let topic = self.direct_chat_topic(other);

        let my_actor = self.agent_id();
        self.initialize_topic(topic, true).await?;

        tracing::info!(
            my_actor = ?my_actor.renamed(),
            other = ?other.renamed(),
            topic = ?topic.renamed(),
            "creating direct chat space"
        );

        tracing::info!(?topic, ?topic, "created direct chat space");

        Ok(())
    }

    /// "Joining" a chat means subscribing to messages for that chat.
    /// This needs to be accompanied by being added as a member of the chat Space by an existing member
    /// -- you're not fully a member until someone adds you.
    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, parent = None, fields(me = ?self.device_id().renamed())))]
    pub async fn join_group(&self, chat_id: ChatId) -> anyhow::Result<()> {
        tracing::info!(?chat_id, "joined group");
        self.initialize_topic(chat_id, true).await
    }

    pub async fn set_profile(&self, profile: Profile) -> anyhow::Result<()> {
        self.author_operation(
            Topic::announcements(self.agent_id()),
            Payload::Announcements(AnnouncementsPayload::SetProfile(profile)),
            Some(&format!("set_profile({})", self.device_id().renamed())),
        )
        .await?;

        Ok(())
    }

    /// Get all messages for a chat from the logs.
    // TODO: Store state instead of regenerating from the logs.
    //       This will be necessary when we switch to double ratchet message encryption.
    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, fields(me = ?self.device_id().renamed())))]
    #[cfg(feature = "testing")]
    pub async fn get_messages(&self, topic: impl Into<ChatId>) -> anyhow::Result<Vec<ChatMessage>> {
        let chat_id = topic.into();
        let mut messages = vec![];

        let authors = self.get_authors(chat_id.into()).await?;

        for (header, payload) in self
            .get_interleaved_logs(chat_id.into(), authors.into_iter().collect())
            .await?
        {
            if let Some(Payload::Chat(ChatPayload::Message(message))) = payload {
                messages.push(ChatMessage::new(message, &header));
            }
        }

        // for (events, author, timestamp) in events {
        //     for event in events {
        //         use crate::Cbor;
        //         match event {
        //             Event::Application { space_id, data } => {
        //                 messages.push(ChatMessage::from_bytes(&data)?)
        //             }
        //             _ => {}
        //         }
        //     }
        // }

        Ok(messages)
    }

    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, fields(me = ?self.device_id().renamed())))]
    pub async fn send_message(
        &self,
        topic: impl Into<ChatId>,
        message: ChatMessageContent,
    ) -> anyhow::Result<(ChatMessage, Header)> {
        let topic = topic.into();

        // NOTE: duplication of timestamp and author.
        //       shouldn't we just encrypt the message itself since the rest is on the header?
        let message = ChatMessageContent::from(message);

        let header = self
            .author_operation(
                topic,
                Payload::Chat(ChatPayload::Message(message.clone())),
                None,
            )
            .await?;

        Ok((ChatMessage::new(message, &header), header))
    }

    pub fn device_id(&self) -> DeviceId {
        self.node_data.device_id()
    }

    pub fn device_group_topic(&self) -> DeviceGroupId {
        Topic::device_group(self.agent_id()).into()
    }

    /// Store someone as a contact, and:
    /// - register their spaces keybundle so we can add them to spaces
    /// - subscribe to their inbox
    /// - store them in the contacts map
    /// - send an invitation to them to do the same
    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, fields(me = ?self.device_id().renamed())))]
    pub async fn add_contact(&self, contact: QrCode) -> anyhow::Result<AgentId> {
        tracing::debug!("adding contact: {:?}", contact);

        // SPACES: Register the member in the spaces manager

        // Must subscribe to the new member's device group in order to receive their
        // group control messages.
        // TODO: is this idempotent? If not we must make sure to do this only once.
        self.initialize_topic(Topic::announcements(contact.agent_id), false)
            .await?;

        // TODO: use all of this commented out stuff when spaces are possible again
        // // XXX: there should be a better way to wait for the device group to be created,
        // //      and this may never happen if the contact is not online.
        // let mut attempts = 0;
        // loop {
        //     if let Some(group) = self.manager.group(contact.chat_actor_id).await? {
        //         if group
        //             .members()
        //             .await?
        //             .iter()
        //             .map(|(id, _)| *id)
        //             .any(|id| id == member_id)
        //         {
        //             break;
        //         }
        //     }

        //     // // see https://github.com/p2panda/p2panda/pull/871
        //     // if let Some(space) = self.manager.space(contact.device_space_id.into()).await? {
        //     //     if space
        //     //         .members()
        //     //         .await?
        //     //         .iter()
        //     //         .map(|(id, _)| *id)
        //     //         .any(|id| id == member_id)
        //     //     {
        //     //         break;
        //     //     }
        //     // }

        //     tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        //     attempts += 1;
        //     if attempts > 20 {
        //         return Err(anyhow!(
        //             "Failed to register contact's device group in 5s. Try again later."
        //         ));
        //     }
        // }
        // // XXX: need sleep a little more for all the messages to be processed
        // tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        // self.initialize_topic(Topic::announcements(actor), false)
        //     .await?;

        let agent = contact.agent_id;
        let direct_topic = self.direct_chat_topic(agent);
        self.initialize_topic(direct_topic, true).await?;

        self.author_operation(
            self.device_group_topic(),
            Payload::DeviceGroup(DeviceGroupPayload::AddContact(contact.clone())),
            Some(&format!("add_contact/invitation({})", agent.renamed())),
        )
        .await?;

        if let Some(inbox_topic) = contact.inbox_topic.clone() {
            self.initialize_topic(inbox_topic.topic, true).await?;
            let qr = self.new_qr_code(ShareIntent::AddContact, false).await?;
            self.author_operation(
                inbox_topic.topic,
                Payload::Inbox(InboxPayload::Contact(qr)),
                Some(&format!("add_contact/invitation({})", agent.renamed())),
            )
            .await?;
        }

        // Only the initiator of contactship should create the direct chat space
        if contact.share_intent == ShareIntent::AddContact && contact.inbox_topic.is_none() {
            self.create_direct_chat_space(agent).await?;
        }

        Ok(agent)
    }

    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, fields(me = ?self.device_id().renamed())))]
    pub async fn remove_contact(&self, _chat_actor_id: ActorId) -> anyhow::Result<()> {
        // TODO: shutdown inbox task, etc.
        todo!("add tombstone to contacts list");
    }
}
