# VALENCE: BRAIN & CORE MENTAL MODEL
> **Document:** `PRD/brain.md`  
> **Project:** Valence — Next-Generation In-Memory State Engine in Pure Rust  
> **Core Stack:** 100% Pure Rust + Linux `io_uring` + `userfaultfd` + Atomics  
> **Target Domain:** Core Systems, High-Performance Storage & Cloud Infrastructure  

---

## 1. Executive Summary & Project Purpose

Modern web services rely on in-memory datastores (Redis, Memcached, Valkey) to serve sub-millisecond responses for user sessions, rate limiters, leaderboards, real-time caches, and transactional locks. By keeping data in physical RAM, these systems bypass slow disk I/O.

### The Production Crisis
While Redis excels on synthetic read-heavy benchmarks, real-world enterprise deployments encounter seven fundamental operational bottlenecks:
1. **The Fork-OOM Killer:** Memory doubling during `BGSAVE` snapshots caused by Linux Copy-on-Write (COW).
2. **The TTL Avalanche:** Event loop freezes when 500,000 keys expire at the exact same millisecond.
3. **The DRAM Cost Wall:** 80% of data is cold, yet sits uncompressed in expensive physical RAM.
4. **The Single-Core Wall:** 1 CPU core runs at 100% while 63 cores sit completely idle.
5. **Rehashing Probe Jitter:** Chained hash tables cause pointer-chasing CPU cache misses and probe duplication during resizing.
6. **Small-Object Heap Fragmentation:** Per-object headers (`robj` + SDS) waste up to 50% of memory in allocator gaps.
7. **Replication Overruns:** Replication backlog buffer wrap-around triggers catastrophic Full Resynchronization network storms.

**Valence** is an experimental state engine engineered from first principles in Pure Rust to systematically resolve all seven bottlenecks.

---

## 2. The Core Architectural Invariant (The Golden Rule)

> **"Foreground request processing must never perform unbounded persistence, expiration, eviction, or memory reorganization work. Every expensive operational task is bounded or decoupled into asynchronous background workers."**

* In Redis, the foreground request thread gets hijacked to copy page tables (persistence), loop through expired keys (expiration), and search for keys to delete (eviction).
* In Valence, the request loop only ever performs **strictly bounded O(1) CPU work**. Heavy operations are pushed to dedicated asynchronous workers.

---

## 3. How Redis Operates Under the Hood (Source Code Level)

To diagnose why incumbent systems fail, we trace the physical execution path of requests and data structures inside Redis source code:

| Subsystem | Source File | How Redis Implements It | Where It Breaks in Production |
| :--- | :--- | :--- | :--- |
| **I/O Reactor** | `src/ae.c` | Single-threaded `epoll_wait()` loop reading commands into client query buffers (`querybuf`). | Only scales to 1 CPU core; long operations freeze all 10,000 connected clients. |
| **Object Wrappers** | `src/server.h`, `src/sds.c` | Wraps every key/value in a 16-byte `robj` pointing to a dynamic SDS string (`sdshdr`). | Storing a 4-byte integer consumes over 56 bytes in RAM after allocator padding. |
| **Hash Table** | `src/dict.c` | Chained buckets with linked lists (`dictEntry`). Incremental rehashing across two tables (`ht_table[0]` and `ht_table[1]`). | Pointer chasing causes continuous L1/L2 cache misses; probes both tables during resizing. |
| **Expiration** | `src/expire.c` | Passive check on read + active sampling loop (`activeExpireCycle`) sampling 20 keys 10 times/sec. | If >25% of sampled keys are expired, it loops aggressively, stalling foreground requests for 400ms+. |
| **Snapshots** | `src/rdb.c` | Calls POSIX `fork()` to spawn a child process; relies on OS Copy-On-Write (COW). | Live writes trigger page faults, duplicating 4KB physical pages. Memory doubles (30GB to 60GB), causing OOM kills. |
| **Replication** | `src/replication.c` | Fixed circular in-memory buffer (`server.repl_backlog`, 1MB–64MB). | If a replica disconnects for 10s under heavy writes, buffer wraps around, forcing a 30GB Full Sync storm. |

---

## 4. The 7 Production Problems & Systematic Solutions

### Problem 1: BGSAVE Fork & Copy-on-Write Memory Doubling (OOM Killer)
* **The Failure:** `fork()` clones page tables (pausing the thread for 50–200ms on a 30GB instance). Live writes trigger Linux Trap 14 (page fault), duplicating 4KB frames. RAM doubles from 30GB to 60GB, triggering the Linux kernel OOM killer (`SIGKILL`).
* **The Solution (Forkless Epoch-Versioned Snapshots with Pre-Update Hooks & Overflow Disk Spill):** Valence completely eliminates POSIX `fork()`. During snapshots, entries carry an atomic mutation epoch. Background fibers stream entries to disk via `io_uring`. If a client writes to an entry before it has been serialized (`entry.epoch <= snapshot_epoch`), an `OnPreUpdate` hook captures the old value into an in-memory shadow ring buffer. If write pressure saturates the shadow ring, excess records spill sequentially to an append-only NVMe overflow log, ensuring snapshot correctness never depends on RAM buffer capacity.
* **Formal Snapshot Invariant:** *"For every key existing at snapshot epoch E, either the live representation remains the E-version until serialization, or its pre-E version is retained until the snapshot has durably captured it."*

### Problem 2: The TTL Avalanche & Active-Expire Latency Freeze
* **The Failure:** 500,000 keys expire at midnight. `activeExpireCycle()` detects >25% expired density and loops continuously. The single thread spends 500ms deleting keys instead of serving sockets. Microservices hit timeout cascades.
* **The Solution (4-Tier Hierarchical Timing Wheels with Atomic Bucket Swapping):** Expirations are organized into 4 circular wheels (1,000 ms, 60 sec, 60 min, 24 hr) with O(1) bitmask math. When a slot ticks, Valence executes an atomic pointer swap to detach the expired bucket and offloads deallocation to a dedicated background channel. High-tier cascading is handled lazily in bounded chunks. Foreground p99 latency stays flat at <20µs.

### Problem 3: The DRAM Cost Wall & Cold Data Bloat
* **The Failure:** In real workloads, 20% of keys get 80% of reads (Zipfian law). The other 80% sit unread for days. Redis forces 100% of data to live in physical DRAM ($150/100GB vs $5/100GB for NVMe SSD), wasting thousands of dollars monthly.
* **The Solution (Transparent NVMe Tiering via Log-Structured Extents & io_uring):** Inactive keys identified by a 2-bit CLOCK algorithm are batched into contiguous 64KB/2MB log-structured extents and written sequentially to NVMe, leaving a compact 12-byte disk fat-pointer in RAM. Cold reads are fetched asynchronously via Linux `io_uring` completion queues without blocking the worker thread. Minimizes flash write amplification and cuts DRAM costs by 75%.

### Problem 4: Single-Threaded Core Saturation Wall
* **The Failure:** Redis runs on a single core. Core 0 maxes out at 100% CPU while Cores 1–63 sit idle. Wrapping tables in mutexes (KeyDB) causes severe lock contention and `futex_wait` kernel stalls.
* **The Solution (Thread-per-Core Shards with Cache Alignment):** One worker thread is pinned to each physical CPU core (`pthread_setaffinity_np`). Each core owns its own hash map with zero shared locks (Strict Single Ownership). Cross-core requests travel through lock-free SPSC ring buffers padded to 64-byte boundaries (`#[repr(align(64))]`), preventing MESI CPU false sharing. Multi-key commands use coordinator WAL transaction fencing to guarantee deadlock freedom and crash consistency.

### Problem 5: Incremental Rehashing Latency Jitter & Cache Misses
* **The Failure:** Chained hash tables scatter `dictEntry` pointers across the heap, incurring ~80ns DRAM cache misses per probe. Resizing maintains two tables, doubling probe depth for millions of operations.
* **The Solution (Swiss Table SIMD Filtering & Segmented Resizing):** Flat contiguous memory layout with 1-byte control bytes storing 7-bit hash fingerprints. SIMD instructions (AVX2/SSE2) filter candidate slots in parallel, drastically reducing key comparisons. Table resizing is segmented into 64KB micro-chunks, completely eliminating dual-table probe duplication.

### Problem 6: Small-Object Memory Fragmentation (robj Metadata Tax)
* **The Failure:** Small strings (20-byte session tokens) incur a 16-byte `robj` header, 3-byte `sdshdr`, 8-byte pointer, and Jemalloc chunk rounding. Over time, external fragmentation causes 50%+ RAM waste (RSS / Logical > 2.0).
* **The Solution (Small String Optimization & Region Slab Arenas):** Values under 24 bytes are inlined directly into the hash slot with zero heap allocation. Larger items use pre-allocated 2MB hugepage-aligned slabs for discrete geometric size classes (32B to 1024B). Slot allocation uses 64-bit integer bitmaps via the CPU hardware intrinsic `trailing_zeros()`. Zero per-object header tax; RSS fragmentation ratio stays strictly <1.05.

### Problem 7: Replication Backlog Overrun & Full-Resync Cascades
* **The Failure:** In-memory circular buffer (`server.repl_backlog`) wraps around during 10s of network lag. Primary falls back to Full Resync: forks a 30GB snapshot, dumps to disk, and saturates the network, causing cascading cluster outages.
* **The Solution (Segmented NVMe WAL with Kernel-Assisted Streaming):** Every mutating write is assigned a strictly monotonic 64-bit `SequenceId` and appended to an in-memory buffer before being batched to an on-disk circular WAL managed via `io_uring`. Replicas reconnect by sequence ID and stream exact missing deltas directly from NVMe to the network socket via `sendfile()` / `splice()`, completely eliminating full-resync storms.

---

## 5. Mental Models & Analogies (How to Explain It Intuitively)

* **The Post Office Analogy (Redis Event Loop):** Redis is a post office with 10,000 customers outside and only **one clerk** running around really fast (`epoll`). If that single clerk stops to photocopy the entire building (`fork()`) or sort 500,000 expired letters (`activeExpireCycle`), every customer outside freezes.
* **The Swiss Watch Analogy (Hierarchical Timing Wheel):** Like the cascading gears in an odometer or a clock (milliseconds to seconds to minutes to hours), expirations advance in O(1) lockstep. When a slot triggers, the clerk unhooks the whole bucket in one atomic motion and hands it to a background cleaner.
* **The Coat Closet Analogy (Storage Tiering):** Your heavy winter coat lives in the basement (NVMe); your daily jacket hangs by the front door (DRAM). You don't build a 5x larger front door closet just to hold winter coats in July.

---

## 6. Formal Consistency Specification

| Operation Type | Formal Guarantee | Implementation Mechanism |
| :--- | :--- | :--- |
| **Single-Key Read / Write** | **Linearizable** | Single-owner actor shard; zero shared locks. |
| **Cross-Shard Transactions** | **Atomic (All-or-Nothing)** | Coordinator WAL logging with explicit PREPARE / COMMIT records. |
| **Point-in-Time Checkpoint** | **Snapshot Isolated at Epoch E** | Versioned entries + pre-update shadow ring and overflow log. |
| **Replica Synchronization** | **Monotonically Ordered** | Strictly increasing 64-bit SequenceId log replay. |
| **Tiered NVMe Read** | **Transparent & Asynchronous** | Non-blocking `io_uring` fetch; read-committed with live shard state. |
