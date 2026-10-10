//! Multi-Shard Coordinator & Cross-Shard Transaction Engine
//!
//! Orchestrates a thread-per-core shared-nothing architecture:
//! - Discovers CPU topology and pins worker threads to hardware cores.
//! - Routes keys to target shards via MurmurHash3 / Jump Consistent Hashing.
//! - Communicates with worker shards over 64-byte cache-padded SPSC ring buffers.
//! - Enforces deterministic shard ID ordering for multi-key operations (MGET, MSET)
//!   to eliminate circular wait states and prevent deadlocks across cores.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::affinity::{spawn_pinned_worker, AffinityError, CoreId, CpuTopology};
use super::router::ShardRouter;
use super::spsc_ring::{spsc_channel, Consumer, Producer, QueueFull};
use crate::snapshot::{ShadowBuffer, SnapshotItem, SnapshotScanner};
use crate::storage::RobinHoodMap;
use crate::timer::TimingWheel;

const DEFAULT_QUEUE_CAPACITY: usize = 1024;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Operations dispatched to a worker shard.
#[derive(Debug, PartialEq, Eq)]
pub enum ShardOp {
    Get { key: Vec<u8> },
    Set { key: Vec<u8>, value: Vec<u8>, ttl_ms: Option<u64> },
    Del { key: Vec<u8> },
    Exists { key: Vec<u8> },
    Expire { key: Vec<u8>, ttl_ms: u64 },
    Ttl { key: Vec<u8> },
    IncrBy { key: Vec<u8>, delta: i64 },
    FlushDb,
    Keys { pattern: Vec<u8> },
    Len,
    SnapshotScan { target_epoch: u64 },
    Shutdown,
}

/// Results returned from a worker shard.
#[derive(Debug, PartialEq, Eq)]
pub enum ShardResult {
    Value(Option<Vec<u8>>),
    Ok,
    Bool(bool),
    Count(usize),
    Ttl(i64),
    Int(i64),
    Keys(Vec<Vec<u8>>),
    Error(String),
    Snapshot(Vec<SnapshotItem>),
}

/// Request envelope sent over the incoming SPSC ring buffer.
pub struct ShardRequest {
    pub id: u64,
    pub op: ShardOp,
}

/// Response envelope sent over the outgoing SPSC ring buffer.
pub struct ShardResponse {
    pub id: u64,
    pub result: ShardResult,
}

/// Errors occurring during shard communication or coordination.
#[derive(Debug, PartialEq, Eq)]
pub enum ShardError {
    Timeout,
    QueueFull,
    WorkerDisconnected,
    AffinityFailed(String),
    CommandError(String),
}

impl fmt::Display for ShardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => write!(f, "Shard operation timed out"),
            Self::QueueFull => write!(f, "Shard SPSC queue is full"),
            Self::WorkerDisconnected => write!(f, "Shard worker thread disconnected"),
            Self::AffinityFailed(msg) => write!(f, "CPU affinity error: {}", msg),
            Self::CommandError(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ShardError {}

impl From<AffinityError> for ShardError {
    fn from(err: AffinityError) -> Self {
        Self::AffinityFailed(err.to_string())
    }
}

/// Channel pair for communicating with a single shard.
struct ShardChannel {
    tx: Producer<ShardRequest>,
    rx: Consumer<ShardResponse>,
}

/// Coordinator handle for a single worker shard.
struct ShardWorkerHandle {
    core_id: CoreId,
    channel: Mutex<ShardChannel>,
}

/// Multi-Shard Coordinator managing pinned worker threads.
pub struct ShardCoordinator {
    router: ShardRouter,
    workers: Vec<ShardWorkerHandle>,
    join_handles: Mutex<Vec<JoinHandle<Result<(), AffinityError>>>>,
    next_request_id: AtomicU64,
}

impl ShardCoordinator {
    /// Creates a new coordinator spawning `num_shards` worker threads,
    /// each pinned to a dedicated CPU core.
    pub fn new(num_shards: usize) -> Result<Self, ShardError> {
        assert!(num_shards > 0, "Number of shards must be strictly positive");

        let topology = CpuTopology::discover();
        let router = ShardRouter::new(num_shards);

        let mut workers = Vec::with_capacity(num_shards);
        let mut join_handles = Vec::with_capacity(num_shards);

        for shard_idx in 0..num_shards {
            let core_id = CoreId(shard_idx % topology.total_cores);

            // Create SPSC channels (Request & Response)
            let (req_tx, req_rx) = spsc_channel::<ShardRequest>(DEFAULT_QUEUE_CAPACITY);
            let (resp_tx, resp_rx) = spsc_channel::<ShardResponse>(DEFAULT_QUEUE_CAPACITY);

            // Spawn the worker pinned to its dedicated hardware core
            let worker_name = format!("valence-shard-{}", shard_idx);
            let handle = spawn_pinned_worker(core_id, Some(&worker_name), move || {
                run_shard_worker(req_rx, resp_tx);
            })
            .map_err(|e| ShardError::AffinityFailed(e.to_string()))?;

            workers.push(ShardWorkerHandle {
                core_id,
                channel: Mutex::new(ShardChannel {
                    tx: req_tx,
                    rx: resp_rx,
                }),
            });
            join_handles.push(handle);
        }

        Ok(Self {
            router,
            workers,
            join_handles: Mutex::new(join_handles),
            next_request_id: AtomicU64::new(1),
        })
    }

    /// Total number of active worker shards.
    #[inline(always)]
    pub fn num_shards(&self) -> usize {
        self.workers.len()
    }

    /// Returns the core ID assigned to a given shard.
    pub fn shard_core_id(&self, shard_idx: usize) -> Option<CoreId> {
        self.workers.get(shard_idx).map(|w| w.core_id)
    }

    /// Routes a key to its assigned shard index.
    #[inline(always)]
    pub fn route_key(&self, key: &[u8]) -> usize {
        self.router.route_key(key)
    }

    /// Dispatches a command to a specific shard and waits for its response.
    fn dispatch_to_shard(&self, shard_idx: usize, op: ShardOp) -> Result<ShardResult, ShardError> {
        let worker = self
            .workers
            .get(shard_idx)
            .ok_or(ShardError::WorkerDisconnected)?;

        let req_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let guard = worker.channel.lock().map_err(|_| ShardError::WorkerDisconnected)?;

        // Push request to shard's incoming ring buffer
        let mut req = ShardRequest { id: req_id, op };
        let push_start = Instant::now();
        loop {
            match guard.tx.push(req) {
                Ok(()) => break,
                Err(QueueFull(returned_req)) => {
                    req = returned_req;
                    if push_start.elapsed() > DEFAULT_TIMEOUT {
                        return Err(ShardError::QueueFull);
                    }
                    std::hint::spin_loop();
                }
            }
        }

        // Wait for matching response on outgoing ring buffer
        let poll_start = Instant::now();
        loop {
            if let Some(resp) = guard.rx.pop() {
                if resp.id == req_id {
                    return Ok(resp.result);
                }
            }
            if poll_start.elapsed() > DEFAULT_TIMEOUT {
                return Err(ShardError::Timeout);
            }
            std::hint::spin_loop();
        }
    }

    /// Executes a GET operation on the keyspace.
    pub fn execute_get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, ShardError> {
        let shard_idx = self.route_key(key);
        let result = self.dispatch_to_shard(shard_idx, ShardOp::Get { key: key.to_vec() })?;
        match result {
            ShardResult::Value(val) => Ok(val),
            _ => Err(ShardError::WorkerDisconnected),
        }
    }

    /// Executes a SET operation on the keyspace with optional millisecond TTL.
    pub fn execute_set(&self, key: &[u8], value: &[u8], ttl_ms: Option<u64>) -> Result<(), ShardError> {
        let shard_idx = self.route_key(key);
        let result = self.dispatch_to_shard(
            shard_idx,
            ShardOp::Set {
                key: key.to_vec(),
                value: value.to_vec(),
                ttl_ms,
            },
        )?;
        match result {
            ShardResult::Ok => Ok(()),
            _ => Err(ShardError::WorkerDisconnected),
        }
    }

    /// Executes a DEL operation on the keyspace. Returns true if key was present.
    pub fn execute_del(&self, key: &[u8]) -> Result<bool, ShardError> {
        let shard_idx = self.route_key(key);
        let result = self.dispatch_to_shard(shard_idx, ShardOp::Del { key: key.to_vec() })?;
        match result {
            ShardResult::Bool(deleted) => Ok(deleted),
            _ => Err(ShardError::WorkerDisconnected),
        }
    }

    /// Checks if a key exists in the keyspace.
    pub fn execute_exists(&self, key: &[u8]) -> Result<bool, ShardError> {
        let shard_idx = self.route_key(key);
        let result = self.dispatch_to_shard(shard_idx, ShardOp::Exists { key: key.to_vec() })?;
        match result {
            ShardResult::Bool(exists) => Ok(exists),
            _ => Err(ShardError::WorkerDisconnected),
        }
    }

    /// Sets a TTL in milliseconds on the specified key. Returns true if key exists.
    pub fn execute_expire(&self, key: &[u8], ttl_ms: u64) -> Result<bool, ShardError> {
        let shard_idx = self.route_key(key);
        let result = self.dispatch_to_shard(shard_idx, ShardOp::Expire { key: key.to_vec(), ttl_ms })?;
        match result {
            ShardResult::Bool(b) => Ok(b),
            _ => Err(ShardError::WorkerDisconnected),
        }
    }

    /// Returns the remaining TTL in seconds for a key: -2 if key does not exist, -1 if no TTL, or seconds remaining.
    pub fn execute_ttl(&self, key: &[u8]) -> Result<i64, ShardError> {
        let shard_idx = self.route_key(key);
        let result = self.dispatch_to_shard(shard_idx, ShardOp::Ttl { key: key.to_vec() })?;
        match result {
            ShardResult::Ttl(ttl) => Ok(ttl),
            _ => Err(ShardError::WorkerDisconnected),
        }
    }

    /// Executes an atomic INCRBY operation on the target shard.
    pub fn execute_incrby(&self, key: &[u8], delta: i64) -> Result<i64, ShardError> {
        let shard_idx = self.route_key(key);
        let result = self.dispatch_to_shard(
            shard_idx,
            ShardOp::IncrBy {
                key: key.to_vec(),
                delta,
            },
        )?;
        match result {
            ShardResult::Int(val) => Ok(val),
            ShardResult::Error(err) => Err(ShardError::CommandError(err)),
            _ => Err(ShardError::WorkerDisconnected),
        }
    }

    /// Flushes all keys across all worker shards.
    pub fn execute_flushdb(&self) -> Result<(), ShardError> {
        for shard_idx in 0..self.num_shards() {
            let result = self.dispatch_to_shard(shard_idx, ShardOp::FlushDb)?;
            if result != ShardResult::Ok {
                return Err(ShardError::WorkerDisconnected);
            }
        }
        Ok(())
    }

    /// Aggregates all matching keys across all worker shards.
    pub fn execute_keys(&self, pattern: &[u8]) -> Result<Vec<Vec<u8>>, ShardError> {
        let mut all_keys = Vec::new();
        for shard_idx in 0..self.num_shards() {
            let result = self.dispatch_to_shard(
                shard_idx,
                ShardOp::Keys {
                    pattern: pattern.to_vec(),
                },
            )?;
            if let ShardResult::Keys(keys) = result {
                all_keys.extend(keys);
            }
        }
        Ok(all_keys)
    }

    /// Executes a multi-key MGET operation with deterministic ascending shard ID ordering.
    ///
    /// Sorting target shard IDs guarantees deadlock-free parallel execution across cores.
    pub fn execute_mget(&self, keys: &[&[u8]]) -> Result<Vec<Option<Vec<u8>>>, ShardError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        // 1. Group keys by destination shard index (ordered deterministically by shard ID)
        let mut shard_keys: BTreeMap<usize, Vec<(usize, Vec<u8>)>> = BTreeMap::new();
        for (orig_idx, &key) in keys.iter().enumerate() {
            let shard_id = self.route_key(key);
            shard_keys
                .entry(shard_id)
                .or_default()
                .push((orig_idx, key.to_vec()));
        }

        let mut results: Vec<Option<Vec<u8>>> = vec![None; keys.len()];

        // 2. Execute across shards in strictly ascending shard ID order
        for (shard_id, items) in shard_keys {
            for (orig_idx, key) in items {
                let res = self.dispatch_to_shard(shard_id, ShardOp::Get { key })?;
                if let ShardResult::Value(val) = res {
                    results[orig_idx] = val;
                }
            }
        }

        Ok(results)
    }

    /// Executes a multi-key MSET operation with deterministic ascending shard ID ordering.
    ///
    /// Prevents circular wait states across shards.
    pub fn execute_mset(&self, pairs: &[(&[u8], &[u8])]) -> Result<(), ShardError> {
        if pairs.is_empty() {
            return Ok(());
        }

        // 1. Group key-value pairs by destination shard index
        let mut shard_pairs: BTreeMap<usize, Vec<(Vec<u8>, Vec<u8>)>> = BTreeMap::new();
        for &(key, val) in pairs {
            let shard_id = self.route_key(key);
            shard_pairs
                .entry(shard_id)
                .or_default()
                .push((key.to_vec(), val.to_vec()));
        }

        // 2. Execute across shards in strictly ascending shard ID order
        for (shard_id, items) in shard_pairs {
            for (key, val) in items {
                let res = self.dispatch_to_shard(
                    shard_id,
                    ShardOp::Set {
                        key,
                        value: val,
                        ttl_ms: None,
                    },
                )?;
                if res != ShardResult::Ok {
                    return Err(ShardError::WorkerDisconnected);
                }
            }
        }

        Ok(())
    }

    /// Computes the total number of keys across all shards.
    pub fn total_keys(&self) -> Result<usize, ShardError> {
        let mut total = 0;
        for shard_idx in 0..self.workers.len() {
            let res = self.dispatch_to_shard(shard_idx, ShardOp::Len)?;
            if let ShardResult::Count(count) = res {
                total += count;
            }
        }
        Ok(total)
    }

    /// Collects consistent point-in-time snapshot items across all shards.
    pub fn collect_snapshot(&self, target_epoch: u64) -> Result<Vec<SnapshotItem>, ShardError> {
        let mut items = Vec::new();
        for shard_idx in 0..self.workers.len() {
            let res = self.dispatch_to_shard(shard_idx, ShardOp::SnapshotScan { target_epoch })?;
            if let ShardResult::Snapshot(shard_items) = res {
                items.extend(shard_items);
            } else {
                return Err(ShardError::WorkerDisconnected);
            }
        }
        Ok(items)
    }

    /// Cleanly signals all shard workers to stop and joins their threads.
    pub fn shutdown(&self) {
        for shard_idx in 0..self.workers.len() {
            let _ = self.dispatch_to_shard(shard_idx, ShardOp::Shutdown);
        }

        let mut handles = match self.join_handles.lock() {
            Ok(h) => h,
            Err(_) => return,
        };

        for handle in handles.drain(..) {
            let _ = handle.join();
        }
    }
}

impl Drop for ShardCoordinator {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// The independent event loop running inside each pinned worker thread.
fn run_shard_worker(rx: Consumer<ShardRequest>, tx: Producer<ShardResponse>) {
    let mut map = RobinHoodMap::new(1024);
    let mut ttl_map: HashMap<Vec<u8>, Instant> = HashMap::new();
    let shadow_buffer = ShadowBuffer::new();

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let mut wheel = TimingWheel::new(now_ms);

    loop {
        let mut processed_work = false;

        if let Some(req) = rx.pop() {
            processed_work = true;

            if req.op == ShardOp::Shutdown {
                let _ = tx.push(ShardResponse {
                    id: req.id,
                    result: ShardResult::Ok,
                });
                break;
            }

            let result = match req.op {
                ShardOp::Get { key } => {
                    // Check if key has expired in ttl_map
                    if let Some(expire_at) = ttl_map.get(&key) {
                        if Instant::now() >= *expire_at {
                            map.delete(&key);
                            ttl_map.remove(&key);
                            ShardResult::Value(None)
                        } else {
                            ShardResult::Value(map.get(&key).map(|v| v.to_vec()))
                        }
                    } else {
                        ShardResult::Value(map.get(&key).map(|v| v.to_vec()))
                    }
                }
                ShardOp::Set { key, value, ttl_ms } => {
                    map.insert(&key, &value);
                    if let Some(ms) = ttl_ms {
                        let expire_at = Instant::now() + Duration::from_millis(ms);
                        let curr_ms = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;
                        wheel.schedule(&key, curr_ms + ms);
                        ttl_map.insert(key, expire_at);
                    } else {
                        ttl_map.remove(&key);
                    }
                    ShardResult::Ok
                }
                ShardOp::Del { key } => {
                    let deleted = map.delete(&key);
                    ttl_map.remove(&key);
                    ShardResult::Bool(deleted)
                }
                ShardOp::Exists { key } => {
                    if let Some(expire_at) = ttl_map.get(&key) {
                        if Instant::now() >= *expire_at {
                            map.delete(&key);
                            ttl_map.remove(&key);
                            ShardResult::Bool(false)
                        } else {
                            ShardResult::Bool(map.contains_key(&key))
                        }
                    } else {
                        ShardResult::Bool(map.contains_key(&key))
                    }
                }
                ShardOp::Expire { key, ttl_ms } => {
                    if map.contains_key(&key) {
                        let deadline = Instant::now() + Duration::from_millis(ttl_ms);
                        ttl_map.insert(key.clone(), deadline);
                        let curr_ms = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;
                        wheel.schedule(&key, curr_ms + ttl_ms);
                        ShardResult::Bool(true)
                    } else {
                        ShardResult::Bool(false)
                    }
                }
                ShardOp::Ttl { key } => {
                    if !map.contains_key(&key) {
                        ShardResult::Ttl(-2)
                    } else if let Some(&expire_time) = ttl_map.get(&key) {
                        let now = Instant::now();
                        if now >= expire_time {
                            ttl_map.remove(&key);
                            map.delete(&key);
                            ShardResult::Ttl(-2)
                        } else {
                            let rem = expire_time.duration_since(now).as_secs() as i64;
                            ShardResult::Ttl(rem)
                        }
                    } else {
                        ShardResult::Ttl(-1)
                    }
                }
                ShardOp::Len => ShardResult::Count(map.len()),
                ShardOp::IncrBy { key, delta } => {
                    let now = Instant::now();
                    if let Some(expire_at) = ttl_map.get(&key) {
                        if now >= *expire_at {
                            map.delete(&key);
                            ttl_map.remove(&key);
                        }
                    }

                    let curr_val = if let Some(val_bytes) = map.get(&key) {
                        match std::str::from_utf8(val_bytes) {
                            Ok(s) => match s.parse::<i64>() {
                                Ok(v) => Ok(v),
                                Err(_) => {
                                    Err("ERR value is not an integer or out of range".to_string())
                                }
                            },
                            Err(_) => {
                                Err("ERR value is not an integer or out of range".to_string())
                            }
                        }
                    } else {
                        Ok(0)
                    };

                    match curr_val {
                        Err(err) => ShardResult::Error(err),
                        Ok(curr) => match curr.checked_add(delta) {
                            Some(new_val) => {
                                let new_val_str = new_val.to_string();
                                map.insert(&key, new_val_str.as_bytes());
                                ShardResult::Int(new_val)
                            }
                            None => {
                                ShardResult::Error("ERR increment or decrement would overflow".to_string())
                            }
                        },
                    }
                }
                ShardOp::FlushDb => {
                    map.clear();
                    ttl_map.clear();
                    ShardResult::Ok
                }
                ShardOp::Keys { pattern } => {
                    let now = Instant::now();
                    let mut matched = Vec::new();
                    for &k in &map.keys() {
                        if let Some(expire_at) = ttl_map.get(k) {
                            if now >= *expire_at {
                                continue;
                            }
                        }
                        if crate::protocol::resp::match_pattern(&pattern, k) {
                            matched.push(k.to_vec());
                        }
                    }
                    ShardResult::Keys(matched)
                }
                ShardOp::SnapshotScan { target_epoch } => {
                    let scan_now_ms = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    let expired = wheel.advance_to(scan_now_ms);
                    for exp_key in expired {
                        map.delete(&exp_key);
                        ttl_map.remove(&exp_key);
                    }
                    let now = Instant::now();
                    let mut expired_keys = Vec::new();
                    for (k, expire_at) in ttl_map.iter() {
                        if now >= *expire_at {
                            expired_keys.push(k.clone());
                        }
                    }
                    for k in expired_keys {
                        map.delete(&k);
                        ttl_map.remove(&k);
                    }

                    let scanner = SnapshotScanner::new(target_epoch, &map, &shadow_buffer);
                    let (items, _stats) = scanner.collect_all();
                    ShardResult::Snapshot(items)
                }
                ShardOp::Shutdown => unreachable!(),
            };

            // Push response back to coordinator
            let mut resp_to_send = ShardResponse { id: req.id, result };
            loop {
                match tx.push(resp_to_send) {
                    Ok(()) => break,
                    Err(QueueFull(returned)) => {
                        resp_to_send = returned;
                        std::hint::spin_loop();
                    }
                }
            }
        }

        // Advance hierarchical timing wheel and evict expired keys
        let curr_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let expired = wheel.advance_to(curr_ms);
        for key in expired {
            map.delete(&key);
            ttl_map.remove(&key);
        }

        if !processed_work {
            thread::yield_now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_coordinator_lifecycle_and_single_key_crud() {
        let coordinator = ShardCoordinator::new(4).expect("Failed to initialize 4-shard coordinator");
        assert_eq!(coordinator.num_shards(), 4);

        // SET & GET
        coordinator.execute_set(b"user:101", b"alice", None).unwrap();
        assert_eq!(
            coordinator.execute_get(b"user:101").unwrap(),
            Some(b"alice".to_vec())
        );

        // EXISTS
        assert!(coordinator.execute_exists(b"user:101").unwrap());
        assert!(!coordinator.execute_exists(b"user:999").unwrap());

        // DEL
        assert!(coordinator.execute_del(b"user:101").unwrap());
        assert_eq!(coordinator.execute_get(b"user:101").unwrap(), None);
        assert!(!coordinator.execute_del(b"user:101").unwrap());

        coordinator.shutdown();
    }

    #[test]
    fn test_coordinator_ttl_expiration() {
        let coordinator = ShardCoordinator::new(2).expect("Failed to create coordinator");

        // Set key with 60ms TTL
        coordinator.execute_set(b"temp:token", b"xyz", Some(60)).unwrap();
        assert_eq!(
            coordinator.execute_get(b"temp:token").unwrap(),
            Some(b"xyz".to_vec())
        );

        // Sleep to allow expiration
        thread::sleep(Duration::from_millis(90));

        // Key should have expired
        assert_eq!(coordinator.execute_get(b"temp:token").unwrap(), None);
        assert!(!coordinator.execute_exists(b"temp:token").unwrap());

        coordinator.shutdown();
    }

    #[test]
    fn test_deterministic_mget_and_mset() {
        let coordinator = ShardCoordinator::new(4).expect("Failed to create coordinator");

        let pairs: Vec<(&[u8], &[u8])> = vec![
            (b"account:1", b"val_1"),
            (b"account:2", b"val_2"),
            (b"account:3", b"val_3"),
            (b"account:4", b"val_4"),
            (b"account:5", b"val_5"),
        ];

        // MSET across all shards
        coordinator.execute_mset(&pairs).unwrap();
        assert_eq!(coordinator.total_keys().unwrap(), 5);

        // MGET in original key order
        let keys: Vec<&[u8]> = vec![b"account:3", b"account:1", b"account:5", b"account:nonexistent"];
        let mget_res = coordinator.execute_mget(&keys).unwrap();

        assert_eq!(mget_res.len(), 4);
        assert_eq!(mget_res[0], Some(b"val_3".to_vec()));
        assert_eq!(mget_res[1], Some(b"val_1".to_vec()));
        assert_eq!(mget_res[2], Some(b"val_5".to_vec()));
        assert_eq!(mget_res[3], None);

        coordinator.shutdown();
    }

    #[test]
    fn test_cross_shard_high_concurrency() {
        let coordinator = Arc::new(ShardCoordinator::new(4).expect("Failed to create coordinator"));
        let num_threads = 4;
        let ops_per_thread = 500;
        let mut handles = Vec::new();

        for thread_idx in 0..num_threads {
            let coord = Arc::clone(&coordinator);
            let handle = thread::spawn(move || {
                for i in 0..ops_per_thread {
                    let key = format!("thread:{}:key:{}", thread_idx, i);
                    let val = format!("val:{}", i);
                    coord.execute_set(key.as_bytes(), val.as_bytes(), None).unwrap();

                    let fetched = coord.execute_get(key.as_bytes()).unwrap();
                    assert_eq!(fetched, Some(val.into_bytes()));
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(coordinator.total_keys().unwrap(), num_threads * ops_per_thread);
        coordinator.shutdown();
    }

    #[test]
    fn test_cross_shard_snapshot_collection() {
        let coordinator = ShardCoordinator::new(4).expect("Failed to create coordinator");

        for i in 0..20 {
            let key = format!("snap:key:{}", i);
            let val = format!("val:{}", i);
            coordinator.execute_set(key.as_bytes(), val.as_bytes(), None).unwrap();
        }

        assert_eq!(coordinator.total_keys().unwrap(), 20);

        let snapshot_items = coordinator.collect_snapshot(1).unwrap();
        assert_eq!(snapshot_items.len(), 20);

        let mut collected_keys: Vec<String> = snapshot_items
            .into_iter()
            .map(|item| String::from_utf8(item.key).unwrap())
            .collect();
        collected_keys.sort();

        let mut expected_keys: Vec<String> = (0..20).map(|i| format!("snap:key:{}", i)).collect();
        expected_keys.sort();

        assert_eq!(collected_keys, expected_keys);

        coordinator.shutdown();
    }
}
