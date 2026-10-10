//! Valence Binary Write-Ahead Log: Record Framing & Checksums
//!
//! Compact binary framing with monotonic Log Sequence Numbers (LSN),
//! epoch fencing, and streaming ECMA-182 CRC64 checksum validation.

use std::fmt;
use std::io;
use crate::snapshot::persistence::Crc64;

/// 2-byte magic prefix marking the start of every binary WAL record ("VL").
pub const WAL_RECORD_MAGIC: [u8; 2] = [0x56, 0x4C];

pub const OP_CODE_SET: u8 = 1;
pub const OP_CODE_DEL: u8 = 2;
pub const OP_CODE_EXPIRE: u8 = 3;
pub const OP_CODE_CHECKPOINT: u8 = 4;

/// Errors occurring during WAL serialization, deserialization, or verification.
#[derive(Debug)]
pub enum WalError {
    Io(io::Error),
    TruncatedRecord,
    InvalidMagic([u8; 2]),
    CorruptChecksum { expected: u64, actual: u64 },
    UnsupportedOp(u8),
    CorruptHeader(String),
    SegmentFull,
}

impl fmt::Display for WalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "WAL I/O error: {}", e),
            Self::TruncatedRecord => write!(f, "Torn or truncated WAL record at tail"),
            Self::InvalidMagic(m) => write!(f, "Invalid record magic: [0x{:02X}, 0x{:02X}]", m[0], m[1]),
            Self::CorruptChecksum { expected, actual } => write!(
                f,
                "WAL CRC64 mismatch: expected 0x{:016X}, computed 0x{:016X}",
                expected, actual
            ),
            Self::UnsupportedOp(op) => write!(f, "Unsupported WAL operation code: {}", op),
            Self::CorruptHeader(msg) => write!(f, "Corrupted WAL header: {}", msg),
            Self::SegmentFull => write!(f, "Current WAL segment has reached maximum capacity"),
        }
    }
}

impl std::error::Error for WalError {}

impl From<io::Error> for WalError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// Mutating operation encoded in a WAL record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalOp {
    Set {
        key: Vec<u8>,
        value: Vec<u8>,
        ttl_ms: Option<u64>,
    },
    Del {
        key: Vec<u8>,
    },
    Expire {
        key: Vec<u8>,
        ttl_ms: u64,
    },
    Checkpoint {
        target_epoch: u64,
    },
}

impl WalOp {
    #[inline(always)]
    pub fn op_code(&self) -> u8 {
        match self {
            Self::Set { .. } => OP_CODE_SET,
            Self::Del { .. } => OP_CODE_DEL,
            Self::Expire { .. } => OP_CODE_EXPIRE,
            Self::Checkpoint { .. } => OP_CODE_CHECKPOINT,
        }
    }
}

/// Complete write-ahead log record with monotonic sequencing, timestamp, and epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalRecord {
    pub lsn: u64,
    pub epoch: u64,
    pub timestamp_ms: u64,
    pub op: WalOp,
}

impl WalRecord {
    /// Constructs a new WAL record.
    pub fn new(lsn: u64, epoch: u64, timestamp_ms: u64, op: WalOp) -> Self {
        Self {
            lsn,
            epoch,
            timestamp_ms,
            op,
        }
    }

    /// Serializes the record into an on-wire byte vector with trailing CRC64 checksum.
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64);

        // 1. Magic (2B)
        buf.extend_from_slice(&WAL_RECORD_MAGIC);

        // 2. Op Code (1B)
        buf.push(self.op.op_code());

        // 3. LSN (8B)
        buf.extend_from_slice(&self.lsn.to_le_bytes());

        // 4. Epoch (8B)
        buf.extend_from_slice(&self.epoch.to_le_bytes());

        // 5. Timestamp (8B)
        buf.extend_from_slice(&self.timestamp_ms.to_le_bytes());

        // 6. Payload by Op Code
        match &self.op {
            WalOp::Set { key, value, ttl_ms } => {
                let ttl_val = ttl_ms.unwrap_or(0);
                buf.extend_from_slice(&ttl_val.to_le_bytes());
                buf.push(if ttl_ms.is_some() { 1 } else { 0 });
                buf.extend_from_slice(&(key.len() as u32).to_le_bytes());
                buf.extend_from_slice(&(value.len() as u32).to_le_bytes());
                buf.extend_from_slice(key);
                buf.extend_from_slice(value);
            }
            WalOp::Del { key } => {
                buf.extend_from_slice(&(key.len() as u32).to_le_bytes());
                buf.extend_from_slice(key);
            }
            WalOp::Expire { key, ttl_ms } => {
                buf.extend_from_slice(&ttl_ms.to_le_bytes());
                buf.extend_from_slice(&(key.len() as u32).to_le_bytes());
                buf.extend_from_slice(key);
            }
            WalOp::Checkpoint { target_epoch } => {
                buf.extend_from_slice(&target_epoch.to_le_bytes());
            }
        }

        // 7. Calculate and append ECMA-182 CRC64 checksum over all preceding bytes
        let mut crc = Crc64::new();
        crc.update(&buf);
        let checksum = crc.finish();
        buf.extend_from_slice(&checksum.to_le_bytes());

        buf
    }

    /// Attempts to decode a single WAL record from a byte buffer.
    ///
    /// Returns `Ok((record, consumed_bytes))` on success.
    /// Returns `Err(WalError::TruncatedRecord)` if buffer is incomplete (torn write boundary).
    /// Returns `Err(WalError::CorruptChecksum)` if CRC64 validation fails.
    pub fn decode(bytes: &[u8]) -> Result<(Self, usize), WalError> {
        // Minimum fixed size: Magic (2) + Op (1) + LSN (8) + Epoch (8) + Time (8) + CRC (8) = 27B
        if bytes.len() < 27 {
            return Err(WalError::TruncatedRecord);
        }

        if bytes[0..2] != WAL_RECORD_MAGIC {
            return Err(WalError::InvalidMagic([bytes[0], bytes[1]]));
        }

        let op_code = bytes[2];
        let lsn = u64::from_le_bytes(bytes[3..11].try_into().unwrap());
        let epoch = u64::from_le_bytes(bytes[11..19].try_into().unwrap());
        let timestamp_ms = u64::from_le_bytes(bytes[19..27].try_into().unwrap());

        let mut offset = 27;

        let op = match op_code {
            OP_CODE_SET => {
                // ttl (8B) + has_ttl (1B) + klen (4B) + vlen (4B) = 17B
                if bytes.len() < offset + 17 {
                    return Err(WalError::TruncatedRecord);
                }
                let ttl_val = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
                offset += 8;
                let has_ttl = bytes[offset] != 0;
                offset += 1;
                let klen = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;
                let vlen = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;

                if bytes.len() < offset + klen + vlen + 8 {
                    return Err(WalError::TruncatedRecord);
                }
                let key = bytes[offset..offset + klen].to_vec();
                offset += klen;
                let value = bytes[offset..offset + vlen].to_vec();
                offset += vlen;

                let ttl_ms = if has_ttl { Some(ttl_val) } else { None };
                WalOp::Set { key, value, ttl_ms }
            }
            OP_CODE_DEL => {
                // klen (4B)
                if bytes.len() < offset + 4 {
                    return Err(WalError::TruncatedRecord);
                }
                let klen = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;

                if bytes.len() < offset + klen + 8 {
                    return Err(WalError::TruncatedRecord);
                }
                let key = bytes[offset..offset + klen].to_vec();
                offset += klen;

                WalOp::Del { key }
            }
            OP_CODE_EXPIRE => {
                // ttl (8B) + klen (4B) = 12B
                if bytes.len() < offset + 12 {
                    return Err(WalError::TruncatedRecord);
                }
                let ttl_ms = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
                offset += 8;
                let klen = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;

                if bytes.len() < offset + klen + 8 {
                    return Err(WalError::TruncatedRecord);
                }
                let key = bytes[offset..offset + klen].to_vec();
                offset += klen;

                WalOp::Expire { key, ttl_ms }
            }
            OP_CODE_CHECKPOINT => {
                // target_epoch (8B)
                if bytes.len() < offset + 8 + 8 {
                    return Err(WalError::TruncatedRecord);
                }
                let target_epoch = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
                offset += 8;

                WalOp::Checkpoint { target_epoch }
            }
            unsupported => return Err(WalError::UnsupportedOp(unsupported)),
        };

        // Read stored CRC64 (8B)
        if bytes.len() < offset + 8 {
            return Err(WalError::TruncatedRecord);
        }
        let stored_crc = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());

        // Validate computed CRC64
        let mut crc = Crc64::new();
        crc.update(&bytes[..offset]);
        let computed_crc = crc.finish();

        if computed_crc != stored_crc {
            return Err(WalError::CorruptChecksum {
                expected: stored_crc,
                actual: computed_crc,
            });
        }

        let total_consumed = offset + 8;
        Ok((
            Self {
                lsn,
                epoch,
                timestamp_ms,
                op,
            },
            total_consumed,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_encode_decode_roundtrip_set() {
        let op = WalOp::Set {
            key: b"user:session:100".to_vec(),
            value: b"auth_token_xyz_987".to_vec(),
            ttl_ms: Some(3600_000),
        };
        let record = WalRecord::new(101, 12, 1700000000, op);
        let encoded = record.encode();

        let (decoded, consumed) = WalRecord::decode(&encoded).expect("Decode failed");
        assert_eq!(consumed, encoded.len());
        assert_eq!(decoded, record);
    }

    #[test]
    fn test_record_encode_decode_roundtrip_del_and_checkpoint() {
        let del_record = WalRecord::new(102, 12, 1700000001, WalOp::Del { key: b"temp_key".to_vec() });
        let encoded_del = del_record.encode();
        let (decoded_del, _) = WalRecord::decode(&encoded_del).unwrap();
        assert_eq!(decoded_del, del_record);

        let chk_record = WalRecord::new(103, 13, 1700000002, WalOp::Checkpoint { target_epoch: 12 });
        let encoded_chk = chk_record.encode();
        let (decoded_chk, _) = WalRecord::decode(&encoded_chk).unwrap();
        assert_eq!(decoded_chk, chk_record);
    }

    #[test]
    fn test_corrupt_checksum_detection() {
        let record = WalRecord::new(1, 1, 100, WalOp::Del { key: b"k".to_vec() });
        let mut encoded = record.encode();

        // Flip a byte in the checksum
        let len = encoded.len();
        encoded[len - 1] ^= 0xFF;

        match WalRecord::decode(&encoded) {
            Err(WalError::CorruptChecksum { .. }) => {}
            other => panic!("Expected CorruptChecksum, got: {:?}", other),
        }
    }

    #[test]
    fn test_truncated_record_detection() {
        let record = WalRecord::new(1, 1, 100, WalOp::Set {
            key: b"long_key_name".to_vec(),
            value: b"long_value_data".to_vec(),
            ttl_ms: None,
        });
        let encoded = record.encode();

        // Pass truncated slice
        for cut in 1..encoded.len() {
            assert!(matches!(
                WalRecord::decode(&encoded[..cut]),
                Err(WalError::TruncatedRecord)
            ));
        }
    }
}
