//! Valence Binary Write-Ahead Log: Writer & Group-Commit Engine
//!
//! Provides thread-safe append-only binary logging with atomic LSN sequencing
//! and configurable durability sync policies.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use super::record::{WalError, WalOp, WalRecord};
use super::segment::WalSegmentManager;

/// Durability synchronization policies for the WAL writer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncPolicy {
    /// Flush and call sync_all (fdatasync/FlushFileBuffers) on every append.
    Always,
    /// Batch writes and sync when unflushed buffer exceeds threshold or on interval.
    GroupCommit { max_unflushed_bytes: usize },
    /// Rely on operating system dirty page writeback with periodic manual sync.
    Periodic,
}

impl Default for SyncPolicy {
    fn default() -> Self {
        Self::GroupCommit {
            max_unflushed_bytes: 64 * 1024, // 64 KB batch flush
        }
    }
}

/// Thread-safe high-throughput WAL writer.
pub struct WalWriter {
    segment_mgr: Mutex<WalSegmentManager>,
    next_lsn: AtomicU64,
    sync_policy: SyncPolicy,
    unflushed_bytes: AtomicUsize,
}

impl WalWriter {
    /// Constructs a new WAL writer in the specified directory.
    pub fn new(
        wal_dir: impl Into<PathBuf>,
        max_segment_size: u64,
        sync_policy: SyncPolicy,
    ) -> Result<Self, WalError> {
        Self::with_starting_lsn(wal_dir, max_segment_size, 1, sync_policy)
    }

    /// Constructs a WAL writer resuming from a specific starting LSN (e.g. after crash recovery).
    pub fn with_starting_lsn(
        wal_dir: impl Into<PathBuf>,
        max_segment_size: u64,
        starting_lsn: u64,
        sync_policy: SyncPolicy,
    ) -> Result<Self, WalError> {
        let mut mgr = WalSegmentManager::new(wal_dir, max_segment_size)?;
        mgr.open_active_segment(starting_lsn)?;

        Ok(Self {
            segment_mgr: Mutex::new(mgr),
            next_lsn: AtomicU64::new(starting_lsn),
            sync_policy,
            unflushed_bytes: AtomicUsize::new(0),
        })
    }

    /// Returns the next monotonic LSN to be assigned.
    #[inline(always)]
    pub fn next_lsn(&self) -> u64 {
        self.next_lsn.load(Ordering::Acquire)
    }

    /// Appends a mutating operation into the log with atomic monotonic LSN.
    ///
    /// Returns the assigned Log Sequence Number (LSN).
    pub fn append(&self, epoch: u64, op: WalOp) -> Result<u64, WalError> {
        let lsn = self.next_lsn.fetch_add(1, Ordering::SeqCst);
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let record = WalRecord::new(lsn, epoch, now_ms, op);
        let encoded = record.encode();
        let bytes_len = encoded.len();

        let mut mgr = self.segment_mgr.lock().map_err(|_| {
            WalError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "WAL lock poisoned",
            ))
        })?;

        mgr.append_raw(&encoded, lsn + 1)?;

        match self.sync_policy {
            SyncPolicy::Always => {
                mgr.sync()?;
                self.unflushed_bytes.store(0, Ordering::Release);
            }
            SyncPolicy::GroupCommit { max_unflushed_bytes } => {
                let prev = self.unflushed_bytes.fetch_add(bytes_len, Ordering::Relaxed);
                if prev + bytes_len >= max_unflushed_bytes {
                    mgr.sync()?;
                    self.unflushed_bytes.store(0, Ordering::Release);
                }
            }
            SyncPolicy::Periodic => {
                self.unflushed_bytes.fetch_add(bytes_len, Ordering::Relaxed);
            }
        }

        Ok(lsn)
    }

    /// Logs a snapshot checkpoint record and forces a physical sync.
    pub fn checkpoint(&self, target_epoch: u64) -> Result<u64, WalError> {
        let lsn = self.append(target_epoch, WalOp::Checkpoint { target_epoch })?;
        self.flush_and_sync()?;
        Ok(lsn)
    }

    /// Forces a flush and hardware sync of all uncommitted writes to disk.
    pub fn flush_and_sync(&self) -> Result<(), WalError> {
        let mut mgr = self.segment_mgr.lock().map_err(|_| {
            WalError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "WAL lock poisoned",
            ))
        })?;
        mgr.sync()?;
        self.unflushed_bytes.store(0, Ordering::Release);
        Ok(())
    }

    /// Prunes WAL segments whose segment ID is strictly older than `min_segment_id`.
    pub fn prune_segments_before(&self, min_segment_id: u64) -> Result<usize, WalError> {
        let mgr = self.segment_mgr.lock().map_err(|_| {
            WalError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "WAL lock poisoned",
            ))
        })?;
        mgr.prune_segments_before(min_segment_id)
    }

    /// Current active segment ID.
    pub fn current_segment_id(&self) -> u64 {
        let mgr = self.segment_mgr.lock().unwrap();
        mgr.current_segment_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::segment::DEFAULT_MAX_SEGMENT_SIZE;
    use std::fs;

    #[test]
    fn test_wal_writer_sequential_appends_and_lsn() {
        let test_dir = std::env::temp_dir().join(format!("val_writer_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&test_dir);

        let writer = WalWriter::new(&test_dir, DEFAULT_MAX_SEGMENT_SIZE, SyncPolicy::Always).unwrap();

        let lsn1 = writer.append(1, WalOp::Set {
            key: b"k1".to_vec(),
            value: b"v1".to_vec(),
            ttl_ms: None,
        }).unwrap();
        assert_eq!(lsn1, 1);

        let lsn2 = writer.append(1, WalOp::Set {
            key: b"k2".to_vec(),
            value: b"v2".to_vec(),
            ttl_ms: Some(500),
        }).unwrap();
        assert_eq!(lsn2, 2);

        let lsn3 = writer.append(2, WalOp::Del { key: b"k1".to_vec() }).unwrap();
        assert_eq!(lsn3, 3);

        assert_eq!(writer.next_lsn(), 4);

        let _ = fs::remove_dir_all(&test_dir);
    }
}
