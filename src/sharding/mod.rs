//! Multi-Core Sharding & Affinity Architecture
//!
//! Provides thread-per-core hardware pinning, consistent key routing,
//! lock-free cache-padded SPSC ring buffers, and cross-shard transaction coordination.

pub mod affinity;
pub mod router;

pub use affinity::{pin_current_thread_to_core, spawn_pinned_worker, AffinityError, CoreId, CpuTopology};
pub use router::{jump_consistent_hash, murmur3_32, ShardRouter};
