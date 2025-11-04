use crate::PK;
use crate::chat::ChatId;

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
}

impl TopicId for Topic {
    fn id(&self) -> [u8; 32] {
        match self {
            Topic::Chat(chat_id) => **chat_id,
            Topic::Inbox(public_key) => *public_key.as_bytes(),
        }
    }
}

impl From<ChatId> for Topic {
    fn from(chat_id: ChatId) -> Self {
        Topic::Chat(chat_id)
    }
}

impl TopicQuery for Topic {}

pub type LogId = Topic;
