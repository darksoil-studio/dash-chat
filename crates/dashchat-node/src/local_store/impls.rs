//! Redb implementations for InboxTopic
//!
//! InboxTopic is serialized as a fixed-width array of 40 bytes:
//! - 8 bytes for the timestamp in nanoseconds
//! - 32 bytes for the topic ID
//!
//! The timestamp is stored as a big-endian 64-bit integer.
//! The topic ID is stored as a 32-byte array.

use super::*;

impl redb::Key for InboxTopic {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        let topic1 = InboxTopic::from_bytes(data1);
        let topic2 = InboxTopic::from_bytes(data2);
        topic1.cmp(&topic2)
    }
}

impl redb::Value for InboxTopic {
    type SelfType<'a>
        = InboxTopic
    where
        Self: 'a;

    type AsBytes<'a>
        = [u8; 40]
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        Some(40)
    }

    fn type_name() -> TypeName {
        TypeName::new("InboxTopic")
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        let timestamp = i64::from_be_bytes(data[0..8].try_into().unwrap());
        let topic = Topic::new(data[8..40].try_into().unwrap());
        InboxTopic {
            expires_at: DateTime::from_timestamp_nanos(timestamp),
            topic,
        }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let mut buf = [0u8; 40];
        buf[0..8].copy_from_slice(
            &value
                .expires_at
                .timestamp_nanos_opt()
                .expect("invalid timestamp")
                .to_be_bytes(),
        );
        buf[8..40].copy_from_slice(&(**value.topic));
        buf
    }
}
