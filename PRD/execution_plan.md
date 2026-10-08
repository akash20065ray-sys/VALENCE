# VALENCE: MASTER EXECUTION PLAN
> **Document:** `PRD/execution_plan.md`  
> **Project:** Valence — Next-Generation In-Memory State Engine in Pure Rust  
> **Structure:** 3 Major Milestones | 8 Phases | 24 Atomic Sub-Phases  

---

## Roadmap Overview

```
[ MILESTONE 1: The Core In-Memory Engine ]      --> Weeks 1–3
  Phase 1: Robin Hood Hash Map & RESP3 Parser (Sub-phases 1.1 - 1.4)
  Phase 2: 4-Tier Hierarchical Timing Wheel   (Sub-phases 2.1 - 2.4)
  ★ MILESTONE WIN: Working database + First viral benchmark plot!

[ MILESTONE 2: Multi-Core Scale & Memory ]      --> Weeks 4–5
  Phase 3: Thread-per-Core Shards & SPSC Rings (Sub-phases 3.1 - 3.4)
  Phase 4: Region Slab Arenas & Bitmaps        (Sub-phases 4.1 - 4.3)
  ★ MILESTONE WIN: Millions of ops/sec with linear CPU scaling!

[ MILESTONE 3: Durability, Tiering & Polish ]   --> Weeks 6–8
  Phase 5: Durability (WAL) & Zero-Fork Snapshots (Sub-phases 5.1 - 5.3)
  Phase 6: Transparent NVMe Tiering (io_uring)    (Sub-phases 6.1 - 6.3)
  Phase 7: Hardware Acceleration (SIMD) & CLI     (Sub-phases 7.1 - 7.2)
  Phase 8: Comparative Benchmark Suite & Polish   (Sub-phases 8.1 - 8.2)
  ★ MILESTONE WIN: Tier-1 Staff-level open source portfolio!
```

---

# MILESTONE 1: The Core In-Memory Engine (Single-Core MVP)

### Phase 1: Robin Hood Hash Map & RESP3 Parser

#### Sub-Phase 1.1: Slot & Entry Layout Definition
* **Target File:** `src/storage/entry.rs`
* **Implementation Tasks:**
  * Define `Slot<K, V>` struct with memory alignment.
  * Fields: `hash: u64`, `dib: u8` (Distance from Ideal Bucket), `state: SlotState` (`Empty = 0`, `Occupied = 1`), `key: K`, `value: V`.
  * Implement helper methods: `is_empty()`, `is_occupied()`, `reset()`.
* **Verification Command:**
  ```bash
  cargo test storage::entry::tests
  ```
* **Acceptance Criteria:** Unit tests verify slot byte alignment and memory footprint without compiler padding bloat.

#### Sub-Phase 1.2: Robin Hood Flat-Array Storage Engine
* **Target File:** `src/storage/robin_hood.rs`
* **Implementation Tasks:**
  * Allocate a continuous contiguous vector of `Slot<Vec<u8>, Vec<u8>>`.
  * Implement Robin Hood insertion: probe slots, compare DIB, swap when incoming item has higher DIB.
  * Implement O(1) backshift deletion: shift adjacent collided slots backward, decrementing DIB; zero dead tombstones.
  * Implement lookup with early termination (stops when probed slot DIB is smaller than search DIB).
* **Verification Command:**
  ```bash
  cargo test storage::robin_hood::tests
  ```
* **Acceptance Criteria:** Insert 100,000 random keys with 50% collisions; verify 100% key-value retrieval and zero tombstones on deletions.

#### Sub-Phase 1.3: Zero-Copy RESP3 Protocol Parser
* **Target File:** `src/protocol/resp.rs`
* **Implementation Tasks:**
  * Implement zero-copy byte-slice parser for Redis RESP3 protocol frames.
  * Parse commands: `*3\r\n$3\r\nSET\r\n$4\r\nkey1\r\n$4\r\nval1\r\n`.
  * Support command types: `SET`, `GET`, `DEL`, `PING`, `EXPIRE`, `TTL`.
  * Return zero-heap-allocated `Command<'a>` enum over input byte buffers.
* **Verification Command:**
  ```bash
  cargo test protocol::resp::tests
  ```
* **Acceptance Criteria:** Parse 50,000 synthetic RESP command buffers; verify zero dynamic heap allocations on the hot path.

#### Sub-Phase 1.4: Non-Blocking Single-Threaded TCP Server
* **Target File:** `src/server/connection.rs` & `src/main.rs`
* **Implementation Tasks:**
  * Create a non-blocking TCP socket listener bound to `127.0.0.1:6379`.
  * Read bytes into static circular receive buffer, pass to RESP parser, execute on Robin Hood map, serialize response.
* **Verification Command:**
  ```bash
  cargo run --bin valence-server
  # In another terminal:
  redis-cli -p 6379 SET test_key "Hello_Valence"
  redis-cli -p 6379 GET test_key
  redis-benchmark -p 6379 -t get,set -n 100000 -q
  ```
* **Acceptance Criteria:** 100% protocol compliance on standard commands; sustains >1.2M ops/sec single-core in `redis-benchmark`.

---

### Phase 2: 4-Tier Hierarchical Timing Wheel (TTL Avalanche Solver)

#### Sub-Phase 2.1: Intrusive Atomic Slot Buckets
* **Target File:** `src/timer/slot.rs`
* **Implementation Tasks:**
  * Build a lock-free intrusive linked list slot bucket.
  * Implement atomic head detachment: swap `slot.head` with `std::ptr::null_mut()` using `Ordering::AcqRel`.
* **Verification Command:**
  ```bash
  cargo test timer::slot::tests
  ```
* **Acceptance Criteria:** 4 concurrent threads enqueue 50,000 expiration nodes while a consumer thread detaches batches; verify zero data races.

#### Sub-Phase 2.2: 4-Tier Cascaded Timing Wheel State Machine
* **Target File:** `src/timer/timing_wheel.rs`
* **Implementation Tasks:**
  * Wheel 1: 1,000 slots (1ms resolution, covers 0–999ms).
  * Wheel 2: 60 slots (1s resolution, covers 1–59s).
  * Wheel 3: 60 slots (1m resolution, covers 1–59m).
  * Wheel 4: 24 slots (1h resolution, covers 1–24h).
  * Implement O(1) scheduling bitmask arithmetic: `slot_idx = (target_ms >> shift) & mask`.
  * Implement tier cascade: higher-tier slots shift down into lower-tier slots as time advances.
* **Verification Command:**
  ```bash
  cargo test timer::timing_wheel::tests
  ```
* **Acceptance Criteria:** Schedule 20,000 timers ranging from 5ms to 3 hours; verify timers trigger within 1ms accuracy.

#### Sub-Phase 2.3: Decoupled Asynchronous Expiration Pipeline
* **Target File:** `src/server/engine.rs`
* **Implementation Tasks:**
  * Hook `SET ... EX <seconds>` and `EXPIRE` commands into the timing wheel.
  * On slot tick, foreground thread swaps bucket head and dispatches batch to a lock-free crossbeam channel.
  * Dedicated background worker drains the channel and frees memory.
* **Verification Command:**
  ```bash
  cargo test server::engine::tests
  ```
* **Acceptance Criteria:** Foreground CPU processing time per tick remains <5µs regardless of the number of expired keys in the bucket.

#### Sub-Phase 2.4: The Avalanche Chaos Benchmark
* **Target File:** `tests/avalanche_chaos.py`
* **Implementation Tasks:**
  * Python script inserting 500,000 keys with identical expiration timestamps.
  * Measure concurrent client read latency during the expiration second.
  * Run side-by-side against Redis 7.2 and Valence; export latency percentiles (p50, p99, p99.9).
* **Verification Command:**
  ```bash
  python tests/avalanche_chaos.py --host 127.0.0.1 --port 6379 --keys 500000
  ```
* **Acceptance Criteria:** Redis latency spikes to 450ms+; Valence p99 latency stays flat at <20µs. Plot generated and saved to `README.md`.

---

# MILESTONE 2: Multi-Core Scale & Zero-Header Memory

### Phase 3: Thread-per-Core Shards & Lock-Free SPSC Ring Buffers

#### Sub-Phase 3.1: CPU Topology Discovery & Core Pinning
* **Target File:** `src/sharding/affinity.rs`
* **Implementation Tasks:**
  * Detect physical CPU cores using `core_affinity`.
  * Spawn N worker threads and pin each to a physical CPU core (`pthread_setaffinity_np`).
* **Verification Command:**
  ```bash
  cargo test sharding::affinity::tests
  ```
* **Acceptance Criteria:** Process thread inspection verifies workers stay pinned to assigned CPU cores with zero thread migration.

#### Sub-Phase 3.2: Keyspace Consistent Hash Router
* **Target File:** `src/sharding/router.rs`
* **Implementation Tasks:**
  * Implement MurmurHash3 key routing: `shard_id = murmur3(key) % num_shards`.
  * Partition keys uniformly across shards with negligible hash skew.
* **Verification Command:**
  ```bash
  cargo test sharding::router::tests
  ```
* **Acceptance Criteria:** Route 1,000,000 keys; verify variance across shards is <1%.

#### Sub-Phase 3.3: 64-Byte Cache-Padded SPSC Ring Buffer
* **Target File:** `src/sharding/spsc_ring.rs`
* **Implementation Tasks:**
  * Build a circular lock-free Single-Producer Single-Consumer queue.
  * Enforce 64-byte alignment: `#[repr(align(64))] pub struct SpscRingBuffer` with 56-byte padding arrays around `AtomicU64` head/tail.
* **Verification Command:**
  ```bash
  cargo test sharding::spsc_ring::tests
  ```
* **Acceptance Criteria:** `perf c2c` confirms zero MESI CPU cache line invalidations between producer and consumer cores.

#### Sub-Phase 3.4: Actor Message Dispatch Coordinator
* **Target File:** `src/server/engine.rs`
* **Implementation Tasks:**
  * Worker thread polls both client network sockets and inbound SPSC queues.
  * Cross-shard requests route to the target core, execute against local Robin Hood map, and reply asynchronously.
* **Verification Command:**
  ```bash
  cargo bench --bench throughput_bench
  ```
* **Acceptance Criteria:** Demonstrates linear throughput scaling from 1 to 4 to 8 to 16 cores with zero mutex lock contention.

---

### Phase 4: Region-Based Slab Arenas (Zero-Header Allocation)

#### Sub-Phase 4.1: 64-Bit Bitmapped Allocator
* **Target File:** `src/memory/bitmap.rs`
* **Implementation Tasks:**
  * Manage 64 slots in a single `u64` bitfield.
  * Use CPU hardware intrinsic `trailing_zeros()` to allocate a slot in 1 clock cycle.
  * Free a slot using bitwise `AND NOT` in 1 clock cycle.
* **Verification Command:**
  ```bash
  cargo test memory::bitmap::tests
  ```
* **Acceptance Criteria:** 10,000,000 allocation/free cycles execute in <50ms; verify zero memory leaks.

#### Sub-Phase 4.2: 2MB Hugepage-Aligned Slab Pools
* **Target File:** `src/memory/slab_arena.rs`
* **Implementation Tasks:**
  * Pre-allocate 2MB memory blocks aligned to Linux Transparent Hugepage (THP) boundaries.
  * Partition into discrete slab classes: 32B, 64B, 128B, 256B, 512B.
* **Verification Command:**
  ```bash
  cargo test memory::slab_arena::tests
  ```
* **Acceptance Criteria:** Allocate records across size classes; verify all memory addresses align with 2MB hugepage boundaries.

#### Sub-Phase 4.3: Storage Engine Slab Integration
* **Target File:** `src/storage/robin_hood.rs`
* **Implementation Tasks:**
  * Connect Robin Hood hash table values to slab arena pointers instead of system heap.
  * Monitor operating system Resident Set Size (RSS) vs. logical memory size.
* **Verification Command:**
  ```bash
  cargo test storage::robin_hood::slab_integration
  ```
* **Acceptance Criteria:** Run 10 million random insert/delete operations; verify memory fragmentation ratio (RSS / Logical) stays strictly <1.05.

---

# MILESTONE 3: Kernel Durability, NVMe Tiering & Production Polish

### Phase 5: Durability (WAL) & Zero-Fork Snapshots (`userfaultfd`)

#### Sub-Phase 5.1: Pipelined Binary Write-Ahead Log
* **Target File:** `src/persistence/wal.rs`
* **Implementation Tasks:**
  * Append every mutating command to an on-disk binary log with group-committed `fdatasync`.
  * Calculate and embed CRC32 checksums on every record header.
  * Implement recovery replay on engine boot.
* **Verification Command:**
  ```bash
  cargo test persistence::wal::tests
  ```
* **Acceptance Criteria:** Simulate crash termination; verify engine restores 1,000,000 records in <15ms with zero state corruption.

#### Sub-Phase 5.2: Linux `userfaultfd` Page-Dirty Intercept Daemon
* **Target File:** `src/persistence/userfaultfd.rs`
* **Implementation Tasks:**
  * Register memory arena with Linux kernel `userfaultfd` write protection (`UFFDIO_WRITEPROTECT`).
  * Intercept page faults in user space: copy original 4KB frame to temporary scratchpad buffer, unprotect page, and unblock writer.
* **Verification Command:**
  ```bash
  cargo test persistence::userfaultfd::tests
  ```
* **Acceptance Criteria:** Live client writes continue uninterrupted during active write-protection with <3µs pause per modified page.

#### Sub-Phase 5.3: Snapshot Persistence Coordinator
* **Target File:** `src/persistence/snapshot.rs`
* **Implementation Tasks:**
  * Background thread streams clean pages to an `.rdb` binary snapshot on disk.
  * Coordinate with `userfaultfd` daemon to persist scratchpad buffers for mutated pages.
* **Verification Command:**
  ```bash
  python tests/cow_memory_test.py --host 127.0.0.1 --port 6379
  ```
* **Acceptance Criteria:** Snapshot a 20 GB database under 50k writes/sec; verify total memory bloat is <5% (zero OOM kills).

---

### Phase 6: Transparent NVMe Storage Tiering (`io_uring`)

#### Sub-Phase 6.1: 2-Bit CLOCK Access Frequency Tracker
* **Target File:** `src/tiering/clock.rs`
* **Implementation Tasks:**
  * Assign a 2-bit counter (0..3) to each hash map slot. Increment to 3 on read/write.
  * Background pointer sweeps the table, decrementing counts to identify cold records under Zipfian read distributions.
* **Verification Command:**
  ```bash
  cargo test tiering::clock::tests
  ```
* **Acceptance Criteria:** Under an 80/20 Zipfian workload, cold keys are reliably identified within 3 clock cycles.

#### Sub-Phase 6.2: Linux `io_uring` Asynchronous Block Paging
* **Target File:** `src/tiering/io_uring.rs`
* **Implementation Tasks:**
  * Initialize `io_uring` completion queues with pre-registered buffers (`IORING_REGISTER_BUFFERS`).
  * Flush cold keys to 4KB aligned NVMe disk blocks and replace in-memory record with a 12-byte disk pointer.
* **Verification Command:**
  ```bash
  cargo test tiering::io_uring::tests
  ```
* **Acceptance Criteria:** Asynchronous disk reads complete in 15µs - 30µs without blocking the worker thread.

#### Sub-Phase 6.3: Cold Read Cache Promotion
* **Target File:** `src/tiering/mod.rs`
* **Implementation Tasks:**
  * Frequently queried cold keys are automatically promoted back to the hot DRAM arena.
* **Verification Command:**
  ```bash
  cargo test tiering::cache_promotion
  ```
* **Acceptance Criteria:** Verify 70% physical DRAM reduction on cold workloads while hot keys maintain sub-microsecond access times.

---

### Phase 7: Hardware Acceleration (SIMD AVX2) & Diagnostic CLI

#### Sub-Phase 7.1: SIMD AVX2 Protocol Delimiter Scanner
* **Target File:** `src/protocol/simd.rs`
* **Implementation Tasks:**
  * Implement 256-bit vector scanning (`_mm256_cmpeq_epi8`) to search for `\r\n` delimiters 32 bytes in parallel.
* **Verification Command:**
  ```bash
  cargo test protocol::simd::tests
  ```
* **Acceptance Criteria:** Parser throughput exceeds 12,000,000 ops/sec per core in micro-benchmarks.

#### Sub-Phase 7.2: Diagnostic Terminal CLI Executable (`--explain`)
* **Target File:** `src/bin/valence_cli.rs`
* **Implementation Tasks:**
  * Build the `valence-cli <command> --explain` tool.
  * Render execution trace trees showing shard ID, core affinity, probe hops, timing wheel status, and latency profiles.
* **Verification Command:**
  ```bash
  cargo run --bin valence-cli -- get "user:session:100" --explain
  ```
* **Acceptance Criteria:** Outputs colored ASCII execution trace in terminal with microsecond breakdowns.

---

### Phase 8: Comparative Benchmark Suite & Open Source Portfolio

#### Sub-Phase 8.1: Automated Benchmark Suite
* **Target File:** `benches/suite.rs` & `tests/run_all_benches.sh`
* **Implementation Tasks:**
  * Script that spins up Redis 7.2, Dragonfly, and Valence, executing identical workloads.
  * Export throughput, p99 latency, and RSS memory usage to CSV/JSON.
* **Verification Command:**
  ```bash
  cargo bench
  ```
* **Acceptance Criteria:** Generates reproducible benchmark data with a single command.

#### Sub-Phase 8.2: Portfolio & GitHub Assets Polish
* **Target File:** `README.md`
* **Implementation Tasks:**
  * Add architecture diagrams, benchmark plots, 15-second terminal demo GIF, and placement whitepaper link.
* **Verification Command:**
  * Visual inspection of GitHub README and documentation.
* **Acceptance Criteria:** Complete repository ready for submission to campus recruiters and senior engineers.
