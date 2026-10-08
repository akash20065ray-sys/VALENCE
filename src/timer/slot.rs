//! Intrusive Lock-Free Expiration Slot Buckets
//!
//! Implements O(1) atomic bucket detachment for the 4-Tier Hierarchical Timing Wheel.
//! Eliminates TTL avalanche latency spikes by swapping entire lists of expired timers
//! in a single atomic instruction (~5-10ns), allowing asynchronous, non-blocking reclamation.

use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::storage::CompactKey;

/// Intrusive Timer Node storing expiration deadline and key reference.
#[repr(C)]
pub struct TimerNode {
    pub key: CompactKey,
    pub expire_at_ms: u64,
    pub next: *mut TimerNode,
}

impl TimerNode {
    /// Allocates a new heap-backed timer node.
    pub fn new(key: &[u8], expire_at_ms: u64) -> *mut Self {
        Box::into_raw(Box::new(Self {
            key: CompactKey::new(key),
            expire_at_ms,
            next: ptr::null_mut(),
        }))
    }

    /// Reclaims a single heap-allocated node.
    ///
    /// # Safety
    /// Must only be called once per node when it is detached from all lists.
    pub unsafe fn free(node: *mut Self) {
        if !node.is_null() {
            drop(Box::from_raw(node));
        }
    }
}

/// Atomic Intrusive Linked List Bucket for a Timing Wheel Slot.
pub struct AtomicTimerBucket {
    head: AtomicPtr<TimerNode>,
}

unsafe impl Send for AtomicTimerBucket {}
unsafe impl Sync for AtomicTimerBucket {}

impl AtomicTimerBucket {
    /// Creates a new empty bucket.
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Pushes a timer node to the head of the bucket using lock-free CAS.
    ///
    /// # Safety
    /// `node` must be a valid, heap-allocated pointer returned by `TimerNode::new`.
    pub fn push(&self, node: *mut TimerNode) {
        assert!(!node.is_null(), "Cannot push null timer node");

        let mut current_head = self.head.load(Ordering::Acquire);
        loop {
            unsafe {
                (*node).next = current_head;
            }

            match self.head.compare_exchange_weak(
                current_head,
                node,
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(actual_head) => current_head = actual_head,
            }
        }
    }

    /// Atomically detaches the entire linked list of timer nodes in O(1) time.
    ///
    /// Returns the raw pointer to the head of the detached list.
    /// The caller obtains exclusive ownership of all nodes in the chain.
    #[inline(always)]
    pub fn detach_all(&self) -> *mut TimerNode {
        self.head.swap(ptr::null_mut(), Ordering::AcqRel)
    }

    /// Checks whether the bucket currently contains any pending timers.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire).is_null()
    }

    /// Drains all detached nodes from a list head into a owned vector, deallocating the nodes.
    ///
    /// # Safety
    /// `head` must be a valid detached pointer chain owned exclusively by the caller.
    pub unsafe fn collect_and_free(mut head: *mut TimerNode) -> Vec<(Vec<u8>, u64)> {
        let mut collected = Vec::new();
        while !head.is_null() {
            let next = (*head).next;
            let key_bytes = (*head).key.as_slice().to_vec();
            let expire_at = (*head).expire_at_ms;

            TimerNode::free(head);
            collected.push((key_bytes, expire_at));

            head = next;
        }
        collected
    }
}

impl Default for AtomicTimerBucket {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AtomicTimerBucket {
    fn drop(&mut self) {
        let detached = self.detach_all();
        if !detached.is_null() {
            unsafe {
                Self::collect_and_free(detached);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_bucket_push_and_detach() {
        let bucket = AtomicTimerBucket::new();
        assert!(bucket.is_empty());

        let n1 = TimerNode::new(b"key1", 100);
        let n2 = TimerNode::new(b"key2", 200);
        let n3 = TimerNode::new(b"key3", 300);

        bucket.push(n1);
        bucket.push(n2);
        bucket.push(n3);

        assert!(!bucket.is_empty());

        let detached = bucket.detach_all();
        assert!(bucket.is_empty());

        let collected = unsafe { AtomicTimerBucket::collect_and_free(detached) };
        assert_eq!(collected.len(), 3);

        // Verify all keys are present
        let keys: Vec<Vec<u8>> = collected.into_iter().map(|(k, _)| k).collect();
        assert!(keys.contains(&b"key1".to_vec()));
        assert!(keys.contains(&b"key2".to_vec()));
        assert!(keys.contains(&b"key3".to_vec()));
    }

    #[test]
    fn test_concurrent_push_and_detach() {
        let bucket = Arc::new(AtomicTimerBucket::new());
        let threads = 4;
        let per_thread = 10_000;
        let mut handles = Vec::new();

        // 4 concurrent producer threads enqueueing timers
        for t in 0..threads {
            let b = bucket.clone();
            handles.push(thread::spawn(move || {
                for i in 0..per_thread {
                    let key = format!("thread_{}_key_{}", t, i);
                    let node = TimerNode::new(key.as_bytes(), 1000 + i as u64);
                    b.push(node);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // Single consumer detaches all
        let detached = bucket.detach_all();
        let collected = unsafe { AtomicTimerBucket::collect_and_free(detached) };

        assert_eq!(collected.len(), threads * per_thread);
        assert!(bucket.is_empty());
    }
}
