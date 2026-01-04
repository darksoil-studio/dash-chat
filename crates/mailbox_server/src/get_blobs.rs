use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{AppState, Blob, LogId, SequenceNumber, TopicId, BLOBS_TABLE};

#[derive(Serialize, Deserialize)]
pub struct GetBlobsRequest {
    pub topics: BTreeMap<TopicId, BTreeMap<LogId, SequenceNumber>>,
}

#[derive(Serialize, Deserialize)]
pub struct GetBlobsResponse {
    #[serde(with = "serde_blobs")]
    pub blobs: BTreeMap<TopicId, BTreeMap<LogId, BTreeMap<SequenceNumber, Blob>>>,
}

mod serde_blobs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::collections::BTreeMap;
    use base64::{Engine as _, engine::general_purpose};

    pub fn serialize<S>(
        map: &BTreeMap<String, BTreeMap<String, BTreeMap<u32, Vec<u8>>>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        let mut topic_map = serializer.serialize_map(Some(map.len()))?;
        for (topic_id, logs) in map {
            let mut log_map = BTreeMap::new();
            for (log_id, sequences) in logs {
                let mut seq_map = BTreeMap::new();
                for (seq_num, blob) in sequences {
                    seq_map.insert(*seq_num, general_purpose::STANDARD.encode(blob));
                }
                log_map.insert(log_id.clone(), seq_map);
            }
            topic_map.serialize_entry(topic_id, &log_map)?;
        }
        topic_map.end()
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<BTreeMap<String, BTreeMap<String, BTreeMap<u32, Vec<u8>>>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: BTreeMap<String, BTreeMap<String, BTreeMap<u32, String>>> =
            BTreeMap::deserialize(deserializer)?;
        let mut result = BTreeMap::new();
        for (topic_id, logs) in map {
            let mut log_map = BTreeMap::new();
            for (log_id, sequences) in logs {
                let mut seq_map = BTreeMap::new();
                for (seq_num, encoded_blob) in sequences {
                    let blob = general_purpose::STANDARD
                        .decode(encoded_blob)
                        .map_err(serde::de::Error::custom)?;
                    seq_map.insert(seq_num, blob);
                }
                log_map.insert(log_id, seq_map);
            }
            result.insert(topic_id, log_map);
        }
        Ok(result)
    }
}

pub async fn get_blobs_for_topics(
    State(state): State<AppState>,
    Json(payload): Json<GetBlobsRequest>,
) -> Result<Json<GetBlobsResponse>, (StatusCode, String)> {
    let mut blobs: BTreeMap<TopicId, BTreeMap<LogId, BTreeMap<SequenceNumber, Blob>>> = BTreeMap::new();

    let read_txn = state.db.begin_read().map_err(|e| {
        tracing::error!("Failed to begin read transaction: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to begin transaction: {}", e),
        )
    })?;

    let table = read_txn.open_table(BLOBS_TABLE).map_err(|e| {
        tracing::error!("Failed to open table: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to open table: {}", e),
        )
    })?;

    for (topic_id, requested_logs) in &payload.topics {
        let mut topic_logs: BTreeMap<LogId, BTreeMap<SequenceNumber, Blob>> = BTreeMap::new();

        // Iterate through ALL blobs for this topic
        let prefix = format!("{}:", topic_id);
        let range_start = prefix.as_str();
        let mut range_end = prefix.clone();
        range_end.push(char::MAX);
        let range_end_str = range_end.as_str();

        for entry in table.range(range_start..range_end_str).map_err(|e| {
            tracing::error!("Failed to create range iterator: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create range iterator: {}", e),
            )
        })? {
            let (key, value) = entry.map_err(|e| {
                tracing::error!("Failed to read entry: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read entry: {}", e),
                )
            })?;

            let key_str: &str = key.value();
            // Key format: "topic_id:log_id:sequence_number:uuid_v7"
            let parts: Vec<&str> = key_str.split(':').collect();
            if parts.len() < 4 {
                tracing::error!("Invalid database key format: {} (expected 4 parts, got {})", key_str, parts.len());
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Invalid database key format: {} (expected 4 parts, got {})", key_str, parts.len()),
                ));
            }

            let log_id = parts[1].to_string();
            let seq_num = parts[2].parse::<SequenceNumber>().map_err(|e| {
                tracing::error!("Failed to parse sequence number from key {}: {}", key_str, e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to parse sequence number from key {}: {}", key_str, e),
                )
            })?;

            // Check if this log was requested with a specific sequence number filter
            let should_include = if let Some(min_seq_num) = requested_logs.get(&log_id) {
                // Log is in the request: only include if seq_num > min_seq_num
                seq_num > *min_seq_num
            } else {
                // Log is NOT in the request: include all blobs for this log
                true
            };

            if should_include {
                topic_logs
                    .entry(log_id)
                    .or_insert_with(BTreeMap::new)
                    .insert(seq_num, value.value().to_vec());
            }
        }

        blobs.insert(topic_id.clone(), topic_logs);
    }

    tracing::debug!("Retrieved blobs for {} topics", payload.topics.len());
    Ok(Json(GetBlobsResponse { blobs }))
}
