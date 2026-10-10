//! Valence Snapshot Engine: Mutation Epochs & Snapshot Registry
//!
//! Provides lock-free monotonic epoch coordination and point-in-time snapshot
//! session management. Eliminates Redis's fork-based memory duplication by
//! tagging every slot with a 64-bit monotonic mutation epoch.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Monotonic mutation epoch identifier.
pub type Epoch = u64;

/// Unique snapshot job identifier.
pub type SnapshotId = u64;

/// Sentinel epoch representing an unversioned or empty entry.
pub const EPOCH_ZERO: Epoch = 0;

/// Default initial epoch when the engine boots.
pub const EPOCH_INIT: Epoch = 1;

/// Sentinel value indicating no snapshots are currently active.
const NO_ACTIVE_SNAPSHOT_EPOCH: u64 = u64::MAX;

/// Lifecycle state of a registered snapshot session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotStatus {
    /// Snapshot is currently streaming out data.
    Active,
    /// Snapshot successfully persisted to disk.
    Completed,
    /// Snapshot was aborted or canceled.
    Aborted,
}

/// Metadata record for a point-in-time snapshot job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotMetadata {
    pub id: SnapshotId,
    pub target_epoch: Epoch,
    pub created_at_ms: u64,
    pub status: SnapshotStatus,
}

/// Action determined by the registry prior to overwriting or deleting an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreUpdateAction {
    /// No active snapshot requires this version; perform direct in-place update.
    DirectOverwrite,
    /// An active snapshot needs this point-in-time version; preserve pre-image.
    PreservePreImage { target_epoch: Epoch },
}

/// Errors returned by the snapshot registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    NotFound(SnapshotId),
    AlreadyCompleted(SnapshotId),
    AlreadyAborted(SnapshotId),
    InvalidEpoch(Epoch),
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "Snapshot #{} not found in registry", id),
            Self::AlreadyCompleted(id) => write!(f, "Snapshot #{} is already completed", id),
            Self::AlreadyAborted(id) => write!(f, "Snapshot #{} is already aborted", id),
            Self::InvalidEpoch(ep) => write!(f, "Invalid snapshot epoch: {}", ep),
        }
    }
}

impl std::error::Error for SnapshotError {}

/// Lock-free monotonic mutation epoch tracker.
///
/// Incremented atomically on state mutations (SET, DEL, etc.).
pub struct EpochTracker {
    current_epoch: AtomicU64,
}

impl EpochTracker {
    /// Constructs a new tracker initialized to `EPOCH_INIT` (1).
    pub fn new() -> Self {
        Self {
            current_epoch: AtomicU64::new(EPOCH_INIT),
        }
    }

    /// Constructs a tracker initialized to a specific starting epoch (e.g. from WAL recovery).
    pub fn with_initial_epoch(initial: Epoch) -> Self {
        Self {
            current_epoch: AtomicU64::new(initial),
        }
    }

    /// Atomically retrieves the current mutation epoch.
    #[inline(always)]
    pub fn current(&self) -> Epoch {
        self.current_epoch.load(Ordering::Acquire)
    }

    /// Atomically increments and returns the new mutation epoch.
    #[inline(always)]
    pub fn advance(&self) -> Epoch {
        self.current_epoch.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Atomically increments the epoch by a batch count.
    #[inline(always)]
    pub fn advance_by(&self, delta: u64) -> Epoch {
        self.current_epoch.fetch_add(delta, Ordering::SeqCst) + delta
    }
}

impl Default for EpochTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Central registry managing point-in-time snapshot sessions.
///
/// Employs a cached atomic `cached_min_active_epoch` to allow worker threads to perform
/// pre-update version checks with zero lock contention on the fast path.
pub struct SnapshotRegistry {
    next_id: AtomicU64,
    cached_min_active_epoch: AtomicU64,
    sessions: RwLock<HashMap<SnapshotId, SnapshotMetadata>>,
}

impl SnapshotRegistry {
    /// Creates a new, empty snapshot registry.
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            cached_min_active_epoch: AtomicU64::new(NO_ACTIVE_SNAPSHOT_EPOCH),
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a new snapshot session at the given target epoch.
    pub fn register(&self, target_epoch: Epoch) -> SnapshotMetadata {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let meta = SnapshotMetadata {
            id,
            target_epoch,
            created_at_ms: now_ms,
            status: SnapshotStatus::Active,
        };

        {
            let mut guard = self.sessions.write().expect("lock poisoned");
            guard.insert(id, meta.clone());
        }

        self.recompute_cached_min_epoch();
        meta
    }

    /// Marks an active snapshot session as successfully completed.
    pub fn complete(&self, id: SnapshotId) -> Result<SnapshotMetadata, SnapshotError> {
        let meta = {
            let mut guard = self.sessions.write().expect("lock poisoned");
            let entry = guard.get_mut(&id).ok_or(SnapshotError::NotFound(id))?;
            match entry.status {
                SnapshotStatus::Completed => return Err(SnapshotError::AlreadyCompleted(id)),
                SnapshotStatus::Aborted => return Err(SnapshotError::AlreadyAborted(id)),
                SnapshotStatus::Active => {
                    entry.status = SnapshotStatus::Completed;
                    entry.clone()
                }
            }
        };

        self.recompute_cached_min_epoch();
        Ok(meta)
    }

    /// Marks an active snapshot session as aborted.
    pub fn abort(&self, id: SnapshotId) -> Result<SnapshotMetadata, SnapshotError> {
        let meta = {
            let mut guard = self.sessions.write().expect("lock poisoned");
            let entry = guard.get_mut(&id).ok_or(SnapshotError::NotFound(id))?;
            match entry.status {
                SnapshotStatus::Completed => return Err(SnapshotError::AlreadyCompleted(id)),
                SnapshotStatus::Aborted => return Err(SnapshotError::AlreadyAborted(id)),
                SnapshotStatus::Active => {
                    entry.status = SnapshotStatus::Aborted;
                    entry.clone()
                }
            }
        };

        self.recompute_cached_min_epoch();
        Ok(meta)
    }

    /// Returns the minimum target epoch across all currently active snapshots.
    ///
    /// Fast lock-free read path via cached atomic.
    #[inline(always)]
    pub fn min_active_epoch(&self) -> Option<Epoch> {
        let val = self.cached_min_active_epoch.load(Ordering::Acquire);
        if val == NO_ACTIVE_SNAPSHOT_EPOCH {
            None
        } else {
            Some(val)
        }
    }

    /// Returns `true` if one or more snapshot sessions are currently active.
    #[inline(always)]
    pub fn has_active_snapshots(&self) -> bool {
        self.cached_min_active_epoch.load(Ordering::Acquire) != NO_ACTIVE_SNAPSHOT_EPOCH
    }

    /// Fast-path pre-update hook: checks if an item with `item_epoch` needs its
    /// pre-image preserved before in-place modification.
    ///
    /// If `item_epoch <= min_active_epoch`, the item was committed before the snapshot
    /// boundary and must be preserved if modified.
    #[inline(always)]
    pub fn inspect_pre_update(&self, item_epoch: Epoch) -> PreUpdateAction {
        let min_epoch = self.cached_min_active_epoch.load(Ordering::Acquire);
        if min_epoch != NO_ACTIVE_SNAPSHOT_EPOCH && item_epoch <= min_epoch && item_epoch > EPOCH_ZERO {
            PreUpdateAction::PreservePreImage {
                target_epoch: min_epoch,
            }
        } else {
            PreUpdateAction::DirectOverwrite
        }
    }

    /// Returns the total number of currently active snapshot jobs.
    pub fn active_snapshot_count(&self) -> usize {
        let guard = self.sessions.read().expect("lock poisoned");
        guard
            .values()
            .filter(|s| s.status == SnapshotStatus::Active)
            .count()
    }

    /// Retrieves metadata for a specific snapshot by ID.
    pub fn get_snapshot(&self, id: SnapshotId) -> Option<SnapshotMetadata> {
        let guard = self.sessions.read().expect("lock poisoned");
        guard.get(&id).cloned()
    }

    /// Returns a list of all active snapshots.
    pub fn active_snapshots(&self) -> Vec<SnapshotMetadata> {
        let guard = self.sessions.read().expect("lock poisoned");
        guard
            .values()
            .filter(|s| s.status == SnapshotStatus::Active)
            .cloned()
            .collect()
    }

    /// Internal helper to recompute the cached minimum active epoch.
    fn recompute_cached_min_epoch(&self) {
        let guard = self.sessions.read().expect("lock poisoned");
        let min = guard
            .values()
            .filter(|s| s.status == SnapshotStatus::Active)
            .map(|s| s.target_epoch)
            .min();

        match min {
            Some(m) => self.cached_min_active_epoch.store(m, Ordering::Release),
            None => self
                .cached_min_active_epoch
                .store(NO_ACTIVE_SNAPSHOT_EPOCH, Ordering::Release),
        }
    }
}

impl Default for SnapshotRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_epoch_tracker_advances_monotonically() {
        let tracker = EpochTracker::new();
        assert_eq!(tracker.current(), 1);

        assert_eq!(tracker.advance(), 2);
        assert_eq!(tracker.current(), 2);

        assert_eq!(tracker.advance(), 3);
        assert_eq!(tracker.current(), 3);

        assert_eq!(tracker.advance_by(5), 8);
        assert_eq!(tracker.current(), 8);
    }

    #[test]
    fn test_snapshot_registry_lifecycle() {
        let registry = SnapshotRegistry::new();
        assert!(!registry.has_active_snapshots());
        assert_eq!(registry.min_active_epoch(), None);

        // Register first snapshot at epoch 10
        let snap1 = registry.register(10);
        assert_eq!(snap1.id, 1);
        assert_eq!(snap1.target_epoch, 10);
        assert_eq!(snap1.status, SnapshotStatus::Active);
        assert!(registry.has_active_snapshots());
        assert_eq!(registry.min_active_epoch(), Some(10));
        assert_eq!(registry.active_snapshot_count(), 1);

        // Register second snapshot at epoch 25
        let snap2 = registry.register(25);
        assert_eq!(snap2.id, 2);
        assert_eq!(registry.min_active_epoch(), Some(10)); // Min is still 10
        assert_eq!(registry.active_snapshot_count(), 2);

        // Complete first snapshot -> Min active epoch advances to 25
        let comp1 = registry.complete(snap1.id).unwrap();
        assert_eq!(comp1.status, SnapshotStatus::Completed);
        assert_eq!(registry.min_active_epoch(), Some(25));
        assert_eq!(registry.active_snapshot_count(), 1);

        // Abort second snapshot -> No active snapshots left
        let ab2 = registry.abort(snap2.id).unwrap();
        assert_eq!(ab2.status, SnapshotStatus::Aborted);
        assert!(!registry.has_active_snapshots());
        assert_eq!(registry.min_active_epoch(), None);
        assert_eq!(registry.active_snapshot_count(), 0);
    }

    #[test]
    fn test_pre_update_hook_decisions() {
        let registry = SnapshotRegistry::new();

        // When no snapshot is active: all overwrites are direct
        assert_eq!(
            registry.inspect_pre_update(5),
            PreUpdateAction::DirectOverwrite
        );

        // Register snapshot at epoch 20
        let _snap = registry.register(20);

        // Item committed at epoch 15 (before or equal to target epoch 20) MUST preserve pre-image
        assert_eq!(
            registry.inspect_pre_update(15),
            PreUpdateAction::PreservePreImage { target_epoch: 20 }
        );
        assert_eq!(
            registry.inspect_pre_update(20),
            PreUpdateAction::PreservePreImage { target_epoch: 20 }
        );

        // Item committed at epoch 25 (after snapshot target epoch 20) does NOT need preservation
        assert_eq!(
            registry.inspect_pre_update(25),
            PreUpdateAction::DirectOverwrite
        );

        // Item at epoch 0 (empty/uncommitted) does not need preservation
        assert_eq!(
            registry.inspect_pre_update(0),
            PreUpdateAction::DirectOverwrite
        );
    }

    #[test]
    fn test_concurrent_epoch_tracking() {
        let tracker = Arc::new(EpochTracker::new());
        let mut handles = Vec::new();

        for _ in 0..8 {
            let t = Arc::clone(&tracker);
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    t.advance();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // Initial epoch 1 + 8 * 1000 advances = 8001
        assert_eq!(tracker.current(), 8001);
    }
}
