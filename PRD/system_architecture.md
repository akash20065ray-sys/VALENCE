# VALENCE: SYSTEM ARCHITECTURE SPECIFICATION
> **Document:** `PRD/system_architecture.md`  
> **Project:** Valence — Next-Generation In-Memory State Engine in Pure Rust  
> **Classification:** High-Performance Core Systems Architecture  

---

## 1. Architectural Overview & System Topology

Valence is engineered around a **shared-nothing, thread-per-core actor model** combined with decoupled asynchronous background subsystems. The system eliminates global locks on the request-processing hot path and bounds all foreground work.

```
                      +------------------------------------------+
                      |         NETWORK CLIENT LAYER (TCP)       |
                      |          (RESP3 Command Stream)          |
                      +------------------------------------------+
                                           |
                                           v
                      +------------------------------------------+
                      |          MURMUR3 SHARD ROUTER            |
                      |  shard_id = Murmur3(key) % num_cores     |
                      +------------------------------------------+
                                           |
         +---------------------------------+---------------------------------+
         | (Local Core 0)                                                    | (Cross-Core -> Core 1)
         v                                                                   v
+-------------------------------+                                 +-------------------------------+
|     CORE 0 WORKER SHARD       |                                 |   64B-ALIGNED SPSC RING QUEUE |
| - Pinned to CPU Core 0        |                                 | - Lock-Free Atomic Head/Tail  |
| - Robin Hood Flat Array       |                                 | - #[repr(align(64))] Padded   |
| - 4-Tier Timing Wheel (O(1))  |                                 +-------------------------------+
| - 2MB Hugepage Slab Arena     |                                                |
| - 2-Bit CLOCK Access Tracker  |                                                v
+-------------------------------+                                 +-------------------------------+
         |             |                                          |     CORE 1 WORKER SHARD       |
         |             |                                          | (Independent Shard State)     |
         |             +-----------------------+                  +-------------------------------+
         v                                     v
+-------------------------------+   +-------------------------------+
|  ASYNC RECLAMATION WORKER     |   |   ASYNCHRONOUS IO_URING TIER  |
| - Consumes detached buckets   |   | - 4KB Block Pages on NVMe     |
| - Batch-frees expired keys    |   | - Pre-registered Disk Buffers |
| - Zero foreground jitter      |   | - 75% Physical DRAM Cut       |
+-------------------------------+   +-------------------------------+
                                               ^
                                               |
                                    +-------------------------------+
                                    |    ZERO-FORK USERFAULTFD      |
                                    | - Epoch E0 Page Interception  |
                                    | - Temporary Scratchpad Copy   |
                                    | - Memory Doubling = 0%        |
                                    +-------------------------------+
```

---

## 2. Production Codebase File Structure

The project is structured as an idiomatic, production-grade Rust workspace designed for zero-cost abstractions, isolated unsafe boundaries, and comprehensive micro-benchmarking:

```
valence/
├── Cargo.toml                          # Multi-target configuration, release LTO, and hardware SIMD flags
├── README.md                           # Systems specification, benchmark plots, and visual CLI GIFs
├── benches/                            # Criterion performance benchmarks
│   ├── hash_bench.rs                   # Robin Hood open-addressing vs std::collections::HashMap probes
│   ├── timing_wheel_bench.rs           # O(1) slot insertion, deletion, and cascade traversal latencies
│   └── throughput_bench.rs             # Multi-core ops/sec scaling under high concurrent client load
├── tests/                              # Chaos and correctness integration test suites
│   ├── redis_compliance_test.rs        # RESP3 protocol conformance against official redis-cli
│   ├── avalanche_chaos.py              # 500k simultaneous key expiration burst stress harness
│   └── cow_memory_test.py              # Memory bloat verification during active snapshotting
├── src/
│   ├── lib.rs                          # Core crate root & internal API exports
│   ├── main.rs                         # Server daemon bootstrapper and OS signal handler
│   ├── bin/
│   │   ├── valence_server.rs           # Production server binary executable
│   │   └── valence_cli.rs              # Diagnostic inspection CLI with microsecond --explain tracer
│   ├── storage/                        # In-Memory Storage Engine
│   │   ├── mod.rs                      # Storage module interface & traits
│   │   ├── entry.rs                    # Slot layout, DIB distance counter, and tombstone-free states
│   │   └── robin_hood.rs               # Flat-array Robin Hood hash map with backshift deletion
│   ├── timer/                          # Expiration Subsystem
│   │   ├── mod.rs                      # Timer module interface
│   │   ├── slot.rs                     # Intrusive lock-free atomic bucket list for batch detachment
│   │   └── timing_wheel.rs             # 4-Tier Hierarchical Timing Wheel (ms -> sec -> min -> hr)
│   ├── memory/                         # Zero-Header Memory Arena
│   │   ├── mod.rs                      # Allocator module interface
│   │   ├── bitmap.rs                   # 64-bit trailing_zeros() single-cycle allocation bitfield
│   │   └── slab_arena.rs               # 2MB hugepage-aligned slab pools for discrete size classes
│   ├── sharding/                       # Shared-Nothing Multi-Core Architecture
│   │   ├── mod.rs                      # Sharding module interface
│   │   ├── affinity.rs                 # CPU topology detection and core pinning (pthread_setaffinity)
│   │   ├── router.rs                   # MurmurHash3 uniform keyspace partitioner
│   │   └── spsc_ring.rs                # 64-byte cache-line aligned lock-free SPSC inter-core ring buffer
│   ├── persistence/                    # Durability & Snapshots
│   │   ├── mod.rs                      # Persistence module interface
│   │   ├── wal.rs                      # Group-committed binary write-ahead log with CRC32 checksums
│   │   └── userfaultfd.rs              # Linux userfaultfd page-dirty intercept daemon (Zero-Fork)
│   ├── tiering/                        # Transparent DRAM/NVMe Tiering
│   │   ├── mod.rs                      # Tiering coordinator
│   │   ├── clock.rs                    # 2-bit CLOCK frequency tracker (Second-Chance FIFO)
│   │   └── io_uring.rs                 # Linux io_uring asynchronous completion ring for block paging
│   ├── protocol/                       # Wire Protocol & Vectorization
│   │   ├── mod.rs                      # Protocol encoder/decoder traits
│   │   ├── resp.rs                     # Zero-copy RESP3 parser for Redis command frames
│   │   └── simd.rs                     # 256-bit AVX2 vector delimiter scanner (_mm256_cmpeq_epi8)
│   └── server/                         # Network Coordination
│       ├── mod.rs                      # Server subsystem interface
│       ├── connection.rs               # Non-blocking TCP client session handler
│       └── engine.rs                   # Shard actor coordinator & background task dispatcher
```

---

## 3. Component Deep Dive & Mechanical Sympathy

### 3.1 In-Memory Storage: Flat-Array Robin Hood Hashing (`storage/`)
* **Memory Layout:** Unlike Redis's pointer-chained buckets, Valence stores slots in a single contiguous array in virtual memory:
  ```rust
  #[repr(C)]
  pub struct Slot<K, V> {
      pub hash: u64,
      pub dib: u8,          // Distance from Ideal Bucket
      pub state: SlotState, // Empty = 0, Occupied = 1
      pub key: K,
      pub value: V,
  }
  ```
* **Search Locality:** Contiguous layout guarantees that sequential probe hops fetch data within the same or adjacent 64-byte CPU cache lines.
* **Backshift Deletion:** When an item at index i is deleted, adjacent items at i+1, i+2 with DIB > 0 are shifted backward by 1 slot. This eliminates dead tombstones and preserves spatial locality.
* **SIMD Control Bytes (Swiss Table Path):** For maximum throughput, 1-byte control bytes store 7-bit hash fingerprints, scanned 16/32 slots per cycle via AVX2/SSE2 vector intrinsics.

### 3.2 Expiration Engine: 4-Tier Hierarchical Timing Wheel (`timer/`)
* **Wheel Dimensions:**
  * **Wheel 1 (Milliseconds):** 1,000 slots (1ms resolution, covers 0–999ms).
  * **Wheel 2 (Seconds):** 60 slots (1s resolution, covers 1–59s).
  * **Wheel 3 (Minutes):** 60 slots (1m resolution, covers 1–59m).
  * **Wheel 4 (Hours):** 24 slots (1h resolution, covers 1–24h).
* **Decoupled Batch Swap & Lazy Cascading:**
  ```rust
  // Executed on the foreground thread in <5 nanoseconds:
  let expired_batch = slot.head.swap(std::ptr::null_mut(), Ordering::AcqRel);
  // Handed to background reclamation channel without traversing the nodes:
  reclamation_sender.send(expired_batch);
  ```
* High-tier wheel cascading is executed lazily in small, bounded chunks off the foreground hot path.

### 3.3 Hardware Alignment & Inter-Core Ring Buffers (`sharding/`)
* **MESI Protocol & False Sharing:** If Core 0 writes to atomic variable `head` and Core 1 writes to atomic variable `tail` while both reside in the same 64-byte memory slice, the L3 cache controller continuously invalidates both cores' L1 caches.
* **Hardware Padding:**
  ```rust
  #[repr(align(64))]
  pub struct SpscRingBuffer<T> {
      pub head: AtomicU64,
      _pad1: [u8; 56], // Isolates head to its own 64-byte cache line

      pub tail: AtomicU64,
      _pad2: [u8; 56], // Isolates tail to its own 64-byte cache line

      pub buffer: Box<[UnsafeCell<Option<T>>]>,
  }
  ```

### 3.4 Zero-Fork Persistence via Epoch-Versioned Snapshots & Linux `userfaultfd` (`persistence/`)
* **Primary Path (User-Space Epoch Checkpointing with OnPreUpdate Hooks):**
  1. Each slot/entry carries a 64-bit atomic version counter.
  2. When snapshot initializes at `snapshot_epoch`, a background fiber streams entries with `version < snapshot_epoch` to disk via `io_uring`.
  3. If a live write targets an unpersisted entry, the worker's `OnPreUpdate` hook captures the old value into an asynchronous ring buffer before applying the update, bounding memory growth to <5% with zero syscalls.
* **Fallback Safety Path (Linux `userfaultfd` Write Protection):**
  1. Snapshot initialization at epoch E0 registers memory arena via `ioctl(uffd, UFFDIO_REGISTER, UFFDIO_REGISTER_MODE_WP)`.
  2. Write protection is enabled on the memory region: `ioctl(uffd, UFFDIO_WRITEPROTECT)`.
  3. Clean 4KB frames stream to disk; client writes to protected pages trap to `userfaultfd` reader, copying the 4KB frame and unprotecting in <3µs.

### 3.5 Asynchronous NVMe Tiering via `io_uring` & Log-Structured Extents (`tiering/`)
* **Ring Buffers:** Valence sets up Linux `io_uring` with 512 completion entries and `IORING_SETUP_SQPOLL`.
* **Log-Structured Extents:** Cold keys are batched in memory and written sequentially in 64KB/2MB extents to NVMe, ensuring a Write Amplification Factor near 1.0.
* **Registered Buffers:** Pre-registers memory pages with `IORING_REGISTER_BUFFERS` to avoid kernel page table translation on each disk operation.
* **2-Bit CLOCK Algorithm:** Second-chance FIFO tracks read frequency; keys reaching 0 are flushed to disk, leaving a compact 12-byte disk pointer in DRAM.

---

## 4. Module Boundary Invariants

1. **Isolated Unsafe Code:** Raw pointer manipulation and SIMD vector instructions are strictly quarantined to `memory/bitmap.rs` and `protocol/simd.rs`, with explicit `// SAFETY:` proofs.
2. **Zero-Allocation Hot Path:** Foreground command execution handlers in `server/engine.rs` must never invoke system heap allocation (`malloc`/`realloc`). All working memory is acquired from pre-allocated slab arenas.
3. **Bounded Foreground Time:** Foreground CPU processing for any operation is strictly bound to <1µs. All unbounded tasks (disk flushes, bulk memory reclamation, tiering demotions) are handled asynchronously.
