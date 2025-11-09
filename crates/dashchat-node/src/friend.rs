use p2panda_core::cbor::{decode_cbor, encode_cbor};
use p2panda_encryption::key_bundle::LongTermKeyBundle;
use p2panda_spaces::ActorId;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Friend {}

#[derive(Clone, Debug, Serialize, Deserialize, derive_more::From)]
#[serde(into = "String", try_from = "String")]
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

impl std::fmt::Display for MemberCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = encode_cbor(&(self.0.clone(), self.1)).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", hex::encode(bytes))
    }
}

impl FromStr for MemberCode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        let (long_term_key_bundle, actor_id) = decode_cbor(bytes.as_slice())?;
        Ok(Self(long_term_key_bundle, actor_id))
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

impl From<MemberCode> for String {
    fn from(code: MemberCode) -> Self {
        code.to_string()
    }
}

impl TryFrom<String> for MemberCode {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(MemberCode::from_str(&value).unwrap())
    }
}
