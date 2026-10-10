//! Integration Tests for Non-Blocking TCP Server & Wire Protocol

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use valence::ServerEngine;

#[test]
fn test_server_tcp_e2e_commands() {
    let mut engine = ServerEngine::bind("127.0.0.1:0").expect("Failed to bind ephemeral port");
    let addr = engine.local_addr();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    // Spawn server tick loop in background thread
    let server_handle = thread::spawn(move || {
        while running_clone.load(Ordering::Relaxed) {
            let events = engine.tick();
            if events == 0 {
                thread::sleep(Duration::from_micros(100));
            }
        }
    });

    // Connect client
    let mut stream = TcpStream::connect(addr).expect("Client failed to connect to server");
    stream.set_nodelay(true).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();

    let mut buf = [0u8; 1024];

    // 1. PING -> +PONG\r\n
    stream.write_all(b"*1\r\n$4\r\nPING\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"+PONG\r\n");

    // 2. SET key "valence_val" -> +OK\r\n
    stream
        .write_all(b"*3\r\n$3\r\nSET\r\n$7\r\nkey_foo\r\n$11\r\nvalence_val\r\n")
        .unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"+OK\r\n");

    // 3. GET key -> $11\r\nvalence_val\r\n
    stream.write_all(b"*2\r\n$3\r\nGET\r\n$7\r\nkey_foo\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"$11\r\nvalence_val\r\n");

    // 4. DEL key -> :1\r\n
    stream.write_all(b"*2\r\n$3\r\nDEL\r\n$7\r\nkey_foo\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b":1\r\n");

    // 5. GET key again -> $-1\r\n (Null bulk string)
    stream.write_all(b"*2\r\n$3\r\nGET\r\n$7\r\nkey_foo\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"$-1\r\n");

    // 6. DEL non-existent key -> :0\r\n
    stream.write_all(b"*2\r\n$3\r\nDEL\r\n$7\r\nkey_foo\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b":0\r\n");

    // 7. PING with argument -> bulk string
    stream.write_all(b"*2\r\n$4\r\nPING\r\n$5\r\nhello\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"$5\r\nhello\r\n");

    // 8. INFO -> contains valence_version
    stream.write_all(b"*1\r\n$4\r\nINFO\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    let info_response = String::from_utf8_lossy(&buf[..n]);
    assert!(info_response.contains("valence_version:0.1.0"));
    assert!(info_response.contains("total_commands_processed:8"));

    // Shutdown test engine
    running.store(false, Ordering::Relaxed);
    let _ = server_handle.join();
}

#[test]
fn test_pipelined_tcp_requests() {
    let mut engine = ServerEngine::bind("127.0.0.1:0").expect("Failed to bind ephemeral port");
    let addr = engine.local_addr();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let server_handle = thread::spawn(move || {
        while running_clone.load(Ordering::Relaxed) {
            let events = engine.tick();
            if events == 0 {
                thread::sleep(Duration::from_micros(100));
            }
        }
    });

    let mut stream = TcpStream::connect(addr).unwrap();
    stream.set_nodelay(true).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();

    // Send 3 pipelined commands in a single TCP packet
    let pipeline = b"*1\r\n$4\r\nPING\r\n*3\r\n$3\r\nSET\r\n$1\r\na\r\n$1\r\n1\r\n*2\r\n$3\r\nGET\r\n$1\r\na\r\n";
    stream.write_all(pipeline).unwrap();

    let mut total_read = 0;
    let mut buf = [0u8; 1024];
    let expected = b"+PONG\r\n+OK\r\n$1\r\n1\r\n";

    while total_read < expected.len() {
        let n = stream.read(&mut buf[total_read..]).unwrap();
        assert!(n > 0);
        total_read += n;
    }

    assert_eq!(&buf[..total_read], expected);

    running.store(false, Ordering::Relaxed);
    let _ = server_handle.join();
}

#[test]
fn test_tcp_expiration_via_timing_wheel() {
    let mut engine = ServerEngine::bind("127.0.0.1:0").expect("Failed to bind ephemeral port");
    let addr = engine.local_addr();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let server_handle = thread::spawn(move || {
        while running_clone.load(Ordering::Relaxed) {
            let events = engine.tick();
            if events == 0 {
                thread::sleep(Duration::from_micros(100));
            }
        }
    });

    let mut stream = TcpStream::connect(addr).unwrap();
    stream.set_nodelay(true).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();

    let mut buf = [0u8; 1024];

    // SET key_exp "fast_vanish" PX 150
    stream
        .write_all(b"*5\r\n$3\r\nSET\r\n$7\r\nkey_exp\r\n$11\r\nfast_vanish\r\n$2\r\nPX\r\n$3\r\n150\r\n")
        .unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"+OK\r\n");

    // Immediate GET -> should exist
    stream.write_all(b"*2\r\n$3\r\nGET\r\n$7\r\nkey_exp\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"$11\r\nfast_vanish\r\n");

    // Sleep 250ms to let the timing wheel advance and detach the atomic bucket
    thread::sleep(Duration::from_millis(250));

    // GET after deadline -> should be null ($-1\r\n)
    stream.write_all(b"*2\r\n$3\r\nGET\r\n$7\r\nkey_exp\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"$-1\r\n");

    running.store(false, Ordering::Relaxed);
    let _ = server_handle.join();
}

#[test]
fn test_universal_compatibility_commands() {
    let mut engine = ServerEngine::bind("127.0.0.1:0").expect("Failed to bind ephemeral port");
    let addr = engine.local_addr();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let server_handle = thread::spawn(move || {
        while running_clone.load(Ordering::Relaxed) {
            let events = engine.tick();
            if events == 0 {
                thread::sleep(Duration::from_micros(100));
            }
        }
    });

    let mut stream = TcpStream::connect(addr).unwrap();
    stream.set_nodelay(true).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();

    let mut buf = [0u8; 1024];

    // 1. SELECT 0 -> +OK\r\n
    stream.write_all(b"*2\r\n$6\r\nSELECT\r\n$1\r\n0\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"+OK\r\n");

    // 2. CLIENT SETNAME my_client -> +OK\r\n
    stream.write_all(b"*3\r\n$6\r\nCLIENT\r\n$7\r\nSETNAME\r\n$9\r\nmy_client\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"+OK\r\n");

    // 3. CLIENT LIST -> bulk string
    stream.write_all(b"*2\r\n$6\r\nCLIENT\r\n$4\r\nLIST\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert!(buf[..n].starts_with(b"$"));

    // 4. INCR visits -> :1\r\n
    stream.write_all(b"*2\r\n$4\r\nINCR\r\n$6\r\nvisits\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b":1\r\n");

    // 5. INCR visits -> :2\r\n
    stream.write_all(b"*2\r\n$4\r\nINCR\r\n$6\r\nvisits\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b":2\r\n");

    // 6. INCRBY visits 10 -> :12\r\n
    stream.write_all(b"*3\r\n$6\r\nINCRBY\r\n$6\r\nvisits\r\n$2\r\n10\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b":12\r\n");

    // 7. DECR visits -> :11\r\n
    stream.write_all(b"*2\r\n$4\r\nDECR\r\n$6\r\nvisits\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b":11\r\n");

    // 8. DECRBY visits 5 -> :6\r\n
    stream.write_all(b"*3\r\n$6\r\nDECRBY\r\n$6\r\nvisits\r\n$1\r\n5\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b":6\r\n");

    // 9. KEYS * -> array containing "visits"
    stream.write_all(b"*2\r\n$4\r\nKEYS\r\n$1\r\n*\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"*1\r\n$6\r\nvisits\r\n");

    // 10. FLUSHDB -> +OK\r\n
    stream.write_all(b"*1\r\n$7\r\nFLUSHDB\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"+OK\r\n");

    // 11. DBSIZE -> :0\r\n
    stream.write_all(b"*1\r\n$6\r\nDBSIZE\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b":0\r\n");

    running.store(false, Ordering::Relaxed);
    let _ = server_handle.join();
}

