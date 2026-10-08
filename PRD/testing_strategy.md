# VALENCE: TESTING & VERIFICATION STRATEGY
> **Document:** `PRD/testing_strategy.md`  
> **Project:** Valence — Next-Generation In-Memory State Engine in Pure Rust  
> **Classification:** High-Performance Systems Quality Assurance  

---

## 1. Testing Philosophy: The 4 Quality Pillars

In a high-throughput systems engine, toy unit tests are insufficient. Valence enforces a 4-pillar quality framework to guarantee correctness, memory safety, and deterministic tail latency:

```
[ PILLAR 1: Unit Correctness ]          --> Invariants & algorithm mathematical correctness
[ PILLAR 2: Protocol Compliance ]       --> 100% interoperability with official Redis tools
[ PILLAR 3: Adversarial Chaos ]         --> Simulated production catastrophes (TTL bursts, COW)
[ PILLAR 4: Micro-Benchmarking SLAs ]   --> Strict p99 latency & throughput enforcement
```

---

## 2. Pillar 1: Unit Testing (`cargo test`)

Unit tests reside directly within the source modules using Rust's `#[cfg(test)]` harness.

### Key Invariants Tested:
1. **Robin Hood & Swiss Table Map Invariants (`storage/`):**
   * *Probe Length Bound:* Verify that no element's Distance from Ideal Bucket (DIB) exceeds log2(N) + 4.
   * *Backshift Deletion Invariant:* Verify that deleting any key correctly shifts subsequent elements backward and leaves zero tombstones.
   * *Load Factor Rehash:* Verify that doubling the table size correctly preserves all keys.
2. **Hierarchical Timing Wheel Invariants (`timer/timing_wheel.rs`):**
   * *Tick Precision:* Verify that a timer scheduled for X milliseconds triggers within X ± 1ms.
   * *Cascade Correctness:* Verify that higher-tier slots (Wheel 2, 3, 4) cascade down into Wheel 1 at exact minute/hour boundaries.
   * *Atomic Head Swapping:* Verify that detaching an expired slot's atomic head in concurrent threads never leaks memory or drops keys.
3. **Slab Allocator Invariants (`memory/slab_arena.rs` & `memory/bitmap.rs`):**
   * *Single-Cycle Allocation:* Verify that `trailing_zeros()` bitmask allocation produces monotonic slots.
   * *Double-Free Protection:* Verify that attempting to double-free a slot returns an explicit error without corrupting the bitmap.

* **Execution Command:**
  ```bash
  cargo test --lib
  ```

---

## 3. Pillar 2: Integration & Protocol Compliance

Valence must be 100% compatible with existing Redis client libraries and tools.

### Test Harness: `tests/redis_compliance_test.rs`
* **Test Matrix:**
  * Runs a live instance of `valence-server` on port 6380.
  * Connects using the official `redis` Rust client, Python `redis-py`, and Node.js `ioredis`.
  * Executes a suite of 500 standard command sequences (`SET`, `GET`, `DEL`, `EXPIRE`, `TTL`, `PING`).
  * Asserts that return values, error strings, and data types match Redis 7.2 byte-for-byte.

* **Official CLI Verification:**
  ```bash
  # Manual CLI check
  redis-cli -p 6380 PING
  redis-cli -p 6380 SET foo "bar" EX 10
  redis-cli -p 6380 TTL foo
  redis-cli -p 6380 GET foo
  ```

---

## 4. Pillar 3: Adversarial & Chaos Testing (Production Stress)

These tests actively simulate the real-world production disasters that cause Redis to crash.

### Chaos Test A: The TTL Avalanche Stress Harness (`tests/avalanche_chaos.py`)
* **The Scenario:** Insert 500,000 keys with identical expiration timestamps set to T + 5 seconds.
* **The Test:** Concurrently generate 50,000 background client `GET` requests per second across the expiration boundary.
* **Pass/Fail Criteria:**
  * **FAIL:** Foreground client p99 latency spikes above 50ms (as seen in Redis).
  * **PASS:** Foreground client p99 latency remains strictly bounded under **<20µs**.
* **Execution Command:**
  ```bash
  python tests/avalanche_chaos.py --host 127.0.0.1 --port 6379 --keys 500000 --concurrency 50
  ```

### Chaos Test B: Persistence Memory Bloat & COW Stress (`tests/cow_memory_test.py`)
* **The Scenario:** Load 20 GB of key-value data into the engine. Trigger a background snapshot.
* **The Stress:** During snapshotting, hammer the server with continuous random updates (`SET key new_val`) at 50,000 writes/second.
* **Pass/Fail Criteria:**
  * **FAIL:** Resident Set Size (RSS) doubles to 40 GB or triggers an OOM panic (as seen in Redis `fork()`).
  * **PASS:** Total process RSS growth remains strictly bounded under **<5%** additional memory overhead.
* **Execution Command:**
  ```bash
  python tests/cow_memory_test.py --host 127.0.0.1 --port 6379 --write-rate 50000 --duration 120
  ```

### Chaos Test C: False Sharing & Cache Invalidation Profiling (`perf c2c`)
* **The Scenario:** Run 16 worker threads concurrently passing requests across SPSC ring buffers.
* **The Measurement:** Use Linux hardware performance counters to detect cache line invalidation cycles (MESI bus stalls).
* **Pass/Fail Criteria:** Zero hitm (hit modified) cache line contention events across thread boundaries.
* **Execution Command:**
  ```bash
  perf c2c record -- cargo bench --bench throughput_bench
  perf c2c report --stdio
  ```

### Chaos Test D: Snapshot Consistency Torture (`tests/snapshot_torture.py`)
* **The Scenario:** 4 shards, 10,000 keys. Hammer engine with concurrent random `MSET`, `MGET`, and `DEL` operations while initiating snapshots every 5 seconds.
* **The Chaos:** Issue random `SIGKILL` termination signals during active checkpointing.
* **Pass/Fail Criteria:** Engine reboots from `dump.rdb` + WAL; recovered state matches an independent linearizable oracle reference model with 100% precision. Zero ghost keys, zero corrupted entries.
* **Execution Command:**
  ```bash
  python tests/snapshot_torture.py --shards 4 --keys 10000 --snapshots 20 --random-kill
  ```

### Chaos Test E: NVMe Fault Injection & Degraded Device Stress (`tests/nvme_fault_injection.rs`)
* **The Scenario:** Trigger active cold-tier paging to NVMe while injecting simulated device errors: `EIO` (hardware controller failure), `ENOSPC` (disk full), and 500ms I/O latency timeouts.
* **Pass/Fail Criteria:** **Cold-tier failure must never corrupt or crash hot DRAM state.** Cold read misses return explicit I/O error frames; hot DRAM key requests continue serving sub-microsecond traffic with zero panics.
* **Execution Command:**
  ```bash
  cargo test --test nvme_fault_injection
  ```

---

## 5. Pillar 4: Performance & Benchmarking SLAs

Every release of Valence must satisfy strict, quantified service-level objectives before merging:

| Metric | Target SLA | Benchmark Method | Comparison vs Redis 7.2 |
| :--- | :---: | :--- | :--- |
| **Single-Core Throughput** | **>= 1.2M ops/sec** | `redis-benchmark -t get,set -n 100000 -q` | ~1.2x faster |
| **16-Core Throughput** | **>= 5.0M ops/sec** | Multi-threaded client benchmark | **~4.5x faster** |
| **p50 Latency (Median)** | **<= 8µs** | Continuous request probe | Parity |
| **p99 Latency (Tail)** | **<= 20µs** | Expiration burst workload | **20,000x faster** (20µs vs 450ms) |
| **p99.9 Latency (Worst-case)** | **<= 100µs** | Concurrent persistence workload | **2,000x faster** |
| **Memory Fragmentation Ratio** | **<= 1.05** | RSS / Logical Memory after 10M churn ops | **2x better** (1.05 vs 1.80+) |
| **DRAM Reduction (Cold Data)** | **>= 70%** | Zipfian 80/20 cold-offload workload | Redis: 0% reduction |

---

## 6. Continuous Verification Command Cheat Sheet

```bash
# 1. Run all unit tests
cargo test --workspace

# 2. Run Criterion performance micro-benchmarks
cargo bench

# 3. Run official Redis benchmark tool
redis-benchmark -h 127.0.0.1 -p 6379 -t get,set -n 200000 -c 50 -q

# 4. Run the full automated chaos test suite
python tests/avalanche_chaos.py
python tests/cow_memory_test.py
```
