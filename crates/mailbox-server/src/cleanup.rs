use redb::{Database, ReadableTable};
use std::sync::Arc;
use std::time::Duration;

use crate::BLOBS_TABLE;

const CLEANUP_INTERVAL: Duration = Duration::from_secs(5 * 60); // 5 minutes
const MESSAGE_MAX_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60); // 7 days

/// Spawns a background task that periodically cleans up old messages
pub fn spawn_cleanup_task(db: Arc<Database>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(CLEANUP_INTERVAL);

        loop {
            interval.tick().await;

            if let Err(e) = cleanup_old_messages(&db).await {
                tracing::error!("Failed to cleanup old messages: {}", e);
            }
        }
    });
}

/// Deletes all messages older than MESSAGE_MAX_AGE
pub async fn cleanup_old_messages(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting cleanup of old messages");

    let cutoff_time = std::time::SystemTime::now() - MESSAGE_MAX_AGE;
    let cutoff_uuid = uuid::Uuid::new_v7(uuid::Timestamp::from_unix(
        uuid::NoContext,
        cutoff_time.duration_since(std::time::UNIX_EPOCH)?.as_secs(),
        0,
    ));

    let write_txn = db.begin_write()?;
    let mut deleted_count = 0;

    {
        let mut table = write_txn.open_table(BLOBS_TABLE)?;

        // Collect keys to delete
        let mut keys_to_delete = Vec::new();

        for entry in table.iter()? {
            let (key, _value) = entry?;
            let key_str: &str = key.value();

            // Key format is "topic_id:log_id:sequence_number:uuid_v7"
            let parts: Vec<&str> = key_str.split(':').collect();

            if parts.len() < 4 {
                tracing::error!(
                    "Invalid database key format during cleanup: {} (expected 4 parts, got {})",
                    key_str,
                    parts.len()
                );
                return Err(format!(
                    "Invalid database key format: {} (expected 4 parts, got {})",
                    key_str,
                    parts.len()
                )
                .into());
            }

            let message_uuid = uuid::Uuid::parse_str(parts[3]).map_err(|e| {
                tracing::error!("Failed to parse UUID from key {}: {}", key_str, e);
                format!("Failed to parse UUID from key {}: {}", key_str, e)
            })?;

            if message_uuid < cutoff_uuid {
                keys_to_delete.push(key_str.to_string());
            }
        }

        // Delete old messages
        for key in &keys_to_delete {
            table.remove(key.as_str())?;
            deleted_count += 1;
        }
    }

    write_txn.commit()?;

    tracing::info!("Cleanup completed: deleted {} old messages", deleted_count);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use redb::ReadableDatabase;
    use tempfile::NamedTempFile;

    fn create_test_db() -> (Database, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::create(temp_file.path()).unwrap();

        let write_txn = db.begin_write().unwrap();
        {
            let _table = write_txn.open_table(BLOBS_TABLE).unwrap();
        }
        write_txn.commit().unwrap();

        (db, temp_file)
    }

    #[tokio::test]
    async fn test_cleanup_old_messages() {
        let (db, _temp_file) = create_test_db();

        // Insert an old message (8 days ago)
        let old_time = std::time::SystemTime::now() - Duration::from_secs(8 * 24 * 60 * 60);
        let old_uuid = uuid::Uuid::new_v7(uuid::Timestamp::from_unix(
            uuid::NoContext,
            old_time
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            0,
        ));
        let old_key = format!("test-topic:log-1:0:{}", old_uuid);

        // Insert a recent message (1 day ago)
        let recent_uuid = uuid::Uuid::now_v7();
        let recent_key = format!("test-topic:log-1:1:{}", recent_uuid);

        {
            let write_txn = db.begin_write().unwrap();
            {
                let mut table = write_txn.open_table(BLOBS_TABLE).unwrap();
                table
                    .insert(old_key.as_str(), b"old message".as_slice())
                    .unwrap();
                table
                    .insert(recent_key.as_str(), b"recent message".as_slice())
                    .unwrap();
            }
            write_txn.commit().unwrap();
        }

        // Verify both messages exist
        {
            let read_txn = db.begin_read().unwrap();
            let table = read_txn.open_table(BLOBS_TABLE).unwrap();
            assert!(table.get(old_key.as_str()).unwrap().is_some());
            assert!(table.get(recent_key.as_str()).unwrap().is_some());
        }

        // Run cleanup
        cleanup_old_messages(&db).await.unwrap();

        // Verify old message is deleted and recent message remains
        {
            let read_txn = db.begin_read().unwrap();
            let table = read_txn.open_table(BLOBS_TABLE).unwrap();
            assert!(table.get(old_key.as_str()).unwrap().is_none());
            assert!(table.get(recent_key.as_str()).unwrap().is_some());
        }
    }
}
