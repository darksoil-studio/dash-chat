use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{AppState, Blob, LogId, SequenceNumber, TopicId, BLOBS_TABLE};

#[derive(Serialize, Deserialize)]
pub struct StoreBlobsRequest {
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

pub async fn store_blobs(
    State(state): State<AppState>,
    Json(payload): Json<StoreBlobsRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let write_txn = state.db.begin_write().map_err(|e| {
        tracing::error!("Failed to begin write transaction: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to begin transaction: {}", e),
        )
    })?;

    let mut blob_count = 0;

    {
        let mut table = write_txn.open_table(BLOBS_TABLE).map_err(|e| {
            tracing::error!("Failed to open table: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open table: {}", e),
            )
        })?;

        for (topic_id, logs) in &payload.blobs {
            for (log_id, sequences) in logs {
                for (seq_num, blob) in sequences {
                    // Key format: "topic_id:log_id:sequence_number:uuid_v7"
                    let key = format!("{}:{}:{}:{}", topic_id, log_id, seq_num, uuid::Uuid::now_v7());

                    table.insert(key.as_str(), blob.as_slice()).map_err(|e| {
                        tracing::error!("Failed to insert blob: {}", e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to insert blob: {}", e),
                        )
                    })?;
                    blob_count += 1;
                }
            }
        }
    }

    write_txn.commit().map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to commit transaction: {}", e),
        )
    })?;

    tracing::debug!("Stored {} blobs", blob_count);
    Ok(StatusCode::CREATED)
}
