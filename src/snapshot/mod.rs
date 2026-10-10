//! Valence Forkless Epoch-Versioned Snapshot Engine
//!
//! Replaces Linux fork()-based snapshots with lock-free monotonic epoch versioning,
//! zero-copy background iterators, and streaming binary persistence with CRC64 checksums.

pub mod epoch;

pub use epoch::{
    Epoch, EpochTracker, PreUpdateAction, SnapshotError, SnapshotId, SnapshotMetadata,
    SnapshotRegistry, SnapshotStatus, EPOCH_INIT, EPOCH_ZERO,
};
