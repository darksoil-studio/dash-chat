use crate::{
    chat::ChatId,
    testing::{AliasedId, ShortId},
};

use p2panda_net::TopicId;
use p2panda_spaces::ActorId;
use p2panda_sync::TopicQuery;
use serde::{Deserialize, Serialize};

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
pub struct Topic([u8; 32]);

impl p2panda_spaces::traits::SpaceId for Topic {}

impl Topic {
    /// The topic ID is the unique chat ID.
    pub fn chat(chat_id: [u8; 32]) -> Self {
        Self(chat_id)
    }

    /// The chat ID for direct chat between two public keys
    /// is the hash of the sorted keys.
    /// This lets both parties derive the same chat ID from the same two keys,
    /// but gives no information about what this topic is for.
    pub fn direct_chat(mut pks: [ActorId; 2]) -> Self {
        pks.sort();
        let mut hasher = blake3::Hasher::new();
        hasher.update(pks[0].as_bytes());
        hasher.update(pks[1].as_bytes());
        Self(hasher.finalize().into())
    }

    /// The topic ID is randomly generated for each new Friend code (QR code).
    pub fn random() -> Self {
        Self(rand::random())
    }

    /// The topic ID is the hashed public key.
    /// This is to prevent collisions with the inbox topic, which also uses the public key.
    pub fn announcements(actor: ActorId) -> Self {
        let hash = blake3::hash(actor.as_bytes());
        Self(hash.into())
    }

    /// The topic ID is unique.
    pub fn device_group(topic_id: DeviceGroupTopic) -> Self {
        Self(*topic_id.as_bytes())
    }
}

impl TopicId for Topic {
    fn id(&self) -> [u8; 32] {
        self.0
    }
}

pub type DeviceGroupTopic = ActorId;

pub type DashChatTopicId = Topic;

impl TopicQuery for Topic {}

pub type LogId = DashChatTopicId;

impl ShortId for Topic {
    const PREFIX: &'static str = "T";
    fn short(&self) -> String {
        let mut k = self.to_string();
        k.truncate(8);
        format!("{}|{}", Self::PREFIX, k)
    }
}

impl AliasedId for Topic {
    fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<Topic> for String {
    fn from(topic: Topic) -> Self {
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
        Ok(Self(
            hex::decode(topic)?
                .try_into()
                .map_err(|e| anyhow::anyhow!("Invalid Topic: {e:?}"))?,
        ))
    }
}

// impl Serialize for DashChatTopicId {
//     fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
//         serializer.collect_str(&base64::display::Base64Display::new(
//             &self.0,
//             &base64::prelude::BASE64_STANDARD,
//         ))
//     }
// }

// use std::convert::TryInto;

// fn to_fixed_size_array<T>(v: Vec<T>) -> Result<[T; 32], String> {
//     let boxed_slice = v.into_boxed_slice();
//     let boxed_array: Box<[T; 32]> = match boxed_slice.try_into() {
//         Ok(ba) => ba,
//         Err(o) => Err(format!(
//             "Expected a Vec of length {} but it was {}",
//             4,
//             o.len()
//         ))?,
//     };
//     Ok(*boxed_array)
// }

// impl<'de> Deserialize<'de> for DashChatTopicId {
//     fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
//         struct Vis;
//         impl serde::de::Visitor<'_> for Vis {
//             type Value = DashChatTopicId;

//             fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//                 formatter.write_str("a base64 string")
//             }

//             fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
//                 let bytes: Vec<u8> = base64::decode(v).map_err(serde::de::Error::custom)?;
//                 let byte_array: [u8; 32] =
//                     to_fixed_size_array(bytes).map_err(serde::de::Error::custom)?;

//                 let topic_id = DashChatTopicId(byte_array);
//                 Ok(topic_id)

//                 // .map(|bytes| )
//                 // .map_err(Error::custom)
//             }
//         }
//         deserializer.deserialize_str(Vis)
//     }
// }
