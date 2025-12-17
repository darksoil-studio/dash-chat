//! Topics for pub/sub.
//!
//! # List of topics
//!
//! ## `Announcements` (ActorId)
//!
//! Each node has their own announcements topic.
//! It is backed by a space with the node as sole Manager, with everyone else having Read access.
//! The node uses this to publish profile updates
//!
//! ## `Auth` (ActorId)
//!
//! KeyBundle and Auth control messages are published to this topic.
//!
//! ## `Space` (SpaceId)
//!
//! All other control messages specific to a space are published to this topic:
//!
//! - SpaceMembership
//! - SpaceUpdate
//! - Application
//!
//!
//!
//! - Published by
//! - `Inbox`: topic for inbox messages (e.g. contact requests)
//! - `DeviceGroup`: topic for device group messages (e.g. device group invitations)
//! - `Chat`: topic for chat messages (e.g. direct chat messages)
//! - `GroupChat`: topic for group chat messages (e.g. group chat messages)
//! - `Untyped`: topic for untyped messages (e.g. messages with no specific topic)

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
// pub trait ChatTopicKind: TopicKind {}

pub type DeviceGroupTopic = ActorId;

pub type GlobalTopic = Topic<kind::Global>;
pub type UntypedTopic = Topic<kind::Untyped>;

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

    // Either direct or group chat
    topic_kind!(Chat);
    // Global topic!
    // TODO: alpha: this needs to be refined
    topic_kind!(Global);

    topic_kind!(Untyped);

    // impl ChatTopicKind for Chat {}
    // impl ChatTopicKind for GroupChat {}
    // impl ChatTopicKind for DirectChat {}
    // impl ChatTopicKind for DeviceGroup {}
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

impl Topic<kind::Global> {
    pub fn global() -> Self {
        Self::new([255; 32]).aliased("GLOBAL")
    }
}

impl Topic<kind::Chat> {
    pub fn random() -> Self {
        Self::new(rand::random())
    }

    pub fn direct_chat(mut pks: [ActorId; 2]) -> Self {
        pks.sort();
        let mut hasher = blake3::Hasher::new();
        hasher.update(pks[0].as_bytes());
        hasher.update(pks[1].as_bytes());
        Self::new(hasher.finalize().into())
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
