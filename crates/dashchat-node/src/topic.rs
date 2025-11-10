use crate::chat::ChatId;
use crate::friend::InboxTopic;
use crate::{PK, friend::Friend};

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
    derive_more::Deref,
)]
pub struct Topic([u8; 32]);

impl Topic {
    /// The topic ID is the unique chat ID.
    pub fn chat(chat_id: ChatId) -> Self {
        Self(chat_id.0)
    }

    /// The topic ID is randomly generated for each new Friend code (QR code).
    pub fn inbox() -> Self {
        Self(rand::random())
    }

    /// The topic ID is the hashed public key.
    /// This is to prevent collisions with the inbox topic, which also uses the public key.
    pub fn announcements(public_key: impl Into<p2panda_core::PublicKey>) -> Self {
        let hash = blake3::hash(public_key.into().as_bytes());
        Self(hash.into())
    }

    /// The topic ID is unique.
    pub fn private(topic_id: PrivateTopicId) -> Self {
        Self(topic_id)
    }
}

impl TopicId for Topic {
    fn id(&self) -> [u8; 32] {
        self.0
    }
}

pub type PrivateTopicId = [u8; 32];

impl From<ChatId> for Topic {
    fn from(chat_id: ChatId) -> Self {
        Topic::chat(chat_id)
    }
}

pub type DashChatTopicId = Topic;

impl TopicQuery for Topic {}

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
