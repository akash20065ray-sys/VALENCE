//! Hierarchical Timing Wheel Module
//!
//! Provides sub-millisecond to 24-hour expiration tracking with lock-free atomic slot detachment
//! and automatic multi-tier cascade, eliminating active TTL avalanche spikes.

pub mod slot;
pub mod timing_wheel;

pub use slot::{AtomicTimerBucket, TimerNode};
pub use timing_wheel::TimingWheel;
