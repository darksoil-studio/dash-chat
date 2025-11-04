mod message;
pub use message::*;
use p2panda_spaces::traits::SpaceId;
use rand::Rng;

#[cfg(test)]
mod tests;

use std::{collections::BTreeSet, convert::Infallible, str::FromStr};

use p2panda_net::ToNetwork;
use serde::{Deserialize, Serialize};

use crate::{PK, ShortId, testing::AliasedId};

#[derive(Clone, Debug)]
pub struct Chat {
    pub(crate) id: BaseId,

    /// The gossip overlay sender for this chat.
    pub(crate) sender: tokio::sync::mpsc::Sender<ToNetwork>,

    /// The processed decrypted messages for this chat.
    pub(crate) messages: BTreeSet<ChatMessage>,

    /// Whether I have been removed from this chat.
    pub(crate) removed: bool,
}

impl Chat {
    pub fn new(id: BaseId, sender: tokio::sync::mpsc::Sender<ToNetwork>) -> Self {
        Self {
            id,
            sender,
            messages: BTreeSet::new(),
            removed: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, derive_more::Deref)]
pub struct AnnouncementsId(BaseId);

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
    derive_more::Deref,
)]
pub struct InboxId(BaseId);

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
    derive_more::Deref,
)]
pub struct DirectId(BaseId);

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
    derive_more::Deref,
)]
pub struct GroupId(BaseId);

impl From<PK> for AnnouncementsId {
    fn from(public_key: PK) -> Self {
        let hash = blake3::hash(public_key.as_bytes());
        Self(BaseId::new(*hash.as_bytes()))
    }
}

impl InboxId {
    /// A node who knows its own ID and the ID of another node can create an InboxId
    /// which both nodes will agree on.
    ///
    /// Inboxes are asymmetric, so both nodes have to create two InboxIds,
    /// one for each direction.
    pub fn new(recipient: PK, sender: PK) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(recipient.as_bytes());
        hasher.update(sender.as_bytes());
        let hash = hasher.finalize();
        Self(BaseId::new(*hash.as_bytes()))
    }
}

impl DirectId {
    /// Create a BaseId for a direct message between two keys.
    /// The users' keys cannot be determined from the BaseId, but both
    /// users know the correct BaseId to use, given the two keys.
    ///
    /// DirectIds are symmetric, so both nodes share the same ID.
    pub fn from_keys(mut keys: [PK; 2]) -> Self {
        keys.sort();
        let mut hasher = blake3::Hasher::new();
        hasher.update(keys[0].as_bytes());
        hasher.update(keys[1].as_bytes());
        let hash = hasher.finalize();
        Self(BaseId::new(*hash.as_bytes()))
    }
}

impl GroupId {
    /// Group IDs are always chosen randomly by the group creator.
    pub fn random() -> Self {
        Self(BaseId(rand::random()))
    }
}

/// General BaseId which is used in various newtype wrappers.
#[derive(
    Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, derive_more::Deref,
)]
#[serde(into = "String", try_from = "String")]
pub struct BaseId(pub [u8; 32]);

impl BaseId {
    pub fn new(topic_id: [u8; 32]) -> Self {
        Self(topic_id)
    }

    pub fn random() -> Self {
        Self(rand::random())
    }

    pub fn from_rng(rng: &mut impl Rng) -> Self {
        Self(rng.random())
    }
}

impl SpaceId for BaseId {}

impl ShortId for BaseId {
    const PREFIX: &'static str = "Ch";
    fn short(&self) -> String {
        let mut k = self.to_string();
        k.truncate(8);
        format!("{}|{}", Self::PREFIX, k)
    }
}

impl From<BaseId> for String {
    fn from(chat_id: BaseId) -> Self {
        chat_id.to_string()
    }
}

impl TryFrom<String> for BaseId {
    type Error = Infallible;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(BaseId::from_str(&value).unwrap())
    }
}

impl std::fmt::Display for BaseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl std::fmt::Debug for BaseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.alias())
    }
}

impl FromStr for BaseId {
    type Err = anyhow::Error;

    fn from_str(topic: &str) -> Result<Self, Self::Err> {
        // maybe base64?
        Ok(Self(
            hex::decode(topic)?
                .try_into()
                .map_err(|e| anyhow::anyhow!("Invalid BaseId: {e:?}"))?,
        ))
    }
}
