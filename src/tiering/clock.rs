//! 2-Bit CLOCK Access Frequency Tracker (Sub-Phase 6.1)
//!
//! Implements a low-overhead, cacheline-compact 2-Bit Second-Chance CLOCK algorithm
//! to track keyspace access frequencies and identify cold records for transparent
//! NVMe disk tiering under Zipfian read distributions.
//!
//! Technical Design & Mechanical Sympathy:
//! - Bit-Packs four 2-bit counters into each byte (32 slots per `u64` word).
//! - On read or write: slot frequency saturates to 3 (Hot).
//! - Circular clock hand sweeps the keyspace:
//!   - If counter > 0: decrements by 1 (giving the slot a second chance).
//!   - If counter == 0: classifies the slot as a Cold Candidate for NVMe offloading.
//! - Cache-friendly: 1,000,000 slots require only ~250 KB of tracking memory.

/// State of a cold candidate identified by the CLOCK sweep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColdCandidate {
    pub slot: usize,
    pub frequency: u8,
}

/// Bit-packed 2-bit counter array tracking slot access frequencies (0..=3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClockArray {
    words: Vec<u64>,
    capacity: usize,
    hand: usize,
}

impl ClockArray {
    /// Constructs a new 2-bit CLOCK array for `capacity` slots.
    pub fn new(capacity: usize) -> Self {
        let num_words = (capacity + 31) / 32;
        Self {
            words: vec![0u64; num_words],
            capacity,
            hand: 0,
        }
    }

    /// Total number of tracked slots.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Current circular clock hand position.
    #[inline(always)]
    pub fn hand_position(&self) -> usize {
        self.hand
    }

    /// Resets the clock hand to index 0.
    #[inline(always)]
    pub fn reset_hand(&mut self) {
        self.hand = 0;
    }

    /// Retrieves the current 2-bit frequency for a slot (0..=3).
    #[inline(always)]
    pub fn get(&self, slot: usize) -> u8 {
        if slot >= self.capacity {
            return 0;
        }
        let word_idx = slot / 32;
        let bit_offset = (slot % 32) * 2;
        ((self.words[word_idx] >> bit_offset) & 0b11) as u8
    }

    /// Records an access event (read or write), saturating the frequency to 3 (Hot).
    #[inline(always)]
    pub fn touch(&mut self, slot: usize) {
        if slot >= self.capacity {
            return;
        }
        let word_idx = slot / 32;
        let bit_offset = (slot % 32) * 2;
        let mask = 0b11u64 << bit_offset;
        self.words[word_idx] |= mask;
    }

    /// Sets an explicit frequency value (0..=3) for a slot.
    #[inline(always)]
    pub fn set(&mut self, slot: usize, freq: u8) {
        if slot >= self.capacity {
            return;
        }
        let word_idx = slot / 32;
        let bit_offset = (slot % 32) * 2;
        let clear_mask = !(0b11u64 << bit_offset);
        let val = (freq.min(3) as u64) << bit_offset;
        self.words[word_idx] = (self.words[word_idx] & clear_mask) | val;
    }

    /// Decrements the frequency counter for a slot if it is greater than 0.
    /// Returns the updated frequency.
    #[inline(always)]
    pub fn decrement(&mut self, slot: usize) -> u8 {
        if slot >= self.capacity {
            return 0;
        }
        let curr = self.get(slot);
        if curr > 0 {
            let next = curr - 1;
            self.set(slot, next);
            next
        } else {
            0
        }
    }

    /// Advances the clock hand by one slot, decrementing positive counts
    /// and returning the inspected slot index and its updated frequency.
    pub fn step(&mut self) -> (usize, u8) {
        if self.capacity == 0 {
            return (0, 0);
        }
        let slot = self.hand;
        let freq = self.get(slot);

        let final_freq = if freq > 0 {
            self.decrement(slot)
        } else {
            0
        };

        self.hand = (self.hand + 1) % self.capacity;
        (slot, final_freq)
    }

    /// Sweeps across the table collecting cold candidates (frequency == 0).
    ///
    /// Stops when `max_candidates` are found or when `max_inspections` have been performed.
    pub fn sweep_batch(&mut self, max_candidates: usize, max_inspections: usize) -> Vec<ColdCandidate> {
        let mut candidates = Vec::with_capacity(max_candidates);
        let mut inspected = 0;

        while inspected < max_inspections && candidates.len() < max_candidates {
            if self.capacity == 0 {
                break;
            }

            let slot = self.hand;
            let freq = self.get(slot);

            if freq == 0 {
                candidates.push(ColdCandidate { slot, frequency: 0 });
            } else {
                self.decrement(slot);
            }

            self.hand = (self.hand + 1) % self.capacity;
            inspected += 1;
        }

        candidates
    }

    /// Executes one full 360-degree cycle of the clock hand over all slots.
    ///
    /// Returns the total count of slots that are currently at frequency 0 (cold).
    pub fn full_cycle(&mut self) -> usize {
        if self.capacity == 0 {
            return 0;
        }

        let mut cold_count = 0;
        for _ in 0..self.capacity {
            let slot = self.hand;
            let freq = self.get(slot);
            if freq == 0 {
                cold_count += 1;
            } else {
                self.decrement(slot);
            }
            self.hand = (self.hand + 1) % self.capacity;
        }
        cold_count
    }

    /// Clears all counters back to 0 and resets the hand.
    pub fn clear(&mut self) {
        self.words.fill(0);
        self.hand = 0;
    }
}

/// Telemetry and coordination engine for 2-Bit CLOCK storage tiering.
#[derive(Debug)]
pub struct ClockTracker {
    array: ClockArray,
    total_touches: u64,
    total_sweeps: u64,
    cold_candidates_found: u64,
    cycles_completed: u64,
}

impl ClockTracker {
    /// Constructs a new tracker with the designated capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            array: ClockArray::new(capacity),
            total_touches: 0,
            total_sweeps: 0,
            cold_candidates_found: 0,
            cycles_completed: 0,
        }
    }

    /// Records an access event for a slot.
    #[inline(always)]
    pub fn record_access(&mut self, slot: usize) {
        self.total_touches += 1;
        self.array.touch(slot);
    }

    /// Returns the frequency of a slot.
    #[inline(always)]
    pub fn get_frequency(&self, slot: usize) -> u8 {
        self.array.get(slot)
    }

    /// Performs an incremental sweep for cold records.
    pub fn sweep_incremental(&mut self, max_candidates: usize, max_steps: usize) -> Vec<ColdCandidate> {
        self.total_sweeps += 1;
        let cands = self.array.sweep_batch(max_candidates, max_steps);
        self.cold_candidates_found += cands.len() as u64;
        cands
    }

    /// Runs a full clock cycle over the entire keyspace.
    pub fn run_full_cycle(&mut self) -> usize {
        self.cycles_completed += 1;
        self.array.full_cycle()
    }

    /// Returns telemetry statistics.
    pub fn stats(&self) -> ClockStats {
        ClockStats {
            capacity: self.array.capacity(),
            hand_position: self.array.hand_position(),
            total_touches: self.total_touches,
            total_sweeps: self.total_sweeps,
            cold_candidates_found: self.cold_candidates_found,
            cycles_completed: self.cycles_completed,
        }
    }
}

/// Observability metrics for the CLOCK tiering subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockStats {
    pub capacity: usize,
    pub hand_position: usize,
    pub total_touches: u64,
    pub total_sweeps: u64,
    pub cold_candidates_found: u64,
    pub cycles_completed: u64,
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_clock_array_basic_touch_and_get() {
        let mut clock = ClockArray::new(64);
        assert_eq!(clock.get(0), 0);
        assert_eq!(clock.get(63), 0);

        clock.touch(5);
        assert_eq!(clock.get(5), 3);

        clock.touch(63);
        assert_eq!(clock.get(63), 3);

        // Out of bounds safety
        clock.touch(100);
        assert_eq!(clock.get(100), 0);
    }

    #[test]
    fn test_clock_array_decrement_and_saturation() {
        let mut clock = ClockArray::new(10);
        clock.touch(2); // Set to 3
        assert_eq!(clock.get(2), 3);

        assert_eq!(clock.decrement(2), 2);
        assert_eq!(clock.get(2), 2);

        assert_eq!(clock.decrement(2), 1);
        assert_eq!(clock.get(2), 1);

        assert_eq!(clock.decrement(2), 0);
        assert_eq!(clock.get(2), 0);

        // Does not underflow below 0
        assert_eq!(clock.decrement(2), 0);
        assert_eq!(clock.get(2), 0);
    }

    #[test]
    fn test_bit_packing_density() {
        let mut clock = ClockArray::new(128);

        // Touch alternating slots
        for i in (0..128).step_by(2) {
            clock.touch(i);
        }

        // Verify even slots are 3 and odd slots are 0
        for i in 0..128 {
            if i % 2 == 0 {
                assert_eq!(clock.get(i), 3, "Slot {} should be hot (3)", i);
            } else {
                assert_eq!(clock.get(i), 0, "Slot {} should be cold (0)", i);
            }
        }
    }

    #[test]
    fn test_clock_hand_circular_sweep() {
        let mut clock = ClockArray::new(4);
        clock.touch(0);
        clock.touch(1);
        clock.touch(2);
        clock.touch(3);

        // Initial sweep
        let (s0, f0) = clock.step();
        assert_eq!(s0, 0);
        assert_eq!(f0, 2);

        let (s1, f1) = clock.step();
        assert_eq!(s1, 1);
        assert_eq!(f1, 2);

        let (s2, f2) = clock.step();
        assert_eq!(s2, 2);
        assert_eq!(f2, 2);

        let (s3, f3) = clock.step();
        assert_eq!(s3, 3);
        assert_eq!(f3, 2);

        // Hand should wrap back to 0
        assert_eq!(clock.hand_position(), 0);
    }

    #[test]
    fn test_zipfian_80_20_cold_key_identification() {
        // Acceptance Criteria: Under an 80/20 Zipfian workload,
        // cold keys are reliably identified within 3 clock cycles.
        let capacity = 1_000;
        let mut tracker = ClockTracker::new(capacity);

        // Partition: First 200 slots (20%) are HOT keys.
        // Remaining 800 slots (80%) are COLD keys.
        let hot_keys_count = 200;
        let cold_keys_count = 800;

        // Initialize: all keys receive an initial touch
        for i in 0..capacity {
            tracker.record_access(i);
        }

        // Simulate 80/20 Zipfian traffic over 3 cycles:
        // Hot keys are repeatedly touched during each cycle, while cold keys are never touched.
        for _cycle in 1..=3 {
            // Hot keys get frequent touches
            for hot_idx in 0..hot_keys_count {
                tracker.record_access(hot_idx);
            }

            // Run a full clock cycle (decrements all untouched keys)
            tracker.run_full_cycle();
        }

        let mut cold_identified = 0;
        for i in hot_keys_count..capacity {
            if tracker.get_frequency(i) == 0 {
                cold_identified += 1;
            }
        }

        // All 800 cold keys must have reached frequency 0 after 3 clock cycles
        assert_eq!(
            cold_identified, cold_keys_count,
            "Acceptance Criteria failed: expected all {} cold keys to reach frequency 0, got {}",
            cold_keys_count, cold_identified
        );

        // Hot keys must have maintained positive frequency
        let mut hot_retained = 0;
        for i in 0..hot_keys_count {
            if tracker.get_frequency(i) > 0 {
                hot_retained += 1;
            }
        }

        assert_eq!(
            hot_retained, hot_keys_count,
            "Hot keys were incorrectly classified as cold"
        );

        let stats = tracker.stats();
        assert_eq!(stats.cycles_completed, 3);
        assert!(stats.total_touches > 0);
    }
}
