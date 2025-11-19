use std::marker::PhantomData;

use crate::testing::{AliasedId, ShortId};

use p2panda_net::TopicId;
use p2panda_spaces::ActorId;
use p2panda_sync::TopicQuery;
use serde::{Deserialize, Serialize,  de::DeserializeOwned};

pub trait TopicKind:
    Default
    + Clone
    + Copy
    + Send
    + Sync
    + Serialize
    + DeserializeOwned
    + std::hash::Hash
    + Eq
    + PartialEq
    + PartialOrd
    + Ord
    + std::fmt::Display
    + std::fmt::Debug
{
}
pub trait ChatTopicKind: TopicKind {}

pub type DeviceGroupTopic = ActorId;

pub mod kind {
    use super::*;

    macro_rules! topic_kind {
        ($name:ident) => {
            #[derive(
                Clone,
                Copy,
                PartialEq,
                Eq,
                PartialOrd,
                Ord,
                Hash,
                Serialize,
                Deserialize,
                derive_more::Display,
                derive_more::Debug,
            )]
            #[display("{}", stringify!($name))]
            #[debug("{}", stringify!($name))]
            pub struct $name;
            impl TopicKind for $name {}
            impl Default for $name {
                fn default() -> Self {
                    Self
                }
            }
        };
    }

    // Profiles
    topic_kind!(Announcements);
    // QR code specific
    topic_kind!(Inbox);
    // Stores linked devices
    // TODO: where does local state (settings, contacts, etc.) across devices?
    topic_kind!(DeviceGroup);
    // Chat messages but also group invitations
    topic_kind!(DirectChat);
    // Group chat messages but also group settings
    topic_kind!(GroupChat);
    // Either direct or group chat
    topic_kind!(Chat);

    topic_kind!(Untyped);

    impl ChatTopicKind for Chat {}
    impl ChatTopicKind for GroupChat {}
    impl ChatTopicKind for DirectChat {}
    impl ChatTopicKind for DeviceGroup {}
}

impl From<Topic<kind::GroupChat>> for Topic<kind::Chat> {
    fn from(topic: Topic<kind::GroupChat>) -> Topic<kind::Chat> {
        Topic::new(topic.id)
    }
}

impl From<Topic<kind::DirectChat>> for Topic<kind::Chat> {
    fn from(topic: Topic<kind::DirectChat>) -> Topic<kind::Chat> {
        Topic::new(topic.id)
    }
}

impl From<Topic<kind::DeviceGroup>> for Topic<kind::Chat> {
    fn from(topic: Topic<kind::DeviceGroup>) -> Topic<kind::Chat> {
        Topic::new(topic.id)
    }
}

#[derive(
    Copy,
    Clone,
    Serialize,
    Deserialize,
    Hash,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    derive_more::Deref,
    derive_more::Display,
    derive_more::Debug,
)]
#[display("{}", hex::encode(self.0))]
#[debug("{}", self.alias())]
pub struct LogId([u8; 32]);

impl p2panda_spaces::traits::SpaceId for LogId {}
impl TopicQuery for LogId {}

impl ShortId for LogId {
    const PREFIX: &'static str = "L";

    fn to_short_string(&self) -> String {
        hex::encode(self.0)
    }
}

impl AliasedId for LogId {
    const SHOW_SHORT_ID: bool = true;

    fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[derive(
    Copy,
    Clone,
    Hash,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    derive_more::Deref,
    derive_more::Display,
    derive_more::Debug,
)]
#[display("{}", hex::encode(self.id))]
#[debug("{}", self.alias())]
pub struct Topic<K: TopicKind = kind::Untyped> {
    #[deref]
    id: [u8; 32],

    kind: PhantomData<K>,
}

impl<K: TopicKind> p2panda_spaces::traits::SpaceId for Topic<K> {}
impl<K: TopicKind> TopicQuery for Topic<K> {}

impl<K: TopicKind> Topic<K> {
    fn new(id: [u8; 32]) -> Self {
        Self {
            id,
            kind: PhantomData::<K>,
        }
    }

    #[deprecated(note = "refactor so this is impossible")]
    pub fn recast<K2: TopicKind>(self) -> Topic<K2> {
        Topic::new(self.id)
    }
}

impl Topic<kind::Chat> {
    pub fn random() -> Self {
        Self::new(rand::random())
    }
}

impl Topic<kind::GroupChat> {
    /// The topic ID is the unique chat ID.
    pub fn group_chat(chat_id: [u8; 32]) -> Self {
        Self::new(chat_id)
    }

    pub fn random() -> Self {
        Self::new(rand::random())
    }
}

impl Topic<kind::DirectChat> {
    /// The chat ID for direct chat between two public keys
    /// is the hash of the sorted keys.
    /// This lets both parties derive the same chat ID from the same two keys,
    /// but gives no information about what this topic is for.
    pub fn direct_chat(mut pks: [ActorId; 2]) -> Self {
        pks.sort();
        let mut hasher = blake3::Hasher::new();
        hasher.update(pks[0].as_bytes());
        hasher.update(pks[1].as_bytes());
        Self::new(hasher.finalize().into())
    }
}

impl Topic<kind::Inbox> {
    /// The topic ID is randomly generated for each new Contact code (QR code).
    pub fn inbox() -> Self {
        Self::new(rand::random())
    }
}

impl Topic<kind::Announcements> {
    /// The topic ID is the actor ID.
    pub fn announcements(actor: ActorId) -> Self {
        Self::new(*actor.as_bytes())
    }
}

impl Topic<kind::DeviceGroup> {
    /// The topic ID is random.
    pub fn random() -> Self {
        Self::new(rand::random())
    }
}

impl Topic<kind::Untyped> {
    pub fn untyped(id: [u8; 32]) -> Self {
        Self {
            id,
            kind: PhantomData,
        }
    }
}

impl TopicId for LogId {
    fn id(&self) -> [u8; 32] {
        self.0
    }
}

impl<K: TopicKind> From<Topic<K>> for LogId {
    fn from(topic: Topic<K>) -> Self {
        Self(topic.id)
    }
}

impl<K: TopicKind> TopicId for Topic<K> {
    fn id(&self) -> [u8; 32] {
        self.id
    }
}

impl<K: TopicKind> ShortId for Topic<K> {
    const PREFIX: &'static str = "T";

    fn prefix() -> String {
        format!("{}:{}", Self::PREFIX, K::default())
    }
}

impl<K: TopicKind> AliasedId for Topic<K> {
    const SHOW_SHORT_ID: bool = true;

    fn as_bytes(&self) -> &[u8] {
        self.id.as_ref()
    }
}

impl<K: TopicKind> From<Topic<K>> for String {
    fn from(topic: Topic<K>) -> Self {
        topic.to_string()
    }
}

impl TryFrom<String> for Topic {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(std::str::FromStr::from_str(&value)?)
    }
}

impl std::str::FromStr for Topic {
    type Err = anyhow::Error;

    fn from_str(topic: &str) -> Result<Self, Self::Err> {
        // maybe base64?
        Ok(Self::new(
            hex::decode(topic)?
                .try_into()
                .map_err(|e| anyhow::anyhow!("Invalid Topic: {e:?}"))?,
        ))
    }
}

impl<K: TopicKind> Serialize for Topic<K> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&hex::encode(&self.id))
    }
}

use std::convert::TryInto;

fn to_fixed_size_array<T>(v: Vec<T>) -> Result<[T; 32], String> {
    let boxed_slice = v.into_boxed_slice();
    let boxed_array: Box<[T; 32]> = match boxed_slice.try_into() {
        Ok(ba) => ba,
        Err(o) => Err(format!(
            "Expected a Vec of length {} but it was {}",
            4,
            o.len()
        ))?,
    };
    Ok(*boxed_array)
}

impl<'de, K: TopicKind> Deserialize<'de> for Topic<K> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Vis<K> {
          phantom_data: PhantomData<K>  
        }
        impl<K: TopicKind> serde::de::Visitor<'_> for Vis<K> {
            type Value = Topic<K>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a hex string")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                let bytes: Vec<u8> = hex::decode(v).map_err(serde::de::Error::custom)?;
                let byte_array: [u8; 32] =
                    to_fixed_size_array(bytes).map_err(serde::de::Error::custom)?;

                let topic_id:Topic<K> = Topic::new(byte_array);
                Ok(topic_id)
            }
        }
        deserializer.deserialize_str(Vis {
            phantom_data: PhantomData::<K>
        })
    }
}
