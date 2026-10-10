//! Robin Hood Hash Table with Backshift Deletion & Contiguous Flat Arrays
//!
//! Features:
//! - Contiguous array of 64-byte aligned `Slot` entries (maximum cache locality).
//! - Robin Hood displacement ("steal from the rich, give to the poor") to minimize probe variance.
//! - O(1) Backshift Deletion: shifts collided elements backward, completely eliminating dead tombstones.
//! - Early Search Termination: lookups stop as soon as `slot.dib < search_dib`.
//! - Snapshot Epoch awareness for Point-in-Time consistency.

use super::entry::{CompactValue, Slot};
use crate::memory::slab_arena::SlabArena;
use twox_hash::XxHash64;
use std::hash::Hasher;

const DEFAULT_INITIAL_CAPACITY: usize = 16;
const MAX_LOAD_FACTOR: f64 = 0.85;

const VALENCE_SEED: u64 = 0x517CC1B727220A95;

/// Compute 64-bit hash for a byte slice using XxHash64.
#[inline(always)]
pub fn hash_key(key: &[u8]) -> u64 {
    let mut hasher = XxHash64::with_seed(VALENCE_SEED);
    hasher.write(key);
    hasher.finish()
}

/// High-Performance Robin Hood Hash Map with Integrated Slab Arena.
pub struct RobinHoodMap {
    slots: Box<[Slot]>,
    capacity: usize,
    mask: usize,
    count: usize,
    epoch: u64,
    slab_arena: Option<SlabArena>,
}

impl RobinHoodMap {
    /// Constructs a new Robin Hood Map with the specified initial capacity (rounded to power of 2).
    pub fn new(initial_capacity: usize) -> Self {
        let capacity = initial_capacity.max(DEFAULT_INITIAL_CAPACITY).next_power_of_two();
        let mask = capacity - 1;
        let mut slots = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            slots.push(Slot::empty());
        }

        Self {
            slots: slots.into_boxed_slice(),
            capacity,
            mask,
            count: 0,
            epoch: 1,
            slab_arena: None,
        }
    }

    /// Constructs a Robin Hood Map with an integrated 2MB hugepage slab arena.
    pub fn with_slab(initial_capacity: usize) -> Self {
        let mut map = Self::new(initial_capacity);
        map.slab_arena = Some(SlabArena::new());
        map
    }

    /// Constructs a Robin Hood Map with a slab arena of custom chunk size.
    pub fn with_slab_chunk(initial_capacity: usize, chunk_size: usize) -> Self {
        let mut map = Self::new(initial_capacity);
        map.slab_arena = Some(SlabArena::with_chunk_size(chunk_size));
        map
    }

    /// Enables slab arena on an existing map if not already present.
    pub fn enable_slab(&mut self) {
        if self.slab_arena.is_none() {
            self.slab_arena = Some(SlabArena::new());
        }
    }

    /// Returns true if this map has an active slab arena.
    pub fn has_slab(&self) -> bool {
        self.slab_arena.is_some()
    }

    /// Returns an immutable reference to the slab arena if enabled.
    pub fn slab_arena(&self) -> Option<&SlabArena> {
        self.slab_arena.as_ref()
    }

    /// Returns a mutable reference to the slab arena if enabled.
    pub fn slab_arena_mut(&mut self) -> Option<&mut SlabArena> {
        self.slab_arena.as_mut()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.count
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline(always)]
    pub fn load_factor(&self) -> f64 {
        self.count as f64 / self.capacity as f64
    }

    /// Sets the active mutation epoch for Snapshot Isolation tracking.
    pub fn set_epoch(&mut self, epoch: u64) {
        self.epoch = epoch;
    }

    /// Returns the current mutation epoch.
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Returns a reference to the raw slot at the specified index, if within bounds.
    #[inline(always)]
    pub fn slot_at(&self, idx: usize) -> Option<&Slot> {
        if idx < self.capacity {
            Some(&self.slots[idx])
        } else {
            None
        }
    }

    /// Retrieves a value slice by key.
    ///
    /// Exploits Robin Hood ordering for early termination:
    /// As soon as a probed slot's DIB is strictly less than our search DIB,
    /// the key cannot exist further down the probe chain.
    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        if self.count == 0 {
            return None;
        }

        let hash = hash_key(key);
        let mut idx = (hash as usize) & self.mask;
        let mut search_dib: u16 = 0;

        loop {
            let slot = &self.slots[idx];

            if slot.is_empty() {
                // Empty bucket encountered: key does not exist.
                return None;
            }

            if slot.is_occupied() {
                // Early Search Termination Invariant:
                // If stored DIB < search DIB, our key would have displaced it if it existed.
                if slot.dib < search_dib {
                    return None;
                }

                if slot.hash == hash && slot.key() == key {
                    return Some(slot.value());
                }
            }

            idx = (idx + 1) & self.mask;
            search_dib += 1;

            if search_dib as usize >= self.capacity {
                return None;
            }
        }
    }

    pub fn contains_key(&self, key: &[u8]) -> bool {
        self.get(key).is_some()
    }

    /// Helper: Prepares a CompactValue, allocating from SlabArena for values > 23B up to 1024B.
    fn create_compact_value(&mut self, val_bytes: &[u8]) -> CompactValue {
        if val_bytes.len() <= 23 {
            CompactValue::new(val_bytes)
        } else if let Some(ref mut arena) = self.slab_arena {
            if val_bytes.len() <= 1024 {
                if let Some(alloc) = arena.allocate_bytes(val_bytes) {
                    CompactValue::new_slab(alloc.as_ptr(), val_bytes.len())
                } else {
                    CompactValue::new(val_bytes)
                }
            } else {
                CompactValue::new(val_bytes)
            }
        } else {
            CompactValue::new(val_bytes)
        }
    }

    /// Inserts a key-value pair. If key already exists, updates value and returns old value.
    pub fn insert(&mut self, key: &[u8], value: &[u8]) -> Option<Vec<u8>> {
        // Grow table if load factor threshold exceeded
        if (self.count + 1) as f64 / self.capacity as f64 >= MAX_LOAD_FACTOR {
            self.grow();
        }

        let hash = hash_key(key);
        let mut idx = (hash as usize) & self.mask;
        let mut incoming_dib: u16 = 0;

        // Check if key already exists to update in place
        let mut probe_idx = idx;
        let mut probe_dib: u16 = 0;
        while (probe_dib as usize) < self.capacity {
            if self.slots[probe_idx].is_empty() {
                break;
            }
            if self.slots[probe_idx].is_occupied() {
                if self.slots[probe_idx].dib < probe_dib {
                    break;
                }
                if self.slots[probe_idx].hash == hash && self.slots[probe_idx].key() == key {
                    let old_val = self.slots[probe_idx].value().to_vec();
                    let old_slab_ptr = self.slots[probe_idx].value.slab_ptr();
                    if let Some((ptr, len)) = old_slab_ptr {
                        if let Some(ref mut arena) = self.slab_arena {
                            let _ = arena.free(ptr, len);
                        }
                    }
                    let new_val = self.create_compact_value(value);
                    self.slots[probe_idx].set_compact_value(new_val, self.epoch);
                    return Some(old_val);
                }
            }
            probe_idx = (probe_idx + 1) & self.mask;
            probe_dib += 1;
        }

        // Inserting new key: prepare incoming slot
        let incoming_val = self.create_compact_value(value);
        let mut incoming = Slot::new_with_value(hash, self.epoch, 0, key, incoming_val);

        loop {
            let slot = &mut self.slots[idx];

            if slot.is_empty() || slot.is_tombstone() {
                // Found an open slot: place incoming item
                incoming.dib = incoming_dib;
                *slot = incoming;
                self.count += 1;
                return None;
            }

            // Robin Hood displacement: "Steal from the rich (lower DIB), give to the poor (higher DIB)"
            if slot.dib < incoming_dib {
                let displaced_dib = slot.dib;
                incoming.dib = incoming_dib;
                std::mem::swap(slot, &mut incoming);
                incoming_dib = displaced_dib;
            }

            idx = (idx + 1) & self.mask;
            incoming_dib += 1;

            if incoming_dib as usize >= self.capacity {
                // Table is saturated; force resize and retry insert
                self.grow();
                return self.insert(key, value);
            }
        }
    }

    /// Deletes a key from the table using Backshift Deletion.
    ///
    /// Backshift Deletion Invariant:
    /// Adjacent collided elements with DIB > 0 are shifted backward by 1 slot,
    /// decrementing their DIB. Completely eliminates dead tombstones and preserves
    /// spatial probe locality.
    pub fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        if self.count == 0 {
            return None;
        }

        let hash = hash_key(key);
        let mut idx = (hash as usize) & self.mask;
        let mut search_dib: u16 = 0;
        let target_idx;

        // Locate target key
        loop {
            let slot = &self.slots[idx];

            if slot.is_empty() {
                return None;
            }

            if slot.is_occupied() {
                if slot.dib < search_dib {
                    return None;
                }

                if slot.hash == hash && slot.key() == key {
                    target_idx = idx;
                    break;
                }
            }

            idx = (idx + 1) & self.mask;
            search_dib += 1;

            if search_dib as usize >= self.capacity {
                return None;
            }
        }

        let old_val = self.slots[target_idx].value().to_vec();
        let old_slab_ptr = self.slots[target_idx].value.slab_ptr();
        if let Some((ptr, len)) = old_slab_ptr {
            if let Some(ref mut arena) = self.slab_arena {
                let _ = arena.free(ptr, len);
            }
        }

        // Perform O(1) Backshift Deletion
        let mut curr = target_idx;
        let mut next = (curr + 1) & self.mask;

        while self.slots[next].is_occupied() && self.slots[next].dib > 0 {
            // Shift next item backward into curr using zero-copy mem::replace
            let mut shifted = std::mem::replace(&mut self.slots[next], Slot::empty());
            shifted.dib -= 1;
            self.slots[curr] = shifted;

            curr = next;
            next = (next + 1) & self.mask;
        }

        // Clear the trailing hole
        self.slots[curr].clear();
        self.count -= 1;

        Some(old_val)
    }

    /// Deletes a key from the table. Returns true if key was present.
    #[inline(always)]
    pub fn delete(&mut self, key: &[u8]) -> bool {
        self.remove(key).is_some()
    }

    /// Clears all entries from the map and resets slab memory pools.
    pub fn clear(&mut self) {
        if let Some(ref mut arena) = self.slab_arena {
            arena.reset();
        }
        for slot in self.slots.iter_mut() {
            *slot = Slot::empty();
        }
        self.count = 0;
    }

    /// Returns all occupied key slices in the map.
    pub fn keys(&self) -> Vec<&[u8]> {
        self.slots
            .iter()
            .filter(|s| s.is_occupied())
            .map(|s| s.key())
            .collect()
    }

    /// Calculates the maximum Distance from Ideal Bucket across all active slots.
    pub fn max_probe_length(&self) -> usize {
        self.slots
            .iter()
            .filter(|s| s.is_occupied())
            .map(|s| s.dib as usize)
            .max()
            .unwrap_or(0)
    }

    /// Verifies the count of dead tombstones in the table (should always be 0 due to backshift deletion).
    pub fn tombstone_count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_tombstone()).count()
    }

    /// Iterates entries consistent with snapshot epoch E.
    pub fn iter_snapshot(&self, snapshot_epoch: u64) -> Vec<(&[u8], &[u8])> {
        self.slots
            .iter()
            .filter(|s| s.is_occupied() && s.epoch <= snapshot_epoch)
            .map(|s| (s.key(), s.value()))
            .collect()
    }

    /// Computes memory metrics including the fragmentation ratio (Physical / Logical).
    pub fn memory_fragmentation_ratio(&self) -> f64 {
        let mut logical_bytes = 0;
        for slot in self.slots.iter() {
            if slot.is_occupied() {
                logical_bytes += slot.key().len() + slot.value().len();
            }
        }

        if logical_bytes == 0 {
            return 1.0;
        }

        let physical_bytes = if let Some(ref arena) = self.slab_arena {
            arena.stats().total_allocated_bytes
        } else {
            logical_bytes
        };

        if physical_bytes == 0 {
            1.0
        } else {
            (physical_bytes as f64) / (logical_bytes as f64)
        }
    }

    /// Doubles the table capacity and re-inserts all active entries in place without value reallocation.
    fn grow(&mut self) {
        let new_capacity = self.capacity * 2;
        let new_mask = new_capacity - 1;
        let mut new_slots = Vec::with_capacity(new_capacity);
        for _ in 0..new_capacity {
            new_slots.push(Slot::empty());
        }

        for slot in self.slots.iter_mut() {
            if slot.is_occupied() {
                let mut incoming = std::mem::replace(slot, Slot::empty());
                let hash = incoming.hash;
                let mut idx = (hash as usize) & new_mask;
                let mut incoming_dib: u16 = 0;

                loop {
                    let target = &mut new_slots[idx];
                    if target.is_empty() {
                        incoming.dib = incoming_dib;
                        *target = incoming;
                        break;
                    }
                    if target.dib < incoming_dib {
                        let displaced_dib = target.dib;
                        incoming.dib = incoming_dib;
                        std::mem::swap(target, &mut incoming);
                        incoming_dib = displaced_dib;
                    }
                    idx = (idx + 1) & new_mask;
                    incoming_dib += 1;
                }
            }
        }

        self.slots = new_slots.into_boxed_slice();
        self.capacity = new_capacity;
        self.mask = new_mask;
    }
}

impl Default for RobinHoodMap {
    fn default() -> Self {
        Self::new(DEFAULT_INITIAL_CAPACITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_crud() {
        let mut map = RobinHoodMap::new(16);
        assert!(map.is_empty());

        assert_eq!(map.insert(b"k1", b"v1"), None);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(b"k1"), Some(&b"v1"[..]));

        // Update existing
        assert_eq!(map.insert(b"k1", b"v1_updated"), Some(b"v1".to_vec()));
        assert_eq!(map.get(b"k1"), Some(&b"v1_updated"[..]));

        // Remove
        assert_eq!(map.remove(b"k1"), Some(b"v1_updated".to_vec()));
        assert_eq!(map.get(b"k1"), None);
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_backshift_deletion_zero_tombstones() {
        let mut map = RobinHoodMap::new(32);

        // Insert 20 keys
        for i in 0..20 {
            let k = format!("key:{}", i);
            let v = format!("val:{}", i);
            map.insert(k.as_bytes(), v.as_bytes());
        }

        // Delete half the keys
        for i in (0..20).step_by(2) {
            let k = format!("key:{}", i);
            assert!(map.remove(k.as_bytes()).is_some());
        }

        // Invariant: Backshift deletion leaves exactly 0 tombstones
        assert_eq!(
            map.tombstone_count(),
            0,
            "Backshift deletion must eliminate all dead tombstones"
        );

        // Verify remaining keys are 100% retrievable
        for i in (1..20).step_by(2) {
            let k = format!("key:{}", i);
            let expected_v = format!("val:{}", i);
            assert_eq!(map.get(k.as_bytes()), Some(expected_v.as_bytes()));
        }
    }

    #[test]
    fn test_early_termination_on_missing_key() {
        let mut map = RobinHoodMap::new(64);
        map.insert(b"apple", b"1");
        map.insert(b"banana", b"2");
        map.insert(b"cherry", b"3");

        assert_eq!(map.get(b"non_existent_key_xyz"), None);
    }

    #[test]
    fn test_high_volume_resize_and_integrity() {
        let mut map = RobinHoodMap::new(16);
        let count = 5_000;

        // Insert 5,000 keys, triggering multiple power-of-two resizes
        for i in 0..count {
            let k = format!("user:session:{}", i);
            let v = format!("token_{}", i * 7);
            map.insert(k.as_bytes(), v.as_bytes());
        }

        assert_eq!(map.len(), count);

        // Verify 100% retrieval
        for i in 0..count {
            let k = format!("user:session:{}", i);
            let expected_v = format!("token_{}", i * 7);
            assert_eq!(
                map.get(k.as_bytes()),
                Some(expected_v.as_bytes()),
                "Failed to retrieve key {}",
                k
            );
        }

        // Verify probe length bound (Robin Hood keeps DIB remarkably small)
        let max_dib = map.max_probe_length();
        assert!(
            max_dib < 25,
            "Max DIB should be strictly bounded (got {})",
            max_dib
        );
    }

    #[test]
    fn test_snapshot_epoch_filtering() {
        let mut map = RobinHoodMap::new(16);
        map.set_epoch(1);
        map.insert(b"k1", b"v1");
        map.insert(b"k2", b"v2");

        // Advance epoch to 2
        map.set_epoch(2);
        map.insert(b"k3", b"v3");

        // Snapshot at epoch 1 sees only k1, k2
        let snap1 = map.iter_snapshot(1);
        assert_eq!(snap1.len(), 2);

        // Snapshot at epoch 2 sees k1, k2, k3
        let snap2 = map.iter_snapshot(2);
        assert_eq!(snap2.len(), 3);
    }

    #[test]
    fn test_slab_integration_inserts_and_deletions() {
        let mut map = RobinHoodMap::with_slab_chunk(64, 64 * 1024);
        assert!(map.has_slab());

        // 1. Inlined small value (<= 23B)
        map.insert(b"small_key", b"short_val");
        assert_eq!(map.get(b"small_key"), Some(&b"short_val"[..]));
        let slot = map.slot_at(map.slots.iter().position(|s| s.key() == b"small_key").unwrap()).unwrap();
        assert!(slot.value.is_inline());

        // 2. Slab-allocated value (> 23B, fits in 64B class)
        let med_val = b"this_is_a_medium_payload_that_exceeds_twenty_three_bytes_exactly";
        map.insert(b"med_key", med_val);
        assert_eq!(map.get(b"med_key"), Some(&med_val[..]));
        let slot_med = map.slot_at(map.slots.iter().position(|s| s.key() == b"med_key").unwrap()).unwrap();
        assert!(slot_med.value.is_slab());

        // 3. Update slab key with new value
        let med_val_2 = b"another_medium_payload_updating_in_place_inside_slab_arena_now";
        let old = map.insert(b"med_key", med_val_2);
        assert_eq!(old, Some(med_val.to_vec()));
        assert_eq!(map.get(b"med_key"), Some(&med_val_2[..]));

        // 4. Delete slab key (verifies O(1) slab free)
        assert!(map.delete(b"med_key"));
        assert_eq!(map.get(b"med_key"), None);

        // 5. High volume inserts across size classes
        for i in 0..500 {
            let k = format!("k:{}", i);
            let v = format!("payload_with_sufficient_length_to_trigger_slab_allocation_{:040}", i);
            map.insert(k.as_bytes(), v.as_bytes());
        }

        for i in 0..500 {
            let k = format!("k:{}", i);
            let expected_v = format!("payload_with_sufficient_length_to_trigger_slab_allocation_{:040}", i);
            assert_eq!(map.get(k.as_bytes()), Some(expected_v.as_bytes()));
        }

        // Verify fragmentation ratio is tightly bounded
        let ratio = map.memory_fragmentation_ratio();
        assert!(ratio >= 1.0 && ratio < 1.35, "Fragmentation ratio: {}", ratio);

        // Clear map and verify reset
        map.clear();
        assert_eq!(map.len(), 0);
        assert_eq!(map.memory_fragmentation_ratio(), 1.0);
    }

    #[test]
    fn test_slab_integration_fragmentation_acceptance() {
        let mut map = RobinHoodMap::with_slab_chunk(256, 128 * 1024);

        // Churn workload: insert and delete 5,000 keys with slab allocations
        for i in 0..5_000 {
            let k = format!("user:session:token:{}", i);
            let v = format!("bearer_token_payload_with_jwt_signature_long_data_{:050}", i);
            map.insert(k.as_bytes(), v.as_bytes());
            if i % 4 == 0 {
                map.delete(k.as_bytes());
            }
        }

        let ratio = map.memory_fragmentation_ratio();
        assert!(
            ratio < 1.15,
            "Fragmentation ratio must stay strictly bounded under high churn (got {:.3})",
            ratio
        );
    }
}
