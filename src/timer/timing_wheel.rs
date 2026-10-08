//! 4-Tier Hierarchical Timing Wheel (TTL Avalanche Solver)
//!
//! Replaces Redis's active TTL scanning loops (which cause 200ms+ latency spikes
//! during key mass-expiration events) with a deterministic O(1) multi-tier cascaded wheel.
//!
//! Tiers:
//! - Tier 1: 1,000 slots (1ms resolution, covers 0-999ms)
//! - Tier 2: 60 slots (1s resolution, covers 1-59s)
//! - Tier 3: 60 slots (1m resolution, covers 1-59m)
//! - Tier 4: 24 slots (1h resolution, covers 1-24h)

use std::ptr;

use super::slot::{AtomicTimerBucket, TimerNode};

pub const TIER1_SLOTS: usize = 1_000;
pub const TIER2_SLOTS: usize = 60;
pub const TIER3_SLOTS: usize = 60;
pub const TIER4_SLOTS: usize = 24;

const MS_PER_SECOND: u64 = 1_000;
const MS_PER_MINUTE: u64 = 60 * MS_PER_SECOND;
const MS_PER_HOUR: u64 = 60 * MS_PER_MINUTE;

/// Hierarchical Timing Wheel State Machine
pub struct TimingWheel {
    tier1: Box<[AtomicTimerBucket]>,
    tier2: Box<[AtomicTimerBucket]>,
    tier3: Box<[AtomicTimerBucket]>,
    tier4: Box<[AtomicTimerBucket]>,
    current_time_ms: u64,
}

impl TimingWheel {
    /// Constructs a new 4-tier timing wheel initialized at `start_time_ms`.
    pub fn new(start_time_ms: u64) -> Self {
        let mut t1 = Vec::with_capacity(TIER1_SLOTS);
        for _ in 0..TIER1_SLOTS {
            t1.push(AtomicTimerBucket::new());
        }

        let mut t2 = Vec::with_capacity(TIER2_SLOTS);
        for _ in 0..TIER2_SLOTS {
            t2.push(AtomicTimerBucket::new());
        }

        let mut t3 = Vec::with_capacity(TIER3_SLOTS);
        for _ in 0..TIER3_SLOTS {
            t3.push(AtomicTimerBucket::new());
        }

        let mut t4 = Vec::with_capacity(TIER4_SLOTS);
        for _ in 0..TIER4_SLOTS {
            t4.push(AtomicTimerBucket::new());
        }

        Self {
            tier1: t1.into_boxed_slice(),
            tier2: t2.into_boxed_slice(),
            tier3: t3.into_boxed_slice(),
            tier4: t4.into_boxed_slice(),
            current_time_ms: start_time_ms,
        }
    }

    /// Current simulated or wall clock time of the wheel in milliseconds.
    #[inline(always)]
    pub fn current_time_ms(&self) -> u64 {
        self.current_time_ms
    }

    /// Schedules a key for expiration at `expire_at_ms`.
    ///
    /// Determines the appropriate wheel tier in O(1) time and pushes the timer node.
    pub fn schedule(&self, key: &[u8], expire_at_ms: u64) {
        let node = TimerNode::new(key, expire_at_ms);
        self.schedule_node(node, expire_at_ms);
    }

    /// Internal node scheduling logic shared by fresh inserts and cascading down-shifts.
    fn schedule_node(&self, node: *mut TimerNode, expire_at_ms: u64) {
        if expire_at_ms <= self.current_time_ms {
            // Already expired: place into current millisecond slot for immediate detachment
            let slot = (self.current_time_ms % TIER1_SLOTS as u64) as usize;
            self.tier1[slot].push(node);
            return;
        }

        let delta = expire_at_ms - self.current_time_ms;

        if delta < MS_PER_SECOND {
            // Tier 1: Milliseconds (0 - 999 ms)
            let slot = (expire_at_ms % TIER1_SLOTS as u64) as usize;
            self.tier1[slot].push(node);
        } else if delta < MS_PER_MINUTE {
            // Tier 2: Seconds (1 - 59 s)
            let sec = expire_at_ms / MS_PER_SECOND;
            let slot = (sec % TIER2_SLOTS as u64) as usize;
            self.tier2[slot].push(node);
        } else if delta < MS_PER_HOUR {
            // Tier 3: Minutes (1 - 59 m)
            let min = expire_at_ms / MS_PER_MINUTE;
            let slot = (min % TIER3_SLOTS as u64) as usize;
            self.tier3[slot].push(node);
        } else {
            // Tier 4: Hours (1 - 24 h)
            let hour = expire_at_ms / MS_PER_HOUR;
            let slot = (hour % TIER4_SLOTS as u64) as usize;
            self.tier4[slot].push(node);
        }
    }

    /// Advances the wheel forward to `now_ms`, executing all cascading tier down-shifts
    /// and detaching all expired timer slots.
    ///
    /// Returns a list of expired key byte vectors.
    pub fn advance_to(&mut self, now_ms: u64) -> Vec<Vec<u8>> {
        if now_ms <= self.current_time_ms {
            return Vec::new();
        }

        let mut expired_keys = Vec::new();

        while self.current_time_ms < now_ms {
            self.current_time_ms += 1;

            // 1. Check cascading transitions
            if self.current_time_ms % MS_PER_SECOND == 0 {
                // Tier 2 -> Tier 1 cascade
                let sec_slot = ((self.current_time_ms / MS_PER_SECOND) % TIER2_SLOTS as u64) as usize;
                self.cascade_bucket(&self.tier2[sec_slot], &mut expired_keys);
            }

            if self.current_time_ms % MS_PER_MINUTE == 0 {
                // Tier 3 -> Tier 2 cascade
                let min_slot = ((self.current_time_ms / MS_PER_MINUTE) % TIER3_SLOTS as u64) as usize;
                self.cascade_bucket(&self.tier3[min_slot], &mut expired_keys);
            }

            if self.current_time_ms % MS_PER_HOUR == 0 {
                // Tier 4 -> Tier 3 cascade
                let hr_slot = ((self.current_time_ms / MS_PER_HOUR) % TIER4_SLOTS as u64) as usize;
                self.cascade_bucket(&self.tier4[hr_slot], &mut expired_keys);
            }

            // 2. Detach expired items for the current millisecond from Tier 1
            let ms_slot = (self.current_time_ms % TIER1_SLOTS as u64) as usize;
            let detached = self.tier1[ms_slot].detach_all();
            if !detached.is_null() {
                unsafe {
                    let items = AtomicTimerBucket::collect_and_free(detached);
                    for (k, _) in items {
                        expired_keys.push(k);
                    }
                }
            }
        }

        expired_keys
    }

    /// Cascades all timer nodes from a detached higher-tier bucket into lower tiers
    /// or directly to expired collection if their deadline has passed.
    fn cascade_bucket(&self, bucket: &AtomicTimerBucket, expired_keys: &mut Vec<Vec<u8>>) {
        let detached = bucket.detach_all();
        if detached.is_null() {
            return;
        }

        let mut curr = detached;
        while !curr.is_null() {
            let next = unsafe { (*curr).next };
            unsafe {
                (*curr).next = ptr::null_mut();
            }

            let expire_at = unsafe { (*curr).expire_at_ms };
            if expire_at <= self.current_time_ms {
                // Expired during cascade
                let key_bytes = unsafe { (*curr).key.as_slice().to_vec() };
                unsafe {
                    TimerNode::free(curr);
                }
                expired_keys.push(key_bytes);
            } else {
                // Re-schedule into a lower tier
                self.schedule_node(curr, expire_at);
            }

            curr = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_wheel_sub_second_expiration() {
        let mut wheel = TimingWheel::new(1_000);

        // Schedule timers at +50ms, +200ms, +500ms
        wheel.schedule(b"k50", 1_050);
        wheel.schedule(b"k200", 1_200);
        wheel.schedule(b"k500", 1_500);

        // Advance +40ms -> None expired
        let exp = wheel.advance_to(1_040);
        assert!(exp.is_empty());

        // Advance past 1050ms -> k50 expired
        let exp = wheel.advance_to(1_060);
        assert_eq!(exp, vec![b"k50".to_vec()]);

        // Advance past 1200ms -> k200 expired
        let exp = wheel.advance_to(1_210);
        assert_eq!(exp, vec![b"k200".to_vec()]);

        // Advance past 1500ms -> k500 expired
        let exp = wheel.advance_to(1_510);
        assert_eq!(exp, vec![b"k500".to_vec()]);
    }

    #[test]
    fn test_timing_wheel_tier2_cascade() {
        let mut wheel = TimingWheel::new(0);

        // Schedule at 5 seconds (5,000 ms) -> places in Tier 2
        wheel.schedule(b"key_5s", 5_000);

        // Advance to 4,000ms -> Tier 2 slot 5 has not yet cascaded
        let exp = wheel.advance_to(4_000);
        assert!(exp.is_empty());

        // Advance to 5,000ms -> Cascades at 5,000ms boundary and triggers
        let exp = wheel.advance_to(5_000);
        assert!(exp.contains(&b"key_5s".to_vec()));
    }

    #[test]
    fn test_timing_wheel_tier3_cascade() {
        let mut wheel = TimingWheel::new(0);

        // Schedule at 2 minutes (120,000 ms) -> places in Tier 3
        wheel.schedule(b"key_2m", 120_000);

        // Advance to 119,000ms -> should not be expired
        let exp = wheel.advance_to(119_000);
        assert!(exp.is_empty());

        // Advance to 120,000ms -> Cascades down through Tier 2 and Tier 1, expires
        let exp = wheel.advance_to(120_000);
        assert!(exp.contains(&b"key_2m".to_vec()));
    }
}
