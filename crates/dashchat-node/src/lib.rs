#![feature(bool_to_result)]

mod chat;
mod friend;
mod node;
mod operation;
mod spaces;
mod stores;
mod topic;
mod util;

#[cfg(test)]
mod tests;

pub mod polestar;

#[cfg(feature = "testing")]
pub mod testing;

use base64::{Engine, prelude::BASE64_STANDARD};
use p2panda_core::IdentityError;

pub use chat::{ChatId, ChatMessage, ChatMessageContent};
pub use node::{Node, NodeConfig, NodeLocalData, Notification};
pub use operation::*;
pub use p2panda_core::PrivateKey;
pub use p2panda_spaces::ActorId;
use p2panda_spaces::OperationId;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
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

// impl Serialize for PK {
//     fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
//         serializer.collect_str(&base64::display::Base64Display::new(
//             self.0.as_bytes(),
//             &BASE64_STANDARD,
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

// impl<'de> Deserialize<'de> for PK {
//     fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
//         struct Vis;
//         impl serde::de::Visitor<'_> for Vis {
//             type Value = PK;

//             fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//                 formatter.write_str("a base64 string")
//             }

//             fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
//                 let bytes: Vec<u8> = base64::decode(v).map_err(Error::custom)?;
//                 let byte_array: [u8; 32] = to_fixed_size_array(bytes).map_err(Error::custom)?;

//                 let public_key =
//                     p2panda_core::PublicKey::from_bytes(&byte_array).map_err(Error::custom)?;
//                 let pk = PK(public_key);
//                 Ok(pk)

//                 // .map(|bytes| )
//                 // .map_err(Error::custom)
//             }
//         }
//         deserializer.deserialize_str(Vis)
//     }
// }

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
        dbg!(bytes.len());
        Ok(p2panda_core::Body::new(bytes.as_slice()))
    }

    fn try_from_body(body: &p2panda_core::Body) -> Result<Self, p2panda_core::cbor::DecodeError> {
        Self::from_bytes(body.to_bytes().as_slice())
    }
}

pub trait ShortId {
    const PREFIX: &'static str;

    fn short(&self) -> String;
}

impl ShortId for p2panda_core::Hash {
    const PREFIX: &'static str = "H";
    fn short(&self) -> String {
        let mut s = self.to_hex();
        s.truncate(8);
        format!("{}|{s}", Self::PREFIX)
    }
}

impl ShortId for p2panda_core::PublicKey {
    const PREFIX: &'static str = "PK";
    fn short(&self) -> String {
        let mut s = self.to_hex();
        s.truncate(8);
        format!("{}|{s}", Self::PREFIX)
    }
}

impl ShortId for OperationId {
    const PREFIX: &'static str = "OP";
    fn short(&self) -> String {
        let mut s = self.to_hex();
        s.truncate(8);
        format!("{}|{s}", Self::PREFIX)
    }
}

pub fn timestamp_now() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("time from operation system")
        .as_secs()
}
