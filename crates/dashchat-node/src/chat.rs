mod message;
pub use message::*;
use p2panda_spaces::traits::SpaceId;
use rand::Rng;

#[cfg(test)]
mod tests;

use std::{collections::BTreeSet, convert::Infallible, str::FromStr};

use p2panda_net::ToNetwork;
use serde::{Deserialize, Serialize};

use crate::{ShortId, testing::AliasedId};

#[derive(Clone, Debug)]
pub struct Chat {
    pub(crate) id: ChatId,

    /// The processed decrypted messages for this chat.
    pub(crate) messages: BTreeSet<ChatMessage>,

    /// Whether I have been removed from this chat.
    pub(crate) removed: bool,
}

impl Chat {
    pub fn new(id: ChatId) -> Self {
        Self {
            id,
            messages: BTreeSet::new(),
            removed: false,
        }
    }
}

#[derive(
    Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, derive_more::Deref,
)]
#[serde(into = "String", try_from = "String")]
pub struct ChatId(pub [u8; 32]);

impl ChatId {
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

impl SpaceId for ChatId {}

impl ShortId for ChatId {
    const PREFIX: &'static str = "Ch";
    fn short(&self) -> String {
        let mut k = self.to_string();
        k.truncate(8);
        format!("{}|{}", Self::PREFIX, k)
    }
}

impl From<ChatId> for String {
    fn from(chat_id: ChatId) -> Self {
        chat_id.to_string()
    }
}

impl TryFrom<String> for ChatId {
    type Error = Infallible;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(ChatId::from_str(&value).unwrap())
    }
}

impl std::fmt::Display for ChatId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl std::fmt::Debug for ChatId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.alias())
    }
}

impl FromStr for ChatId {
    type Err = anyhow::Error;

    fn from_str(topic: &str) -> Result<Self, Self::Err> {
        // maybe base64?
        Ok(Self(
            hex::decode(topic)?
                .try_into()
                .map_err(|e| anyhow::anyhow!("Invalid ChatId: {e:?}"))?,
        ))
    }
}
