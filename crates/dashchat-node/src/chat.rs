mod message;
use std::collections::BTreeSet;

pub use message::*;

use crate::Topic;

#[derive(Clone, Debug)]
pub struct Chat {
    pub(crate) id: Topic<crate::topic::kind::Chat>,

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

pub type ChatId = Topic<crate::topic::kind::Chat>;
pub type GroupChatId = Topic<crate::topic::kind::GroupChat>;
pub type DirectChatId = Topic<crate::topic::kind::DirectChat>;
pub type DeviceGroupId = Topic<crate::topic::kind::DeviceGroup>;
