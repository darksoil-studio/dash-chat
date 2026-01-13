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
pub struct GetBlobsForTopicResponse {
    // The blobs that the client does not have
    pub blobs: BTreeMap<LogId, BTreeMap<SequenceNumber, Blob>>,
    // The blobs that the server is missing from the client's request
    pub missing: BTreeMap<LogId, Vec<SequenceNumber>>,
}

#[derive(Serialize, Deserialize)]
pub struct GetBlobsResponse {
    pub blobs_by_topic: BTreeMap<TopicId, GetBlobsForTopicResponse>,
}

pub async fn get_blobs_for_topics(
    State(state): State<AppState>,
    Json(payload): Json<GetBlobsRequest>,
) -> Result<Json<GetBlobsResponse>, (StatusCode, String)> {
    let mut blobs_by_topic: BTreeMap<TopicId, GetBlobsForTopicResponse> = BTreeMap::new();

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
        let mut max_seq_per_log: BTreeMap<LogId, SequenceNumber> = BTreeMap::new();

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

            // Track the maximum sequence number for each log
            max_seq_per_log
                .entry(log_id.clone())
                .and_modify(|max| *max = (*max).max(seq_num))
                .or_insert(seq_num);

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
                    .insert(seq_num, Blob::from(value.value().to_vec()));
            }
        }

        // Calculate missing blobs: for each requested log, check if server has gaps
        let mut missing: BTreeMap<LogId, Vec<SequenceNumber>> = BTreeMap::new();
        for (log_id, client_max_seq) in requested_logs {
            if let Some(&server_max_seq) = max_seq_per_log.get(log_id) {
                // If server's highest sequence number is less than client's,
                // the server is missing blobs
                if server_max_seq < *client_max_seq {
                    let missing_seq_nums: Vec<SequenceNumber> =
                        ((server_max_seq + 1)..=*client_max_seq)
                            .map(|seq| seq as SequenceNumber)
                            .collect();
                    tracing::debug!(
                        "Server missing {} blobs for log {} in topic {} (sequences: {:?})",
                        missing_seq_nums.len(),
                        log_id,
                        topic_id,
                        missing_seq_nums
                    );
                    missing.insert(log_id.clone(), missing_seq_nums);
                }
            } else {
                // Server has no blobs for this log at all - request from 0 to client_max_seq
                let missing_seq_nums: Vec<SequenceNumber> = (0..=*client_max_seq)
                    .map(|seq| seq as SequenceNumber)
                    .collect();
                tracing::debug!(
                    "Server has no blobs for log {} in topic {} (need sequences: {:?})",
                    log_id,
                    topic_id,
                    missing_seq_nums
                );
                missing.insert(log_id.clone(), missing_seq_nums);
            }
        }

        blobs_by_topic.insert(
            topic_id.clone(),
            GetBlobsForTopicResponse {
                blobs: topic_logs,
                missing,
            },
        );
    }

    tracing::debug!("Retrieved blobs for {} topics", payload.topics.len());
    Ok(Json(GetBlobsResponse { blobs_by_topic }))
}
