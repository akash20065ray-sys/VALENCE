//! Valence Binary Write-Ahead Log: Segment File Management
//!
//! Handles 64MB fixed-boundary segment rotation, deterministic segment file headers,
//! atomic fsync/sync_all syncing, and pre-checkpoint segment pruning.

use std::fs::{self, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::record::WalError;

/// 16-byte magic header prefix at the start of every Valence WAL segment.
pub const WAL_SEGMENT_MAGIC: &[u8; 16] = b"VALENCE_WAL_V1\0\0";
pub const WAL_SEGMENT_VERSION: u32 = 1;
pub const DEFAULT_MAX_SEGMENT_SIZE: u64 = 64 * 1024 * 1024; // 64 MB

/// Header stored at the beginning of each `.valwal` segment file (44 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalSegmentHeader {
    pub version: u32,
    pub segment_id: u64,
    pub base_lsn: u64,
    pub created_at_ms: u64,
}

impl WalSegmentHeader {
    pub fn new(segment_id: u64, base_lsn: u64) -> Self {
        let created_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            version: WAL_SEGMENT_VERSION,
            segment_id,
            base_lsn,
            created_at_ms,
        }
    }

    pub fn encode(&self) -> [u8; 44] {
        let mut buf = [0u8; 44];
        buf[0..16].copy_from_slice(WAL_SEGMENT_MAGIC);
        buf[16..20].copy_from_slice(&self.version.to_le_bytes());
        buf[20..28].copy_from_slice(&self.segment_id.to_le_bytes());
        buf[28..36].copy_from_slice(&self.base_lsn.to_le_bytes());
        buf[36..44].copy_from_slice(&self.created_at_ms.to_le_bytes());
        buf
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, WalError> {
        if bytes.len() < 44 {
            return Err(WalError::CorruptHeader("Header shorter than 44 bytes".to_string()));
        }
        if &bytes[0..16] != WAL_SEGMENT_MAGIC {
            return Err(WalError::CorruptHeader("Invalid segment magic".to_string()));
        }
        let version = u32::from_le_bytes(bytes[16..20].try_into().unwrap());
        let segment_id = u64::from_le_bytes(bytes[20..28].try_into().unwrap());
        let base_lsn = u64::from_le_bytes(bytes[28..36].try_into().unwrap());
        let created_at_ms = u64::from_le_bytes(bytes[36..44].try_into().unwrap());

        Ok(Self {
            version,
            segment_id,
            base_lsn,
            created_at_ms,
        })
    }
}

/// Generates standardized segment file name: `wal_0000000000000001.valwal`.
pub fn format_segment_filename(segment_id: u64) -> String {
    format!("wal_{:016}.valwal", segment_id)
}

/// Parses segment ID from a file path.
pub fn parse_segment_id(path: &Path) -> Option<u64> {
    let stem = path.file_stem()?.to_str()?;
    if stem.starts_with("wal_") {
        stem[4..].parse::<u64>().ok()
    } else {
        None
    }
}

/// Manages active and archived WAL segment files on disk.
pub struct WalSegmentManager {
    wal_dir: PathBuf,
    max_segment_size: u64,
    current_segment_id: u64,
    current_file: Option<File>,
    current_bytes: u64,
}

impl WalSegmentManager {
    /// Initializes the segment manager in the given directory.
    pub fn new(wal_dir: impl Into<PathBuf>, max_segment_size: u64) -> Result<Self, WalError> {
        let wal_dir = wal_dir.into();
        fs::create_dir_all(&wal_dir)?;

        Ok(Self {
            wal_dir,
            max_segment_size,
            current_segment_id: 1,
            current_file: None,
            current_bytes: 0,
        })
    }

    /// Lists all segment files in ascending segment ID order.
    pub fn list_segments(&self) -> Result<Vec<(u64, PathBuf)>, WalError> {
        let mut segments = Vec::new();
        if self.wal_dir.exists() {
            for entry in fs::read_dir(&self.wal_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "valwal") {
                    if let Some(id) = parse_segment_id(&path) {
                        segments.push((id, path));
                    }
                }
            }
        }
        segments.sort_by_key(|&(id, _)| id);
        Ok(segments)
    }

    /// Opens the latest active segment or creates segment 1 if none exists.
    pub fn open_active_segment(&mut self, next_lsn: u64) -> Result<(), WalError> {
        let existing = self.list_segments()?;
        if let Some(&(last_id, ref path)) = existing.last() {
            let metadata = fs::metadata(path)?;
            self.current_segment_id = last_id;
            self.current_bytes = metadata.len();

            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(path)?;
            file.seek(SeekFrom::End(0))?;
            self.current_file = Some(file);
        } else {
            self.rotate_new_segment(1, next_lsn)?;
        }
        Ok(())
    }

    /// Forces rotation to a brand new segment.
    pub fn rotate_new_segment(&mut self, segment_id: u64, base_lsn: u64) -> Result<(), WalError> {
        if let Some(old_file) = self.current_file.take() {
            old_file.sync_all()?;
        }

        let path = self.wal_dir.join(format_segment_filename(segment_id));
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        let header = WalSegmentHeader::new(segment_id, base_lsn);
        let header_bytes = header.encode();
        file.write_all(&header_bytes)?;
        file.sync_all()?;

        self.current_segment_id = segment_id;
        self.current_bytes = header_bytes.len() as u64;
        self.current_file = Some(file);

        Ok(())
    }

    /// Appends raw bytes into the active segment, triggering rotation if the segment fills up.
    pub fn append_raw(&mut self, bytes: &[u8], next_lsn: u64) -> Result<(), WalError> {
        if self.current_file.is_none() {
            self.open_active_segment(next_lsn)?;
        }

        // Check if append would exceed max segment size
        if self.current_bytes + (bytes.len() as u64) > self.max_segment_size {
            let next_segment_id = self.current_segment_id + 1;
            self.rotate_new_segment(next_segment_id, next_lsn)?;
        }

        if let Some(ref mut file) = self.current_file {
            file.write_all(bytes)?;
            self.current_bytes += bytes.len() as u64;
        }

        Ok(())
    }

    /// Flushes and issues hardware sync (`FlushFileBuffers` / `fdatasync`).
    pub fn sync(&mut self) -> Result<(), WalError> {
        if let Some(ref mut file) = self.current_file {
            file.sync_all()?;
        }
        Ok(())
    }

    /// Current active segment ID.
    pub fn current_segment_id(&self) -> u64 {
        self.current_segment_id
    }

    /// Deletes all segment files strictly older than `min_segment_id`.
    pub fn prune_segments_before(&self, min_segment_id: u64) -> Result<usize, WalError> {
        let mut pruned = 0;
        let segments = self.list_segments()?;
        for (id, path) in segments {
            if id < min_segment_id {
                let _ = fs::remove_file(path);
                pruned += 1;
            }
        }
        Ok(pruned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_header_encode_decode() {
        let header = WalSegmentHeader::new(42, 1000);
        let encoded = header.encode();
        let decoded = WalSegmentHeader::decode(&encoded).expect("Decode failed");
        assert_eq!(decoded.version, WAL_SEGMENT_VERSION);
        assert_eq!(decoded.segment_id, 42);
        assert_eq!(decoded.base_lsn, 1000);
    }

    #[test]
    fn test_segment_rotation_on_size_limit() {
        let test_dir = std::env::temp_dir().join(format!("val_seg_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&test_dir);

        // Max segment size = 120 bytes (header is 44B)
        let mut mgr = WalSegmentManager::new(&test_dir, 120).unwrap();
        mgr.open_active_segment(1).unwrap();
        assert_eq!(mgr.current_segment_id(), 1);

        // Append 40 bytes -> fits in segment 1 (44 + 40 = 84B)
        let payload1 = [0xAAu8; 40];
        mgr.append_raw(&payload1, 2).unwrap();
        assert_eq!(mgr.current_segment_id(), 1);

        // Append another 50 bytes -> (84 + 50 = 134 > 120) -> triggers rotation to segment 2!
        let payload2 = [0xBBu8; 50];
        mgr.append_raw(&payload2, 3).unwrap();
        assert_eq!(mgr.current_segment_id(), 2);

        let segments = mgr.list_segments().unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].0, 1);
        assert_eq!(segments[1].0, 2);

        let _ = fs::remove_dir_all(&test_dir);
    }
}
