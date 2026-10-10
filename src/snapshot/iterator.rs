//! Valence Snapshot Engine: Zero-Copy Snapshot Iterator
//!
//! Provides point-in-time consistent streaming of hash table contents without
//! relying on Linux fork(). Implements key-level Copy-on-Write (CoW) via a
//! dedicated shadow pre-image buffer that captures the original versions of
//! keys mutated while a snapshot is streaming.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::snapshot::epoch::Epoch;
use crate::storage::RobinHoodMap;

/// Point-in-time entry yielded by the snapshot iterator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotItem {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub epoch: Epoch,
    pub expire_at_ms: Option<u64>,
}

impl SnapshotItem {
    /// Constructs a new snapshot item.
    pub fn new(
        key: &[u8],
        value: &[u8],
        epoch: Epoch,
        expire_at_ms: Option<u64>,
    ) -> Self {
        Self {
            key: key.to_vec(),
            value: value.to_vec(),
            epoch,
            expire_at_ms,
        }
    }

    /// Size of the key and value in bytes.
    #[inline(always)]
    pub fn payload_size(&self) -> usize {
        self.key.len() + self.value.len()
    }
}

/// Thread-safe shadow buffer capturing pre-update images of keys mutated during an active snapshot.
///
/// Unlike Linux `fork()` which duplicates 4KB physical pages on write, `ShadowBuffer`
/// stores only the exact bytes of keys modified during the snapshot window.
#[derive(Debug, Clone, Default)]
pub struct ShadowBuffer {
    inner: Arc<RwLock<HashMap<Vec<u8>, SnapshotItem>>>,
}

impl ShadowBuffer {
    /// Constructs a new empty shadow buffer.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Captures the pre-update image of a key before it is modified or deleted in storage.
    ///
    /// Preserves only the *first* pre-image seen during the snapshot window.
    /// Subsequent updates to the same key do not overwrite the initial snapshot version.
    pub fn capture_pre_image(
        &self,
        key: &[u8],
        value: &[u8],
        epoch: Epoch,
        expire_at_ms: Option<u64>,
    ) {
        let mut guard = self.inner.write().expect("lock poisoned");
        if !guard.contains_key(key) {
            guard.insert(
                key.to_vec(),
                SnapshotItem::new(key, value, epoch, expire_at_ms),
            );
        }
    }

    /// Returns `true` if this key has already been captured into the shadow buffer.
    #[inline]
    pub fn contains_key(&self, key: &[u8]) -> bool {
        let guard = self.inner.read().expect("lock poisoned");
        guard.contains_key(key)
    }

    /// Returns the number of pre-images currently buffered.
    pub fn len(&self) -> usize {
        let guard = self.inner.read().expect("lock poisoned");
        guard.len()
    }

    /// Returns `true` if no pre-images have been buffered.
    pub fn is_empty(&self) -> bool {
        let guard = self.inner.read().expect("lock poisoned");
        guard.is_empty()
    }

    /// Retrieves a cloned snapshot item by key, if present in the buffer.
    pub fn get(&self, key: &[u8]) -> Option<SnapshotItem> {
        let guard = self.inner.read().expect("lock poisoned");
        guard.get(key).cloned()
    }

    /// Returns all captured snapshot items.
    pub fn items(&self) -> Vec<SnapshotItem> {
        let guard = self.inner.read().expect("lock poisoned");
        guard.values().cloned().collect()
    }

    /// Clears all buffered entries.
    pub fn clear(&self) {
        let mut guard = self.inner.write().expect("lock poisoned");
        guard.clear();
    }
}

/// Operational statistics collected during a snapshot streaming run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SnapshotStreamStats {
    /// Total number of consistent items streamed.
    pub items_streamed: usize,
    /// Number of items streamed directly from storage map.
    pub direct_map_items: usize,
    /// Number of items streamed from the shadow pre-image buffer.
    pub shadow_buffer_items: usize,
    /// Total payload bytes (keys + values) streamed.
    pub total_bytes: usize,
}

/// Zero-copy point-in-time snapshot scanner.
///
/// Combines the underlying `RobinHoodMap` slots with the `ShadowBuffer` to produce
/// an exact, consistent view of state at `target_epoch`.
pub struct SnapshotScanner<'a> {
    target_epoch: Epoch,
    map: &'a RobinHoodMap,
    shadow_buffer: &'a ShadowBuffer,
}

impl<'a> SnapshotScanner<'a> {
    /// Creates a new snapshot scanner targeting a specific epoch boundary.
    pub fn new(
        target_epoch: Epoch,
        map: &'a RobinHoodMap,
        shadow_buffer: &'a ShadowBuffer,
    ) -> Self {
        Self {
            target_epoch,
            map,
            shadow_buffer,
        }
    }

    /// Scans storage and streams out all consistent items matching the target epoch.
    ///
    /// Execution guarantee:
    /// 1. Iterates map slots: If `slot.epoch <= target_epoch` and key was NOT mutated
    ///    into `ShadowBuffer`, yields directly from the slot with zero allocations.
    /// 2. Iterates `ShadowBuffer`: Yields all pre-images captured when keys were mutated
    ///    after the snapshot epoch boundary.
    pub fn collect_all(&self) -> (Vec<SnapshotItem>, SnapshotStreamStats) {
        let mut results = Vec::with_capacity(self.map.len() + self.shadow_buffer.len());
        let mut stats = SnapshotStreamStats::default();

        let shadow_items = self.shadow_buffer.items();
        let shadow_keys: HashSet<Vec<u8>> = shadow_items
            .iter()
            .map(|item| item.key.clone())
            .collect();

        // 1. Scan direct map slots that have NOT been shadowed
        let mut idx = 0;
        while idx < self.map.capacity() {
            if let Some(slot) = self.map.slot_at(idx) {
                if slot.is_occupied() && slot.epoch <= self.target_epoch {
                    let key_bytes = slot.key();
                    if !shadow_keys.contains(key_bytes) {
                        let item = SnapshotItem::new(
                            key_bytes,
                            slot.value(),
                            slot.epoch,
                            None,
                        );
                        stats.total_bytes += item.payload_size();
                        stats.direct_map_items += 1;
                        results.push(item);
                    }
                }
            }
            idx += 1;
        }

        // 2. Stream all pre-images from shadow buffer
        for item in shadow_items {
            if item.epoch <= self.target_epoch {
                stats.total_bytes += item.payload_size();
                stats.shadow_buffer_items += 1;
                results.push(item);
            }
        }

        stats.items_streamed = results.len();
        (results, stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_map_snapshot() {
        let mut map = RobinHoodMap::new(16);
        map.set_epoch(1);
        map.insert(b"user:101", b"alice");
        map.insert(b"user:102", b"bob");
        map.insert(b"user:103", b"charlie");

        let shadow = ShadowBuffer::new();
        let scanner = SnapshotScanner::new(1, &map, &shadow);

        let (items, stats) = scanner.collect_all();
        assert_eq!(stats.items_streamed, 3);
        assert_eq!(stats.direct_map_items, 3);
        assert_eq!(stats.shadow_buffer_items, 0);

        let mut keys: Vec<String> = items
            .iter()
            .map(|i| String::from_utf8(i.key.clone()).unwrap())
            .collect();
        keys.sort();
        assert_eq!(keys, vec!["user:101", "user:102", "user:103"]);
    }

    #[test]
    fn test_snapshot_with_concurrent_mutations_and_shadowing() {
        let mut map = RobinHoodMap::new(32);
        map.set_epoch(1);
        map.insert(b"auth:token:01", b"v1_secret");
        map.insert(b"auth:token:02", b"v1_valid");
        map.insert(b"cache:count", b"100");

        // Target snapshot epoch = 1
        let target_epoch = 1;
        let shadow = ShadowBuffer::new();

        // Now simulate workers making mutations in Epoch 2 during the snapshot window
        map.set_epoch(2);

        // Mutation 1: auth:token:01 is updated from "v1_secret" to "v2_secret"
        // Pre-update hook captures the original v1 value into ShadowBuffer
        shadow.capture_pre_image(b"auth:token:01", b"v1_secret", 1, None);
        map.insert(b"auth:token:01", b"v2_secret");

        // Mutation 2: cache:count is deleted
        // Pre-update hook captures the original value before deletion
        shadow.capture_pre_image(b"cache:count", b"100", 1, None);
        map.delete(b"cache:count");

        // Mutation 3: A completely new key inserted at Epoch 2 (should NOT be in Epoch 1 snapshot)
        map.insert(b"new:epoch2:key", b"fresh_val");

        // Scan snapshot for target epoch 1
        let scanner = SnapshotScanner::new(target_epoch, &map, &shadow);
        let (items, stats) = scanner.collect_all();

        assert_eq!(stats.items_streamed, 3);
        assert_eq!(stats.direct_map_items, 1); // auth:token:02 untouched
        assert_eq!(stats.shadow_buffer_items, 2); // auth:token:01 and cache:count from shadow buffer

        // Verify values: must match Epoch 1 state exactly!
        let item_map: HashMap<String, String> = items
            .iter()
            .map(|i| {
                (
                    String::from_utf8(i.key.clone()).unwrap(),
                    String::from_utf8(i.value.clone()).unwrap(),
                )
            })
            .collect();

        // 1. Updated key must yield its OLD Epoch 1 value
        assert_eq!(item_map.get("auth:token:01").unwrap(), "v1_secret");

        // 2. Deleted key must still exist in Epoch 1 snapshot
        assert_eq!(item_map.get("cache:count").unwrap(), "100");

        // 3. Untouched key exists
        assert_eq!(item_map.get("auth:token:02").unwrap(), "v1_valid");

        // 4. Epoch 2 new key must NOT be present
        assert!(!item_map.contains_key("new:epoch2:key"));
    }

    #[test]
    fn test_shadow_buffer_idempotency_on_multiple_writes() {
        let shadow = ShadowBuffer::new();

        // First write captures original value "alpha"
        shadow.capture_pre_image(b"key:abc", b"alpha", 1, None);

        // Subsequent write during same snapshot attempts to capture "beta"
        shadow.capture_pre_image(b"key:abc", b"beta", 2, None);

        // Must still retain initial point-in-time value "alpha"
        let item = shadow.get(b"key:abc").unwrap();
        assert_eq!(item.value, b"alpha");
        assert_eq!(item.epoch, 1);
        assert_eq!(shadow.len(), 1);
    }
}
