//! Transparent NVMe Storage Tiering
//!
//! Sub-Phases:
//! - 6.1: 2-Bit CLOCK Access Frequency Tracker (`clock.rs`)
//! - 6.2: Asynchronous Block Paging (`io_uring.rs` / block storage)
//! - 6.3: Cold Read Cache Promotion (`cache_promotion.rs`)

pub mod clock;

pub use clock::{ClockArray, ClockTracker, ColdCandidate};
