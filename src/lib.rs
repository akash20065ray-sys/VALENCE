//! Valence: Next-Generation In-Memory State Engine in Pure Rust
//!
//! Architected to eliminate seven production bottlenecks in incumbent in-memory engines:
//! 1. Fork-OOM Killer -> Forkless Epoch-Versioned Snapshots & Pre-Update Hooks
//! 2. TTL Avalanche -> 4-Tier Hierarchical Timing Wheels with Atomic Detachment
//! 3. DRAM Cost Wall -> Transparent NVMe Tiering via Log-Structured Extents & io_uring
//! 4. Single-Core Saturation -> Thread-per-Core Shards with 64B Cache-Padded SPSC Rings
//! 5. Rehashing Jitter -> Swiss Table SIMD Filtering & Robin Hood Flat Arrays
//! 6. Allocator Fragmentation -> Small Value Inlining (SSO) & 2MB Geometric Slab Arenas
//! 7. Replication Overrun -> Segmented NVMe WAL with Kernel-Assisted Streaming

pub mod protocol;
pub mod server;
pub mod sharding;
pub mod storage;
pub mod timer;

pub use protocol::{Command, ParseError, RespEncoder, RespParser};
pub use server::{ClientConnection, ServerEngine, ServerStats};
pub use sharding::{
    jump_consistent_hash, murmur3_32, pin_current_thread_to_core, spawn_pinned_worker,
    AffinityError, CoreId, CpuTopology, ShardRouter,
};
pub use storage::entry::{CompactKey, CompactValue, Slot, SlotState};
pub use storage::robin_hood::RobinHoodMap;
pub use timer::{AtomicTimerBucket, TimerNode, TimingWheel};
