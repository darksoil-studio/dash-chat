use axum::{extract::State, http::StatusCode, Json};
use redb::{Database, ReadableTable};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::{AppState, Blob, LogId, SequenceNumber, TopicId, BLOBS_TABLE, WATERMARKS_TABLE};

#[derive(Serialize, Deserialize)]
pub struct StoreBlobsRequest {
    pub blobs: BTreeMap<TopicId, BTreeMap<LogId, BTreeMap<SequenceNumber, Blob>>>,
}

pub async fn store_blobs(
    State(state): State<AppState>,
    Json(payload): Json<StoreBlobsRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let db = state.db.clone();
    // Use spawn_blocking because redb's begin_write() is a blocking call that waits
    // for exclusive write access. Running this directly in async context would block
    // tokio worker threads and cause deadlocks under concurrent load.
    tokio::task::spawn_blocking(move || store_blobs_inner(&db, &payload))
        .await
        .map_err(|e| {
            tracing::error!("Task join error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .map_err(|e| {
            tracing::error!("{}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e)
        })?;
    Ok(StatusCode::CREATED)
}

fn store_blobs_inner(db: &Database, request: &StoreBlobsRequest) -> Result<(), String> {
    let write_txn = db
        .begin_write()
        .map_err(|e| format!("Failed to begin transaction: {}", e))?;

    let mut blob_count = 0;

    {
        let mut blobs_table = write_txn
            .open_table(BLOBS_TABLE)
            .map_err(|e| format!("Failed to open blobs table: {}", e))?;

        let mut watermarks_table = write_txn
            .open_table(WATERMARKS_TABLE)
            .map_err(|e| format!("Failed to open watermarks table: {}", e))?;

        for (topic_id, logs) in &request.blobs {
            for (log_id, sequences) in logs {
                let topic_log_key = format!("{}:{}", topic_id, log_id);

                // Get current watermark for this topic:log
                let current_watermark = watermarks_table
                    .get(topic_log_key.as_str())
                    .map_err(|e| format!("Failed to read watermark: {}", e))?
                    .map(|v| v.value());

                // Collect sequence numbers being stored (BTreeMap is already sorted)
                let mut stored_seqs: BTreeSet<SequenceNumber> = BTreeSet::new();

                for (seq_num, blob) in sequences {
                    // Key format: "topic_id:log_id:sequence_number:uuid_v7"
                    let key = format!(
                        "{}:{}:{}:{}",
                        topic_id,
                        log_id,
                        seq_num,
                        uuid::Uuid::now_v7()
                    );

                    blobs_table
                        .insert(key.as_str(), blob.as_slice())
                        .map_err(|e| format!("Failed to insert blob: {}", e))?;
                    stored_seqs.insert(*seq_num);
                    blob_count += 1;
                }

                // Update watermark for this topic:log
                let new_watermark = compute_new_watermark(
                    &blobs_table,
                    topic_id,
                    log_id,
                    current_watermark,
                    &stored_seqs,
                )?;

                if let Some(wm) = new_watermark {
                    // Only update if watermark changed or was newly established
                    if current_watermark != Some(wm) {
                        watermarks_table
                            .insert(topic_log_key.as_str(), wm)
                            .map_err(|e| format!("Failed to update watermark: {}", e))?;
                        tracing::debug!(
                            "Updated watermark for {}:{} from {:?} to {}",
                            topic_id,
                            log_id,
                            current_watermark,
                            wm
                        );
                    }
                }
            }
        }
    }

    write_txn
        .commit()
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    tracing::debug!("Stored {} blobs", blob_count);
    Ok(())
}

/// Computes the new watermark after storing blobs.
/// Returns None if no watermark can be established (no sequence 0).
fn compute_new_watermark(
    blobs_table: &redb::Table<&str, &[u8]>,
    topic_id: &str,
    log_id: &str,
    current_watermark: Option<SequenceNumber>,
    new_sequences: &BTreeSet<SequenceNumber>,
) -> Result<Option<SequenceNumber>, String> {
    let mut watermark = match current_watermark {
        Some(current_watermark) => {
            // Check if new sequences or existing blobs don't extend current watermark
            // No extension from new sequences, but check if existing blobs can extend
            if !new_sequences.contains(&(current_watermark + 1))
                && !blob_exists(blobs_table, topic_id, log_id, current_watermark + 1)?
            {
                return Ok(Some(current_watermark)); // No extension possible
            }
            current_watermark
        }
        None => {
            // No watermark yet - need sequence 0 to start
            if !new_sequences.contains(&0) && !blob_exists(blobs_table, topic_id, log_id, 0)? {
                return Ok(None); // Can't establish watermark without seq 0
            }
            // Start from "before 0" so first iteration checks seq 0
            // We use wrapping subtraction to get u64::MAX, then first check is for 0
            u64::MAX
        }
    };

    // Extend watermark by checking consecutive sequences
    loop {
        let next_seq = watermark.wrapping_add(1);

        // First check new sequences (cheaper), then existing blobs
        if new_sequences.contains(&next_seq)
            || blob_exists(blobs_table, topic_id, log_id, next_seq)?
        {
            watermark = next_seq;
        } else {
            break;
        }
    }

    Ok(Some(watermark))
}

/// Checks if a blob exists for the given topic:log:seq
fn blob_exists(
    table: &redb::Table<&str, &[u8]>,
    topic_id: &str,
    log_id: &str,
    seq_num: SequenceNumber,
) -> Result<bool, String> {
    // Use range query with prefix "topic_id:log_id:seq_num:"
    let prefix = format!("{}:{}:{}:", topic_id, log_id, seq_num);
    let range_start = prefix.as_str();
    let mut range_end = prefix.clone();
    range_end.push(char::MAX);

    let mut iter = table
        .range(range_start..range_end.as_str())
        .map_err(|e| format!("Failed to create range iterator: {}", e))?;

    Ok(iter.next().is_some())
}
