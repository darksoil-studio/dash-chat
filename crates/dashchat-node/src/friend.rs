use chrono::{DateTime, Utc};
use p2panda_core::cbor::{decode_cbor, encode_cbor};
use p2panda_encryption::key_bundle::LongTermKeyBundle;
use p2panda_spaces::ActorId;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::Topic;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct Friend {
    pub member_code: MemberCode,
    pub inbox_topic: InboxTopic,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InboxTopic {
    pub expires_at: DateTime<Utc>,
    pub topic: Topic,
}

#[derive(Clone, Debug, Serialize, Deserialize, derive_more::From)]
pub struct MemberCode(LongTermKeyBundle, ActorId);

impl MemberCode {
    pub fn new(key_bundle: LongTermKeyBundle, actor_id: ActorId) -> Self {
        Self(key_bundle, actor_id)
    }

    pub fn id(&self) -> ActorId {
        self.1
    }

    pub fn key_bundle(&self) -> &LongTermKeyBundle {
        &self.0
    }
}

impl std::fmt::Display for Friend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes =
            encode_cbor(&(&self.member_code, &self.inbox_topic)).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", hex::encode(bytes))
    }
}

impl FromStr for Friend {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        let (member_code, inbox_topic) = decode_cbor(bytes.as_slice())?;
        Ok(Friend {
            member_code,
            inbox_topic,
        })
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

impl From<Friend> for String {
    fn from(code: Friend) -> Self {
        code.to_string()
    }
}

impl TryFrom<String> for Friend {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Friend::from_str(&value).unwrap())
    }
}
