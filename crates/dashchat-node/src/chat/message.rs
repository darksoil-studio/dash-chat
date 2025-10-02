use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::{Cbor, PK};

/// A standalone chat message suitable for sending to the frontend.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub content: ChatMessageContent,
    pub author: PK,
    pub timestamp: u64,
}

impl Cbor for ChatMessage {}

impl PartialOrd for ChatMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.timestamp
                .cmp(&other.timestamp)
                .then(self.content.cmp(&other.content))
                .then(self.author.cmp(&other.author)),
        )
    }
}

impl Ord for ChatMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp
            .cmp(&other.timestamp)
            .then(self.content.cmp(&other.content))
            .then(self.author.cmp(&other.author))
    }
}

#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, derive_more::From, derive_more::Deref,
)]
pub struct ChatMessageContent(String);

impl From<&str> for ChatMessageContent {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
