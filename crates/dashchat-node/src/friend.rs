use p2panda_core::cbor::{decode_cbor, encode_cbor};
use p2panda_encryption::key_bundle::LongTermKeyBundle;
use p2panda_net::ToNetwork;
use p2panda_spaces::ActorId;
use p2panda_spaces::member::Member;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Friend {
    // pub member: Member,
    pub network_tx: tokio::sync::mpsc::Sender<ToNetwork>,
}

#[derive(Clone, Debug, Serialize, Deserialize, derive_more::From)]
#[serde(into = "String", try_from = "String")]
pub struct MemberCode(LongTermKeyBundle, ActorId);

impl From<Member> for MemberCode {
    fn from(member: Member) -> Self {
        Self(member.key_bundle().clone(), member.id())
    }
}

impl From<MemberCode> for Member {
    fn from(member_code: MemberCode) -> Self {
        Member::new(member_code.1, member_code.0)
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
