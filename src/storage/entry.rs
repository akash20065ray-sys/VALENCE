//! Valence Storage Engine: Cache-Line Aligned Entry Slot
//!
//! Provides the fundamental storage unit for Valence's hash table.
//! Designed for mechanical sympathy: exactly 64 bytes in size and 64-byte aligned
//! to ensure that every slot maps precisely to a single CPU L1/L2 cache line.

use std::fmt;
use std::ops::Deref;

/// Slot state lifecycle flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SlotState {
    /// Slot is completely empty and uninitialized.
    Empty = 0,
    /// Slot contains active, valid key-value data.
    Occupied = 1,
    /// Slot was previously occupied but has been deleted.
    Tombstone = 2,
}

const INLINE_KEY_CAPACITY: usize = 15;
const INLINE_VAL_CAPACITY: usize = 23;

const FLAG_KEY_INLINE: u8 = 0b0000_0001;
const FLAG_VAL_INLINE: u8 = 0b0000_0010;

/// Compact key representation with Small String Optimization (SSO).
/// Exactly 16 bytes:
/// - If len <= 15: stored inline with zero heap allocation.
/// - If len > 15: stored as a pointer + length on the heap.
#[repr(C)]
pub struct CompactKey {
    len: u8,
    data: [u8; INLINE_KEY_CAPACITY],
}

impl CompactKey {
    pub fn new(bytes: &[u8]) -> Self {
        if bytes.len() <= INLINE_KEY_CAPACITY {
            let mut data = [0u8; INLINE_KEY_CAPACITY];
            data[..bytes.len()].copy_from_slice(bytes);
            Self {
                len: bytes.len() as u8,
                data,
            }
        } else {
            // Allocate heap buffer: pointer (8B) + len (8B) packed into 16 bytes
            let boxed = bytes.to_vec().into_boxed_slice();
            let raw_ptr = Box::into_raw(boxed);
            let ptr_val = (raw_ptr as *mut u8 as usize).to_ne_bytes();
            let len_val = bytes.len().to_ne_bytes();

            let mut data = [0u8; INLINE_KEY_CAPACITY];
            // Store 7 bytes of length in data[0..7] and ptr in data[7..15]
            data[0..7].copy_from_slice(&len_val[0..7]);
            data[7..15].copy_from_slice(&ptr_val);

            Self {
                len: 0xFF, // Marker indicating heap-allocated pointer
                data,
            }
        }
    }

    #[inline(always)]
    pub fn is_inline(&self) -> bool {
        self.len != 0xFF
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        if self.is_inline() {
            &self.data[..self.len as usize]
        } else {
            let mut ptr_bytes = [0u8; 8];
            ptr_bytes.copy_from_slice(&self.data[7..15]);
            let ptr = usize::from_ne_bytes(ptr_bytes) as *const u8;

            let mut len_bytes = [0u8; 8];
            len_bytes[0..7].copy_from_slice(&self.data[0..7]);
            let len = usize::from_ne_bytes(len_bytes);

            unsafe { std::slice::from_raw_parts(ptr, len) }
        }
    }
}

impl Deref for CompactKey {
    type Target = [u8];
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl Clone for CompactKey {
    fn clone(&self) -> Self {
        Self::new(self.as_slice())
    }
}

impl Drop for CompactKey {
    fn drop(&mut self) {
        if !self.is_inline() {
            let mut ptr_bytes = [0u8; 8];
            ptr_bytes.copy_from_slice(&self.data[7..15]);
            let ptr = usize::from_ne_bytes(ptr_bytes) as *mut u8;

            let mut len_bytes = [0u8; 8];
            len_bytes[0..7].copy_from_slice(&self.data[0..7]);
            let len = usize::from_ne_bytes(len_bytes);

            unsafe {
                let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr, len));
            }
        }
    }
}

impl fmt::Debug for CompactKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", String::from_utf8_lossy(self.as_slice()))
    }
}

impl PartialEq for CompactKey {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl Eq for CompactKey {}

/// Compact value representation with Small Value Inlining (SSO).
/// Exactly 24 bytes:
/// - If len <= 23: stored inline with zero heap allocation.
/// - If len > 23: stored as pointer + length + capacity on the heap.
#[repr(C)]
pub struct CompactValue {
    len: u8,
    data: [u8; INLINE_VAL_CAPACITY],
}

impl CompactValue {
    pub fn new(bytes: &[u8]) -> Self {
        if bytes.len() <= INLINE_VAL_CAPACITY {
            let mut data = [0u8; INLINE_VAL_CAPACITY];
            data[..bytes.len()].copy_from_slice(bytes);
            Self {
                len: bytes.len() as u8,
                data,
            }
        } else {
            let boxed = bytes.to_vec().into_boxed_slice();
            let raw_ptr = Box::into_raw(boxed);
            let ptr_val = (raw_ptr as *mut u8 as usize).to_ne_bytes();
            let len_val = bytes.len().to_ne_bytes();

            let mut data = [0u8; INLINE_VAL_CAPACITY];
            // Store ptr in data[0..8] and len in data[8..16]
            data[0..8].copy_from_slice(&ptr_val);
            data[8..16].copy_from_slice(&len_val);

            Self {
                len: 0xFF, // Marker indicating heap pointer
                data,
            }
        }
    }

    #[inline(always)]
    pub fn is_inline(&self) -> bool {
        self.len != 0xFF
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        if self.is_inline() {
            &self.data[..self.len as usize]
        } else {
            let mut ptr_bytes = [0u8; 8];
            ptr_bytes.copy_from_slice(&self.data[0..8]);
            let ptr = usize::from_ne_bytes(ptr_bytes) as *const u8;

            let mut len_bytes = [0u8; 8];
            len_bytes.copy_from_slice(&self.data[8..16]);
            let len = usize::from_ne_bytes(len_bytes);

            unsafe { std::slice::from_raw_parts(ptr, len) }
        }
    }
}

impl Deref for CompactValue {
    type Target = [u8];
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl Clone for CompactValue {
    fn clone(&self) -> Self {
        Self::new(self.as_slice())
    }
}

impl Drop for CompactValue {
    fn drop(&mut self) {
        if !self.is_inline() {
            let mut ptr_bytes = [0u8; 8];
            ptr_bytes.copy_from_slice(&self.data[0..8]);
            let ptr = usize::from_ne_bytes(ptr_bytes) as *mut u8;

            let mut len_bytes = [0u8; 8];
            len_bytes.copy_from_slice(&self.data[8..16]);
            let len = usize::from_ne_bytes(len_bytes);

            unsafe {
                let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr, len));
            }
        }
    }
}

impl fmt::Debug for CompactValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", String::from_utf8_lossy(self.as_slice()))
    }
}

impl PartialEq for CompactValue {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl Eq for CompactValue {}

/// The Fundamental Cache-Line Aligned Hash Table Slot.
///
/// Hardware Layout Invariant:
/// - Size: Exactly 64 bytes.
/// - Alignment: 64 bytes.
/// - Guarantees that probing this slot fetches exactly 1 CPU cache line with zero
///   straddling across cache boundaries.
#[repr(C, align(64))]
pub struct Slot {
    /// Full 64-bit hash of the key for fast collision rejection before key equality check.
    pub hash: u64,
    /// Mutation epoch in which the current live value was committed (for Snapshot Isolation).
    pub epoch: u64,
    /// Distance from Ideal Bucket (DIB) for Robin Hood displacement & bounded probe lengths.
    pub dib: u16,
    /// State flag: Empty, Occupied, or Tombstone.
    pub state: SlotState,
    /// Auxiliary bitflags (inline status, compression, etc.).
    pub flags: u8,
    /// Explicit padding to align `key` cleanly at 24-byte offset.
    _pad: [u8; 4],
    /// Compact key with SSO support (16 bytes).
    pub key: CompactKey,
    /// Compact value with SSO support (24 bytes).
    pub value: CompactValue,
}

impl Slot {
    /// Constructs a clean, empty slot.
    pub const fn empty() -> Self {
        Self {
            hash: 0,
            epoch: 0,
            dib: 0,
            state: SlotState::Empty,
            flags: 0,
            _pad: [0; 4],
            key: CompactKey {
                len: 0,
                data: [0; INLINE_KEY_CAPACITY],
            },
            value: CompactValue {
                len: 0,
                data: [0; INLINE_VAL_CAPACITY],
            },
        }
    }

    /// Constructs an occupied slot.
    pub fn new(hash: u64, epoch: u64, dib: u16, key_bytes: &[u8], val_bytes: &[u8]) -> Self {
        let key = CompactKey::new(key_bytes);
        let value = CompactValue::new(val_bytes);
        let mut flags = 0u8;
        if key.is_inline() {
            flags |= FLAG_KEY_INLINE;
        }
        if value.is_inline() {
            flags |= FLAG_VAL_INLINE;
        }

        Self {
            hash,
            epoch,
            dib,
            state: SlotState::Occupied,
            flags,
            _pad: [0; 4],
            key,
            value,
        }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.state == SlotState::Empty
    }

    #[inline(always)]
    pub fn is_occupied(&self) -> bool {
        self.state == SlotState::Occupied
    }

    #[inline(always)]
    pub fn is_tombstone(&self) -> bool {
        self.state == SlotState::Tombstone
    }

    #[inline(always)]
    pub fn key(&self) -> &[u8] {
        self.key.as_slice()
    }

    #[inline(always)]
    pub fn value(&self) -> &[u8] {
        self.value.as_slice()
    }

    /// Updates the value and records the mutation epoch.
    pub fn set_value(&mut self, new_val_bytes: &[u8], new_epoch: u64) {
        self.value = CompactValue::new(new_val_bytes);
        self.epoch = new_epoch;
        if self.value.is_inline() {
            self.flags |= FLAG_VAL_INLINE;
        } else {
            self.flags &= !FLAG_VAL_INLINE;
        }
    }

    /// Marks the slot as a tombstone while preserving mutation epoch for snapshot consistency.
    pub fn mark_tombstone(&mut self, epoch: u64) {
        self.state = SlotState::Tombstone;
        self.epoch = epoch;
    }

    /// Clears the slot completely back to empty.
    pub fn clear(&mut self) {
        self.hash = 0;
        self.epoch = 0;
        self.dib = 0;
        self.state = SlotState::Empty;
        self.flags = 0;
        self.key = CompactKey::new(&[]);
        self.value = CompactValue::new(&[]);
    }
}

impl Default for Slot {
    fn default() -> Self {
        Self::empty()
    }
}

impl Clone for Slot {
    fn clone(&self) -> Self {
        Self {
            hash: self.hash,
            epoch: self.epoch,
            dib: self.dib,
            state: self.state,
            flags: self.flags,
            _pad: [0; 4],
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }
}

impl fmt::Debug for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Slot")
            .field("hash", &self.hash)
            .field("epoch", &self.epoch)
            .field("dib", &self.dib)
            .field("state", &self.state)
            .field("key", &String::from_utf8_lossy(self.key()))
            .field("value", &String::from_utf8_lossy(self.value()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};

    #[test]
    fn test_slot_size_and_alignment() {
        // Architectural Invariant: Exactly 64 bytes and 64-byte aligned
        assert_eq!(
            size_of::<Slot>(),
            64,
            "Slot must be exactly 64 bytes to fit one CPU cache line"
        );
        assert_eq!(
            align_of::<Slot>(),
            64,
            "Slot must be 64-byte aligned to prevent cache line straddling"
        );
        assert_eq!(size_of::<CompactKey>(), 16);
        assert_eq!(size_of::<CompactValue>(), 24);
    }

    #[test]
    fn test_slot_lifecycle() {
        let mut slot = Slot::empty();
        assert!(slot.is_empty());
        assert!(!slot.is_occupied());

        // Occupy slot
        slot = Slot::new(0xDEADBEEF, 1, 0, b"user:42", b"Akash");
        assert!(slot.is_occupied());
        assert_eq!(slot.key(), b"user:42");
        assert_eq!(slot.value(), b"Akash");
        assert_eq!(slot.epoch, 1);
        assert_eq!(slot.dib, 0);

        // Update value
        slot.set_value(b"Akash_Updated", 2);
        assert_eq!(slot.value(), b"Akash_Updated");
        assert_eq!(slot.epoch, 2);

        // Mark tombstone
        slot.mark_tombstone(3);
        assert!(slot.is_tombstone());
        assert_eq!(slot.epoch, 3);

        // Clear
        slot.clear();
        assert!(slot.is_empty());
    }

    #[test]
    fn test_small_value_inlining() {
        // Values <= 23 bytes must be inlined
        let small_val = b"inline_token_12345";
        assert!(small_val.len() <= INLINE_VAL_CAPACITY);
        let compact = CompactValue::new(small_val);
        assert!(compact.is_inline(), "Values <= 23 bytes must be inlined");
        assert_eq!(compact.as_slice(), small_val);

        // Keys <= 15 bytes must be inlined
        let small_key = b"session:99";
        assert!(small_key.len() <= INLINE_KEY_CAPACITY);
        let compact_key = CompactKey::new(small_key);
        assert!(compact_key.is_inline(), "Keys <= 15 bytes must be inlined");
        assert_eq!(compact_key.as_slice(), small_key);
    }

    #[test]
    fn test_large_value_heap_fallback() {
        // Value > 23 bytes must fall back to heap without corruption
        let large_val = b"this_is_a_very_long_value_exceeding_the_inline_budget_for_benchmarking_purposes";
        assert!(large_val.len() > INLINE_VAL_CAPACITY);
        let compact = CompactValue::new(large_val);
        assert!(!compact.is_inline());
        assert_eq!(compact.as_slice(), large_val);

        // Clone large value
        let cloned = compact.clone();
        assert_eq!(cloned.as_slice(), large_val);

        // Key > 15 bytes must fall back to heap
        let large_key = b"very_long_user_session_identifier_key";
        assert!(large_key.len() > INLINE_KEY_CAPACITY);
        let compact_key = CompactKey::new(large_key);
        assert!(!compact_key.is_inline());
        assert_eq!(compact_key.as_slice(), large_key);
    }
}
