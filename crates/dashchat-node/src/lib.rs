#![feature(bool_to_result)]

mod chat;
mod friend;
mod node;
mod payload;
pub mod spaces;
pub mod stores;
pub mod topic;
mod util;

pub mod polestar;

#[cfg(feature = "testing")]
pub mod testing;

use p2panda_core::IdentityError;

pub use chat::{ChatId, ChatMessage, ChatMessageContent};
pub use friend::{QrCode, ShareIntent};
pub use node::{Node, NodeConfig, NodeLocalData, Notification};
pub use p2panda_core::PrivateKey;
pub use p2panda_spaces::ActorId;
pub use payload::*;
pub use topic::{DashChatTopicId, Topic};

use crate::testing::AliasedId;

#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    derive_more::Deref,
    derive_more::From,
    derive_more::Into,
)]
pub struct PK(p2panda_core::PublicKey);

impl std::fmt::Debug for PK {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", AliasedId::alias(&self.0))
    }
}

impl std::fmt::Display for PK {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", AliasedId::alias(&self.0))
    }
}

impl PK {
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, IdentityError> {
        Ok(Self(p2panda_core::PublicKey::from_bytes(bytes)?))
    }
}

impl From<ActorId> for PK {
    fn from(id: ActorId) -> Self {
        Self(p2panda_core::PublicKey::from_bytes(id.as_bytes()).unwrap())
    }
}

impl From<PK> for ActorId {
    fn from(pk: PK) -> Self {
        Self::from_bytes(pk.0.as_bytes()).unwrap()
    }
}

pub trait Cbor: serde::Serialize + serde::de::DeserializeOwned {
    fn as_bytes(&self) -> Result<Vec<u8>, p2panda_core::cbor::EncodeError> {
        p2panda_core::cbor::encode_cbor(&self)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, p2panda_core::cbor::DecodeError> {
        p2panda_core::cbor::decode_cbor(bytes)
    }
}

pub trait AsBody: Cbor {
    fn try_into_body(&self) -> Result<p2panda_core::Body, p2panda_core::cbor::EncodeError> {
        let bytes = self.as_bytes()?;
        Ok(p2panda_core::Body::new(bytes.as_slice()))
    }

    fn try_from_body(body: &p2panda_core::Body) -> Result<Self, p2panda_core::cbor::DecodeError> {
        Self::from_bytes(body.to_bytes().as_slice())
    }
}

pub fn timestamp_now() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("time from operation system")
        .as_secs()
}
