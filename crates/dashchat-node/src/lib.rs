#![feature(bool_to_result)]

mod chat;
mod forge;
mod friend;
mod network;
mod node;
mod operation;
mod spaces;
mod store;
mod util;

pub mod polestar;

#[cfg(feature = "testing")]
pub mod testing;

use p2panda_core::IdentityError;

pub use chat::{ChatId, ChatMessage, ChatMessageContent};
pub use node::{Node, NodeConfig, Notification};
pub use operation::{InvitationMessage, Payload};
pub use p2panda_core::PrivateKey;
pub use p2panda_spaces::ActorId;
use p2panda_spaces::OperationId;
pub use spaces::MemberCode;

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
    derive_more::Display,
    derive_more::Deref,
    derive_more::From,
    derive_more::Into,
)]
pub struct PK(p2panda_core::PublicKey);

impl std::fmt::Debug for PK {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", ShortId::short(&self.0))
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
        Ok(p2panda_core::Body::new(self.as_bytes()?.as_slice()))
    }

    fn try_from_body(body: p2panda_core::Body) -> Result<Self, p2panda_core::cbor::DecodeError> {
        Self::from_bytes(body.to_bytes().as_slice())
    }
}

pub trait ShortId {
    const PREFIX: &'static str;
    fn short(&self) -> String;
}

impl ShortId for p2panda_core::Hash {
    const PREFIX: &'static str = "H|";
    fn short(&self) -> String {
        let mut s = self.to_hex();
        s.truncate(8);
        format!("{}{s}", Self::PREFIX)
    }
}

impl ShortId for p2panda_core::PublicKey {
    const PREFIX: &'static str = "PK|";
    fn short(&self) -> String {
        let mut s = self.to_hex();
        s.truncate(8);
        format!("{}{s}", Self::PREFIX)
    }
}

impl ShortId for OperationId {
    const PREFIX: &'static str = "OP|";
    fn short(&self) -> String {
        let mut s = self.to_hex();
        s.truncate(8);
        format!("{}{s}", Self::PREFIX)
    }
}

pub fn timestamp_now() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("time from operation system")
        .as_secs()
}
