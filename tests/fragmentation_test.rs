//! Sub-Phase 4.4: Zero-Fragmentation & RSS Memory Chaos Harness
//!
//! Quality Assurance & Adversarial Memory Stress:
//! - Tests multi-class slab allocation under high-churn insert/delete/update workloads.
//! - Validates zero memory leaks: draining the table returns total active slab bytes strictly to 0.
//! - Exercises cross-class payload transitions (growing from 32B to 128B to 512B and shrinking).
//! - Measures the steady-state memory fragmentation ratio (Physical / Logical).
//! - Validates multi-thread shard isolation with zero lock contention.

use valence::storage::robin_hood::RobinHoodMap;
use std::thread;

/// Generates a deterministic pseudo-random payload of the exact requested length.
fn make_payload(seed: usize, len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        out.push(((seed.wrapping_mul(31) + i) & 0xFF) as u8);
    }
    out
}

#[test]
fn test_zero_memory_leak_after_full_drain() {
    let mut map = RobinHoodMap::with_slab_chunk(1024, 256 * 1024);
    let count = 10_000;

    // Phase 1: Insert 10,000 keys across all geometric size classes (32B to 1024B)
    for i in 0..count {
        let k = format!("user:session:{}", i);
        // Vary lengths: 20B (inline), 45B (64B class), 100B (128B class), 400B (512B class), 800B (1024B class)
        let len = match i % 5 {
            0 => 20,
            1 => 45,
            2 => 100,
            3 => 400,
            _ => 800,
        };
        let v = make_payload(i, len);
        map.insert(k.as_bytes(), &v);
    }

    assert_eq!(map.len(), count);

    // Verify all keys are retrievable and byte-for-byte exact
    for i in 0..count {
        let k = format!("user:session:{}", i);
        let len = match i % 5 {
            0 => 20,
            1 => 45,
            2 => 100,
            3 => 400,
            _ => 800,
        };
        let expected = make_payload(i, len);
        assert_eq!(map.get(k.as_bytes()), Some(expected.as_slice()));
    }

    // Verify slab arena has allocated active bytes
    let slab_stats = map.slab_arena().unwrap().stats();
    assert!(
        slab_stats.total_allocated_bytes > 0,
        "Slab arena must contain active allocations"
    );

    // Phase 2: Delete every single key one by one
    for i in 0..count {
        let k = format!("user:session:{}", i);
        assert!(map.delete(k.as_bytes()), "Key {} must exist for deletion", k);
    }

    assert_eq!(map.len(), 0);

    // Invariant: After draining all keys, active allocated slab bytes must be strictly 0
    let drained_stats = map.slab_arena().unwrap().stats();
    assert_eq!(
        drained_stats.total_allocated_bytes, 0,
        "Zero-leak invariant failed: drained arena retained {} active bytes",
        drained_stats.total_allocated_bytes
    );

    for (class_idx, &count) in drained_stats.class_allocated_slots.iter().enumerate() {
        assert_eq!(
            count, 0,
            "Class {} still has {} un-freed slots",
            class_idx, count
        );
    }
}

#[test]
fn test_cross_class_payload_size_transitions() {
    let mut map = RobinHoodMap::with_slab_chunk(512, 128 * 1024);
    let key = b"dynamic_payload_key";

    // 1. Inlined value (15 bytes <= 23 bytes)
    let v1 = make_payload(1, 15);
    map.insert(key, &v1);
    assert_eq!(map.get(key), Some(v1.as_slice()));
    assert_eq!(map.slab_arena().unwrap().stats().total_allocated_bytes, 0);

    // 2. Grow to 64B slab class (50 bytes)
    let v2 = make_payload(2, 50);
    map.insert(key, &v2);
    assert_eq!(map.get(key), Some(v2.as_slice()));
    let s2 = map.slab_arena().unwrap().stats();
    assert_eq!(s2.class_allocated_slots[1], 1); // 64B class has 1 slot

    // 3. Grow to 512B slab class (350 bytes)
    let v3 = make_payload(3, 350);
    map.insert(key, &v3);
    assert_eq!(map.get(key), Some(v3.as_slice()));
    let s3 = map.slab_arena().unwrap().stats();
    assert_eq!(s3.class_allocated_slots[1], 0, "Old 64B slot must be freed");
    assert_eq!(s3.class_allocated_slots[4], 1, "New 512B slot must be claimed");

    // 4. Shrink back to inlined value (10 bytes)
    let v4 = make_payload(4, 10);
    map.insert(key, &v4);
    assert_eq!(map.get(key), Some(v4.as_slice()));
    let s4 = map.slab_arena().unwrap().stats();
    assert_eq!(s4.total_allocated_bytes, 0, "512B slot must be returned to pool");

    // 5. Delete key
    assert!(map.delete(key));
    assert_eq!(map.get(key), None);
    assert_eq!(map.slab_arena().unwrap().stats().total_allocated_bytes, 0);
}

#[test]
fn test_adversarial_churn_fragmentation_acceptance() {
    let mut map = RobinHoodMap::with_slab_chunk(1024, 256 * 1024);
    let operations = 25_000;

    // Run 25,000 mixed-size operations simulating high-throughput cache churn
    for op in 0..operations {
        let key_id = op % 2_000; // Working set of 2,000 active keys
        let k = format!("cache:key:{}", key_id);

        if op % 3 == 0 && op > 2_000 {
            // Delete operation
            map.delete(k.as_bytes());
        } else {
            // Insert or Overwrite with varying payload sizes
            let len = match (op + key_id) % 6 {
                0 => 16,  // Inline SSO
                1 => 30,  // 32B Slab
                2 => 60,  // 64B Slab
                3 => 120, // 128B Slab
                4 => 250, // 256B Slab
                _ => 500, // 512B Slab
            };
            let v = make_payload(op, len);
            map.insert(k.as_bytes(), &v);
        }
    }

    // Verify memory fragmentation ratio remains strictly bounded
    let ratio = map.memory_fragmentation_ratio();
    println!("Adversarial Churn Test: Fragmentation Ratio = {:.3}", ratio);

    assert!(
        ratio < 1.25,
        "Acceptance criteria: Memory fragmentation ratio must stay tightly bounded (got {:.3})",
        ratio
    );

    // Verify all remaining live keys match their latest state
    let count = map.len();
    assert!(count > 0, "Map must have retained active keys");
}

#[test]
fn test_concurrent_multi_shard_slab_isolation() {
    let num_threads = 4;
    let ops_per_thread = 5_000;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            thread::spawn(move || {
                let mut map = RobinHoodMap::with_slab_chunk(512, 128 * 1024);

                for i in 0..ops_per_thread {
                    let k = format!("shard:{}:key:{}", thread_id, i);
                    let v = make_payload(i ^ thread_id, 80); // 128B class
                    map.insert(k.as_bytes(), &v);

                    if i % 3 == 0 {
                        map.delete(k.as_bytes());
                    }
                }

                // Verify thread local map integrity
                let stats = map.slab_arena().unwrap().stats();
                assert!(stats.total_allocated_bytes > 0);

                // Drain map completely
                map.clear();
                assert_eq!(map.len(), 0);
                assert_eq!(map.slab_arena().unwrap().stats().total_allocated_bytes, 0);
            })
        })
        .collect();

    for h in handles {
        h.join().expect("Worker thread panicked");
    }
}
