//! Valence Snapshot Engine: Binary Disk Persistence & Crash Recovery
//!
//! Provides streaming on-disk serialization and atomic restoration of point-in-time
//! snapshots. Features an atomic two-phase write protocol (.tmp -> rename),
//! deterministic binary framing, and an ECMA-182 CRC64 checksum to protect against
//! data corruption and partial disk writes.

use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::snapshot::epoch::Epoch;
use crate::snapshot::iterator::SnapshotItem;
use crate::storage::RobinHoodMap;

/// Magic signature at the beginning of every Valence snapshot file (16 bytes).
pub const SNAPSHOT_MAGIC: &[u8; 16] = b"VALENCE_SNAP_V1\0";

/// Magic footer signature preceding the CRC64 checksum (8 bytes).
pub const SNAPSHOT_FOOTER_MAGIC: &[u8; 8] = b"VAL_EOF\0";

/// Current binary format specification version.
pub const FORMAT_VERSION: u32 = 1;

/// Standard ECMA-182 CRC64 polynomial.
const CRC64_POLY: u64 = 0x42F0E1EBA9EA3693;

/// Compile-time generation of the 256-entry CRC64 lookup table.
const fn make_crc_table() -> [u64; 256] {
    let mut table = [0u64; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = (i as u64) << 56;
        let mut j = 0;
        while j < 8 {
            if (crc & 0x8000_0000_0000_0000) != 0 {
                crc = (crc << 1) ^ CRC64_POLY;
            } else {
                crc <<= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

static CRC_TABLE: [u64; 256] = make_crc_table();

/// Streaming CRC64 checksum calculator.
#[derive(Debug, Clone, Copy, Default)]
pub struct Crc64 {
    crc: u64,
}

impl Crc64 {
    /// Creates a new CRC64 calculator initialized to zero.
    pub const fn new() -> Self {
        Self { crc: 0 }
    }

    /// Updates the checksum with incoming byte slice.
    #[inline]
    pub fn update(&mut self, data: &[u8]) {
        for &byte in data {
            let table_idx = ((self.crc >> 56) ^ (byte as u64)) as usize & 0xFF;
            self.crc = (self.crc << 8) ^ CRC_TABLE[table_idx];
        }
    }

    /// Returns the finalized checksum value.
    #[inline(always)]
    pub fn finish(&self) -> u64 {
        self.crc
    }
}

/// Parsed metadata header of a Valence snapshot file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotHeader {
    pub version: u32,
    pub flags: u32,
    pub target_epoch: Epoch,
    pub created_at_ms: u64,
    pub item_count: u64,
}

/// Metadata describing a written snapshot file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotFileMeta {
    pub file_path: PathBuf,
    pub target_epoch: Epoch,
    pub item_count: u64,
    pub total_file_bytes: u64,
    pub checksum: u64,
}

/// Errors occurring during snapshot serialization or restoration.
#[derive(Debug)]
pub enum PersistenceError {
    Io(io::Error),
    InvalidMagic,
    InvalidFooter,
    UnsupportedVersion(u32),
    ChecksumMismatch { expected: u64, actual: u64 },
    UnexpectedEof,
    CorruptEntry(String),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "Snapshot I/O error: {}", e),
            Self::InvalidMagic => write!(f, "Invalid snapshot magic header: not a Valence snapshot"),
            Self::InvalidFooter => write!(f, "Invalid snapshot footer: truncated file"),
            Self::UnsupportedVersion(v) => write!(f, "Unsupported snapshot version: {}", v),
            Self::ChecksumMismatch { expected, actual } => {
                write!(f, "CRC64 checksum mismatch: expected 0x{:016X}, got 0x{:016X}", expected, actual)
            }
            Self::UnexpectedEof => write!(f, "Unexpected EOF while reading snapshot stream"),
            Self::CorruptEntry(msg) => write!(f, "Corrupted snapshot entry: {}", msg),
        }
    }
}

impl std::error::Error for PersistenceError {}

impl From<io::Error> for PersistenceError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// High-performance streaming snapshot writer.
pub struct SnapshotWriter;

impl SnapshotWriter {
    /// Writes a point-in-time snapshot to disk using atomic two-phase commit.
    ///
    /// Writes to a temporary file (`.tmp`) first, flushes and syncs to storage,
    /// then atomically renames to the final destination to guarantee zero partial writes.
    pub fn write_snapshot(
        target_path: &Path,
        target_epoch: Epoch,
        items: &[SnapshotItem],
    ) -> Result<SnapshotFileMeta, PersistenceError> {
        let tmp_path = target_path.with_extension("val.tmp");
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = File::create(&tmp_path)?;
        let mut writer = BufWriter::with_capacity(64 * 1024, file);
        let mut crc = Crc64::new();
        let mut bytes_written: u64 = 0;

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // 1. Write Header (48 bytes total)
        // Magic: 16B
        writer.write_all(SNAPSHOT_MAGIC)?;
        crc.update(SNAPSHOT_MAGIC);
        bytes_written += SNAPSHOT_MAGIC.len() as u64;

        // Version: 4B
        let ver_bytes = FORMAT_VERSION.to_le_bytes();
        writer.write_all(&ver_bytes)?;
        crc.update(&ver_bytes);
        bytes_written += 4;

        // Flags: 4B
        let flags: u32 = 0;
        let flags_bytes = flags.to_le_bytes();
        writer.write_all(&flags_bytes)?;
        crc.update(&flags_bytes);
        bytes_written += 4;

        // Target Epoch: 8B
        let epoch_bytes = target_epoch.to_le_bytes();
        writer.write_all(&epoch_bytes)?;
        crc.update(&epoch_bytes);
        bytes_written += 8;

        // Created At Timestamp: 8B
        let time_bytes = now_ms.to_le_bytes();
        writer.write_all(&time_bytes)?;
        crc.update(&time_bytes);
        bytes_written += 8;

        // Item Count: 8B
        let count_bytes = (items.len() as u64).to_le_bytes();
        writer.write_all(&count_bytes)?;
        crc.update(&count_bytes);
        bytes_written += 8;

        // 2. Stream Payload Entries
        for item in items {
            // Key length (4B) + Key bytes
            let key_len = item.key.len() as u32;
            let klen_bytes = key_len.to_le_bytes();
            writer.write_all(&klen_bytes)?;
            crc.update(&klen_bytes);
            writer.write_all(&item.key)?;
            crc.update(&item.key);
            bytes_written += 4 + item.key.len() as u64;

            // Value length (4B) + Value bytes
            let val_len = item.value.len() as u32;
            let vlen_bytes = val_len.to_le_bytes();
            writer.write_all(&vlen_bytes)?;
            crc.update(&vlen_bytes);
            writer.write_all(&item.value)?;
            crc.update(&item.value);
            bytes_written += 4 + item.value.len() as u64;

            // Mutation Epoch (8B)
            let item_epoch_bytes = item.epoch.to_le_bytes();
            writer.write_all(&item_epoch_bytes)?;
            crc.update(&item_epoch_bytes);
            bytes_written += 8;

            // Expiry flag (1B) + optional Expiry timestamp (8B)
            if let Some(exp_ms) = item.expire_at_ms {
                writer.write_all(&[1u8])?;
                crc.update(&[1u8]);
                let exp_bytes = exp_ms.to_le_bytes();
                writer.write_all(&exp_bytes)?;
                crc.update(&exp_bytes);
                bytes_written += 1 + 8;
            } else {
                writer.write_all(&[0u8])?;
                crc.update(&[0u8]);
                bytes_written += 1;
            }
        }

        // 3. Write Footer Magic (8B)
        writer.write_all(SNAPSHOT_FOOTER_MAGIC)?;
        crc.update(SNAPSHOT_FOOTER_MAGIC);
        bytes_written += SNAPSHOT_FOOTER_MAGIC.len() as u64;

        // 4. Write Finalized CRC64 Checksum (8B)
        let checksum = crc.finish();
        let crc_bytes = checksum.to_le_bytes();
        writer.write_all(&crc_bytes)?;
        bytes_written += 8;

        // Flush and fsync
        writer.flush()?;
        let f = writer.into_inner().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        f.sync_all()?;

        // Atomic Rename to target destination
        fs::rename(&tmp_path, target_path)?;

        Ok(SnapshotFileMeta {
            file_path: target_path.to_path_buf(),
            target_epoch,
            item_count: items.len() as u64,
            total_file_bytes: bytes_written,
            checksum,
        })
    }
}

/// Snapshot reader and verification engine.
pub struct SnapshotReader;

impl SnapshotReader {
    /// Reads and parses an on-disk snapshot file, validating header and CRC64 checksum.
    pub fn read_snapshot(path: &Path) -> Result<(SnapshotHeader, Vec<SnapshotItem>), PersistenceError> {
        let file = File::open(path)?;
        let mut reader = BufReader::with_capacity(64 * 1024, file);
        let mut crc = Crc64::new();

        // 1. Validate Header Magic (16B)
        let mut magic = [0u8; 16];
        reader.read_exact(&mut magic)?;
        if &magic != SNAPSHOT_MAGIC {
            return Err(PersistenceError::InvalidMagic);
        }
        crc.update(&magic);

        // Version (4B)
        let mut ver_bytes = [0u8; 4];
        reader.read_exact(&mut ver_bytes)?;
        crc.update(&ver_bytes);
        let version = u32::from_le_bytes(ver_bytes);
        if version != FORMAT_VERSION {
            return Err(PersistenceError::UnsupportedVersion(version));
        }

        // Flags (4B)
        let mut flags_bytes = [0u8; 4];
        reader.read_exact(&mut flags_bytes)?;
        crc.update(&flags_bytes);
        let flags = u32::from_le_bytes(flags_bytes);

        // Target Epoch (8B)
        let mut epoch_bytes = [0u8; 8];
        reader.read_exact(&mut epoch_bytes)?;
        crc.update(&epoch_bytes);
        let target_epoch = u64::from_le_bytes(epoch_bytes);

        // Created At (8B)
        let mut time_bytes = [0u8; 8];
        reader.read_exact(&mut time_bytes)?;
        crc.update(&time_bytes);
        let created_at_ms = u64::from_le_bytes(time_bytes);

        // Item Count (8B)
        let mut count_bytes = [0u8; 8];
        reader.read_exact(&mut count_bytes)?;
        crc.update(&count_bytes);
        let item_count = u64::from_le_bytes(count_bytes);

        let header = SnapshotHeader {
            version,
            flags,
            target_epoch,
            created_at_ms,
            item_count,
        };

        // 2. Read Entries
        let mut items = Vec::with_capacity(item_count as usize);
        for _ in 0..item_count {
            // Key
            let mut klen_bytes = [0u8; 4];
            reader.read_exact(&mut klen_bytes)?;
            crc.update(&klen_bytes);
            let klen = u32::from_le_bytes(klen_bytes) as usize;
            let mut key = vec![0u8; klen];
            reader.read_exact(&mut key)?;
            crc.update(&key);

            // Value
            let mut vlen_bytes = [0u8; 4];
            reader.read_exact(&mut vlen_bytes)?;
            crc.update(&vlen_bytes);
            let vlen = u32::from_le_bytes(vlen_bytes) as usize;
            let mut value = vec![0u8; vlen];
            reader.read_exact(&mut value)?;
            crc.update(&value);

            // Epoch
            let mut ep_bytes = [0u8; 8];
            reader.read_exact(&mut ep_bytes)?;
            crc.update(&ep_bytes);
            let epoch = u64::from_le_bytes(ep_bytes);

            // Expiry
            let mut has_exp = [0u8; 1];
            reader.read_exact(&mut has_exp)?;
            crc.update(&has_exp);

            let expire_at_ms = if has_exp[0] == 1 {
                let mut exp_bytes = [0u8; 8];
                reader.read_exact(&mut exp_bytes)?;
                crc.update(&exp_bytes);
                Some(u64::from_le_bytes(exp_bytes))
            } else {
                None
            };

            items.push(SnapshotItem {
                key,
                value,
                epoch,
                expire_at_ms,
            });
        }

        // 3. Validate Footer Magic (8B)
        let mut footer = [0u8; 8];
        reader.read_exact(&mut footer)?;
        crc.update(&footer);
        if &footer != SNAPSHOT_FOOTER_MAGIC {
            return Err(PersistenceError::InvalidFooter);
        }

        // 4. Validate CRC64 Checksum (8B)
        let mut stored_crc_bytes = [0u8; 8];
        reader.read_exact(&mut stored_crc_bytes)?;
        let stored_crc = u64::from_le_bytes(stored_crc_bytes);

        let calculated_crc = crc.finish();
        if stored_crc != calculated_crc {
            return Err(PersistenceError::ChecksumMismatch {
                expected: stored_crc,
                actual: calculated_crc,
            });
        }

        Ok((header, items))
    }

    /// Restores a snapshot file directly into an active storage map.
    pub fn restore_into_map(path: &Path, map: &mut RobinHoodMap) -> Result<SnapshotHeader, PersistenceError> {
        let (header, items) = Self::read_snapshot(path)?;

        for item in items {
            map.set_epoch(item.epoch);
            map.insert(&item.key, &item.value);
        }

        map.set_epoch(header.target_epoch);
        Ok(header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_crc64_deterministic() {
        let mut crc1 = Crc64::new();
        crc1.update(b"123456789");

        let mut crc2 = Crc64::new();
        crc2.update(b"123456789");

        assert_eq!(crc1.finish(), crc2.finish());
        assert_ne!(crc1.finish(), 0);
    }

    #[test]
    fn test_write_and_read_snapshot_roundtrip() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let items = vec![
            SnapshotItem::new(b"auth:session_991", b"jwt_payload_data", 10, Some(1730000000000)),
            SnapshotItem::new(b"ratelimit:ip_10.0.0.1", b"42", 12, Some(1730000060000)),
            SnapshotItem::new(b"cache:db_user_1", b"{\"name\":\"alice\"}", 8, None),
        ];

        let meta = SnapshotWriter::write_snapshot(&path, 15, &items).unwrap();
        assert_eq!(meta.item_count, 3);
        assert_eq!(meta.target_epoch, 15);
        assert!(meta.total_file_bytes > 0);

        let (header, loaded_items) = SnapshotReader::read_snapshot(&path).unwrap();
        assert_eq!(header.target_epoch, 15);
        assert_eq!(header.item_count, 3);
        assert_eq!(header.version, FORMAT_VERSION);
        assert_eq!(loaded_items, items);
    }

    #[test]
    fn test_corrupt_snapshot_checksum_failure() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let items = vec![
            SnapshotItem::new(b"key1", b"val1", 1, None),
            SnapshotItem::new(b"key2", b"val2", 2, None),
        ];

        SnapshotWriter::write_snapshot(&path, 5, &items).unwrap();

        // Mutate the CRC64 checksum byte at the end of the file
        let mut data = fs::read(&path).unwrap();
        let last_idx = data.len() - 1;
        data[last_idx] ^= 0xFF; // Flip bits in trailing CRC checksum
        fs::write(&path, &data).unwrap();

        // Reading must fail with ChecksumMismatch
        let res = SnapshotReader::read_snapshot(&path);
        match res {
            Err(PersistenceError::ChecksumMismatch { .. }) => {}
            other => panic!("Expected ChecksumMismatch error, got: {:?}", other),
        }
    }

    #[test]
    fn test_restore_into_robin_hood_map() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let items = vec![
            SnapshotItem::new(b"user:001", b"data1", 3, None),
            SnapshotItem::new(b"user:002", b"data2", 4, None),
            SnapshotItem::new(b"user:003", b"data3", 5, None),
        ];

        SnapshotWriter::write_snapshot(&path, 7, &items).unwrap();

        let mut map = RobinHoodMap::new(16);
        assert_eq!(map.len(), 0);

        let header = SnapshotReader::restore_into_map(&path, &mut map).unwrap();
        assert_eq!(header.target_epoch, 7);
        assert_eq!(map.len(), 3);
        assert_eq!(map.get(b"user:001"), Some(b"data1".as_slice()));
        assert_eq!(map.get(b"user:002"), Some(b"data2".as_slice()));
        assert_eq!(map.get(b"user:003"), Some(b"data3".as_slice()));
        assert_eq!(map.epoch(), 7);
    }
}
