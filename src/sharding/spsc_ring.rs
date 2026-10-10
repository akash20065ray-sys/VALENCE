//! 64-Byte Cache-Padded SPSC Ring Buffer
//!
//! Provides a lock-free, zero-allocation Single-Producer Single-Consumer (SPSC)
//! circular ring buffer. Atomic head and tail indices are isolated on separate
//! 64-byte cache lines using `#[repr(align(64))]` to eliminate false sharing
//! across the CPU's MESI cache coherence interconnect.

use std::cell::Cell;
use std::fmt;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Cache-line padded wrapper ensuring 64-byte CPU cache alignment.
///
/// On x86_64 and ARM64 architectures, CPU caches operate on 64-byte lines.
/// Placing concurrently modified atomics (e.g., queue head and tail) in the
/// same 64-byte block triggers severe MESI cache invalidation storm ("false sharing").
/// Wrapping atomics in `CachePadded` forces the compiler to pad and align the
/// struct to a dedicated 64-byte cache line.
#[repr(align(64))]
pub struct CachePadded<T> {
    pub value: T,
}

impl<T> CachePadded<T> {
    pub const fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> std::ops::Deref for CachePadded<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> std::ops::DerefMut for CachePadded<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: fmt::Debug> fmt::Debug for CachePadded<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CachePadded({:?})", self.value)
    }
}

/// Error returned when attempting to push into a full ring buffer.
/// Returns ownership of the unpushed item.
#[derive(Debug, PartialEq, Eq)]
pub struct QueueFull<T>(pub T);

/// Internal shared state of the SPSC circular queue.
struct RingInner<T> {
    buffer: Box<[std::cell::UnsafeCell<MaybeUninit<T>>]>,
    capacity: usize,
    mask: usize,
    /// Head index: modified solely by the Consumer.
    head: CachePadded<AtomicUsize>,
    /// Tail index: modified solely by the Producer.
    tail: CachePadded<AtomicUsize>,
}

unsafe impl<T: Send> Send for RingInner<T> {}
unsafe impl<T: Send> Sync for RingInner<T> {}

impl<T> Drop for RingInner<T> {
    fn drop(&mut self) {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        for i in head..tail {
            let slot_idx = i & self.mask;
            unsafe {
                let ptr = self.buffer[slot_idx].get();
                std::ptr::drop_in_place((*ptr).as_mut_ptr());
            }
        }
    }
}

/// Single-Producer handle for the lock-free ring buffer.
///
/// Implements `Send` but intentionally `!Sync` to enforce the single-producer invariant.
pub struct Producer<T> {
    inner: Arc<RingInner<T>>,
    /// Cached copy of the consumer's head to avoid loading the atomic head on every push.
    cached_head: Cell<usize>,
}

unsafe impl<T: Send> Send for Producer<T> {}

impl<T> Producer<T> {
    /// Pushes an item into the ring buffer.
    ///
    /// If the queue is full, returns `Err(QueueFull(item))`.
    /// Executes in O(1) time without heap allocations or locks.
    pub fn push(&self, item: T) -> Result<(), QueueFull<T>> {
        let tail = self.inner.tail.load(Ordering::Relaxed);
        let capacity = self.inner.capacity;

        // Check against cached head first (Lamport / FastForward optimization)
        if tail.wrapping_sub(self.cached_head.get()) >= capacity {
            let actual_head = self.inner.head.load(Ordering::Acquire);
            self.cached_head.set(actual_head);
            if tail.wrapping_sub(actual_head) >= capacity {
                return Err(QueueFull(item));
            }
        }

        let slot_idx = tail & self.inner.mask;
        unsafe {
            let slot = self.inner.buffer[slot_idx].get();
            (*slot).write(item);
        }

        // Publish written item to consumer
        self.inner.tail.store(tail.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    /// Total capacity of the ring buffer.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.inner.capacity
    }

    /// Approximate number of unconsumed elements currently in the buffer.
    pub fn len(&self) -> usize {
        let tail = self.inner.tail.load(Ordering::Relaxed);
        let head = self.inner.head.load(Ordering::Relaxed);
        tail.saturating_sub(head)
    }

    /// Returns `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if the queue appears to be full.
    pub fn is_full(&self) -> bool {
        self.len() >= self.inner.capacity
    }
}

/// Single-Consumer handle for the lock-free ring buffer.
///
/// Implements `Send` but intentionally `!Sync` to enforce the single-consumer invariant.
pub struct Consumer<T> {
    inner: Arc<RingInner<T>>,
    /// Cached copy of the producer's tail to avoid loading the atomic tail on every pop.
    cached_tail: Cell<usize>,
}

unsafe impl<T: Send> Send for Consumer<T> {}

impl<T> Consumer<T> {
    /// Pops an item from the ring buffer.
    ///
    /// Returns `None` if the queue is currently empty.
    /// Executes in O(1) time without heap allocations or locks.
    pub fn pop(&self) -> Option<T> {
        let head = self.inner.head.load(Ordering::Relaxed);

        // Check against cached tail first
        if head == self.cached_tail.get() {
            let actual_tail = self.inner.tail.load(Ordering::Acquire);
            self.cached_tail.set(actual_tail);
            if head == actual_tail {
                return None;
            }
        }

        let slot_idx = head & self.inner.mask;
        let item = unsafe {
            let slot = self.inner.buffer[slot_idx].get();
            (*slot).assume_init_read()
        };

        // Publish consumed slot to producer
        self.inner.head.store(head.wrapping_add(1), Ordering::Release);
        Some(item)
    }

    /// Total capacity of the ring buffer.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.inner.capacity
    }

    /// Approximate number of unconsumed elements in the queue.
    pub fn len(&self) -> usize {
        let tail = self.inner.tail.load(Ordering::Relaxed);
        let head = self.inner.head.load(Ordering::Relaxed);
        tail.saturating_sub(head)
    }

    /// Returns `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        let head = self.inner.head.load(Ordering::Relaxed);
        let tail = self.inner.tail.load(Ordering::Relaxed);
        head == tail
    }
}

/// Creates a 64-byte cache-padded lock-free SPSC channel.
///
/// The requested capacity is rounded up to the nearest power of two.
/// Returns a `(Producer<T>, Consumer<T>)` pair.
pub fn spsc_channel<T: Send>(requested_capacity: usize) -> (Producer<T>, Consumer<T>) {
    let capacity = requested_capacity.max(2).next_power_of_two();
    let mask = capacity - 1;

    let mut storage = Vec::with_capacity(capacity);
    for _ in 0..capacity {
        storage.push(std::cell::UnsafeCell::new(MaybeUninit::uninit()));
    }

    let inner = Arc::new(RingInner {
        buffer: storage.into_boxed_slice(),
        capacity,
        mask,
        head: CachePadded::new(AtomicUsize::new(0)),
        tail: CachePadded::new(AtomicUsize::new(0)),
    });

    (
        Producer {
            inner: Arc::clone(&inner),
            cached_head: Cell::new(0),
        },
        Consumer {
            inner,
            cached_tail: Cell::new(0),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};
    use std::thread;

    #[test]
    fn test_cache_padding_and_alignment() {
        assert_eq!(align_of::<CachePadded<AtomicUsize>>(), 64);
        assert_eq!(size_of::<CachePadded<AtomicUsize>>(), 64);

        let (prod, cons) = spsc_channel::<usize>(128);
        let head_addr = &prod.inner.head.value as *const _ as usize;
        let tail_addr = &prod.inner.tail.value as *const _ as usize;

        // Head and tail must reside on distinct 64-byte cache lines
        assert_eq!(head_addr % 64, 0, "Head must be 64-byte cache-aligned");
        assert_eq!(tail_addr % 64, 0, "Tail must be 64-byte cache-aligned");
        assert!(
            head_addr.abs_diff(tail_addr) >= 64,
            "Head and tail must be separated by at least 64 bytes to eliminate false sharing"
        );
        drop(cons);
    }

    #[test]
    fn test_single_threaded_push_pop() {
        let (prod, cons) = spsc_channel::<u64>(4);
        assert_eq!(prod.capacity(), 4);
        assert!(prod.is_empty());

        assert!(prod.push(10).is_ok());
        assert!(prod.push(20).is_ok());
        assert!(prod.push(30).is_ok());
        assert!(prod.push(40).is_ok());
        assert_eq!(prod.len(), 4);

        // 5th push should fail because capacity is 4
        let err = prod.push(50);
        assert_eq!(err, Err(QueueFull(50)));

        assert_eq!(cons.pop(), Some(10));
        assert_eq!(cons.pop(), Some(20));
        assert_eq!(cons.len(), 2);

        // Can push again after popping
        assert!(prod.push(50).is_ok());
        assert_eq!(cons.pop(), Some(30));
        assert_eq!(cons.pop(), Some(40));
        assert_eq!(cons.pop(), Some(50));
        assert_eq!(cons.pop(), None);
        assert!(cons.is_empty());
    }

    #[test]
    fn test_wrap_around_ring() {
        let (prod, cons) = spsc_channel::<usize>(4);
        for i in 0..10_000 {
            assert!(prod.push(i).is_ok());
            assert_eq!(cons.pop(), Some(i));
        }
        assert!(cons.is_empty());
    }

    #[test]
    fn test_drop_unconsumed_items() {
        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct DropDetector(#[allow(dead_code)] usize);
        impl Drop for DropDetector {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        DROP_COUNT.store(0, Ordering::SeqCst);
        {
            let (prod, cons) = spsc_channel::<DropDetector>(16);
            for i in 0..10 {
                let _ = prod.push(DropDetector(i));
            }
            // Pop 3 items
            let _ = cons.pop();
            let _ = cons.pop();
            let _ = cons.pop();
            assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 3);
            // Drop producer and consumer; remaining 7 items must be dropped cleanly
        }
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_multithreaded_spsc_stress() {
        let total_items = 500_000;
        let (prod, cons) = spsc_channel::<usize>(1024);

        let producer_handle = thread::spawn(move || {
            for i in 0..total_items {
                let mut val = i;
                loop {
                    match prod.push(val) {
                        Ok(()) => break,
                        Err(QueueFull(v)) => {
                            val = v;
                            std::hint::spin_loop();
                        }
                    }
                }
            }
        });

        let consumer_handle = thread::spawn(move || {
            let mut received = 0;
            while received < total_items {
                if let Some(val) = cons.pop() {
                    assert_eq!(val, received, "Items must be received strictly in-order");
                    received += 1;
                } else {
                    std::hint::spin_loop();
                }
            }
            received
        });

        producer_handle.join().unwrap();
        let total_received = consumer_handle.join().unwrap();
        assert_eq!(total_received, total_items);
    }
}
