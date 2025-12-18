pub(crate) mod author_operation;
mod stream_processing;

use std::collections::{BTreeSet, HashMap};
use std::pin::Pin;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use chrono::{Duration, Utc};
use futures::Stream;
use futures::stream::FuturesUnordered;
use named_id::*;
use p2panda_auth::Access;
use p2panda_core::cbor::encode_cbor;
use p2panda_core::{Body, Operation, PrivateKey, PublicKey};
use p2panda_discovery::Discovery;
use p2panda_discovery::mdns::LocalDiscovery;
use p2panda_encryption::Rng;
use p2panda_encryption::crypto::x25519::SecretKey;
use p2panda_net::config::GossipConfig;
use p2panda_net::{
    FromNetwork, Network, NetworkBuilder, ResyncConfiguration, SyncConfiguration, ToNetwork,
};
use p2panda_spaces::ActorId;
use p2panda_spaces::event::Event;
use p2panda_spaces::traits::AuthoredMessage;
use p2panda_store::{LogStore, MemoryStore};
use p2panda_stream::partial::operations::PartialOrder;
use p2panda_stream::{DecodeExt, IngestExt};
use p2panda_sync::log_sync::LogSyncProtocol;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};
use tokio_stream::{StreamExt, wrappers::ReceiverStream};
use tracing::Instrument;

use crate::chat::Chat;
use crate::chat::{ChatMessage, ChatMessageContent};
use crate::contact::{InboxTopic, QrCode, ShareIntent};
use crate::payload::{
    AnnouncementsPayload, ChatPayload, Extensions, InboxPayload, Payload, Profile,
    decode_gossip_message, encode_gossip_message,
};
use crate::spaces::{DashForge, DashManager, DashSpace};
use crate::stores::{AuthorStore, OpStore, SpacesStore};
use crate::testing::alias_space_messages;
use crate::topic::{LogId, Topic, kind};
use crate::util::actor_to_pubkey;
use crate::{
    AsBody, ChatId, DeviceGroupId, DeviceGroupPayload, DirectChatId, GroupChatId, Header,
    timestamp_now,
};

pub use stream_processing::Notification;

// const RELAY_ENDPOINT: &str = "https://wasser.liebechaos.org";

const NETWORK_ID: [u8; 32] = [88; 32];

const MAX_MESSAGE_SIZE: usize = 1000 * 10; // 10kb max. UDP payload size

#[derive(Clone, Debug)]
pub struct NodeConfig {
    pub resync: ResyncConfiguration,
    pub contact_code_expiry: Duration,
}

impl Default for NodeConfig {
    fn default() -> Self {
        let resync = ResyncConfiguration::new().interval(3).poll_interval(1);
        Self {
            resync,
            contact_code_expiry: Duration::days(7),
        }
    }
}

pub type Orderer = PartialOrder<
    LogId,
    Extensions,
    MemoryStore<LogId, Extensions>,
    p2panda_stream::partial::MemoryStore<p2panda_core::Hash>,
>;

/// A group ID along with the individual "representative" for the group.
///
/// This is only necessary as a workaround for the fact that we can't add groups as managers.
/// When adding a group as a manager, we need to add the group with Write access
/// and an individual from that group as a Manager. When p2panda allows groups as managers,
/// we don't need the "representative" anymore.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReppedGroup {
    pub group: ActorId,
    pub individual: PublicKey,
}

#[derive(Clone)]
pub struct NodeState {
    pub(crate) chats: Arc<RwLock<HashMap<ChatId, Chat>>>,
    pub(crate) contacts: Arc<RwLock<HashMap<PublicKey, QrCode>>>,
    pub(crate) chat_actor_id: ActorId,
}

#[derive(Clone)]
pub struct NodeLocalData {
    pub private_key: PrivateKey,
    /// Used to create the device group space
    // TODO: use as space ID once device groups are even possible.
    // for now this is only used as the topic for the device group's messages.
    // see https://github.com/p2panda/p2panda/pull/871
    pub device_space_id: DeviceGroupId,
    pub active_inbox_topics: Arc<RwLock<BTreeSet<InboxTopic>>>,
}

impl NodeLocalData {
    pub fn new_random() -> Self {
        let private_key = PrivateKey::new();
        Self {
            private_key,
            device_space_id: DeviceGroupId::random(),
            active_inbox_topics: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }
}

#[derive(Clone)]
pub struct Node {
    pub op_store: OpStore,
    // pub ordering_store: p2panda_stream::partial::MemoryStore<p2panda_core::Hash>,
    pub network: Network<LogId>,
    author_store: AuthorStore<LogId>,
    /// TODO: should not be necessary, only used to manually persist messages from other nodes
    spaces_store: SpacesStore,
    pub(crate) manager: DashManager,

    config: NodeConfig,
    local_data: NodeLocalData,
    notification_tx: Option<mpsc::Sender<Notification>>,

    /// Add new subscription streams
    stream_tx: mpsc::Sender<Pin<Box<dyn Stream<Item = Operation<Extensions>> + Send + 'static>>>,

    pub(crate) initialized_topics: Arc<RwLock<HashMap<LogId, mpsc::Sender<ToNetwork>>>>,

    /// TODO: some of the stuff in here is only for testing.
    /// The channel senders are needed but any stateful stuff should go.
    pub(crate) nodestate: NodeState,
}

impl Node {
    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, fields(me = ?PublicKey::from(local_data.private_key.public_key()))))]
    pub async fn new(
        local_data: NodeLocalData,
        config: NodeConfig,
        notification_tx: Option<mpsc::Sender<Notification>>,
    ) -> Result<Self> {
        let rng = Rng::default();
        let NodeLocalData {
            private_key,
            active_inbox_topics,
            device_space_id,
        } = local_data.clone();
        let public_key = PublicKey::from(private_key.public_key());

        let mdns = LocalDiscovery::new();

        let op_store = MemoryStore::<LogId, Extensions>::new();
        let author_store: AuthorStore<LogId> = AuthorStore::new();

        let sync_protocol = LogSyncProtocol::new(author_store.clone(), op_store.clone());
        let sync_config = SyncConfiguration::new(sync_protocol).resync(config.resync.clone());

        let mut new_peers = mdns.subscribe(NETWORK_ID).unwrap();

        // author_store
        //     .add_author(Topic::inbox(public_key.clone()), public_key)
        //     .await;

        let network_builder = NetworkBuilder::new(NETWORK_ID)
            .private_key(private_key.clone())
            .discovery(mdns)
            .gossip(GossipConfig {
                max_message_size: MAX_MESSAGE_SIZE,
            })
            // .relay(relay_url, false, 0)
            .sync(sync_config);

        // if config.bootstrap {
        //     network_builder = network_builder.bootstrap();
        // }

        // if let Some(bootstrap) = config.use_bootstrap {
        //     network_builder = network_builder.direct_address(bootstrap, vec![], None);
        // }

        let op_store = OpStore::new(op_store);

        let chat_actor_id: ActorId = PrivateKey::from_bytes(&rng.random_array()?)
            .public_key()
            .into();

        let network = network_builder.build().await.context("spawn p2p network")?;
        let spaces_store = SpacesStore::new();
        let (manager, device_group_msgs) = DashManager::new(
            private_key.clone(),
            chat_actor_id.clone(),
            spaces_store.clone(),
            op_store.clone(),
            rng,
        )
        .await
        .unwrap();

        // // nope
        // manager.register_member(&manager.me().await?).await?;

        let (stream_tx, stream_rx) = mpsc::channel(100);

        let node = Self {
            op_store: op_store.clone(),
            author_store: author_store.clone(),
            spaces_store,
            network,
            manager: manager.clone(),
            config,
            local_data,
            notification_tx,
            stream_tx,
            initialized_topics: Arc::new(RwLock::new(HashMap::new())),
            nodestate: NodeState {
                chats: Arc::new(RwLock::new(HashMap::new())),
                contacts: Arc::new(RwLock::new(HashMap::new())),
                chat_actor_id,
            },
        };

        node.spawn_stream_process_loop(stream_rx, author_store.clone());

        node.initialize_topic(
            device_space_id.with_name(&format!("device_group({})", public_key.renamed())),
            true,
        )
        .await?;

        node.initialize_topic(Topic::global().with_name("GLOBAL"), true)
            .await?;

        node.initialize_topic(
            // Topic::announcements(node.repped_group().group)
            //     .with_name(&format!("announce({})", public_key.renamed())),
            Topic::global(),
            true,
        )
        .await?;

        for topic in active_inbox_topics.read().await.iter() {
            node.initialize_topic(
                topic
                    .topic
                    .clone()
                    .with_name(&format!("inbox({})", public_key.renamed())),
                false,
            )
            .await?;
        }

        node.process_authored_ingested_space_messages(device_group_msgs)
            .await?;

        let author_store = author_store.clone();
        let me = public_key.clone();

        // TODO: accomodate new inbox topics as they are added
        // TODO: remove expired inbox topics from processing and from local data
        // TODO: what is the actual right way to do this?
        tokio::spawn(
            async move {
                while let Some(Ok(peer)) = new_peers.next().await {
                    let pubkey = PublicKey::from_bytes(peer.node_addr.node_id.as_bytes()).unwrap();

                    let mut inboxes = active_inbox_topics.write().await;
                    inboxes.retain(|it| it.expires_at > Utc::now());
                    for inbox in inboxes.iter() {
                        if author_store
                            .authors(&inbox.topic.into())
                            .await
                            .map(|authors| !authors.contains(&me))
                            .unwrap_or(true)
                        {
                            tracing::debug!(?pubkey, ?inbox, "new peer");
                            author_store.add_author(inbox.topic.into(), me).await;
                        }
                    }
                }
                tracing::warn!("new_peers stream ended");
            }
            .instrument(tracing::info_span!("new_peers_loop")),
        );

        // TODO: locally store list of groups and initialize them when the node starts

        Ok(node)
    }

    pub async fn get_interleaved_logs(
        &self,
        topic_id: LogId,
        authors: Vec<PublicKey>,
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
        log_id: LogId,
        author: PublicKey,
    ) -> anyhow::Result<Vec<(Header, Option<Body>)>> {
        let heights = self
            .op_store
            .get_log_heights(&log_id)
            .await?
            .into_iter()
            .map(|(pk, height)| (PublicKey::from(pk).renamed(), height))
            .collect::<Vec<_>>();
        tracing::info!(?heights, "log HEIGHTS for {log_id:?}");
        match self.op_store.get_log(&author, &log_id, None).await? {
            Some(log) => Ok(log),
            None => {
                let author = PublicKey::from(author);
                tracing::warn!("No log found for topic {log_id:?} and author {author:?}");
                Ok(vec![])
            }
        }
    }

    pub async fn get_authors(
        &self,
        topic_id: LogId,
    ) -> anyhow::Result<std::collections::HashSet<PublicKey>> {
        match self.author_store.authors(&topic_id).await {
            Some(authors) => Ok(authors.into_iter().map(|a| a.into()).collect()),
            None => {
                tracing::warn!("No authors found for topic {topic_id:?}");
                Ok(Default::default())
            }
        }
    }

    /// Create a new contact QR code with configured expiry time,
    /// subscribe to the inbox topic for it, and register the topic as active.
    pub async fn new_qr_code(
        &self,
        share_intent: ShareIntent,
        inbox: bool,
    ) -> anyhow::Result<QrCode> {
        let member = self.manager.me().await?;
        assert_eq!(member.id(), self.public_key().into());
        let mut topics = self.local_data.active_inbox_topics.write().await;
        let inbox_topic = if inbox {
            let inbox_topic = InboxTopic {
                topic: Topic::global(),
                // topic: Topic::inbox().with_name(&format!("inbox({})", self.public_key().renamed())),
                expires_at: Utc::now() + self.config.contact_code_expiry,
            };
            self.initialize_topic(inbox_topic.topic, false).await?;
            topics.insert(inbox_topic.clone());
            Some(inbox_topic)
        } else {
            None
        };

        Ok(QrCode {
            member_code: member.into(),
            inbox_topic,
            device_space_id: self.local_data.device_space_id.clone(),
            chat_actor_id: self.repped_group().group,
            share_intent,
        })
    }

    pub fn chat_actor_id(&self) -> ActorId {
        self.nodestate.chat_actor_id
    }

    pub fn repped_group(&self) -> ReppedGroup {
        ReppedGroup {
            group: self
                .chat_actor_id()
                .with_name(&format!("group({})", self.public_key().renamed())),
            individual: self.public_key(),
        }
    }

    /// Get the topic for a direct chat between two public keys.
    ///
    /// The topic is the hashed sorted public keys.
    /// Anyone who knows the two public keys can derive the same topic.
    // TODO: is this a problem? Should we use a random topic instead?
    pub fn direct_chat_topic(&self, other: ActorId) -> DirectChatId {
        return Topic::global().recast();

        let me = self.repped_group().group;
        // TODO: use two secrets from each party to construct the topic
        let topic = Topic::direct_chat([me, other]);
        if me > other {
            topic.with_name(&format!("direct({},{})", other.renamed(), me.renamed()))
        } else {
            topic.with_name(&format!("direct({},{})", me.renamed(), other.renamed()))
        }
    }

    /// Create a new chat Space, and subscribe to the Topic for this chat.
    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, fields(me = ?self.public_key())))]
    pub async fn create_group_chat_space(&self, topic: GroupChatId) -> anyhow::Result<()> {
        self.initialize_topic(topic, true).await?;
        let repped = self.repped_group();

        let (space, msgs, _event) = self
            .manager
            .create_space(
                topic,
                &[
                    (repped.individual.into(), Access::manage()),
                    (repped.group, Access::write()),
                ],
            )
            .await?;

        alias_space_messages(
            &format!(
                "create_group_chat({}, {})",
                space.id().renamed(),
                space.group_id().await?.renamed()
            ),
            topic,
            msgs.iter(),
        );

        self.process_authored_ingested_space_messages(msgs).await?;

        tracing::info!(?topic, ?topic, "created group chat space");

        Box::pin(self.repair_spaces_and_publish()).await?;

        Ok(())
    }

    /// Create a new direct chat Space.
    /// Note that only one node should create the space!
    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, fields(me = ?self.public_key())))]
    pub async fn create_direct_chat_space(&self, other: ActorId) -> anyhow::Result<()> {
        let topic = self.direct_chat_topic(other);

        let my_actor = self.repped_group().group;
        self.initialize_topic(topic, true).await?;

        tracing::info!(
            my_actor = ?my_actor.renamed(),
            other = ?other.renamed(),
            topic = ?topic.renamed(),
            "creating direct chat space"
        );

        let Some(g1) = self.manager.group(my_actor).await? else {
            tracing::error!(
                my_actor = ?my_actor.renamed(),
                "group not found for my actor"
            );
            return Err(anyhow!("group not found for my actor"));
        };
        let Some(g2) = self.manager.group(other).await? else {
            tracing::error!(other = ?other.renamed(), "group not found for other actor");
            return Err(anyhow!("group not found for other actor"));
        };

        tracing::debug!(
            g1_id = ?g1.id().renamed(),
            g2_id = ?g2.id().renamed(),
            g1_members = ?g1.members().await?.iter().map(|(id, _)| id.renamed()).collect::<Vec<_>>(),
            g2_members = ?g2.members().await?.iter().map(|(id, _)| id.renamed()).collect::<Vec<_>>(),
            "group members"
        );

        let (_space, msgs, _event) = Box::pin(self.manager.create_space(
            topic,
            &[(my_actor, Access::write()), (other, Access::write())],
        ))
        .await?;

        alias_space_messages("create_direct_chat", topic, msgs.iter());

        self.process_authored_ingested_space_messages(msgs).await?;

        tracing::info!(?topic, ?topic, "created direct chat space");

        Box::pin(self.repair_spaces_and_publish()).await?;

        Ok(())
    }

    /// "Joining" a chat means subscribing to messages for that chat.
    /// This needs to be accompanied by being added as a member of the chat Space by an existing member
    /// -- you're not fully a member until someone adds you.
    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, parent = None, fields(me = ?self.public_key())))]
    pub async fn join_group(&self, chat_id: ChatId) -> anyhow::Result<()> {
        tracing::info!(?chat_id, "joined group");
        self.initialize_topic(chat_id, true).await
    }

    pub async fn set_profile(&self, profile: Profile) -> anyhow::Result<()> {
        self.author_operation(
            Topic::global(),
            // Topic::announcements(self.repped_group().group),
            Payload::Announcements(AnnouncementsPayload::SetProfile(profile)),
            Some(&format!("set_profile({})", self.public_key().renamed())),
        )
        .await?;

        Ok(())
    }

    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, fields(me = ?self.public_key())))]
    pub async fn add_member(
        &self,
        topic: impl Into<ChatId>,
        repped_group: ReppedGroup,
    ) -> anyhow::Result<()> {
        let topic = topic.into();
        let space = self
            .manager
            .space(topic)
            .await?
            .ok_or_else(|| anyhow!("Chat has no Space: {topic}"))?;

        // TODO: we need an access level for only adding but not removing members
        // TODO: even worse, we need to be able to add groups as managers at all!!!
        //
        // XXX: since we can't add groups as managers, we add the group with Write access
        //      and the individual from that group as a Manager.
        let (msgs1, _events) = space
            .add(repped_group.individual.into(), Access::manage())
            .await?;
        let (msgs2, _events) = space.add(repped_group.group, Access::write()).await?;

        let mut msgs = vec![];
        msgs.extend(msgs1);
        msgs.extend(msgs2);

        alias_space_messages("add_member", topic, msgs.iter());

        let _header = self
            .author_operation(
                self.direct_chat_topic(repped_group.group),
                Payload::Chat(ChatPayload::JoinGroup(topic)),
                Some(&format!("add_member/invitation({})", topic.renamed())),
            )
            .await?;

        self.process_authored_ingested_space_messages(msgs).await?;

        Box::pin(self.repair_spaces_and_publish()).await?;

        Ok(())
    }

    /// Get all messages for a chat from the logs.
    // TODO: Store state instead of regenerating from the logs.
    //       This will be necessary when we switch to double ratchet message encryption.
    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, fields(me = ?self.public_key())))]
    #[cfg(feature = "testing")]
    pub async fn get_messages(&self, topic: impl Into<ChatId>) -> anyhow::Result<Vec<ChatMessage>> {
        let chat_id = topic.into();
        let mut events = vec![];
        let mut messages = vec![];

        let members = self
            .space(chat_id)
            .await?
            .members()
            .await?
            .iter()
            .filter_map(|(id, access)| {
                if access.level >= p2panda_auth::AccessLevel::Write {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let authors = self.get_authors(chat_id.into()).await?;

        for (header, payload) in self
            .get_interleaved_logs(chat_id.into(), authors.into_iter().collect())
            .await?
        {
            use crate::spaces::SpaceOperation;

            if let Some(payload) = payload {
                if let Some(space_msg) = SpaceOperation::from_payload(&header, &payload)? {
                    use crate::spaces::SpacesArgs;

                    match space_msg.args {
                        SpacesArgs::Application { .. } => {
                            use p2panda_spaces::traits::AuthoredMessage;

                            let es = self.manager.process(&space_msg).await?;
                            events.push((es, space_msg.author(), space_msg.timestamp()));
                        }
                        _ => {}
                    }
                }
            }
        }

        for (events, author, timestamp) in events {
            for event in events {
                use crate::Cbor;
                match event {
                    Event::Application { space_id, data } => {
                        messages.push(ChatMessage::from_bytes(&data)?)
                    }
                    _ => {}
                }
            }
        }

        Ok(messages)
    }

    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, fields(me = ?self.public_key())))]
    pub async fn send_message(
        &self,
        topic: impl Into<ChatId>,
        message: ChatMessageContent,
    ) -> anyhow::Result<(ChatMessage, Header)> {
        let topic = topic.into();
        let space = self
            .manager
            .space(topic)
            .await?
            .ok_or_else(|| anyhow!("Chat has no Space: {topic}"))?;

        // NOTE: duplication of timestamp and author.
        //       shouldn't we just encrypt the message itself since the rest is on the header?
        let message = ChatMessage {
            content: message,
            author: self.public_key().into(),
            timestamp: timestamp_now(),
        };
        let encrypted = space.publish(&encode_cbor(&message.clone())?).await?;

        alias_space_messages("send_message", topic, vec![&encrypted]);

        let op = encrypted.into_operation()?;

        let header = self.process_authored_ingested_operation(op).await?;

        Box::pin(self.repair_spaces_and_publish()).await?;

        Ok((message, header))
    }

    pub fn public_key(&self) -> PublicKey {
        self.local_data.private_key.public_key().into()
    }

    pub fn device_group_topic(&self) -> DeviceGroupId {
        return Topic::global().recast();
        self.local_data.device_space_id
    }

    /// Store someone as a contact, and:
    /// - register their spaces keybundle so we can add them to spaces
    /// - subscribe to their inbox
    /// - store them in the contacts map
    /// - send an invitation to them to do the same
    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, fields(me = ?self.public_key())))]
    pub async fn add_contact(&self, contact: QrCode) -> anyhow::Result<ActorId> {
        tracing::debug!("adding contact: {:?}", contact);
        let member = contact.member_code.clone();
        let member_id = member.id();
        let actor = contact.chat_actor_id;

        // Register the member in the spaces manager
        let spaces_member = member.into();
        self.manager
            .register_member(&spaces_member)
            .await
            .map_err(|e| anyhow!("Failed to register contact: {e:?}"))?;

        // Must subscribe to the new member's device group in order to receive their
        // group control messages.
        // TODO: is this idempotent? If not we must make sure to do this only once.
        self.initialize_topic(contact.device_space_id, false)
            .await?;

        // XXX: there should be a better way to wait for the device group to be created,
        //      and this may never happen if the contact is not online.
        let mut attempts = 0;
        loop {
            if let Some(group) = self.manager.group(contact.chat_actor_id).await? {
                if group
                    .members()
                    .await?
                    .iter()
                    .map(|(id, _)| *id)
                    .any(|id| id == member_id)
                {
                    break;
                }
            }

            // // TODO: use this when spaces are possible again
            // // see https://github.com/p2panda/p2panda/pull/871
            // if let Some(space) = self.manager.space(contact.device_space_id.into()).await? {
            //     if space
            //         .members()
            //         .await?
            //         .iter()
            //         .map(|(id, _)| *id)
            //         .any(|id| id == member_id)
            //     {
            //         break;
            //     }
            // }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            attempts += 1;
            if attempts > 20 {
                return Err(anyhow!(
                    "Failed to register contact's device group in 5s. Try again later."
                ));
            }
        }
        // XXX: need sleep a little more for all the messages to be processed
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        // self.initialize_topic(Topic::announcements(actor), false)
        //     .await?;

        let direct_topic = self.direct_chat_topic(actor);
        self.initialize_topic(direct_topic, true).await?;

        // TODO: needed?
        let pubkey: PublicKey = actor_to_pubkey(contact.member_code.id());
        self.author_store
            .add_author(direct_topic.into(), pubkey)
            .await;

        self.author_operation(
            self.device_group_topic(),
            Payload::DeviceGroup(DeviceGroupPayload::AddContact(contact.clone())),
            Some(&format!("add_contact/invitation({})", actor.renamed())),
        )
        .await?;

        if let Some(inbox_topic) = contact.inbox_topic.clone() {
            // self.initialize_topic(inbox_topic.topic, true).await?;
            let qr = self.new_qr_code(ShareIntent::AddContact, false).await?;
            self.author_operation(
                inbox_topic.topic,
                Payload::Inbox(InboxPayload::Contact(qr)),
                Some(&format!("add_contact/invitation({})", actor.renamed())),
            )
            .await?;
        }

        // Only the initiator of contactship should create the direct chat space
        if contact.share_intent == ShareIntent::AddContact && contact.inbox_topic.is_none() {
            self.create_direct_chat_space(actor).await?;
        }

        Ok(actor)
    }

    #[cfg_attr(feature = "instrument", tracing::instrument(skip_all, fields(me = ?self.public_key())))]
    pub async fn remove_contact(&self, chat_actor_id: ActorId) -> anyhow::Result<()> {
        // TODO: shutdown inbox task, etc.
        todo!("add tombstone to contacts list");
    }

    pub async fn space(&self, topic: impl Into<ChatId>) -> anyhow::Result<DashSpace> {
        let topic = topic.into();
        let space = self.manager.space(topic).await?;
        space.ok_or_else(|| anyhow!("Chat has no Space: {topic}"))
    }
}
