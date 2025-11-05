use crate::PK;
use crate::chat::ChatId;

use p2panda_core::Hash;
use p2panda_net::TopicId;
use p2panda_sync::TopicQuery;
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Copy,
    Clone,
    Serialize,
    Deserialize,
    Hash,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    derive_more::Display,
)]
#[display("{:?}", self)]
pub enum Topic {
    Chat(ChatId),
    Inbox(PK),
    Announcements(PK),
}

impl TopicId for Topic {
    fn id(&self) -> [u8; 32] {
        match self {
            Topic::Chat(chat_id) => **chat_id,
            Topic::Inbox(public_key) => *public_key.as_bytes(),
            Topic::Announcements(public_key) => *public_key.as_bytes(),
        }
    }
}

impl From<ChatId> for Topic {
    fn from(chat_id: ChatId) -> Self {
        Topic::Chat(chat_id)
    }
}

impl From<Topic> for DashChatTopicId {
    fn from(topic: Topic) -> Self {
        DashChatTopicId::from(Hash::from_bytes(topic.id().clone()))
    }
}


impl From<Hash> for DashChatTopicId {
    fn from(hash: Hash) -> Self {
        DashChatTopicId(hash)
    }
}

impl TopicQuery for Topic {}

#[derive(
    Clone,
    Debug,
    Copy,
    Hash,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    // derive_more::Display,
)]
pub struct DashChatTopicId(pub Hash);

impl TopicQuery for DashChatTopicId {}

impl TopicId for DashChatTopicId {
    fn id(&self) -> [u8; 32] {
        self.0.as_bytes().clone()
    }
}

pub type LogId = DashChatTopicId;

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
