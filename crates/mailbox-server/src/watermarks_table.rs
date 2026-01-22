use redb::{Key, TableDefinition, TypeName, Value};
use std::cmp::Ordering;
use std::fmt;

/// Error type for watermarks key operations
#[derive(Debug, thiserror::Error)]
pub enum WatermarksKeyError {
    #[error("Topic ID contains invalid character ':': {0}")]
    InvalidTopicId(String),
    #[error("Author contains invalid character ':': {0}")]
    InvalidAuthor(String),
    #[error("Failed to parse key: {0}")]
    ParseError(String),
}

/// Key for WATERMARKS_TABLE: "topic_id:author"
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WatermarksKey {
    // NOTE: order of these fields matters!
    pub topic_id: String,
    pub author: String,
}

impl WatermarksKey {
    /// Creates a new WatermarksKey with validation
    pub fn new(topic_id: String, author: String) -> Result<Self, WatermarksKeyError> {
        if topic_id.contains(':') {
            return Err(WatermarksKeyError::InvalidTopicId(topic_id));
        }
        if author.contains(':') {
            return Err(WatermarksKeyError::InvalidAuthor(author));
        }
        Ok(Self { topic_id, author })
    }

    /// Parses a WatermarksKey from its string representation
    pub fn parse(s: &str) -> Result<Self, WatermarksKeyError> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(WatermarksKeyError::ParseError(format!(
                "Expected 2 parts, got {}",
                parts.len()
            )));
        }

        Ok(Self {
            topic_id: parts[0].to_string(),
            author: parts[1].to_string(),
        })
    }
}

impl fmt::Display for WatermarksKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.topic_id, self.author)
    }
}

impl Value for WatermarksKey {
    type SelfType<'a> = WatermarksKey;
    type AsBytes<'a> = String;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        let s = std::str::from_utf8(data).expect("Invalid UTF-8 in WatermarksKey");
        WatermarksKey::parse(s).expect("Invalid WatermarksKey format")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        value.to_string()
    }

    fn type_name() -> TypeName {
        TypeName::new("mailbox_server::WatermarksKey")
    }
}

impl Key for WatermarksKey {
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        let key1 = WatermarksKey::from_bytes(data1);
        let key2 = WatermarksKey::from_bytes(data2);
        key1.cmp(&key2)
    }
}

// Watermarks table: tracks highest contiguous sequence number per topic:author
// Key format: "topic_id:author"
// Value: highest contiguous sequence number (0..=watermark are all present)
pub const WATERMARKS_TABLE: TableDefinition<WatermarksKey, u64> =
    TableDefinition::new("watermarks");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watermarks_key_roundtrip() {
        let key = WatermarksKey::new("topic1".into(), "author1".into()).unwrap();
        let serialized = key.to_string();
        let parsed = WatermarksKey::parse(&serialized).unwrap();
        assert_eq!(key, parsed);
    }

    #[test]
    fn test_watermarks_key_rejects_colon_in_topic() {
        let result = WatermarksKey::new("topic:bad".into(), "author".into());
        assert!(matches!(result, Err(WatermarksKeyError::InvalidTopicId(_))));
    }

    #[test]
    fn test_watermarks_key_rejects_colon_in_author() {
        let result = WatermarksKey::new("topic".into(), "author:bad".into());
        assert!(matches!(result, Err(WatermarksKeyError::InvalidAuthor(_))));
    }

    #[test]
    fn test_watermarks_key_ordering() {
        let key_a = WatermarksKey::new("topic".into(), "a".into()).unwrap();
        let key_b = WatermarksKey::new("topic".into(), "b".into()).unwrap();
        assert!(key_a < key_b);
    }
}
