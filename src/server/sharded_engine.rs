//! Multi-Core Sharded Server Engine Event Loop
//!
//! Multiplexes non-blocking TCP client connections over a thread-per-core pinned
//! shared-nothing architecture:
//! - Dispatches single-key queries to dedicated pinned hardware cores via Jump Consistent Hash.
//! - Executes deterministic multi-shard transactions (MGET, MSET) in ascending shard ID order.
//! - Tracks monotonic mutation epochs for forkless point-in-time snapshot isolation.
//! - Orchestrates streaming binary persistence with ECMA-182 CRC64 integrity validation.

use std::io;
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::protocol::resp::{Command, ParseError, RespEncoder, RespParser};
use crate::server::connection::ClientConnection;
use crate::sharding::affinity::CpuTopology;
use crate::sharding::coordinator::ShardCoordinator;
use crate::snapshot::epoch::{EpochTracker, SnapshotRegistry};
use crate::snapshot::persistence::SnapshotWriter;

/// Operational metrics for the sharded server engine.
#[derive(Debug, Clone)]
pub struct ShardedServerStats {
    pub start_time: Instant,
    pub total_ops: u64,
    pub total_reads: u64,
    pub total_writes: u64,
    pub total_deletes: u64,
    pub clients_connected: usize,
}

impl Default for ShardedServerStats {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            total_ops: 0,
            total_reads: 0,
            total_writes: 0,
            total_deletes: 0,
            clients_connected: 0,
        }
    }
}

/// Production thread-per-core multi-shard network engine.
pub struct ShardedServerEngine {
    listener: TcpListener,
    coordinator: Arc<ShardCoordinator>,
    epoch_tracker: Arc<EpochTracker>,
    snapshot_registry: Arc<SnapshotRegistry>,
    connections: Vec<ClientConnection>,
    stats: ShardedServerStats,
    bind_addr: SocketAddr,
    num_shards: usize,
    last_save_time: Arc<AtomicU64>,
    data_dir: PathBuf,
}

impl ShardedServerEngine {
    /// Binds the sharded engine to the specified address with auto-discovered CPU cores.
    pub fn bind(addr: &str) -> io::Result<Self> {
        let topology = CpuTopology::discover();
        let num_shards = topology.total_cores.max(1);
        Self::bind_with_shards(addr, num_shards)
    }

    /// Binds the engine to the specified address with an explicit shard count.
    pub fn bind_with_shards(addr: &str, num_shards: usize) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        let bind_addr = listener.local_addr()?;

        let coordinator = ShardCoordinator::new(num_shards)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(Self {
            listener,
            coordinator: Arc::new(coordinator),
            epoch_tracker: Arc::new(EpochTracker::new()),
            snapshot_registry: Arc::new(SnapshotRegistry::new()),
            connections: Vec::with_capacity(512),
            stats: ShardedServerStats::default(),
            bind_addr,
            num_shards,
            last_save_time: Arc::new(AtomicU64::new(now_sec)),
            data_dir: PathBuf::from("./data"),
        })
    }

    /// Overrides the directory where point-in-time snapshots (.val) are persisted.
    pub fn with_data_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.data_dir = path.into();
        self
    }

    /// Bound local socket address.
    #[inline(always)]
    pub fn local_addr(&self) -> SocketAddr {
        self.bind_addr
    }

    /// Number of active worker shards.
    #[inline(always)]
    pub fn num_shards(&self) -> usize {
        self.num_shards
    }

    /// Reference to the underlying multi-shard coordinator.
    #[inline(always)]
    pub fn coordinator(&self) -> &Arc<ShardCoordinator> {
        &self.coordinator
    }

    /// Reference to the monotonic mutation epoch tracker.
    #[inline(always)]
    pub fn epoch_tracker(&self) -> &Arc<EpochTracker> {
        &self.epoch_tracker
    }

    /// Reference to the active snapshot registry.
    #[inline(always)]
    pub fn snapshot_registry(&self) -> &Arc<SnapshotRegistry> {
        &self.snapshot_registry
    }

    /// Operational metrics snapshot.
    #[inline(always)]
    pub fn stats(&self) -> &ShardedServerStats {
        &self.stats
    }

    /// Single non-blocking engine tick multiplexing incoming connections and dispatching frames.
    ///
    /// Returns the number of events (accepted conns, bytes read/written) processed.
    pub fn tick(&mut self) -> usize {
        let mut events = 0;

        // 1. Accept new incoming TCP connections
        loop {
            match self.listener.accept() {
                Ok((stream, _addr)) => {
                    if let Ok(conn) = ClientConnection::new(stream) {
                        self.connections.push(conn);
                        events += 1;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
                    continue;
                }
                Err(_) => {
                    break;
                }
            }
        }

        // 2. Poll and process active client connections
        let mut i = 0;
        while i < self.connections.len() {
            let conn = &mut self.connections[i];

            // Non-blocking read from network socket
            let read_res = conn.read_from_socket();
            if let Ok(n) = read_res {
                if n > 0 {
                    events += n;
                }
            }

            // Parse and execute complete RESP frames
            if conn.rx_len > 0 {
                Self::process_connection_frames(
                    conn,
                    &self.coordinator,
                    &self.epoch_tracker,
                    &self.snapshot_registry,
                    &self.last_save_time,
                    &self.data_dir,
                    self.num_shards,
                    &mut self.stats,
                );
            }

            // Flush buffered response bytes to socket
            let write_res = conn.flush_to_socket();
            if let Ok(n) = write_res {
                if n > 0 {
                    events += n;
                }
            }

            // Clean up disconnected clients
            if conn.is_closed {
                self.connections.swap_remove(i);
            } else {
                i += 1;
            }
        }

        self.stats.clients_connected = self.connections.len();
        events
    }

    /// Parses and processes all pending RESP frames for a client connection.
    #[inline]
    fn process_connection_frames(
        conn: &mut ClientConnection,
        coordinator: &Arc<ShardCoordinator>,
        epoch_tracker: &Arc<EpochTracker>,
        snapshot_registry: &Arc<SnapshotRegistry>,
        last_save_time: &Arc<AtomicU64>,
        data_dir: &Path,
        num_shards: usize,
        stats: &mut ShardedServerStats,
    ) {
        loop {
            if conn.rx_len == 0 {
                break;
            }

            match RespParser::parse(&conn.rx_buf[..conn.rx_len]) {
                Ok(Some((cmd, consumed))) => {
                    Self::execute_command(
                        &mut conn.tx_buf,
                        cmd,
                        coordinator,
                        epoch_tracker,
                        snapshot_registry,
                        last_save_time,
                        data_dir,
                        num_shards,
                        stats,
                    );

                    // Slide unread bytes to start of buffer
                    conn.rx_buf.copy_within(consumed..conn.rx_len, 0);
                    conn.rx_len -= consumed;
                }
                Ok(None) | Err(ParseError::Incomplete) => {
                    break;
                }
                Err(e) => {
                    let err_msg = match e {
                        ParseError::InvalidFrame(s) => format!("ERR Protocol error: {}", s),
                        ParseError::BadInteger => "ERR Value is not an integer".to_string(),
                        _ => "ERR Protocol syntax error".to_string(),
                    };
                    conn.tx_buf.extend_from_slice(&RespEncoder::error(&err_msg));
                    conn.rx_len = 0;
                    break;
                }
            }
        }
    }

    /// Dispatches parsed command against the multi-shard coordinator and snapshot engine.
    fn execute_command(
        tx_buf: &mut Vec<u8>,
        cmd: Command,
        coordinator: &Arc<ShardCoordinator>,
        epoch_tracker: &Arc<EpochTracker>,
        snapshot_registry: &Arc<SnapshotRegistry>,
        last_save_time: &Arc<AtomicU64>,
        data_dir: &Path,
        num_shards: usize,
        stats: &mut ShardedServerStats,
    ) {
        stats.total_ops += 1;

        match cmd {
            Command::Ping(msg) => {
                if let Some(m) = msg {
                    tx_buf.extend_from_slice(&RespEncoder::bulk_string(m));
                } else {
                    tx_buf.extend_from_slice(&RespEncoder::pong());
                }
            }
            Command::Get { key } => {
                stats.total_reads += 1;
                match coordinator.execute_get(key) {
                    Ok(Some(val)) => tx_buf.extend_from_slice(&RespEncoder::bulk_string(&val)),
                    Ok(None) => tx_buf.extend_from_slice(&RespEncoder::null()),
                    Err(e) => tx_buf.extend_from_slice(&RespEncoder::error(&format!("ERR Shard error: {}", e))),
                }
            }
            Command::Set { key, value, expire_ms } => {
                stats.total_writes += 1;
                epoch_tracker.advance();
                match coordinator.execute_set(key, value, expire_ms) {
                    Ok(()) => tx_buf.extend_from_slice(&RespEncoder::ok()),
                    Err(e) => tx_buf.extend_from_slice(&RespEncoder::error(&format!("ERR Shard error: {}", e))),
                }
            }
            Command::Del { key } => {
                stats.total_deletes += 1;
                epoch_tracker.advance();
                match coordinator.execute_del(key) {
                    Ok(deleted) => tx_buf.extend_from_slice(&RespEncoder::integer(if deleted { 1 } else { 0 })),
                    Err(e) => tx_buf.extend_from_slice(&RespEncoder::error(&format!("ERR Shard error: {}", e))),
                }
            }
            Command::Exists { key } => {
                stats.total_reads += 1;
                match coordinator.execute_exists(key) {
                    Ok(exists) => tx_buf.extend_from_slice(&RespEncoder::integer(if exists { 1 } else { 0 })),
                    Err(e) => tx_buf.extend_from_slice(&RespEncoder::error(&format!("ERR Shard error: {}", e))),
                }
            }
            Command::Expire { key, seconds } => {
                stats.total_writes += 1;
                epoch_tracker.advance();
                match coordinator.execute_expire(key, seconds * 1000) {
                    Ok(ok) => tx_buf.extend_from_slice(&RespEncoder::integer(if ok { 1 } else { 0 })),
                    Err(e) => tx_buf.extend_from_slice(&RespEncoder::error(&format!("ERR Shard error: {}", e))),
                }
            }
            Command::Ttl { key } => {
                stats.total_reads += 1;
                match coordinator.execute_ttl(key) {
                    Ok(ttl) => tx_buf.extend_from_slice(&RespEncoder::integer(ttl)),
                    Err(e) => tx_buf.extend_from_slice(&RespEncoder::error(&format!("ERR Shard error: {}", e))),
                }
            }
            Command::MGet { keys } => {
                stats.total_reads += keys.len() as u64;
                match coordinator.execute_mget(&keys) {
                    Ok(results) => {
                        let refs: Vec<Option<&[u8]>> = results.iter().map(|v| v.as_deref()).collect();
                        tx_buf.extend_from_slice(&RespEncoder::array(&refs));
                    }
                    Err(e) => tx_buf.extend_from_slice(&RespEncoder::error(&format!("ERR Shard error: {}", e))),
                }
            }
            Command::MSet { pairs } => {
                stats.total_writes += pairs.len() as u64;
                epoch_tracker.advance();
                match coordinator.execute_mset(&pairs) {
                    Ok(()) => tx_buf.extend_from_slice(&RespEncoder::ok()),
                    Err(e) => tx_buf.extend_from_slice(&RespEncoder::error(&format!("ERR Shard error: {}", e))),
                }
            }
            Command::DbSize => {
                stats.total_reads += 1;
                match coordinator.total_keys() {
                    Ok(count) => tx_buf.extend_from_slice(&RespEncoder::integer(count as i64)),
                    Err(e) => tx_buf.extend_from_slice(&RespEncoder::error(&format!("ERR Shard error: {}", e))),
                }
            }
            Command::Command => {
                tx_buf.extend_from_slice(b"*0\r\n");
            }
            Command::Info => {
                let uptime_sec = stats.start_time.elapsed().as_secs();
                let total_keys = coordinator.total_keys().unwrap_or(0);
                let curr_epoch = epoch_tracker.current();
                let info_str = format!(
                    "# Server\r\n\
                    valence_version:0.1.0\r\n\
                    mode:sharded\r\n\
                    os:Windows\r\n\
                    arch:x86_64\r\n\
                    uptime_in_seconds:{}\r\n\
                    num_shards:{}\r\n\
                    active_epoch:{}\r\n\
                    \r\n\
                    # Clients\r\n\
                    connected_clients:{}\r\n\
                    \r\n\
                    # Stats\r\n\
                    total_commands_processed:{}\r\n\
                    total_reads:{}\r\n\
                    total_writes:{}\r\n\
                    total_deletes:{}\r\n\
                    \r\n\
                    # Keyspace\r\n\
                    db0:keys={}\r\n",
                    uptime_sec,
                    num_shards,
                    curr_epoch,
                    stats.clients_connected,
                    stats.total_ops,
                    stats.total_reads,
                    stats.total_writes,
                    stats.total_deletes,
                    total_keys
                );
                tx_buf.extend_from_slice(&RespEncoder::bulk_string(info_str.as_bytes()));
            }
            Command::Save => {
                let target_epoch = epoch_tracker.current();
                let meta = snapshot_registry.register(target_epoch);

                match coordinator.collect_snapshot(target_epoch) {
                    Ok(items) => {
                        let now_ms = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis();
                        let filename = format!("snapshot_epoch_{}_{}.val", target_epoch, now_ms);
                        let file_path = data_dir.join(filename);

                        match SnapshotWriter::write_snapshot(&file_path, target_epoch, &items) {
                            Ok(_) => {
                                let _ = snapshot_registry.complete(meta.id);
                                let now_sec = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();
                                last_save_time.store(now_sec, Ordering::SeqCst);
                                tx_buf.extend_from_slice(&RespEncoder::ok());
                            }
                            Err(e) => {
                                let _ = snapshot_registry.abort(meta.id);
                                tx_buf.extend_from_slice(&RespEncoder::error(&format!("ERR Save failed: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        let _ = snapshot_registry.abort(meta.id);
                        tx_buf.extend_from_slice(&RespEncoder::error(&format!("ERR Snapshot collection failed: {}", e)));
                    }
                }
            }
            Command::Bgsave => {
                let target_epoch = epoch_tracker.current();
                let meta = snapshot_registry.register(target_epoch);
                let coord_clone = Arc::clone(coordinator);
                let reg_clone = Arc::clone(snapshot_registry);
                let last_save_clone = Arc::clone(last_save_time);
                let dir_clone = data_dir.to_path_buf();

                thread::spawn(move || {
                    match coord_clone.collect_snapshot(target_epoch) {
                        Ok(items) => {
                            let now_ms = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis();
                            let filename = format!("snapshot_epoch_{}_{}.val", target_epoch, now_ms);
                            let file_path = dir_clone.join(filename);

                            match SnapshotWriter::write_snapshot(&file_path, target_epoch, &items) {
                                Ok(_) => {
                                    let _ = reg_clone.complete(meta.id);
                                    let now_sec = SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs();
                                    last_save_clone.store(now_sec, Ordering::SeqCst);
                                }
                                Err(_) => {
                                    let _ = reg_clone.abort(meta.id);
                                }
                            }
                        }
                        Err(_) => {
                            let _ = reg_clone.abort(meta.id);
                        }
                    }
                });

                tx_buf.extend_from_slice(&RespEncoder::simple_string("Background saving started"));
            }
            Command::Lastsave => {
                let last = last_save_time.load(Ordering::Acquire);
                tx_buf.extend_from_slice(&RespEncoder::integer(last as i64));
            }
            Command::Unknown(cmd_name) => {
                let name_str = String::from_utf8_lossy(cmd_name);
                let err = format!("unknown command '{}', with args beginning with: ", name_str);
                tx_buf.extend_from_slice(&RespEncoder::error(&err));
            }
        }
    }

    /// Runs the non-blocking sharded engine event loop.
    pub fn run(&mut self) -> io::Result<()> {
        println!("============================================================");
        println!("Valence Sharded Server Engine listening on {}", self.bind_addr);
        println!("Architecture: Thread-Per-Core ({} pinned shards)", self.num_shards);
        println!("Persistence:  Forkless Epoch Snapshots (.val with CRC64)");
        println!("============================================================");

        let mut last_metric_log = Instant::now();

        loop {
            let events = self.tick();

            if last_metric_log.elapsed() >= Duration::from_secs(10) {
                let total_keys = self.coordinator.total_keys().unwrap_or(0);
                println!(
                    "[STATUS] Clients: {} | Total Ops: {} (R: {}, W: {}, D: {}) | Keys: {} | Shards: {} | Epoch: {}",
                    self.stats.clients_connected,
                    self.stats.total_ops,
                    self.stats.total_reads,
                    self.stats.total_writes,
                    self.stats.total_deletes,
                    total_keys,
                    self.num_shards,
                    self.epoch_tracker.current()
                );
                last_metric_log = Instant::now();
            }

            if events == 0 {
                thread::sleep(Duration::from_micros(100));
            }
        }
    }
}
