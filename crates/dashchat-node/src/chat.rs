mod message;
pub use message::*;
use p2panda_spaces::{ActorId, traits::SpaceId};
use rand::Rng;

use std::{collections::BTreeSet, convert::Infallible, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
    PK, Topic,
    testing::{AliasedId, ShortId},
};

#[derive(Clone, Debug)]
pub struct Chat {
    pub(crate) id: Topic,

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

// TODO: remove
pub type ChatId = Topic;
