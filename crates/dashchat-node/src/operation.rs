use p2panda_core::cbor::{DecodeError, EncodeError, decode_cbor, encode_cbor};
use p2panda_core::{Body, Extension, PruneFlag};
use serde::{Deserialize, Serialize};

use crate::chat::ChatId;
use crate::friend::Friend;
use crate::spaces::SpaceControlMessage;
use crate::topic::LogId;
use crate::{AsBody, Cbor};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Extensions {
    pub log_id: LogId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub avatar: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum AnnouncementsPayload {
    SetProfile(Profile),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum InboxPayload {
    /// Invites the recipient to add the sender as a friend.
    Friend,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChatPayload {
    /// Space control messages, including actual chat messages
    Space(Vec<SpaceControlMessage>),

    /// Instructs the recipient to subscribe to the group chat topic.
    /// This is only sent in direct chat messages.
    /// It's invalid to send in a group chat, because you must be
    /// friends with the recipient for this to be actionable.
    ///
    /// The reason for including this message in the ChatPayload
    /// is that it can only be sent to friends, and we want it to be
    /// long-lasting, so using an Inbox is not an option.
    JoinGroup(ChatId),
}

impl From<Vec<SpaceControlMessage>> for ChatPayload {
    fn from(msgs: Vec<SpaceControlMessage>) -> Self {
        Self::Space(msgs)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PrivatePayload {
    AddFriend(Friend),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Payload {
    /// Pushing data out to my friends.
    Announcements(AnnouncementsPayload),

    /// Data sent to someone who is not your friend
    Inbox(InboxPayload),

    /// Group chat data, including direct 1:1 chats
    Chat(ChatPayload),

    /// Data only seen within your private device group.
    /// No other person sees these.
    Private(PrivatePayload),
}

impl Cbor for Payload {}
impl AsBody for Payload {}

pub type Header = p2panda_core::Header<Extensions>;

impl Extension<LogId> for Extensions {
    fn extract(header: &Header) -> Option<LogId> {
        Some(header.extensions.log_id.clone())
    }
}

impl Extension<PruneFlag> for Extensions {
    fn extract(_header: &Header) -> Option<PruneFlag> {
        Some(PruneFlag::new(false))
    }
}

pub fn encode_gossip_message(header: &Header, body: Option<&Body>) -> Result<Vec<u8>, EncodeError> {
    encode_cbor(&(header.to_bytes(), body.map(|body| body.to_bytes())))
}

pub fn decode_gossip_message(bytes: &[u8]) -> Result<(Vec<u8>, Option<Vec<u8>>), DecodeError> {
    decode_cbor(bytes)
}
