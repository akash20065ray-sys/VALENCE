//! Valence Binary Write-Ahead Log (WAL) Subsystem
//!
//! Provides deterministic binary logging with monotonic Log Sequence Numbers (LSN),
//! 64MB fixed-boundary segment rotation, group commit sync policies, and
//! torn-write resilient crash recovery replay.

pub mod record;
pub mod recovery;
pub mod segment;
pub mod writer;

pub use record::{WalError, WalOp, WalRecord, WAL_RECORD_MAGIC};
pub use recovery::{WalRecovery, WalRecoveryReport};
pub use segment::{
    format_segment_filename, parse_segment_id, WalSegmentHeader, WalSegmentManager,
    DEFAULT_MAX_SEGMENT_SIZE, WAL_SEGMENT_MAGIC, WAL_SEGMENT_VERSION,
};
pub use writer::{SyncPolicy, WalWriter};
