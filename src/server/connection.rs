//! Client Connection & Command Execution Handler
//!
//! Handles non-blocking TCP socket reads, zero-copy RESP frame parsing,
//! execution against the underlying Robin Hood storage map, and wire response buffering.

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::protocol::resp::{Command, ParseError, RespEncoder, RespParser};
use crate::storage::RobinHoodMap;
use crate::timer::TimingWheel;

const RECEIVE_BUFFER_SIZE: usize = 64 * 1024; // 64 KB receive buffer

/// Operational metrics for the server instance.
#[derive(Debug, Clone)]
pub struct ServerStats {
    pub start_time: Instant,
    pub total_ops: u64,
    pub total_reads: u64,
    pub total_writes: u64,
    pub total_deletes: u64,
    pub clients_connected: usize,
}

impl Default for ServerStats {
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

/// Active client TCP connection state.
pub struct ClientConnection {
    pub stream: TcpStream,
    pub rx_buf: Vec<u8>,
    pub rx_len: usize,
    pub tx_buf: Vec<u8>,
    pub is_closed: bool,
}

impl ClientConnection {
    /// Initializes a new client connection in non-blocking mode with TCP_NODELAY.
    pub fn new(stream: TcpStream) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        stream.set_nodelay(true)?;

        let mut rx_buf = Vec::with_capacity(RECEIVE_BUFFER_SIZE);
        rx_buf.resize(RECEIVE_BUFFER_SIZE, 0);

        Ok(Self {
            stream,
            rx_buf,
            rx_len: 0,
            tx_buf: Vec::with_capacity(4096),
            is_closed: false,
        })
    }

    /// Non-blocking read from the TCP socket into the receive buffer.
    ///
    /// Returns:
    /// - `Ok(bytes_read)`
    /// - `Ok(0)` if connection was cleanly closed by remote peer.
    /// - Handles `WouldBlock` and `Interrupted` transparently.
    pub fn read_from_socket(&mut self) -> io::Result<usize> {
        if self.rx_len >= self.rx_buf.len() {
            // Expand receive buffer if full and cannot hold further data
            self.rx_buf.resize(self.rx_buf.len() * 2, 0);
        }

        match self.stream.read(&mut self.rx_buf[self.rx_len..]) {
            Ok(0) => {
                self.is_closed = true;
                Ok(0)
            }
            Ok(n) => {
                self.rx_len += n;
                Ok(n)
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(0),
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => Ok(0),
            Err(e) => {
                self.is_closed = true;
                Err(e)
            }
        }
    }

    /// Non-blocking write of any buffered response bytes to the TCP socket.
    pub fn flush_to_socket(&mut self) -> io::Result<usize> {
        if self.tx_buf.is_empty() {
            return Ok(0);
        }

        match self.stream.write(&self.tx_buf) {
            Ok(n) => {
                self.tx_buf.drain(..n);
                Ok(n)
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(0),
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => Ok(0),
            Err(e) => {
                self.is_closed = true;
                Err(e)
            }
        }
    }

    /// Parses and processes all complete RESP command frames available in the receive buffer.
    pub fn process_incoming(
        &mut self,
        map: &mut RobinHoodMap,
        ttl_map: &mut HashMap<Vec<u8>, Instant>,
        wheel: &TimingWheel,
        stats: &mut ServerStats,
    ) {
        loop {
            if self.rx_len == 0 {
                break;
            }

            match RespParser::parse(&self.rx_buf[..self.rx_len]) {
                Ok(Some((cmd, consumed))) => {
                    Self::execute_command(&mut self.tx_buf, cmd, map, ttl_map, wheel, stats);

                    // Slide buffer forward
                    self.rx_buf.copy_within(consumed..self.rx_len, 0);
                    self.rx_len -= consumed;
                }
                Ok(None) => {
                    // Incomplete frame; await additional network packets
                    break;
                }
                Err(ParseError::Incomplete) => {
                    break;
                }
                Err(e) => {
                    // Protocol error: send descriptive error frame and clear buffer
                    let err_msg = match e {
                        ParseError::InvalidFrame(s) => format!("ERR Protocol error: {}", s),
                        ParseError::BadInteger => "ERR Value is not an integer".to_string(),
                        _ => "ERR Protocol syntax error".to_string(),
                    };
                    self.tx_buf.extend_from_slice(&RespEncoder::error(&err_msg));
                    self.rx_len = 0;
                    break;
                }
            }
        }
    }

    /// Dispatches parsed command against the storage engine.
    fn execute_command(
        tx_buf: &mut Vec<u8>,
        cmd: Command,
        map: &mut RobinHoodMap,
        ttl_map: &mut HashMap<Vec<u8>, Instant>,
        wheel: &TimingWheel,
        stats: &mut ServerStats,
    ) {
        stats.total_ops += 1;

        // Lazy TTL eviction check helper
        let now = Instant::now();

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

                // Check TTL expiration
                if let Some(&expire_time) = ttl_map.get(key) {
                    if now >= expire_time {
                        ttl_map.remove(key);
                        map.delete(key);
                        tx_buf.extend_from_slice(&RespEncoder::null());
                        return;
                    }
                }

                if let Some(val) = map.get(key) {
                    tx_buf.extend_from_slice(&RespEncoder::bulk_string(val));
                } else {
                    tx_buf.extend_from_slice(&RespEncoder::null());
                }
            }
            Command::Set { key, value, expire_ms } => {
                stats.total_writes += 1;
                map.insert(key, value);

                if let Some(ms) = expire_ms {
                    let deadline = now + Duration::from_millis(ms);
                    ttl_map.insert(key.to_vec(), deadline);
                    let wall_now_ms = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    wheel.schedule(key, wall_now_ms + ms);
                } else {
                    ttl_map.remove(key);
                }

                tx_buf.extend_from_slice(&RespEncoder::ok());
            }
            Command::Del { key } => {
                stats.total_deletes += 1;
                ttl_map.remove(key);
                let existed = map.delete(key);
                tx_buf.extend_from_slice(&RespEncoder::integer(if existed { 1 } else { 0 }));
            }
            Command::Expire { key, seconds } => {
                if map.contains_key(key) {
                    let deadline = now + Duration::from_secs(seconds);
                    ttl_map.insert(key.to_vec(), deadline);
                    let wall_now_ms = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    wheel.schedule(key, wall_now_ms + seconds * 1000);
                    tx_buf.extend_from_slice(&RespEncoder::integer(1));
                } else {
                    tx_buf.extend_from_slice(&RespEncoder::integer(0));
                }
            }
            Command::Ttl { key } => {
                if !map.contains_key(key) {
                    tx_buf.extend_from_slice(&RespEncoder::integer(-2));
                } else if let Some(&expire_time) = ttl_map.get(key) {
                    if now >= expire_time {
                        ttl_map.remove(key);
                        map.delete(key);
                        tx_buf.extend_from_slice(&RespEncoder::integer(-2));
                    } else {
                        let remaining = expire_time.duration_since(now).as_secs() as i64;
                        tx_buf.extend_from_slice(&RespEncoder::integer(remaining));
                    }
                } else {
                    tx_buf.extend_from_slice(&RespEncoder::integer(-1));
                }
            }
            Command::Command => {
                // Return empty array for Redis-CLI handshake
                tx_buf.extend_from_slice(b"*0\r\n");
            }
            Command::Info => {
                let uptime_sec = stats.start_time.elapsed().as_secs();
                let load_factor = map.load_factor();
                let info_str = format!(
                    "# Server\r\n\
                    valence_version:0.1.0\r\n\
                    os:Windows\r\n\
                    arch:x86_64\r\n\
                    uptime_in_seconds:{}\r\n\
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
                    db0:keys={},capacity={},load_factor={:.2}\r\n",
                    uptime_sec,
                    stats.clients_connected,
                    stats.total_ops,
                    stats.total_reads,
                    stats.total_writes,
                    stats.total_deletes,
                    map.len(),
                    map.capacity(),
                    load_factor
                );
                tx_buf.extend_from_slice(&RespEncoder::bulk_string(info_str.as_bytes()));
            }
            Command::Unknown(cmd_name) => {
                let name_str = String::from_utf8_lossy(cmd_name);
                let err = format!("unknown command '{}', with args beginning with: ", name_str);
                tx_buf.extend_from_slice(&RespEncoder::error(&err));
            }
        }
    }
}
