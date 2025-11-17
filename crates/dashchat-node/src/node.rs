mod author_operation;
mod stream_processing;

use std::collections::{BTreeSet, HashMap};
use std::pin::Pin;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use chrono::{Duration, Utc};
use futures::Stream;
use futures::stream::FuturesUnordered;
use p2panda_auth::Access;
use p2panda_core::cbor::encode_cbor;
use p2panda_core::{Body, Header, Operation, PrivateKey, PublicKey};
use p2panda_discovery::Discovery;
use p2panda_discovery::mdns::LocalDiscovery;
use p2panda_encryption::Rng;
use p2panda_encryption::crypto::x25519::SecretKey;
use p2panda_net::config::GossipConfig;
use p2panda_net::{
    FromNetwork, Network, NetworkBuilder, ResyncConfiguration, SyncConfiguration, ToNetwork,
};
use p2panda_spaces::event::Event;
use p2panda_spaces::{ActorId, OperationId};
use p2panda_store::{LogStore, MemoryStore};
use p2panda_stream::partial::operations::PartialOrder;
use p2panda_stream::{DecodeExt, IngestExt};
use p2panda_sync::log_sync::LogSyncProtocol;
use tokio::sync::{RwLock, mpsc};
use tokio::task;
use tokio_stream::{StreamExt, wrappers::ReceiverStream};
use tracing::Instrument;

use crate::chat::Chat;
use crate::chat::{ChatMessage, ChatMessageContent};
use crate::friend::{InboxTopic, QrCode, ShareIntent};
use crate::payload::{
    AnnouncementsPayload, ChatPayload, Extensions, InboxPayload, Payload, Profile,
    decode_gossip_message, encode_gossip_message,
};
use crate::spaces::{DashForge, DashManager, DashSpace};
use crate::stores::{AuthorStore, OpStore, SpacesStore};
use crate::testing::{AliasedId, alias_space_messages};
use crate::topic::{LogId, Topic, kind};
use crate::{AsBody, ChatId, DeviceGroupPayload, PK, timestamp_now};

pub use stream_processing::Notification;

// const RELAY_ENDPOINT: &str = "https://wasser.liebechaos.org";

const NETWORK_ID: [u8; 32] = [88; 32];

const MAX_MESSAGE_SIZE: usize = 1000 * 10; // 10kb max. UDP payload size

#[derive(Clone, Debug)]
pub struct NodeConfig {
    pub resync: ResyncConfiguration,
    pub friend_code_expiry: Duration,
}

impl Default for NodeConfig {
    fn default() -> Self {
        let resync = ResyncConfiguration::new().interval(3).poll_interval(1);
        Self {
            resync,
            friend_code_expiry: Duration::days(7),
        }
    }
}

pub type Orderer = PartialOrder<
    LogId,
    Extensions,
    OpStore,
    p2panda_stream::partial::MemoryStore<p2panda_core::Hash>,
>;

#[derive(Clone)]
pub struct NodeState {
    pub(crate) chats: Arc<RwLock<HashMap<ChatId, Chat>>>,
    pub(crate) friends: Arc<RwLock<HashMap<PK, QrCode>>>,
    pub(crate) chat_actor_id: ActorId,
}

#[derive(Clone)]
pub struct NodeLocalData {
    pub private_key: PrivateKey,
    /// Used to create the device group space
    // TODO: maybe not needed to be stored, the important thing is the topic?
    pub device_space_id: ChatId,
    pub active_inbox_topics: Arc<RwLock<BTreeSet<InboxTopic>>>,
}

impl NodeLocalData {
    pub fn new_random() -> Self {
        let private_key = PrivateKey::new();
        Self {
            private_key,
            device_space_id: ChatId::random(),
            active_inbox_topics: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }
}

#[derive(Clone)]
pub struct Node {
    pub op_store: OpStore,
    pub ordering: Arc<RwLock<Orderer>>,
    // pub ordering_store: p2panda_stream::partial::MemoryStore<p2panda_core::Hash>,
    pub network: Network<LogId>,
    author_store: AuthorStore<LogId>,
    /// TODO: should not be necessary, only used to manually persist messages from other nodes
    spaces_store: SpacesStore,
    pub(crate) manager: DashManager,
    /// mapping from space operations to header hashes, so that dependencies
    /// can be declared
    space_dependencies: Arc<RwLock<HashMap<OperationId, p2panda_core::Hash>>>,
    config: NodeConfig,
    local_data: NodeLocalData,
    notification_tx: Option<mpsc::Sender<Notification>>,

    /// Add new subscription streams
    stream_tx: mpsc::Sender<Pin<Box<dyn Stream<Item = Operation<Extensions>> + Send + 'static>>>,

    gossip: Arc<RwLock<HashMap<LogId, mpsc::Sender<ToNetwork>>>>,

    /// TODO: some of the stuff in here is only for testing.
    /// The channel senders are needed but any stateful stuff should go.
    pub(crate) nodestate: NodeState,
}

impl Node {
    #[tracing::instrument(skip_all, fields(me = ?PK::from(local_data.private_key.public_key())))]
    pub async fn new(
        local_data: NodeLocalData,
        config: NodeConfig,
        notification_tx: Option<mpsc::Sender<Notification>>,
    ) -> Result<Self> {
        let rng = Rng::default();
        let NodeLocalData {
            private_key,
            device_space_id,
            active_inbox_topics,
        } = local_data.clone();
        let credentials = p2panda_spaces::Credentials::from_keys(
            private_key.clone(),
            SecretKey::from_rng(&rng).context("Failed to generate secret key")?,
        );
        let public_key = PK::from(private_key.public_key());

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

        let network = network_builder.build().await.context("spawn p2p network")?;

        let spaces_store = SpacesStore::new();

        let forge = DashForge {
            public_key: private_key.public_key(),
            store: spaces_store.clone(),
        };

        let key_store = p2panda_spaces::test_utils::TestKeyStore::new();
        let manager = DashManager::new(spaces_store.clone(), key_store, forge, credentials, rng)
            .await
            .unwrap();

        // // nope
        // manager.register_member(&manager.me().await?).await?;

        let (chat_actor_id, device_group_msgs) = {
            let (space, msgs, _event) = manager
                .create_space(device_space_id, &[(manager.id(), Access::manage())])
                .await?;

            alias_space_messages("create_device_group", msgs.iter());

            (space.group_id().await?, msgs)
        };

        let (stream_tx, mut stream_rx) = mpsc::channel(100);

        let op_store = OpStore::new(op_store);
        let node = Self {
            op_store: op_store.clone(),
            ordering: Arc::new(RwLock::new(Orderer::new(
                op_store.clone(),
                Default::default(),
            ))),
            author_store: author_store.clone(),
            spaces_store,
            network,
            manager: manager.clone(),
            space_dependencies: Arc::new(RwLock::new(HashMap::new())),
            config,
            local_data,
            notification_tx,
            stream_tx,
            gossip: Arc::new(RwLock::new(HashMap::new())),
            nodestate: NodeState {
                chats: Arc::new(RwLock::new(HashMap::new())),
                friends: Arc::new(RwLock::new(HashMap::new())),
                chat_actor_id,
            },
        };

        node.spawn_stream_process_loop(stream_rx, author_store.clone());

        node.initialize_topic(
            Topic::device_group(chat_actor_id)
                .aliased(&format!("device_group({})", public_key.alias())),
            true,
        )
        .await?;

        node.initialize_topic(
            Topic::announcements(node.chat_actor_id()).aliased("announce"),
            true,
        )
        .await?;

        for topic in active_inbox_topics.read().await.iter() {
            node.initialize_topic(topic.topic.clone().aliased("inbox!"), false)
                .await?;
        }

        {
            // let _header = node
            //     .author_operation(
            //         Topic::device_group(device_space_id),
            //         Payload::DeviceGroup(DeviceGroupPayload::CreateDeviceGroup),
            //         Some(&format!(
            //             "create_device_group/space-control({})",
            //             device_space_id.alias()
            //         )),
            //     )
            //     .await?;

            let _header = node
                .author_operation(
                    Topic::device_group(chat_actor_id),
                    Payload::Chat(ChatPayload::Space(device_group_msgs.into())),
                    Some(&format!(
                        "create_device_group/space-control({})",
                        chat_actor_id.alias()
                    )),
                )
                .await?;
        }

        let author_store = author_store.clone();
        let me = public_key.clone();

        // TODO: accomodate new inbox topics as they are added
        // TODO: remove expired inbox topics from processing and from local data
        // TODO: what is the actual right way to do this?
        tokio::spawn(
            async move {
                while let Some(Ok(peer)) = new_peers.next().await {
                    let pubkey = PK::from_bytes(peer.node_addr.node_id.as_bytes()).unwrap();

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
    ) -> anyhow::Result<Vec<(Header<Extensions>, Option<Payload>)>> {
        let mut logs = Vec::new();
        for author in self.get_authors(topic_id).await? {
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
        Ok(logs)
    }

    pub async fn get_log(
        &self,
        topic_id: LogId,
        author: PublicKey,
    ) -> anyhow::Result<Vec<(Header<Extensions>, Option<Body>)>> {
        match self.op_store.get_log(&author, &topic_id, None).await? {
            Some(log) => Ok(log),
            None => {
                tracing::warn!("No log found for topic {topic_id:?} and author {author}");
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

    /// Create a new friend QR code with configured expiry time,
    /// subscribe to the inbox topic for it, and register the topic as active.
    pub async fn new_qr_code(
        &self,
        share_intent: ShareIntent,
        inbox: bool,
    ) -> anyhow::Result<QrCode> {
        let member = self.manager.me().await?;
        let mut topics = self.local_data.active_inbox_topics.write().await;
        let inbox_topic = if inbox {
            let inbox_topic = InboxTopic {
                topic: Topic::inbox().aliased("inbox"),
                expires_at: Utc::now() + self.config.friend_code_expiry,
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
            chat_actor_id: self.chat_actor_id(),
            share_intent,
        })
    }

    pub fn chat_actor_id(&self) -> ActorId {
        self.nodestate.chat_actor_id
    }

    /// Get the topic for a direct chat between two public keys.
    ///
    /// The topic is the hashed sorted public keys.
    /// Anyone who knows the two public keys can derive the same topic.
    // TODO: is this a problem? Should we use a random topic instead?
    pub fn direct_chat_topic(&self, other: ActorId) -> Topic<kind::DirectChat> {
        let me = self.chat_actor_id();
        let topic = Topic::direct_chat([me, other]);
        if me > other {
            topic.aliased(&format!("direct({},{})", other, me))
        } else {
            topic.aliased(&format!("direct({},{})", me, other))
        }
    }

    /// Create a new chat Space, and subscribe to the Topic for this chat.
    #[tracing::instrument(skip_all, fields(me = ?self.public_key()))]
    pub async fn create_group_chat_space(&self, topic: impl Into<ChatId>) -> anyhow::Result<()> {
        let topic = topic.into();
        self.initialize_topic(topic.aliased("chat"), true).await?;

        let (_space, msgs, _event) = self
            .manager
            .create_space(topic, &[(self.chat_actor_id(), Access::manage())])
            .await?;

        alias_space_messages("create_group", msgs.iter());

        let _header = self
            .author_operation(
                topic,
                Payload::Chat(ChatPayload::Space(msgs.into())),
                Some(&format!("create_group/space-control({})", topic.alias())),
            )
            .await?;

        tracing::info!(?topic, ?topic, "created group chat space");

        Ok(())
    }

    /// Create a new direct chat Space.
    /// Note that only one node should create the space!
    #[tracing::instrument(skip_all, fields(me = ?self.public_key()))]
    pub async fn create_direct_chat_space(&self, other: ActorId) -> anyhow::Result<()> {
        let topic = self.direct_chat_topic(other);

        let my_actor = self.chat_actor_id();
        self.initialize_topic(topic.aliased("chat"), true).await?;

        tracing::info!(
            my_actor = my_actor.alias(),
            other = other.alias(),
            "creating direct chat space"
        );

        let g1 = self.manager.group(my_actor).await?;
        let g2 = self.manager.group(other).await?;
        if g1.is_none() {
            tracing::error!(my_actor = my_actor.alias(), "group not found for my actor");
            return Err(anyhow!("group not found for my actor"));
        }
        if g2.is_none() {
            tracing::error!(other = other.alias(), "group not found for other actor");
            return Err(anyhow!("group not found for other actor"));
        }

        let (_space, msgs, _event) = self
            .manager
            .create_space(
                topic.into(),
                &[(my_actor, Access::manage()), (other, Access::manage())],
            )
            .await?;

        alias_space_messages("create_group", msgs.iter());

        let _header = self
            .author_operation(
                topic,
                Payload::Chat(ChatPayload::Space(msgs.into())),
                Some(&format!("create_group/space-control({})", topic.alias())),
            )
            .await?;

        tracing::info!(?topic, ?topic, "created direct chat space");

        Ok(())
    }

    /// "Joining" a chat means subscribing to messages for that chat.
    /// This needs to be accompanied by being added as a member of the chat Space by an existing member
    /// -- you're not fully a member until someone adds you.
    #[tracing::instrument(skip_all, parent = None, fields(me = ?self.public_key()))]
    pub async fn join_group(&self, chat_id: ChatId) -> anyhow::Result<()> {
        tracing::info!(?chat_id, "joined group");
        self.initialize_topic(chat_id, true).await
    }

    pub async fn set_profile(&self, profile: Profile) -> anyhow::Result<()> {
        self.author_operation(
            Topic::announcements(self.chat_actor_id()),
            Payload::Announcements(AnnouncementsPayload::SetProfile(profile)),
            Some(&format!("set_profile({})", self.public_key().alias())),
        )
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, fields(me = ?self.public_key()))]
    pub async fn add_member(&self, topic: impl Into<ChatId>, actor: ActorId) -> anyhow::Result<()> {
        let topic = topic.into();
        let (msgs, _events) = self
            .manager
            .space(topic)
            .await?
            .ok_or_else(|| anyhow!("Chat has no Space: {topic}"))?
            // TODO: we need an access level for only adding but not removing members
            // TODO: even worse, we need to be able to add groups as managers at all!!!
            .add(actor, Access::write())
            .await?;

        alias_space_messages("add_member", msgs.iter());

        let _header = self
            .author_operation(
                self.direct_chat_topic(actor),
                Payload::Chat(ChatPayload::JoinGroup(topic)),
                Some(&format!("add_member/invitation({})", topic.alias())),
            )
            .await?;

        let _header = self
            .author_operation(
                topic,
                Payload::Chat(ChatPayload::Space(msgs)),
                Some(&format!("add_member/space-control({})", topic.alias())),
            )
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, fields(me = ?self.public_key()))]
    #[cfg(feature = "testing")]
    pub async fn get_messages(&self, topic: impl Into<ChatId>) -> anyhow::Result<Vec<ChatMessage>> {
        let chat_id = topic.into();
        Ok(self
            .get_interleaved_logs(chat_id.into())
            .await?
            .into_iter()
            .flat_map(|(header, payload)| match payload {
                Some(Payload::Chat(ChatPayload::Space(msgs))) => {
                    use crate::spaces::SpacesArgs;

                    msgs.into_iter()
                        .filter_map(|m| match m.spaces_args {
                            SpacesArgs::Application { ciphertext, .. } => Some(ChatMessage {
                                content: ChatMessageContent::from(format!(
                                    "TODO: decrypt ciphertext: len={}",
                                    ciphertext.len()
                                )),
                                author: PK::from(header.public_key),
                                timestamp: header.timestamp,
                            }),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                }
                _ => vec![],
            })
            .collect())
    }

    #[tracing::instrument(skip_all, fields(me = ?self.public_key()))]
    pub async fn send_message(
        &self,
        topic: impl Into<ChatId>,
        message: ChatMessageContent,
    ) -> anyhow::Result<(ChatMessage, Header<Extensions>)> {
        let topic = topic.into();
        let space = self
            .manager
            .space(topic)
            .await?
            .ok_or_else(|| anyhow!("Chat has no Space: {topic}"))?;

        // NOTE: duplication of timestamp and author
        let message = ChatMessage {
            content: message,
            author: self.public_key().into(),
            timestamp: timestamp_now(),
        };
        let encrypted = space.publish(&encode_cbor(&message.clone())?).await?;
        let encrypted_hash = encrypted.hash.clone();

        alias_space_messages("send_message", vec![&encrypted]);

        let topic = topic.into();

        let header = self
            .author_operation(
                topic,
                Payload::Chat(vec![encrypted].into()),
                Some(&format!(
                    "send_message/encrypted(chat={}, msg={})",
                    topic.alias(),
                    encrypted_hash.alias()
                )),
            )
            .await?;

        Ok((message, header))
    }

    pub fn public_key(&self) -> PK {
        self.local_data.private_key.public_key().into()
    }

    pub fn device_group_topic(&self) -> Topic<kind::DeviceGroup> {
        Topic::device_group(self.nodestate.chat_actor_id)
    }

    /// Store someone as a friend, and:
    /// - register their spaces keybundle so we can add them to spaces
    /// - subscribe to their inbox
    /// - store them in the friends map
    /// - send an invitation to them to do the same
    #[tracing::instrument(skip_all, fields(me = ?self.public_key()))]
    pub async fn add_friend(&self, friend: QrCode) -> anyhow::Result<ActorId> {
        tracing::debug!("adding friend: {:?}", friend);
        let member = friend.member_code.clone();
        let member_id = member.id();
        let actor = friend.chat_actor_id;

        // Register the member in the spaces manager
        let spaces_member = member.into();
        self.manager
            .register_member(&spaces_member)
            .await
            .map_err(|e| anyhow!("Failed to register friend: {e:?}"))?;

        // Must subscribe to the new member's device group in order to receive their
        // group control messages.
        // TODO: is this idempotent? If not we must make sure to do this only once.
        self.initialize_topic(Topic::device_group(friend.chat_actor_id), false)
            .await?;

        // XXX: there should be a better way to wait for the device group to be created,
        //      and this may never happen if the friend is not online.
        let mut attempts = 0;
        loop {
            if let Some(group) = self.manager.group(actor).await? {
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
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            attempts += 1;
            if attempts > 20 {
                return Err(anyhow!(
                    "Failed to register friend's device group in 5s. Try again later."
                ));
            }
        }

        self.initialize_topic(Topic::announcements(actor).aliased("announce"), false)
            .await?;

        self.initialize_topic(self.direct_chat_topic(actor), true)
            .await?;

        self.author_operation(
            self.device_group_topic(),
            Payload::DeviceGroup(DeviceGroupPayload::AddFriend(friend.clone())),
            Some(&format!("add_friend/invitation({})", actor.alias())),
        )
        .await?;

        if let Some(inbox_topic) = friend.inbox_topic.clone() {
            self.initialize_topic(inbox_topic.topic.aliased("inbox"), true)
                .await?;
            let qr = self.new_qr_code(ShareIntent::AddFriend, false).await?;
            self.author_operation(
                inbox_topic.topic,
                Payload::Inbox(InboxPayload::Friend(qr)),
                Some(&format!("add_friend/invitation({})", actor.alias())),
            )
            .await?;
        }

        // Only the initiator of friendship should create the direct chat space
        if friend.share_intent == ShareIntent::AddFriend && friend.inbox_topic.is_none() {
            self.create_direct_chat_space(actor).await?;
        }

        Ok(actor)
    }

    #[tracing::instrument(skip_all, fields(me = ?self.public_key()))]
    pub async fn remove_friend(&self, chat_actor_id: ActorId) -> anyhow::Result<()> {
        // TODO: shutdown inbox task, etc.
        todo!("add tombstone to friends list");
    }

    pub async fn space(&self, topic: impl Into<ChatId>) -> anyhow::Result<DashSpace> {
        let topic = topic.into();
        let space = self.manager.space(topic).await?;
        space.ok_or_else(|| anyhow!("Chat has no Space: {topic}"))
    }
}
