use redb::{Key, TableDefinition, TypeName, Value};
use std::cmp::Ordering;
use std::fmt;
use uuid::Uuid;

use crate::watermarks_table::WatermarksKey;

/// Error type for blobs key operations
#[derive(Debug, thiserror::Error)]
pub enum BlobsKeyError {
    #[error("Topic ID contains invalid character ':': {0}")]
    InvalidTopicId(String),
    #[error("Author contains invalid character ':': {0}")]
    InvalidAuthor(String),
    #[error("Failed to parse key: {0}")]
    ParseError(String),
}

/// Key for BLOBS_TABLE: "topic_id:author:sequence_number:uuid_v7"
/// Sequence number is zero-padded to 20 digits for correct lexicographic ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobsKey {
    pub topic_id: String,
    pub author: String,
    pub sequence_number: u64,
    pub uuid: Uuid,
}

impl BlobsKey {
    /// Creates a new BlobsKey with validation
    pub fn new(
        topic_id: String,
        author: String,
        sequence_number: u64,
        uuid: Uuid,
    ) -> Result<Self, BlobsKeyError> {
        if topic_id.contains(':') {
            return Err(BlobsKeyError::InvalidTopicId(topic_id));
        }
        if author.contains(':') {
            return Err(BlobsKeyError::InvalidAuthor(author));
        }
        Ok(Self {
            topic_id,
            author,
            sequence_number,
            uuid,
        })
    }

    /// Creates a new BlobsKey with a fresh UUID v7 timestamp
    pub fn new_now(
        topic_id: String,
        author: String,
        sequence_number: u64,
    ) -> Result<Self, BlobsKeyError> {
        Self::new(topic_id, author, sequence_number, Uuid::now_v7())
    }

    /// Parses a BlobsKey from its string representation
    pub fn parse(s: &str) -> Result<Self, BlobsKeyError> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 4 {
            return Err(BlobsKeyError::ParseError(format!(
                "Expected 4 parts, got {}",
                parts.len()
            )));
        }

        let topic_id = parts[0].to_string();
        let author = parts[1].to_string();
        let sequence_number = parts[2].parse::<u64>().map_err(|e| {
            BlobsKeyError::ParseError(format!("Invalid sequence number '{}': {}", parts[2], e))
        })?;
        let uuid = Uuid::parse_str(parts[3]).map_err(|e| {
            BlobsKeyError::ParseError(format!("Invalid UUID '{}': {}", parts[3], e))
        })?;

        Ok(Self {
            topic_id,
            author,
            sequence_number,
            uuid,
        })
    }

    /// Extracts the WatermarksKey (topic_id:author) from this BlobsKey
    pub fn watermarks_key(&self) -> WatermarksKey {
        // Safe to unwrap because BlobsKey already validated no colons
        WatermarksKey::new(self.topic_id.clone(), self.author.clone()).unwrap()
    }
}

impl fmt::Display for BlobsKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{:020}:{}",
            self.topic_id, self.author, self.sequence_number, self.uuid
        )
    }
}

impl Value for BlobsKey {
    type SelfType<'a> = BlobsKey;
    type AsBytes<'a> = String;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        let s = std::str::from_utf8(data).expect("Invalid UTF-8 in BlobsKey");
        BlobsKey::parse(s).expect("Invalid BlobsKey format")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        value.to_string()
    }

    fn type_name() -> TypeName {
        TypeName::new("mailbox_server::BlobsKey")
    }
}

impl Key for BlobsKey {
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        data1.cmp(data2)
    }
}

/// Partial key for prefix-based range queries on BLOBS_TABLE
#[derive(Debug, Clone)]
pub enum BlobsKeyPrefix {
    /// Match all keys for a topic: "topic_id:"
    Topic(String),
    /// Match all keys for a topic:author: "topic_id:author:"
    TopicAuthor(String, String),
    /// Match all keys for a topic:author:seq: "topic_id:author:seq:"
    TopicAuthorSeq(String, String, u64),
}

impl BlobsKeyPrefix {
    /// Returns a BlobsKey for the lower bound of a range query.
    /// Uses minimal values (empty author, seq 0, nil UUID) for unspecified parts.
    pub fn range_start_key(&self) -> BlobsKey {
        match self {
            BlobsKeyPrefix::Topic(topic) => BlobsKey {
                topic_id: topic.clone(),
                author: String::new(),
                sequence_number: 0,
                uuid: Uuid::nil(),
            },
            BlobsKeyPrefix::TopicAuthor(topic, author) => BlobsKey {
                topic_id: topic.clone(),
                author: author.clone(),
                sequence_number: 0,
                uuid: Uuid::nil(),
            },
            BlobsKeyPrefix::TopicAuthorSeq(topic, author, seq) => BlobsKey {
                topic_id: topic.clone(),
                author: author.clone(),
                sequence_number: *seq,
                uuid: Uuid::nil(),
            },
        }
    }

    /// Returns a BlobsKey for the upper bound of a range query (exclusive).
    /// Uses maximal values for unspecified parts.
    pub fn range_end_key(&self) -> BlobsKey {
        match self {
            BlobsKeyPrefix::Topic(topic) => BlobsKey {
                topic_id: topic.clone(),
                // U+10FFFF is the highest Unicode code point, sorts after all valid authors
                author: String::from("\u{10FFFF}"),
                sequence_number: u64::MAX,
                uuid: Uuid::max(),
            },
            BlobsKeyPrefix::TopicAuthor(topic, author) => BlobsKey {
                topic_id: topic.clone(),
                author: author.clone(),
                sequence_number: u64::MAX,
                uuid: Uuid::max(),
            },
            BlobsKeyPrefix::TopicAuthorSeq(topic, author, seq) => BlobsKey {
                topic_id: topic.clone(),
                author: author.clone(),
                sequence_number: *seq,
                uuid: Uuid::max(),
            },
        }
    }

    /// Returns the start bound for a range query as a string
    pub fn range_start(&self) -> String {
        match self {
            BlobsKeyPrefix::Topic(topic) => format!("{}:", topic),
            BlobsKeyPrefix::TopicAuthor(topic, author) => format!("{}:{}:", topic, author),
            BlobsKeyPrefix::TopicAuthorSeq(topic, author, seq) => {
                format!("{}:{}:{:020}:", topic, author, seq)
            }
        }
    }

    /// Returns the end bound for a range query (appends char::MAX)
    pub fn range_end(&self) -> String {
        let mut end = self.range_start();
        end.push(char::MAX);
        end
    }
}

// Database key format: "topic_id:author:sequence_number:uuid_v7"
// The UUID v7 suffix is used for cleanup based on message age
// Sequence numbers are zero-padded to 20 digits for correct lexicographic ordering
pub const BLOBS_TABLE: TableDefinition<BlobsKey, &[u8]> = TableDefinition::new("blobs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blobs_key_roundtrip() {
        let uuid = Uuid::now_v7();
        let key = BlobsKey::new("topic1".into(), "author1".into(), 42, uuid).unwrap();
        let serialized = key.to_string();
        let parsed = BlobsKey::parse(&serialized).unwrap();
        assert_eq!(key, parsed);
    }

    #[test]
    fn test_blobs_key_zero_padding() {
        let uuid = Uuid::now_v7();
        let key = BlobsKey::new("topic".into(), "author".into(), 5, uuid).unwrap();
        let serialized = key.to_string();
        assert!(serialized.contains(":00000000000000000005:"));
    }

    #[test]
    fn test_blobs_key_ordering() {
        let uuid = Uuid::now_v7();
        let key9 = BlobsKey::new("topic".into(), "author".into(), 9, uuid).unwrap();
        let key10 = BlobsKey::new("topic".into(), "author".into(), 10, uuid).unwrap();

        // With zero-padding, 9 should sort before 10
        assert!(key9.to_string() < key10.to_string());
    }

    #[test]
    fn test_blobs_key_rejects_colon_in_topic() {
        let uuid = Uuid::now_v7();
        let result = BlobsKey::new("topic:bad".into(), "author".into(), 0, uuid);
        assert!(matches!(result, Err(BlobsKeyError::InvalidTopicId(_))));
    }

    #[test]
    fn test_blobs_key_rejects_colon_in_author() {
        let uuid = Uuid::now_v7();
        let result = BlobsKey::new("topic".into(), "author:bad".into(), 0, uuid);
        assert!(matches!(result, Err(BlobsKeyError::InvalidAuthor(_))));
    }

    #[test]
    fn test_watermarks_key_from_blobs_key() {
        let uuid = Uuid::now_v7();
        let blobs_key = BlobsKey::new("topic".into(), "author".into(), 42, uuid).unwrap();
        let watermarks_key = blobs_key.watermarks_key();
        assert_eq!(watermarks_key.topic_id, "topic");
        assert_eq!(watermarks_key.author, "author");
    }

    #[test]
    fn test_prefix_range() {
        let prefix = BlobsKeyPrefix::TopicAuthor("topic".into(), "author".into());
        assert_eq!(prefix.range_start(), "topic:author:");
        assert!(prefix.range_end().starts_with("topic:author:"));
        assert!(prefix.range_end().len() > prefix.range_start().len());
    }

    #[test]
    fn test_prefix_range_with_seq() {
        let prefix = BlobsKeyPrefix::TopicAuthorSeq("topic".into(), "author".into(), 5);
        assert_eq!(prefix.range_start(), "topic:author:00000000000000000005:");
    }
}
