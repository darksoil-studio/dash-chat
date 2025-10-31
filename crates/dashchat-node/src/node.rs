mod author_operation;
mod stream_processing;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use p2panda_auth::Access;
use p2panda_core::cbor::encode_cbor;
use p2panda_core::{Header, PrivateKey};
use p2panda_discovery::Discovery;
use p2panda_discovery::mdns::LocalDiscovery;
use p2panda_encryption::Rng;
use p2panda_encryption::crypto::x25519::SecretKey;
use p2panda_net::config::GossipConfig;
use p2panda_net::{
    FromNetwork, Network, NetworkBuilder, ResyncConfiguration, SyncConfiguration, ToNetwork,
};
use p2panda_spaces::OperationId;
use p2panda_spaces::event::Event;
use p2panda_store::{LogStore, MemoryStore};
use p2panda_stream::partial::operations::PartialOrder;
use p2panda_stream::{DecodeExt, IngestExt};
use p2panda_sync::log_sync::LogSyncProtocol;
use tokio::sync::{RwLock, mpsc};
use tokio::task;
use tokio_stream::{StreamExt, wrappers::ReceiverStream};
use tracing::Instrument;

use crate::chat::{Chat, ChatId};
use crate::chat::{ChatMessage, ChatMessageContent};
use crate::friend::{Friend, MemberCode};
use crate::network::{AuthorStore, LogId, Topic};
use crate::operation::{
    Extensions, InvitationMessage, Payload, decode_gossip_message, encode_gossip_message,
};
use crate::spaces::{DashForge, DashManager, DashSpace};
use crate::stores::{OpStore, SpacesStore};
use crate::testing::{AliasedId, alias_space_messages};
use crate::{AsBody, Cbor, PK, timestamp_now};

pub use stream_processing::Notification;

// const RELAY_ENDPOINT: &str = "https://wasser.liebechaos.org";

const NETWORK_ID: [u8; 32] = [88; 32];

const MAX_MESSAGE_SIZE: usize = 1000 * 10; // 10kb max. UDP payload size

#[derive(Clone, Debug)]
pub struct NodeConfig {
    pub resync: ResyncConfiguration,
}

impl Default for NodeConfig {
    fn default() -> Self {
        let resync = ResyncConfiguration::new().interval(3).poll_interval(1);
        Self { resync }
    }
}

pub type Orderer = PartialOrder<
    LogId,
    Extensions,
    OpStore,
    p2panda_stream::partial::MemoryStore<p2panda_core::Hash>,
>;

#[derive(Clone)]
pub struct Node {
    pub op_store: OpStore,
    pub ordering: Arc<RwLock<HashMap<Topic, Orderer>>>,
    // pub ordering_store: p2panda_stream::partial::MemoryStore<p2panda_core::Hash>,
    pub network: Network<Topic>,
    chats: Arc<RwLock<HashMap<ChatId, Chat>>>,
    author_store: AuthorStore<Topic>,
    /// TODO: should not be necessary, only used to manually persist messages from other nodes
    spaces_store: SpacesStore,
    manager: DashManager,
    /// mapping from space operations to header hashes, so that dependencies
    /// can be declared
    space_dependencies: Arc<RwLock<HashMap<OperationId, p2panda_core::Hash>>>,
    private_key: PrivateKey,
    friends: Arc<RwLock<HashMap<PK, Friend>>>,
    notification_tx: Option<mpsc::Sender<Notification>>,
}

impl Node {
    #[tracing::instrument(skip_all, fields(me = ?PK::from(private_key.public_key())))]
    pub async fn new(
        private_key: PrivateKey,
        config: NodeConfig,
        notification_tx: Option<mpsc::Sender<Notification>>,
    ) -> Result<Self> {
        let rng = Rng::default();
        let credentials = p2panda_spaces::Credentials::from_keys(
            private_key.clone(),
            SecretKey::from_rng(&rng).context("Failed to generate secret key")?,
        );
        let public_key = PK::from(private_key.public_key());

        let mdns = LocalDiscovery::new();

        let op_store = MemoryStore::<LogId, Extensions>::new();
        let author_store = AuthorStore::new();

        // TODO: unnecessary
        author_store
            .add_author(Topic::Inbox(public_key.clone()), public_key)
            .await;

        // let relay_url = RELAY_ENDPOINT.parse()?;

        let sync_protocol = LogSyncProtocol::new(author_store.clone(), op_store.clone());
        let sync_config = SyncConfiguration::new(sync_protocol).resync(config.resync.clone());

        let mut new_peers = mdns.subscribe(NETWORK_ID).unwrap();

        if true {
            let author_store = author_store.clone();
            let me = public_key.clone();
            tokio::spawn(
                async move {
                    while let Some(Ok(peer)) = new_peers.next().await {
                        let pubkey = PK::from_bytes(peer.node_addr.node_id.as_bytes()).unwrap();
                        if author_store
                            .authors(&Topic::Inbox(pubkey.clone()))
                            .await
                            .map(|authors| !authors.contains(&me))
                            .unwrap_or(true)
                        {
                            tracing::debug!("new peer: {}", pubkey);
                            author_store
                                .add_author(Topic::Inbox(pubkey.clone()), me)
                                .await;
                        }
                    }
                    tracing::warn!("new_peers stream ended");
                }
                .instrument(tracing::info_span!("new_peers_loop")),
            );
        }

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
        let chats = Arc::new(RwLock::new(HashMap::new()));

        let spaces_store = SpacesStore::new();

        let forge = DashForge {
            public_key: private_key.public_key(),
            store: spaces_store.clone(),
        };

        let key_store = p2panda_spaces::test_utils::TestKeyStore::new();
        let manager = DashManager::new(spaces_store.clone(), key_store, forge, credentials, rng)
            .await
            .unwrap();

        let op_store = OpStore::new(op_store);
        let node = Self {
            op_store: op_store.clone(),
            ordering: Arc::new(RwLock::new(HashMap::new())),
            author_store,
            spaces_store,
            network,
            chats,
            manager: manager.clone(),
            space_dependencies: Arc::new(RwLock::new(HashMap::new())),
            private_key,
            friends: Arc::new(RwLock::new(HashMap::new())),
            notification_tx,
        };

        // // TODO: this doesn't seem to make a difference
        // manager.register_member(&node.me().await?.into()).await?;

        node.initialize_inbox(public_key).await?;

        // TODO: locally store list of groups and initialize them when the node starts

        Ok(node)
    }

    pub async fn me(&self) -> anyhow::Result<MemberCode> {
        let me = self.manager.me().await?;
        Ok(MemberCode::new(me.key_bundle().clone(), me.id()))
    }

    /// Create a new chat Space, and subscribe to the Topic for this chat.
    #[tracing::instrument(skip_all, fields(me = ?self.public_key()))]
    pub async fn create_group(&self, chat_id: ChatId) -> anyhow::Result<Chat> {
        let chat = self.initialize_group(chat_id).await?;

        let (_space, msgs, _event) = self
            .manager
            .create_space(
                chat_id,
                &[(self.private_key.public_key().into(), Access::manage())],
            )
            .await?;

        alias_space_messages("create_group", msgs.iter());

        let _header = self
            .author_operation(
                chat_id.into(),
                Payload::SpaceControl(msgs),
                Some(&format!("create_group/space-control({})", chat_id.alias())),
            )
            .await?;

        Ok(chat)
    }

    /// "Joining" a chat means subscribing to messages for that chat.
    /// This needs to be accompanied by being added as a member of the chat Space by an existing member
    /// -- you're not fully a member until someone adds you.
    #[tracing::instrument(skip_all, parent = None, fields(me = ?self.public_key()))]
    pub async fn join_group(&self, chat_id: ChatId) -> anyhow::Result<Chat> {
        let chat = self.initialize_group(chat_id).await?;

        Ok(chat)
    }

    pub async fn get_groups(&self) -> anyhow::Result<Vec<ChatId>> {
        let groups = self.chats.read().await.keys().cloned().collect();
        Ok(groups)
    }

    #[tracing::instrument(skip_all, fields(me = ?self.public_key()))]
    pub async fn add_member(&self, chat_id: ChatId, pubkey: PK) -> anyhow::Result<()> {
        let (msgs, _events) = self
            .manager
            .space(chat_id)
            .await?
            .ok_or_else(|| anyhow!("Chat has no Space: {chat_id}"))?
            // TODO: we need an access level for only adding but not removing members
            .add(pubkey.into(), Access::manage())
            .await?;

        alias_space_messages("add_member", msgs.iter());

        let _header = self
            .author_operation(
                pubkey.into(),
                Payload::Invitation(InvitationMessage::JoinGroup(chat_id)),
                Some(&format!("add_member/invitation({})", chat_id.alias())),
            )
            .await?;

        let _header = self
            .author_operation(
                chat_id.into(),
                Payload::SpaceControl(msgs),
                Some(&format!("add_member/space-control({})", chat_id.alias())),
            )
            .await?;

        Ok(())
    }

    pub async fn get_members(
        &self,
        chat_id: ChatId,
    ) -> anyhow::Result<Vec<(p2panda_spaces::ActorId, Access)>> {
        if let Some(space) = self.manager.space(chat_id).await? {
            Ok(space.members().await?)
        } else {
            tracing::warn!("Chat has no Space: {chat_id}");
            Ok(vec![])
        }
    }

    #[tracing::instrument(skip_all, fields(me = ?self.public_key()))]
    pub async fn get_messages(&self, chat_id: ChatId) -> anyhow::Result<Vec<ChatMessage>> {
        let chats = self.chats.read().await;
        let chat = chats
            .get(&chat_id)
            .ok_or_else(|| anyhow!("Chat not found: {chat_id}"))?;

        let mut msgs: Vec<ChatMessage> = chat.messages.iter().cloned().collect();
        let original = msgs.clone();
        msgs.sort();
        assert_eq!(msgs, original);

        Ok(msgs)
    }

    #[tracing::instrument(skip_all, fields(me = ?self.public_key()))]
    pub async fn send_message(
        &self,
        chat_id: ChatId,
        message: ChatMessageContent,
    ) -> anyhow::Result<(ChatMessage, Header<Extensions>)> {
        let space = self
            .manager
            .space(chat_id)
            .await?
            .ok_or_else(|| anyhow!("Chat has no Space: {chat_id}"))?;

        // NOTE: duplication of timestamp and author
        let message = ChatMessage {
            content: message,
            author: self.public_key(),
            timestamp: timestamp_now(),
        };
        let encrypted = space.publish(&encode_cbor(&message.clone())?).await?;
        let encrypted_hash = encrypted.hash.clone();

        alias_space_messages("send_message", vec![&encrypted]);

        let topic = chat_id.into();

        let header = self
            .author_operation(
                topic,
                Payload::SpaceControl(vec![encrypted]),
                Some(&format!(
                    "send_message/encrypted(chat={}, msg={})",
                    chat_id.alias(),
                    encrypted_hash.alias()
                )),
            )
            .await?;

        Ok((message, header))
    }

    pub fn public_key(&self) -> PK {
        self.private_key.public_key().into()
    }

    /// Store someone as a friend, and:
    /// - register their spaces keybundle so we can add them to spaces
    /// - subscribe to their inbox
    /// - store them in the friends map
    /// - send an invitation to them to do the same
    #[tracing::instrument(skip_all, fields(me = ?self.public_key()))]
    pub async fn add_friend(&self, member: MemberCode) -> anyhow::Result<PK> {
        tracing::debug!("adding friend: {:?}", member);
        let public_key = PK::try_from(member.id()).expect("actor id is public key");

        // Register the member in the spaces manager
        let spaces_member = p2panda_spaces::Member::new(member.id(), member.key_bundle().clone());
        self.manager
            .register_member(&spaces_member)
            .await
            .map_err(|e| anyhow!("Failed to register friend: {e:?}"))?;

        let network_tx = self
            .initialize_inbox(PK::try_from(member.id()).expect("actor id is public key"))
            .await?;

        // Store the friend
        self.friends.write().await.insert(
            public_key.clone(),
            Friend {
                // member: member.clone(),
                network_tx,
            },
        );

        self.author_operation(
            public_key.clone().into(),
            Payload::Invitation(InvitationMessage::Friend),
            Some(&format!("add_friend/invitation({})", public_key.alias())),
        )
        .await?;

        Ok(public_key)
    }

    pub async fn get_friends(&self) -> anyhow::Result<Vec<PK>> {
        let friends = self.friends.read().await;
        Ok(friends.keys().cloned().collect())
    }

    #[tracing::instrument(skip_all, fields(me = ?self.public_key()))]
    pub async fn remove_friend(&self, public_key: PK) -> anyhow::Result<()> {
        // TODO: shutdown inbox task, etc.
        self.friends.write().await.remove(&public_key);
        Ok(())
    }

    pub async fn space(&self, chat_id: ChatId) -> anyhow::Result<DashSpace> {
        let space = self.manager.space(chat_id).await?;
        space.ok_or_else(|| anyhow!("Chat has no Space: {chat_id}"))
    }
}
