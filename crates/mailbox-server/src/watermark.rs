use redb::{Database, ReadableDatabase, ReadableTable};
use std::collections::{BTreeMap, BTreeSet};

use crate::{SequenceNumber, BLOBS_TABLE, WATERMARKS_TABLE};

/// Computes initial watermarks by scanning all existing blobs.
/// Called once at startup to ensure watermarks are in sync with stored blobs.
///
/// Note: We only need the keys to extract sequence numbers, but redb doesn't
/// provide a keys-only iterator. Keys and values share the same B-tree pages,
/// so the page bytes are loaded together. We avoid deserializing values by
/// dropping the AccessGuard<V> without calling .value().
pub fn compute_initial_watermarks(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Computing initial watermarks from existing blobs");

    // Step 1: Collect all sequence numbers per topic:log
    let mut sequences_per_log: BTreeMap<String, BTreeSet<SequenceNumber>> = BTreeMap::new();

    {
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(BLOBS_TABLE)?;

        // Note: redb's iter() returns (key, value) pairs. We only access the key.
        // The value's AccessGuard is dropped without deserialization.
        for entry in table.iter()? {
            let (key, value) = entry?;
            drop(value);

            let key_str: &str = key.value();

            // Key format: "topic_id:log_id:sequence_number:uuid_v7"
            let parts: Vec<&str> = key_str.split(':').collect();
            if parts.len() < 4 {
                continue; // Skip malformed keys
            }

            let topic_log_key = format!("{}:{}", parts[0], parts[1]);
            let seq_num: SequenceNumber = parts[2].parse()?;

            sequences_per_log
                .entry(topic_log_key)
                .or_default()
                .insert(seq_num);
        }
    }

    // Step 2: Compute watermarks and write them
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(WATERMARKS_TABLE)?;

        for (topic_log_key, sequences) in sequences_per_log {
            if let Some(watermark) = compute_contiguous_watermark(&sequences) {
                table.insert(topic_log_key.as_str(), watermark)?;
            }
        }
    }
    write_txn.commit()?;

    tracing::info!("Initial watermarks computed successfully");
    Ok(())
}

/// Computes the highest contiguous sequence number from a set of sequences.
/// Returns None if sequence 0 is not present.
/// Returns Some(n) where n is the highest value such that 0..=n are all present.
pub fn compute_contiguous_watermark(
    sequences: &BTreeSet<SequenceNumber>,
) -> Option<SequenceNumber> {
    if !sequences.contains(&0) {
        return None;
    }

    let mut watermark: SequenceNumber = 0;
    for &seq in sequences.iter() {
        if seq == watermark + 1 {
            watermark = seq;
        } else if seq > watermark + 1 {
            break; // Gap found
        }
    }
    Some(watermark)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_contiguous_watermark_empty() {
        let sequences = BTreeSet::new();
        assert_eq!(compute_contiguous_watermark(&sequences), None);
    }

    #[test]
    fn test_compute_contiguous_watermark_no_zero() {
        let sequences: BTreeSet<u64> = [1, 2, 3].into_iter().collect();
        assert_eq!(compute_contiguous_watermark(&sequences), None);
    }

    #[test]
    fn test_compute_contiguous_watermark_only_zero() {
        let sequences: BTreeSet<u64> = [0].into_iter().collect();
        assert_eq!(compute_contiguous_watermark(&sequences), Some(0));
    }

    #[test]
    fn test_compute_contiguous_watermark_contiguous() {
        let sequences: BTreeSet<u64> = [0, 1, 2, 3, 4].into_iter().collect();
        assert_eq!(compute_contiguous_watermark(&sequences), Some(4));
    }

    #[test]
    fn test_compute_contiguous_watermark_with_gap() {
        let sequences: BTreeSet<u64> = [0, 1, 2, 5, 6].into_iter().collect();
        assert_eq!(compute_contiguous_watermark(&sequences), Some(2));
    }

    #[test]
    fn test_compute_contiguous_watermark_with_gap_unordered() {
        let sequences: BTreeSet<u64> = [0, 2, 5, 1, 6].into_iter().collect();
        assert_eq!(compute_contiguous_watermark(&sequences), Some(2));
    }

    #[test]
    fn test_compute_contiguous_watermark_gap_at_start() {
        let sequences: BTreeSet<u64> = [0, 2, 3, 4].into_iter().collect();
        assert_eq!(compute_contiguous_watermark(&sequences), Some(0));
    }
}
