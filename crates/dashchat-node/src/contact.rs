use chrono::{DateTime, Utc};
use p2panda_core::cbor::{decode_cbor, encode_cbor};
use p2panda_encryption::key_bundle::LongTermKeyBundle;
use p2panda_spaces::ActorId;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::Topic;

/// The content for a QR code or deep link.
///
/// These codes are used to introduce two nodes for the purpose of either establishing
/// mutual friendship, or linking these two devices together under the same identity.
///
/// The flow has some similarities in either case. In both cases, an "inbox" is established
/// for the lifetime of the QR code, so that the QR code recipient can send its own
/// data back to the sender, without needing to exchange QR codes in both directions.
///
/// When linking a device, the QR code sender adds the recipient to the device group.
/// Whenever a person joins a chat group, they join with their device group, so that all of
/// their devices can participate in the chat. The ActorId of the group is the unified
/// identity which that person uses to join chat groups.
///
/// When adding a contact, no groups are joined, it's only for the purpose of exchanging
/// pubkeys and key bundles, so that chat groups can be joined in the future.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
// #[serde(into = "String", try_from = "String")]
pub struct QrCode {
    /// Pubkey and key bundle of this node: allows adding this node to encrypted spaces.
    pub member_code: MemberCode,
    /// Topic for receiving messages from this node during the lifetime of the QR code.
    /// The initiator will specify an InboxTopic, and the recipient will send back a QR
    /// code without an associated inbox, because after this exchange the two nodes
    /// can communicate directly.
    pub inbox_topic: Option<InboxTopic>,
    /// Topic for the device group of this node.
    pub device_space_id: Topic<crate::topic::kind::DeviceGroup>,
    /// Actor ID to add to spaces
    pub chat_actor_id: ActorId,
    /// The intent of the QR code: whether to add this node as a contact or a device.
    pub share_intent: ShareIntent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShareIntent {
    AddDevice,
    AddContact,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InboxTopic {
    pub expires_at: DateTime<Utc>,
    pub topic: Topic<crate::topic::kind::Inbox>,
}

/// Just add serialization around [`p2panda_spaces::Member`]`
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, derive_more::From)]
pub struct MemberCode {
    pub actor_id: ActorId,
    pub key_bundle: LongTermKeyBundle,
}

impl MemberCode {
    pub fn id(&self) -> ActorId {
        self.actor_id
    }

    pub fn key_bundle(&self) -> &LongTermKeyBundle {
        &self.key_bundle
    }
}

impl std::fmt::Display for QrCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = encode_cbor(&(
            &self.member_code,
            &self.inbox_topic,
            &self.device_space_id,
            &self.chat_actor_id,
            &self.share_intent,
        ))
        .map_err(|_| std::fmt::Error)?;
        write!(f, "{}", hex::encode(bytes))
    }
}

impl FromStr for QrCode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        let (member_code, inbox_topic, device_space_id, chat_actor_id, share_intent) =
            decode_cbor(bytes.as_slice())?;
        Ok(QrCode {
            member_code,
            inbox_topic,
            device_space_id,
            chat_actor_id,
            share_intent,
        })
    }
}

impl From<p2panda_spaces::Member> for MemberCode {
    fn from(member: p2panda_spaces::Member) -> Self {
        Self {
            key_bundle: member.key_bundle().clone(),
            actor_id: member.id(),
        }
    }
}

impl From<MemberCode> for p2panda_spaces::Member {
    fn from(member_code: MemberCode) -> Self {
        p2panda_spaces::Member::new(member_code.id(), member_code.key_bundle().clone())
    }
}

impl From<QrCode> for String {
    fn from(code: QrCode) -> Self {
        code.to_string()
    }
}

impl TryFrom<String> for QrCode {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(QrCode::from_str(&value).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use crate::topic::kind;

    use super::*;

    #[test]
    fn test_contact_roundtrip() {
        let pubkey = p2panda_encryption::crypto::x25519::PublicKey::from_bytes([11; 32]);
        let contact = QrCode {
            member_code: MemberCode {
                actor_id: ActorId::from_bytes(&[22; 32]).unwrap(),
                key_bundle: LongTermKeyBundle::new(
                    pubkey,
                    p2panda_encryption::key_bundle::PreKey::new(
                        pubkey,
                        p2panda_encryption::key_bundle::Lifetime::new(3600),
                    ),
                    p2panda_encryption::crypto::xeddsa::XSignature::from_bytes([33; 64]),
                ),
            },
            inbox_topic: Some(InboxTopic {
                topic: Topic::inbox(),
                expires_at: Utc::now() + chrono::Duration::seconds(3600),
            }),
            device_space_id: Topic::<kind::DeviceGroup>::random(),
            chat_actor_id: ActorId::from_bytes(&[44; 32]).unwrap(),
            share_intent: ShareIntent::AddDevice,
        };
        let encoded = contact.to_string();
        let decoded = QrCode::from_str(&encoded).unwrap();

        assert_eq!(contact, decoded);
    }
}
