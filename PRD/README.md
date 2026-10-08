# VALENCE: PRODUCT REQUIREMENTS & SYSTEM SPECIFICATION (PRD)

Welcome to the **Valence** Project Requirements & Engineering Specification repository. This folder contains the complete technical blueprint, architectural designs, and implementation plans for building Valence from first principles.

---

## PRD Documents Index

| Document | Purpose & Contents |
| :--- | :--- |
| **[brain.md](file:///c:/project_placement/PRD/brain.md)** | Core mental model, Redis C-source internals (`ae.c`, `dict.c`, `expire.c`, `rdb.c`, `replication.c`), the 7 production failure modes, and their systematic first-principles solutions. |
| **[system_architecture.md](file:///c:/project_placement/PRD/system_architecture.md)** | Complete Rust workspace Cargo layout, hardware-level mechanical sympathy (64B cache line isolation, thread pinning), Linux kernel subsystems (`userfaultfd`, `io_uring`), and component interactions. |
| **[execution_plan.md](file:///c:/project_placement/PRD/execution_plan.md)** | Master implementation roadmap divided into 3 Milestones, 8 Phases, and 24 Atomic Sub-Phases with exact target files, tasks, and verification commands (`cargo test`, `redis-benchmark`, chaos scripts). |
| **[protocol_spec.md](file:///c:/project_placement/PRD/protocol_spec.md)** | Wire-level RESP3 protocol specification, command syntax (`SET`, `GET`, `DEL`, `EXPIRE`, `EXPLAIN`), frame formats, connection state machine, and error frames. |
| **[testing_strategy.md](file:///c:/project_placement/PRD/testing_strategy.md)** | 4-pillar quality strategy, unit tests, Redis compliance testing, chaos stress harnesses (TTL avalanche, COW memory bloat), and latency/throughput SLA targets. |

---

## Core Project Invariant

> **"Foreground request processing must never perform unbounded persistence, expiration, eviction, or memory reorganization work. Every expensive operational task is bounded or decoupled into asynchronous background workers."**

---

## The 7 Production Problems Solved

1. **BGSAVE Fork & Copy-on-Write Memory Doubling** → *Forkless Epoch-Versioned Snapshots & `userfaultfd` Page Interception*.
2. **TTL Avalanche & Sampling Loop Freezes** → *4-Tier Hierarchical Timing Wheels (HTW) with Lock-Free Asynchronous Reclamation*.
3. **The DRAM Cost Wall & Cold-Data Bloat** → *Transparent DRAM/NVMe Storage Tiering via Log-Structured Extents and Linux `io_uring`*.
4. **Single-Threaded Core Saturation Wall** → *Shared-Nothing Thread-per-Core Shards with Cache-Padded SPSC Ring Buffers*.
5. **Incremental Rehashing Latency Jitter** → *Swiss Table SIMD Vectorization & Robin Hood Open-Addressing with Contiguous Flat Arrays*.
6. **Small-Object Heap Fragmentation** → *Fixed-Size Region Slab Arenas with 64-Bit Bitmapped Allocators & Small Value Inlining*.
7. **Replication Backlog Overrun Cascades** → *Persistent Ring-Buffer WAL on NVMe with Zero-Copy `sendfile` Streaming*.

---

## Master Artifacts

* **10-Page Master Architecture Whitepaper (PDF):** `C:\Users\akash\Downloads\VALENCE_SYSTEMS_ENGINE_WHITEPAPER.pdf`
* **Local Source HTML:** `c:\project_placement\valence_whitepaper.html`
