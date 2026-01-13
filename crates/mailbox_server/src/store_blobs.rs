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
    pub blobs: BTreeMap<TopicId, BTreeMap<LogId, BTreeMap<SequenceNumber, Blob>>>,
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
