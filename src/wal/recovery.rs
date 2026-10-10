//! Valence Binary Write-Ahead Log: Crash Recovery & Fast Replay Engine
//!
//! Scans segment files in ascending sequence order, validates headers and CRC64
//! checksums, handles torn writes gracefully, and replays operations into shards.

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use super::record::{WalError, WalOp, WalRecord};
use super::segment::{parse_segment_id, WalSegmentHeader};
use crate::sharding::coordinator::ShardCoordinator;
use crate::snapshot::epoch::EpochTracker;

/// Summary statistics for a completed WAL recovery scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalRecoveryReport {
    pub records_replayed: usize,
    pub max_lsn: u64,
    pub max_epoch: u64,
    pub segments_scanned: usize,
    pub torn_write_detected: bool,
}

/// Recovery engine for discovering, scanning, and replaying WAL records.
pub struct WalRecovery;

impl WalRecovery {
    /// Reads and decodes all valid committed records across all segment files in `wal_dir`.
    ///
    /// Stops cleanly upon encountering the first torn write or corrupt block at the tail.
    pub fn read_all_records(wal_dir: &Path) -> Result<(Vec<WalRecord>, WalRecoveryReport), WalError> {
        let mut report = WalRecoveryReport {
            records_replayed: 0,
            max_lsn: 0,
            max_epoch: 1,
            segments_scanned: 0,
            torn_write_detected: false,
        };

        let mut all_records = Vec::new();
        if !wal_dir.exists() {
            return Ok((all_records, report));
        }

        // 1. Discover and sort segments
        let mut segment_paths = Vec::new();
        for entry in fs::read_dir(wal_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "valwal") {
                if let Some(id) = parse_segment_id(&path) {
                    segment_paths.push((id, path));
                }
            }
        }
        segment_paths.sort_by_key(|&(id, _)| id);

        // 2. Scan each segment in ascending order
        for (_seg_id, path) in segment_paths {
            report.segments_scanned += 1;
            let mut file = File::open(&path)?;
            let mut file_bytes = Vec::new();
            file.read_to_end(&mut file_bytes)?;

            // Read 44-byte segment header
            if file_bytes.len() < 44 {
                report.torn_write_detected = true;
                break;
            }

            if WalSegmentHeader::decode(&file_bytes[..44]).is_err() {
                report.torn_write_detected = true;
                break;
            }

            let mut offset = 44;
            while offset < file_bytes.len() {
                match WalRecord::decode(&file_bytes[offset..]) {
                    Ok((record, consumed)) => {
                        report.max_lsn = report.max_lsn.max(record.lsn);
                        report.max_epoch = report.max_epoch.max(record.epoch);
                        report.records_replayed += 1;
                        all_records.push(record);
                        offset += consumed;
                    }
                    Err(WalError::TruncatedRecord) | Err(WalError::CorruptChecksum { .. }) => {
                        // Reached crash boundary / torn write at tail
                        report.torn_write_detected = true;
                        break;
                    }
                    Err(WalError::InvalidMagic(_)) => {
                        // Padding zeros or partial write
                        report.torn_write_detected = true;
                        break;
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            if report.torn_write_detected {
                break;
            }
        }

        Ok((all_records, report))
    }

    /// Replays all committed WAL records directly into the multi-shard keyspace.
    pub fn replay_into_coordinator(
        wal_dir: &Path,
        coordinator: &ShardCoordinator,
        epoch_tracker: &EpochTracker,
    ) -> Result<WalRecoveryReport, WalError> {
        let (records, report) = Self::read_all_records(wal_dir)?;

        for record in records {
            match record.op {
                WalOp::Set { key, value, ttl_ms } => {
                    let _ = coordinator.execute_set(&key, &value, ttl_ms);
                }
                WalOp::Del { key } => {
                    let _ = coordinator.execute_del(&key);
                }
                WalOp::Expire { key, ttl_ms } => {
                    let _ = coordinator.execute_expire(&key, ttl_ms);
                }
                WalOp::Checkpoint { .. } => {}
            }
        }

        if report.max_epoch > epoch_tracker.current() {
            let delta = report.max_epoch - epoch_tracker.current();
            epoch_tracker.advance_by(delta);
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wal::segment::DEFAULT_MAX_SEGMENT_SIZE;
    use crate::wal::writer::{SyncPolicy, WalWriter};

    #[test]
    fn test_wal_recovery_roundtrip_and_torn_write_resilience() {
        let test_dir = std::env::temp_dir().join(format!("val_rec_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&test_dir);

        let writer = WalWriter::new(&test_dir, DEFAULT_MAX_SEGMENT_SIZE, SyncPolicy::Always).unwrap();

        // Write 5 records
        for i in 1..=5 {
            let key = format!("user:{}", i).into_bytes();
            let val = format!("data:{}", i).into_bytes();
            writer.append(i as u64, WalOp::Set { key, value: val, ttl_ms: None }).unwrap();
        }
        writer.flush_and_sync().unwrap();

        // 1. Recover without corruption
        let (records, report) = WalRecovery::read_all_records(&test_dir).unwrap();
        assert_eq!(records.len(), 5);
        assert_eq!(report.records_replayed, 5);
        assert_eq!(report.max_lsn, 5);
        assert!(!report.torn_write_detected);

        // 2. Append simulated torn write (truncated bytes at tail of active segment)
        let seg_path = test_dir.join(crate::wal::segment::format_segment_filename(1));
        let mut file = fs::OpenOptions::new().append(true).open(&seg_path).unwrap();
        use std::io::Write;
        file.write_all(&[0x56, 0x4C, 0x01, 0x99]).unwrap(); // Partial header
        file.sync_all().unwrap();

        // 3. Recover with torn write: should successfully recover the 5 valid records and stop
        let (records_after_torn, report_after_torn) = WalRecovery::read_all_records(&test_dir).unwrap();
        assert_eq!(records_after_torn.len(), 5);
        assert!(report_after_torn.torn_write_detected);

        let _ = fs::remove_dir_all(&test_dir);
    }
}
