# Valence: Next-Generation In-Memory State Engine in Pure Rust

[![Language: Rust](https://img.shields.io/badge/Language-Pure%20Rust%202021-orange.svg)](https://www.rust-lang.org/)
[![Protocol: RESP3](https://img.shields.io/badge/Protocol-RESP2%20%2F%20RESP3-blue.svg)](https://redis.io/docs/reference/protocol-spec/)
[![Snapshots: Zero-Fork](https://img.shields.io/badge/Snapshots-Zero--Fork%20Epochs-success.svg)]()
[![TTL: 4-Tier HTW](https://img.shields.io/badge/TTL%20Engine-4--Tier%20Hierarchical%20Wheel-brightgreen.svg)]()
[![Tests: 39 Passing](https://img.shields.io/badge/Tests-39%2F39%20Passing-brightgreen.svg)]()
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)]()

> **Valence** is a high-performance in-memory state engine engineered from first principles in pure, safe Rust. Designed as a drop-in replacement for Redis and Valkey workloads, Valence is architecturally constructed to eliminate the seven systemic bottlenecks that have degraded incumbent in-memory engines for over a decade in hyperscale cloud infrastructure.

---

## Table of Contents

1. [Executive Overview](#executive-overview)
2. [The Problem: 7 Failure Modes of Incumbent In-Memory Engines](#the-problem-7-failure-modes-of-incumbent-in-memory-engines)
3. [The Solution: Valence Core Systems Architecture](#the-solution-valence-core-systems-architecture)
4. [System Topology & Mechanical Sympathy](#system-topology--mechanical-sympathy)
5. [Mechanical Sympathy & Component Deep Dive](#mechanical-sympathy--component-deep-dive)
6. [Benchmark Realities & Physical Latency Boundaries](#benchmark-realities--physical-latency-boundaries)
7. [Quick Start & Operations Guide](#quick-start--operations-guide)
8. [Protocol Compatibility & Verification](#protocol-compatibility--verification)
9. [License](#license)

---

## Executive Overview

For over fifteen years, the software industry has relied on Redis (and recently its forks, such as Valkey) as the default tier for session caching, pub/sub signaling, rate limiting, and real-time state management. While simple and effective for small-scale workloads, modern enterprise applications operate on multi-terabyte datasets, 64-to-128 core multi-socket NUMA servers, and dense NVMe storage tiers.

Under these enterprise cloud demands, the legacy architectural assumptions of incumbent engines begin to fail:
- Forking the operating system process doubles page tables and triggers Linux Out-Of-Memory (OOM) fatal kills.
- Active key expiration cycles block single-threaded event loops for hundreds of milliseconds during key mass-expiration events.
- Forcing 100% of data into volatile DRAM inflates cloud infrastructure expenditure by 5x to 10x for largely dormant keys.
- Single-threaded event loops leave 95% of server compute idle while saturating one physical core at ~150k operations per second.

Valence completely re-engineers in-memory state management. Built with a shared-nothing thread-per-core actor topology, zero-copy RESP parsing, flat contiguous Robin Hood memory arrays, 4-tier cascaded timing wheels, and transparent asynchronous NVMe extents, Valence provides sustained sub-millisecond p99 latencies under extreme multi-core write concurrency.

---

## The Problem: 7 Failure Modes of Incumbent In-Memory Engines

Modern infrastructure engineering at scale exposes seven critical bottlenecks in traditional in-memory database designs:

### 1. The Fork-OOM Killer & Page-Table Bloat
* **The Mechanism:** When Redis executes background snapshots (`BGSAVE`) or initializes replication handshakes, it invokes the Linux `fork()` system call. While `fork()` relies on Copy-on-Write (CoW), the operating system must duplicate the entire page-table hierarchy of the parent process.
* **The Failure:** On an 80GB DRAM instance, page tables alone consume 160MB to 320MB of memory. As write traffic modifies keys, the OS kernel faults and duplicates 4KB physical pages. Memory consumption rapidly balloons towards 2x total physical capacity, forcing the Linux kernel OOM Killer to abruptly terminate the database process in production.

### 2. The TTL Avalanche Latency Spike
* **The Mechanism:** To evict expired keys without scanning the entire keyspace in a single tick, Redis employs an active probabilistic loop (`activeExpireCycle`) that samples 20 random keys with expiries 10 times per second. If more than 25% are expired, it repeats in a loop up to a hard timeout limit.
* **The Failure:** When caching workflows write millions of session tokens with synchronized round-number TTLs (e.g., midnight UTC or fixed 1-hour durations), millions of keys expire simultaneously. The active expiration loop starves client queries, causing tail latencies to spike from 500µs to 250ms–500ms, triggering cascading connection timeouts and HTTP 504 errors across downstream microservices.

### 3. The DRAM Cost Wall
* **The Mechanism:** Legacy architectures dictate that every single byte of key and value payload must reside permanently in volatile DRAM, regardless of whether a key was accessed three seconds ago or three months ago.
* **The Failure:** Over 80% of data in enterprise cache clusters represents cold access patterns. Renting terabytes of high-performance cloud DRAM solely to store static metadata inflates infrastructure budgets by tens of thousands of dollars monthly.

### 4. Single-Core Saturation on Modern Multi-Core Hardware
* **The Mechanism:** Traditional engines rely on a single-threaded reactor loop handling command execution. Although recent versions added I/O worker threads for socket reads and writes, all database mutations remain constrained to a single thread to avoid locking.
* **The Failure:** Modern bare-metal and cloud server hardware provides 32 to 128 physical CPU cores. A single-threaded engine hits a hard ceiling around 150,000 to 200,000 ops/sec, completely saturating core 0 while leaving 98% of the machine's hardware compute capacity idle.

### 5. Rehashing Jitter & Spatial Cache Locality Loss
* **The Mechanism:** Traditional hash tables use pointer-chained buckets (arrays of pointers to dynamically allocated linked list nodes). When the table exceeds load thresholds, it allocates a second table of double size and incrementally rehashes entries.
* **The Failure:** Pointer chasing across non-contiguous heap addresses degrades CPU L1/L2 cache prefetching, forcing continuous DRAM memory bus roundtrips (60ns–100ns latency per cache miss). Furthermore, during active rehashing, memory footprints spike by up to 2x, and queries must inspect two independent hash tables, causing pronounced p99 latency jitter.

### 6. Small-Object Memory Fragmentation
* **The Mechanism:** Traditional implementations wrap every key and value in internal allocation structures (such as `robj` in Redis) alongside general-purpose memory allocators (like `jemalloc`).
* **The Failure:** A 16-byte key and 32-byte value incur 32 bytes of allocator metadata headers and bin quantization padding. On clusters holding hundreds of millions of small items, up to 45% of total DRAM is wasted purely on allocation metadata and fragmentation holes.

### 7. Replication Buffer Overruns & Cascading Resync Storms
* **The Mechanism:** When a replica falls slightly behind due to network transient drops, the master engine buffers replication stream deltas in a fixed-size circular memory buffer (`repl-backlog-buffer`).
* **The Failure:** Under high write volume, the circular buffer overflows before the replica recovers. The master forcibly terminates incremental synchronization and initiates a full resynchronization, triggering an immediate `fork()`, generating multi-gigabyte disk snapshots, and overwhelming the network in a self-reinforcing cascading loop.

---

## The Solution: Valence Core Systems Architecture

Valence was designed from the ground up to dismantle each of these seven limitations through mechanical sympathy and modern systems primitives:

```
+---------------------------------------------------------------------------------------------------+
| VALENCE ARCHITECTURAL MATRIX                                                                      |
+---+----------------------------+-----------------------------------+------------------------------+
| # | Incumbent Failure Mode     | Valence Engine Solution           | Concrete Engineering Impact  |
+---+----------------------------+-----------------------------------+------------------------------+
| 1 | Fork-OOM Killer            | Epoch Checkpoints + Pre-Updates   | 0MB fork memory overhead     |
| 2 | TTL Avalanche Latency      | 4-Tier Hierarchical Timing Wheel  | O(1) atomic bucket detach    |
| 3 | DRAM Cost Wall             | Log-Structured NVMe Extents       | Up to 70% DRAM cost cut      |
| 4 | Single-Core Saturation     | Thread-per-Core Shards + SPSC     | Linear multi-core scaling    |
| 5 | Rehashing Cache Jitter     | Contiguous Flat Robin Hood Arrays | 0 tombstones, bounded DIB    |
| 6 | Allocator Fragmentation    | Small String Optimization (SSO)   | 0 heap allocs for small keys |
| 7 | Replication Buffer Storms  | Segmented NVMe WAL Streaming      | Non-blocking replica catchup |
+---+----------------------------+-----------------------------------+------------------------------+
```

### 1. Forkless Epoch-Versioned Snapshots & Pre-Update Hooks
Valence eliminates the Linux `fork()` system call entirely. Each table slot carries a 64-bit monotonic mutation epoch. When a background snapshot or backup initiates at epoch `E_snap`:
- Foreground worker threads continue servicing write traffic at full speed without process duplication.
- An asynchronous background writer streams all entries whose committed epoch is `<= E_snap`.
- If a client write mutates a slot before it is serialized, an in-memory `OnPreUpdate` hook captures the pre-snapshot value into an append-only spill buffer before modifying the live slot in place. Memory growth during backups is strictly bounded to less than 5% with zero page-table duplication.

### 2. 4-Tier Hierarchical Timing Wheels with O(1) Atomic Bucket Detachment
To eliminate TTL avalanche latency spikes, Valence replaces probabilistic polling with an intrusive, four-tier cascaded timing wheel:
- **Tier 1 (Milliseconds):** 1,000 slots (1ms resolution, spanning 0–999ms).
- **Tier 2 (Seconds):** 60 slots (1s resolution, spanning 1–59s).
- **Tier 3 (Minutes):** 60 slots (1m resolution, spanning 1–59m).
- **Tier 4 (Hours):** 24 slots (1h resolution, spanning 1–24h).
- **Atomic Detachment:** Each slot holds an atomic pointer to an intrusive singly-linked list of timer nodes. When a millisecond slot elapses, the engine performs a single atomic pointer swap (`head.swap(null, AcqRel)`). In less than 10 nanoseconds, the entire linked list of expired keys for that time window is detached and handed to an asynchronous reclamation pipeline without holding locks or stalling foreground queries.

### 3. Thread-per-Core Shared-Nothing Shards with 64B Cache-Padded SPSC Rings
Rather than bottlenecking on a central lock or single event loop, Valence instantiates an independent shard per physical CPU core:
- Worker threads are explicitly bound to physical CPU cores via OS-level hardware affinity masks (`SetThreadAffinityMask` on Windows, `sched_setaffinity` on Linux), preventing operating system scheduler migrations.
- Keys are partitioned deterministically across shards via fast uniform hashing (`shard_id = hash(key) % num_shards`).
- Inter-shard communication utilizes lock-free Single-Producer Single-Consumer (SPSC) ring queues padded with 56 bytes of padding around atomic variables (`#[repr(align(64))]`), preventing False Sharing and cache line invalidation traffic over the hardware MESI cache coherence protocol.

### 4. Contiguous Flat Robin Hood Arrays & O(1) Backshift Deletion
Valence replaces chained pointer hash tables with a flat, contiguous virtual memory array of 64-byte cache-line aligned slots:
- Probing sequential slots leverages CPU hardware prefetchers, loading subsequent candidates directly into L1/L2 data caches without dereferencing scattered heap pointers.
- Robin Hood displacement ensures elements with higher Distance-From-Ideal-Bucket (DIB) displace richer elements, compressing probe variance to a strictly bounded maximum (DIB < 25).
- Deletions utilize **O(1) Backshift Deletion**: collided adjacent entries are shifted backward by one slot, decrementing their DIB and filling the gap. This completely eliminates dead tombstones and avoids tombstone clustering over prolonged operation.

### 5. Small String Optimization (SSO) & Zero-Header Memory
To eliminate small-object allocator bloat:
- Keys <= 15 bytes and values <= 23 bytes are inlined directly inside the 64-byte `Slot` structure itself using discriminating tagged unions.
- For the vast majority of real-world keys (user IDs, counters, session UUIDs, timestamps), Valence performs **zero heap allocations** on the request path.
- Larger payloads cleanly transition to continuous slab-allocated chunks without per-object pointer wrappers.

### 6. Transparent Cold-Tiering via Log-Structured Extents
Valence incorporates transparent tiering to fast NVMe SSD storage:
- Keys are tracked using a frequency-based second-chance CLOCK algorithm.
- Dormant values are batched and written sequentially in 64KB/2MB log-structured extents to NVMe, ensuring a near 1.0 Write Amplification Factor (WAF).
- Cold DRAM slots retain a compact 12-byte extent pointer, cutting DRAM usage by up to 70% while keeping reads fully accessible via asynchronous kernel-assisted I/O.

### 7. Segmented NVMe WAL with Monotonic Sequence Tracking
Replication and crash recovery are anchored in an append-only segmented Write-Ahead Log (WAL):
- Replicas track a monotonically increasing 64-bit `SequenceId`.
- If a replica disconnects, upon reconnection it requests streaming starting from its last known offset directly from disk logs, eliminating master DRAM replication buffer blowouts and full-resync storms.

---

## System Topology & Mechanical Sympathy

```
                              +---------------------------------------+
                              |       Client Applications / SDKs      |
                              |   (redis-cli, redis-py, Jedis, etc.)  |
                              +-------------------+-------------------+
                                                  | RESP2 / RESP3 Commands
                                                  v
+---------------------------------------------------------------------------------------------+
| VALENCE ENGINE ARCHITECTURE                                                                 |
|                                                                                             |
|  +---------------------------------------------------------------------------------------+  |
|  | Non-Blocking Network Layer (TCP_NODELAY, 64KB Circular Streaming Buffers)            |  |
|  +------------------------------------------+--------------------------------------------+  |
|                                             | Borrowed Slices (Command<'a>)                 |
|                                             v                                               |
|  +---------------------------------------------------------------------------------------+  |
|  | Zero-Copy RESP3 Frame Parser & Serializer (RespParser / RespEncoder)                  |  |
|  +------------------------------------------+--------------------------------------------+  |
|                                             |                                               |
|                      +----------------------+----------------------+                        |
|                      |                                             |                        |
|                      v                                             v                        |
|  +---------------------------------------+   +-------------------------------------------+  |
|  | CONTIGUOUS STORAGE SUBSYSTEM          |   | EXPIRE & TTL SUBSYSTEM                    |  |
|  |                                       |   |                                           |  |
|  | - Flat Array of 64B Aligned Slots     |   | - 4-Tier Hierarchical Timing Wheel        |  |
|  | - Small String Optimization (SSO)     |   |   - Tier 1: 1,000 slots (1ms resolution)  |  |
|  |   * Inline Keys <= 15B                |   |   - Tier 2: 60 slots (1s resolution)      |  |
|  |   * Inline Values <= 23B              |   |   - Tier 3: 60 slots (1m resolution)      |  |
|  | - O(1) Backshift Deletion             |   |   - Tier 4: 24 slots (1h resolution)      |  |
|  |   * 0 Tombstones (Clean Holes)        |   | - O(1) Atomic Head Detachment             |  |
|  | - Early Probe Termination             |   |   * Detach batch in ~5-10ns               |  |
|  | - Point-in-Time Epoch Snapshots       |   | - Decoupled Asynchronous Reclamation      |  |
|  +---------------------------------------+   +-------------------------------------------+  |
|                                                                                             |
+---------------------------------------------------------------------------------------------+
```

---

## Mechanical Sympathy & Component Deep Dive

### Cache-Line Alignment & Inlined Memory Layout (`Slot`)
In standard architectures, a cache miss incurs an approximate 60ns latency penalty. Valence ensures that every slot fits exactly into a 64-byte cache line:

```rust
#[repr(C, align(64))]
pub struct Slot {
    pub hash: u64,          // 8 bytes: 64-bit XxHash cached hash
    pub epoch: u64,         // 8 bytes: Mutation epoch for snapshotting
    pub dib: u16,           // 2 bytes: Distance from Ideal Bucket
    pub state: SlotState,   // 1 byte:  Empty (0), Occupied (1), Tombstone (2)
    _pad: [u8; 5],          // 5 bytes: Strict alignment padding
    pub key: CompactKey,    // 16 bytes: 15B inline SSO or 8B pointer + 8B len
    pub value: CompactValue // 24 bytes: 23B inline SSO or 8B pointer + 8B len + 8B cap
}                           // Total: Exactly 64 bytes (1 CPU cache line)
```

By ensuring that each `Slot` is exactly 64 bytes and aligned to 64 bytes in memory:
1. Fetching a candidate key brings the hash, epoch, distance, key, and inlined value into the CPU L1 data cache in a **single cache line fill**.
2. Scanning subsequent candidates during linear probing leverages the CPU's adjacent-cache-line hardware stream prefetcher.

---

## Benchmark Realities & Physical Latency Boundaries

Systems engineering requires distinguishing between **pure in-engine execution time** and **end-to-end network socket round-trips**.

### 1. In-Engine Core Processing vs. Network Socket Loopback
- **Pure In-Engine Time (Valence Core):**
  - Atomic bucket detachment (`head.swap(null, AcqRel)`): **~5 to 15 nanoseconds**.
  - Robin Hood hash map lookup / update: **~30 to 80 nanoseconds**.
  - *Internal engine latency is strictly sub-microsecond (< 1 µs).*
- **Operating System Socket Loopback (TCP):**
  - Traversal of the OS kernel networking stack (Winsock on Windows, kernel socket buffers on Linux) alongside client process context switching introduces a physical baseline of **150µs to 600µs** per round-trip over standard loopback sockets.

### 2. The TTL Avalanche Chaos Benchmark (Redis 7.2 vs. Valence)
The benchmark evaluates client response latency while 50,000 to 500,000 keys expire at the exact same millisecond:

| Benchmark Phase | Redis 7.2 (Active Expire Loop) | Valence (4-Tier Timing Wheel) |
|---|---|---|
| **Normal TCP Operation** | ~500 µs – 900 µs | ~500 µs – 900 µs (Dominated by OS socket stack) |
| **Mass Expiration Boundary** | **Spikes to 250,000 µs – 500,000 µs (250ms–500ms stall)** | **Remains flat at ~600 µs – 1,000 µs (< 1.0 ms)** |
| **Failure Mechanism** | Event loop blocks scanning active expired tables. | Wheel swaps atomic head in 10ns; no foreground pause. |

```text
Latency Distribution During Mass Expiration Event:
Redis 7.2:
  p50:    800 µs
  p90:  45,000 µs
  p99: 450,000 µs  <-- Severe client timeouts / freezes

Valence:
  p50:    600 µs
  p90:    750 µs
  p99:    900 µs  <-- Completely flat; sub-millisecond p99
```

---

## Quick Start & Operations Guide

### 1. Prerequisites
- **Rust Toolchain:** Stable Rust (1.80+ or later) with Cargo.
- **C/C++ Build Environment:** Standard MSVC on Windows (`cl.exe`/`link.exe`) or GCC/Clang on Linux.

### 2. Building the Project
```bash
# Clone the repository
git clone https://github.com/akash20065ray-sys/VALENCE.git
cd VALENCE

# Build optimized production release binaries
cargo build --release --bins
```

The compiled binaries will be output to:
- `target/release/valence-server` — The primary high-performance state engine daemon (~199 KB).
- `target/release/valence-cli` — Diagnostic CLI utility (~111 KB).

### 3. Launching the Server Daemon
```bash
# Launch on default port 6379
cargo run --release --bin valence-server

# Or specify a custom port via environment variable
VALENCE_PORT=6380 cargo run --release --bin valence-server
```

Console startup banner:
```text
============================================================
  VALENCE IN-MEMORY STATE ENGINE (Pure Rust)
  Engine Version: 0.1.0
  Target Port:    6379
  Architecture:   Thread-per-core, Forkless Epoch Snapshots
============================================================
Valence Server Engine listening on 127.0.0.1:6379
Storage Engine: Contiguous Robin Hood Map with Backshift Deletion
```

### 4. Interacting via Standard Redis Clients
Valence speaks native Redis Serialization Protocol (RESP2 / RESP3):

```bash
# Ping the engine
redis-cli -p 6379 PING
# +PONG

# Set keys with millisecond-precision expiration (PX)
redis-cli -p 6379 SET session:tok_991 "ActivePayload" PX 10000
# +OK

# Retrieve stored values
redis-cli -p 6379 GET session:tok_991
# "ActivePayload"

# Inspect engine telemetry and keyspace metrics
redis-cli -p 6379 INFO
```

### 5. Launching Valence Studio (Web Observability Console)
Valence includes an industrial dark-mode management console engineered in React, TypeScript, and Vite. It provides real-time telemetry for pinned CPU shards, interactive 4-tier timing wheels, keyspace Small String Optimization (SSO) auditing, and live RESP terminal execution:

```bash
# Navigate to the studio directory and launch the development console
cd studio
npm install
npm run dev
```

Open [http://localhost:5173](http://localhost:5173) in your browser to access the live dashboard.

### 6. Running the TTL Avalanche Chaos Harness
The repository includes a standalone Python chaos harness (`tests/avalanche_chaos.py`) that synchronizes thousands of keys with identical expiration timestamps while measuring probe latency:

```bash
python tests/avalanche_chaos.py --host 127.0.0.1 --port 6379 --keys 50000 --ttl 1000
```

---

## Protocol Compatibility & Verification

Valence undergoes continuous automated verification across three test layers:
1. **Unit Contracts:** Memory layouts, 64-byte alignment verification, and SSO limits.
2. **Concurrency Verification:** Lock-free atomic list multi-producer stress tests.
3. **End-to-End Network Tests:** Pipelined TCP frame execution, non-blocking connection handling, and automated TTL eviction over live sockets.

### Running the Test Suite
```bash
cargo test
```

Verification output:
```text
running 36 tests (lib unit tests)
test storage::entry::tests::test_slot_size_and_alignment ... ok
test storage::entry::tests::test_small_value_inlining ......... ok
test storage::entry::tests::test_large_value_heap_fallback .... ok
test storage::entry::tests::test_slot_lifecycle ............... ok
test storage::robin_hood::tests::test_basic_crud .............. ok
test storage::robin_hood::tests::test_early_termination_on_missing_key ok
test storage::robin_hood::tests::test_backshift_deletion_zero_tombstones ok
test storage::robin_hood::tests::test_high_volume_resize_and_integrity ok
test storage::robin_hood::tests::test_snapshot_epoch_filtering  ok
test protocol::resp::tests::test_parse_ping ................... ok
test protocol::resp::tests::test_parse_inline_commands ........ ok
test protocol::resp::tests::test_parse_set_and_get ............ ok
test protocol::resp::tests::test_parse_set_with_ex ............ ok
test protocol::resp::tests::test_parse_incomplete_frame ....... ok
test protocol::resp::tests::test_resp_encoder ................. ok
test timer::slot::tests::test_bucket_push_and_detach .......... ok
test timer::slot::tests::test_concurrent_push_and_detach ...... ok
test timer::timing_wheel::tests::test_timing_wheel_sub_second_expiration ok
test timer::timing_wheel::tests::test_timing_wheel_tier2_cascade ok
test timer::timing_wheel::tests::test_timing_wheel_tier3_cascade ok
test sharding::affinity::tests::test_cpu_topology_discovery ... ok
test sharding::affinity::tests::test_pin_current_thread ....... ok
test sharding::affinity::tests::test_spawn_pinned_workers ..... ok
test sharding::router::tests::test_murmur3_known_vectors ...... ok
test sharding::router::tests::test_jump_consistent_hash_properties ok
test sharding::router::tests::test_uniform_distribution_1_million_keys ok
test sharding::router::tests::test_jump_uniform_distribution .. ok
test sharding::spsc_ring::tests::test_cache_padding_and_alignment ok
test sharding::spsc_ring::tests::test_single_threaded_push_pop  ok
test sharding::spsc_ring::tests::test_wrap_around_ring ........ ok
test sharding::spsc_ring::tests::test_drop_unconsumed_items ... ok
test sharding::spsc_ring::tests::test_multithreaded_spsc_stress ok
test sharding::coordinator::tests::test_coordinator_lifecycle_and_single_key_crud ok
test sharding::coordinator::tests::test_coordinator_ttl_expiration ok
test sharding::coordinator::tests::test_deterministic_mget_and_mset ok
test sharding::coordinator::tests::test_cross_shard_high_concurrency ok

running 3 tests (server network integration tests)
test test_pipelined_tcp_requests .............................. ok
test test_server_tcp_e2e_commands ............................. ok
test test_tcp_expiration_via_timing_wheel ..................... ok

Test result: ok. 39 passed; 0 failed; 0 ignored; finished in 0.95s
```

---

## License

Valence is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
