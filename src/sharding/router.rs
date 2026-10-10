//! Keyspace Consistent Hash Router
//!
//! Maps database keys across independent worker shards using MurmurHash3
//! and Google's Jump Consistent Hashing to achieve near-perfect uniform distribution
//! (<1% variance) and minimal key migrations during shard scaling.

const DEFAULT_SEED: u32 = 0x9747B28C;

/// Canonical MurmurHash3 32-bit implementation in pure safe Rust.
///
/// Exhibits excellent avalanche behavior: single bit perturbations in the input
/// produce uniformly distributed 32-bit hashes with zero bias.
#[inline]
pub fn murmur3_32(key: &[u8], seed: u32) -> u32 {
    let c1: u32 = 0xcc9e2d51;
    let c2: u32 = 0x1b873593;
    let r1: u32 = 15;
    let r2: u32 = 13;
    let m: u32 = 5;
    let n: u32 = 0xe6546b64;

    let mut hash = seed;
    let chunks = key.chunks_exact(4);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let mut k = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        k = k.wrapping_mul(c1);
        k = k.rotate_left(r1);
        k = k.wrapping_mul(c2);

        hash ^= k;
        hash = hash.rotate_left(r2);
        hash = hash.wrapping_mul(m).wrapping_add(n);
    }

    let mut k: u32 = 0;
    match remainder.len() {
        3 => {
            k ^= (remainder[2] as u32) << 16;
            k ^= (remainder[1] as u32) << 8;
            k ^= remainder[0] as u32;
            k = k.wrapping_mul(c1);
            k = k.rotate_left(r1);
            k = k.wrapping_mul(c2);
            hash ^= k;
        }
        2 => {
            k ^= (remainder[1] as u32) << 8;
            k ^= remainder[0] as u32;
            k = k.wrapping_mul(c1);
            k = k.rotate_left(r1);
            k = k.wrapping_mul(c2);
            hash ^= k;
        }
        1 => {
            k ^= remainder[0] as u32;
            k = k.wrapping_mul(c1);
            k = k.rotate_left(r1);
            k = k.wrapping_mul(c2);
            hash ^= k;
        }
        _ => {}
    }

    hash ^= key.len() as u32;
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x85ebca6b);
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(0xc2b2ae35);
    hash ^= hash >> 16;

    hash
}

/// Google's Jump Consistent Hash (Lamping & Veach, 2014).
///
/// Given a 64-bit key hash and number of shards, returns a shard index in `0..num_shards`.
/// Guarantees:
/// - O(ln(num_shards)) computational cost with zero memory overhead.
/// - Exact 1/N key migration when expanding from N to N+1 shards.
#[inline]
pub fn jump_consistent_hash(mut key: u64, num_shards: usize) -> usize {
    assert!(num_shards > 0, "Number of shards must be strictly positive");
    let mut b: i64 = -1;
    let mut j: i64 = 0;
    let num_shards_i64 = num_shards as i64;

    while j < num_shards_i64 {
        b = j;
        key = key.wrapping_mul(2862933555777941757).wrapping_add(1);
        let rand_val = ((key >> 33) + 1) as f64;
        j = (((b + 1) as f64) * ((1i64 << 31) as f64) / rand_val) as i64;
    }

    b as usize
}

/// Keyspace Consistent Hash Router across thread-per-core shards.
#[derive(Debug, Clone)]
pub struct ShardRouter {
    num_shards: usize,
    seed: u32,
}

impl ShardRouter {
    /// Creates a new router for the specified shard count.
    pub fn new(num_shards: usize) -> Self {
        Self::with_seed(num_shards, DEFAULT_SEED)
    }

    /// Creates a new router with a custom hash seed.
    pub fn with_seed(num_shards: usize, seed: u32) -> Self {
        assert!(num_shards > 0, "Shard count must be greater than 0");
        Self { num_shards, seed }
    }

    /// Total number of active shards.
    #[inline(always)]
    pub fn num_shards(&self) -> usize {
        self.num_shards
    }

    /// Routes a key to its assigned shard index using MurmurHash3 modulo arithmetic.
    #[inline(always)]
    pub fn route_key(&self, key: &[u8]) -> usize {
        let hash = murmur3_32(key, self.seed);
        (hash as usize) % self.num_shards
    }

    /// Routes a key using Jump Consistent Hashing for elastic resharding support.
    #[inline(always)]
    pub fn route_key_jump(&self, key: &[u8]) -> usize {
        let h1 = murmur3_32(key, self.seed) as u64;
        let h2 = murmur3_32(key, self.seed.wrapping_add(0x5bd1e995)) as u64;
        let combined_64 = (h1 << 32) | h2;
        jump_consistent_hash(combined_64, self.num_shards)
    }

    /// Computes distribution variance across shards for a set of keys.
    ///
    /// Returns the maximum percentage deviation from the ideal mean:
    /// `max_deviation = max(|shard_count - ideal_mean|) / ideal_mean`
    pub fn calculate_max_deviation(counts: &[usize]) -> f64 {
        if counts.is_empty() {
            return 0.0;
        }

        let total: usize = counts.iter().sum();
        let ideal_mean = total as f64 / counts.len() as f64;
        if ideal_mean == 0.0 {
            return 0.0;
        }

        let max_diff = counts
            .iter()
            .map(|&c| (c as f64 - ideal_mean).abs())
            .fold(0.0, f64::max);

        max_diff / ideal_mean
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_murmur3_known_vectors() {
        // Known test vectors for MurmurHash3 32-bit (seed 0)
        assert_eq!(murmur3_32(b"", 0), 0);
        assert_eq!(murmur3_32(b"hello", 0), 0x248bfa47);
        assert_eq!(murmur3_32(b"world", 0), 0xfb963cfb);
    }

    #[test]
    fn test_jump_consistent_hash_properties() {
        let shards = 8;
        let k1 = 123456789;
        let shard = jump_consistent_hash(k1, shards);
        assert!(shard < shards);

        // Deterministic
        assert_eq!(jump_consistent_hash(k1, shards), shard);

        // When increasing shards from 8 to 9, key either stays at same shard or moves to shard 8
        let new_shard = jump_consistent_hash(k1, shards + 1);
        assert!(new_shard == shard || new_shard == shards);
    }

    #[test]
    fn test_uniform_distribution_1_million_keys() {
        let num_shards = 8;
        let router = ShardRouter::new(num_shards);
        let total_keys = 1_000_000;
        let mut shard_counts = vec![0usize; num_shards];

        // Route 1,000,000 synthetic keys
        for i in 0..total_keys {
            let key = format!("user:session:key:{}", i);
            let shard = router.route_key(key.as_bytes());
            shard_counts[shard] += 1;
        }

        let max_deviation = ShardRouter::calculate_max_deviation(&shard_counts);
        let ideal_per_shard = total_keys / num_shards;

        // Verify variance across shards is strictly < 1% (acceptance criteria)
        assert!(
            max_deviation < 0.01,
            "Hash routing skew too high: max deviation is {:.4}% (counts: {:?}, ideal: {})",
            max_deviation * 100.0,
            shard_counts,
            ideal_per_shard
        );
    }

    #[test]
    fn test_jump_uniform_distribution() {
        let num_shards = 16;
        let router = ShardRouter::new(num_shards);
        let total_keys = 500_000;
        let mut shard_counts = vec![0usize; num_shards];

        for i in 0..total_keys {
            let key = format!("device:sensor:metric:{}", i * 17);
            let shard = router.route_key_jump(key.as_bytes());
            shard_counts[shard] += 1;
        }

        let max_deviation = ShardRouter::calculate_max_deviation(&shard_counts);
        assert!(
            max_deviation < 0.02,
            "Jump hash skew too high: max deviation is {:.4}%",
            max_deviation * 100.0
        );
    }
}
