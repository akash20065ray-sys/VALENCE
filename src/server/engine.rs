//! Single-Threaded Non-Blocking Engine Event Loop
//!
//! Multiplexes incoming TCP connections, parses RESP protocol frames,
//! and executes atomic mutations against the Robin Hood storage map.

use std::collections::HashMap;
use std::io;
use std::net::{SocketAddr, TcpListener};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::server::connection::{ClientConnection, ServerStats};
use crate::storage::RobinHoodMap;
use crate::timer::TimingWheel;

pub struct ServerEngine {
    listener: TcpListener,
    map: RobinHoodMap,
    ttl_map: HashMap<Vec<u8>, Instant>,
    wheel: TimingWheel,
    connections: Vec<ClientConnection>,
    stats: ServerStats,
    bind_addr: SocketAddr,
}

impl ServerEngine {
    /// Binds the server engine to the specified address.
    pub fn bind(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        let bind_addr = listener.local_addr()?;

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Ok(Self {
            listener,
            map: RobinHoodMap::new(1024),
            ttl_map: HashMap::new(),
            wheel: TimingWheel::new(now_ms),
            connections: Vec::with_capacity(256),
            stats: ServerStats::default(),
            bind_addr,
        })
    }

    /// Returns the bound local socket address.
    pub fn local_addr(&self) -> SocketAddr {
        self.bind_addr
    }

    /// Single non-blocking engine tick.
    ///
    /// Returns the number of events processed during this tick.
    pub fn tick(&mut self) -> usize {
        let mut events = 0;

        // 1. Accept new incoming client TCP connections
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

            // Read available network data
            let read_res = conn.read_from_socket();
            if let Ok(n) = read_res {
                if n > 0 {
                    events += n;
                }
            }

            // Process parsed RESP command frames
            if conn.rx_len > 0 {
                conn.process_incoming(&mut self.map, &mut self.ttl_map, &self.wheel, &mut self.stats);
            }

            // Flush buffered response frames
            let write_res = conn.flush_to_socket();
            if let Ok(n) = write_res {
                if n > 0 {
                    events += n;
                }
            }

            // Clean up disconnected or errored clients
            if conn.is_closed {
                self.connections.swap_remove(i);
            } else {
                i += 1;
            }
        }

        // 3. Advance timing wheel and detach expired timers in O(1) time
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let expired = self.wheel.advance_to(now_ms);
        if !expired.is_empty() {
            events += expired.len();
            for key in expired {
                self.map.delete(&key);
                self.ttl_map.remove(&key);
                self.stats.total_deletes += 1;
            }
        }

        self.stats.clients_connected = self.connections.len();
        events
    }

    /// Runs the server event loop continuously.
    pub fn run(&mut self) -> io::Result<()> {
        println!("Valence Server Engine listening on {}", self.bind_addr);
        println!("Storage Engine: Contiguous Robin Hood Map with Backshift Deletion");

        let mut last_metric_log = Instant::now();

        loop {
            let events = self.tick();

            // Periodic operational logging every 10 seconds
            if last_metric_log.elapsed() >= Duration::from_secs(10) {
                println!(
                    "[STATUS] Clients: {} | Total Ops: {} (R: {}, W: {}, D: {}) | Keys: {} | Capacity: {} | Load: {:.2}",
                    self.stats.clients_connected,
                    self.stats.total_ops,
                    self.stats.total_reads,
                    self.stats.total_writes,
                    self.stats.total_deletes,
                    self.map.len(),
                    self.map.capacity(),
                    self.map.load_factor()
                );
                last_metric_log = Instant::now();
            }

            // If no I/O events occurred, sleep briefly (100 µs) to prevent 100% CPU burn
            if events == 0 {
                std::thread::sleep(Duration::from_micros(100));
            }
        }
    }
}
