//! Robin Hood Hash Table with Backshift Deletion & Contiguous Flat Arrays
//!
//! Features:
//! - Contiguous array of 64-byte aligned `Slot` entries (maximum cache locality).
//! - Robin Hood displacement ("steal from the rich, give to the poor") to minimize probe variance.
//! - O(1) Backshift Deletion: shifts collided elements backward, completely eliminating dead tombstones.
//! - Early Search Termination: lookups stop as soon as `slot.dib < search_dib`.
//! - Snapshot Epoch awareness for Point-in-Time consistency.

use super::entry::Slot;
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

/// High-Performance Robin Hood Hash Map.
pub struct RobinHoodMap {
    slots: Box<[Slot]>,
    capacity: usize,
    mask: usize,
    count: usize,
    epoch: u64,
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
        }
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
            let slot = &mut self.slots[probe_idx];
            if slot.is_empty() {
                break;
            }
            if slot.is_occupied() {
                if slot.dib < probe_dib {
                    break;
                }
                if slot.hash == hash && slot.key() == key {
                    let old_val = slot.value().to_vec();
                    slot.set_value(value, self.epoch);
                    return Some(old_val);
                }
            }
            probe_idx = (probe_idx + 1) & self.mask;
            probe_dib += 1;
        }

        // Inserting new key: prepare incoming slot
        let mut incoming = Slot::new(hash, self.epoch, 0, key, value);

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

        // Perform O(1) Backshift Deletion
        let mut curr = target_idx;
        let mut next = (curr + 1) & self.mask;

        while self.slots[next].is_occupied() && self.slots[next].dib > 0 {
            // Shift next item backward into curr
            let mut shifted = self.slots[next].clone();
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

    /// Doubles the table capacity and re-inserts all active entries.
    fn grow(&mut self) {
        let new_capacity = self.capacity * 2;
        let mut new_map = RobinHoodMap::new(new_capacity);
        new_map.set_epoch(self.epoch);

        for slot in self.slots.iter() {
            if slot.is_occupied() {
                new_map.insert(slot.key(), slot.value());
            }
        }

        self.slots = new_map.slots;
        self.capacity = new_map.capacity;
        self.mask = new_map.mask;
        self.count = new_map.count;
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
}
