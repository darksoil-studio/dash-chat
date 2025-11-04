use p2panda_core::cbor::{decode_cbor, encode_cbor};
use p2panda_encryption::key_bundle::LongTermKeyBundle;
use p2panda_net::ToNetwork;
use p2panda_spaces::ActorId;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::chat::InboxId;

#[derive(Clone, Debug)]
pub struct Friend {
    /// Send a message to a friend's inbox.
    pub inbox_tx: tokio::sync::mpsc::Sender<ToNetwork>,
}

#[derive(Clone, Debug, Serialize, Deserialize, derive_more::From, derive_more::Deref)]
#[serde(into = "String", try_from = "String")]
pub struct FriendCode {
    #[deref]
    pub member: MemberCode,
    salt: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize, derive_more::From)]
pub struct MemberCode(LongTermKeyBundle, ActorId);

impl FriendCode {
    pub fn new_salted(key_bundle: LongTermKeyBundle, actor_id: ActorId) -> Self {
        let salt = rand::random();
        Self {
            member: MemberCode(key_bundle, actor_id),
            salt,
        }
    }
}

impl MemberCode {
    pub fn id(&self) -> ActorId {
        self.1
    }

    pub fn key_bundle(&self) -> &LongTermKeyBundle {
        &self.0
    }
}

impl std::fmt::Display for FriendCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = encode_cbor(&self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", hex::encode(bytes))
    }
}

impl FromStr for FriendCode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        let friend_code = decode_cbor(bytes.as_slice())?;
        Ok(friend_code)
    }
}

impl From<p2panda_spaces::Member> for MemberCode {
    fn from(member: p2panda_spaces::Member) -> Self {
        Self(member.key_bundle().clone(), member.id())
    }
}

impl From<MemberCode> for p2panda_spaces::Member {
    fn from(member_code: MemberCode) -> Self {
        p2panda_spaces::Member::new(member_code.id(), member_code.key_bundle().clone())
    }
}

impl From<FriendCode> for String {
    fn from(code: FriendCode) -> Self {
        code.to_string()
    }
}

impl TryFrom<String> for FriendCode {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(FriendCode::from_str(&value).unwrap())
    }
}
