//! Integration Tests for Thread-per-Core Sharded TCP Server & Persistence

use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use valence::{ShardedServerEngine, SnapshotReader};

#[test]
fn test_sharded_server_tcp_e2e_commands() {
    let test_dir = std::env::temp_dir().join(format!("val_sharded_tcp_{}", std::process::id()));
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    let mut engine = ShardedServerEngine::bind_with_data_dir("127.0.0.1:0", 4, &test_dir)
        .expect("Failed to bind 4-shard server");
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

    let mut stream = TcpStream::connect(addr).expect("Client failed to connect to server");
    stream.set_nodelay(true).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();

    let mut buf = [0u8; 2048];

    // 1. PING -> +PONG\r\n
    stream.write_all(b"*1\r\n$4\r\nPING\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"+PONG\r\n");

    // 2. SET shard_key_1 "shard_val_1" -> +OK\r\n
    stream
        .write_all(b"*3\r\n$3\r\nSET\r\n$11\r\nshard_key_1\r\n$11\r\nshard_val_1\r\n")
        .unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"+OK\r\n");

    // 3. GET shard_key_1 -> $11\r\nshard_val_1\r\n
    stream.write_all(b"*2\r\n$3\r\nGET\r\n$11\r\nshard_key_1\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"$11\r\nshard_val_1\r\n");

    // 4. EXISTS shard_key_1 -> :1\r\n
    stream.write_all(b"*2\r\n$6\r\nEXISTS\r\n$11\r\nshard_key_1\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b":1\r\n");

    // 5. EXISTS non_existent -> :0\r\n
    stream.write_all(b"*2\r\n$6\r\nEXISTS\r\n$12\r\nnon_existent\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b":0\r\n");

    // 6. MSET across multiple shards -> +OK\r\n
    stream
        .write_all(b"*5\r\n$4\r\nMSET\r\n$2\r\nk1\r\n$2\r\nv1\r\n$2\r\nk2\r\n$2\r\nv2\r\n")
        .unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"+OK\r\n");

    // 7. MGET across multiple shards -> *3\r\n$2\r\nv1\r\n$2\r\nv2\r\n$-1\r\n
    stream
        .write_all(b"*4\r\n$4\r\nMGET\r\n$2\r\nk1\r\n$2\r\nk2\r\n$7\r\nmissing\r\n")
        .unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"*3\r\n$2\r\nv1\r\n$2\r\nv2\r\n$-1\r\n");

    // 8. DBSIZE -> :3\r\n (shard_key_1, k1, k2)
    stream.write_all(b"*1\r\n$6\r\nDBSIZE\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b":3\r\n");

    // 9. DEL shard_key_1 -> :1\r\n
    stream.write_all(b"*2\r\n$3\r\nDEL\r\n$11\r\nshard_key_1\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b":1\r\n");

    // 10. INFO -> verify sharded info
    stream.write_all(b"*1\r\n$4\r\nINFO\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    let info_text = String::from_utf8_lossy(&buf[..n]);
    assert!(info_text.contains("mode:sharded"));
    assert!(info_text.contains("num_shards:4"));
    assert!(info_text.contains("db0:keys=2"));

    running.store(false, Ordering::Relaxed);
    let _ = server_handle.join();
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_sharded_server_ttl_expiration() {
    let test_dir = std::env::temp_dir().join(format!("val_sharded_ttl_{}", std::process::id()));
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    let mut engine = ShardedServerEngine::bind_with_data_dir("127.0.0.1:0", 2, &test_dir)
        .expect("Failed to bind 2-shard server");
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

    let mut stream = TcpStream::connect(addr).expect("Client failed to connect");
    stream.set_nodelay(true).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();

    let mut buf = [0u8; 1024];

    // SET temporary_key "secret"
    stream
        .write_all(b"*3\r\n$3\r\nSET\r\n$13\r\ntemporary_key\r\n$6\r\nsecret\r\n")
        .unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"+OK\r\n");

    // EXPIRE temporary_key 1 (1 second) -> :1\r\n
    stream
        .write_all(b"*3\r\n$6\r\nEXPIRE\r\n$13\r\ntemporary_key\r\n$1\r\n1\r\n")
        .unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b":1\r\n");

    // TTL temporary_key -> :1\r\n (or 0)
    stream.write_all(b"*2\r\n$3\r\nTTL\r\n$13\r\ntemporary_key\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    let ttl_resp = String::from_utf8_lossy(&buf[..n]);
    assert!(ttl_resp.starts_with(':'));

    // Sleep 1200ms for expiration
    thread::sleep(Duration::from_millis(1200));

    // GET temporary_key -> $-1\r\n (expired)
    stream.write_all(b"*2\r\n$3\r\nGET\r\n$13\r\ntemporary_key\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"$-1\r\n");

    running.store(false, Ordering::Relaxed);
    let _ = server_handle.join();
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_sharded_server_forkless_save_and_lastsave() {
    let test_dir = std::env::temp_dir().join(format!("valence_test_snap_{}", std::process::id()));
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    let mut engine = ShardedServerEngine::bind_with_data_dir("127.0.0.1:0", 4, &test_dir)
        .expect("Failed to bind 4-shard server");
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

    let mut stream = TcpStream::connect(addr).expect("Client failed to connect");
    stream.set_nodelay(true).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();

    let mut buf = [0u8; 1024];

    // Populate keys across shards
    for i in 0..15 {
        let key = format!("key_{}", i);
        let val = format!("val_{}", i);
        let cmd = format!(
            "*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
            key.len(),
            key,
            val.len(),
            val
        );
        stream.write_all(cmd.as_bytes()).unwrap();
        let n = stream.read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"+OK\r\n");
    }

    // Trigger synchronous SAVE -> +OK\r\n
    stream.write_all(b"*1\r\n$4\r\nSAVE\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"+OK\r\n");

    // Check LASTSAVE -> integer timestamp > 0
    stream.write_all(b"*1\r\n$8\r\nLASTSAVE\r\n").unwrap();
    let n = stream.read(&mut buf).unwrap();
    let lastsave_str = String::from_utf8_lossy(&buf[..n]);
    assert!(lastsave_str.starts_with(':'));

    // Verify snapshot file exists on disk and is non-empty
    let entries = fs::read_dir(&test_dir).unwrap();
    let snap_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "val"))
        .collect();
    assert_eq!(snap_files.len(), 1, "Exactly one .val file should be generated");

    // Read and verify using SnapshotReader (verifies magic, header, CRC64 checksum, and all 15 items)
    let (header, items) = SnapshotReader::read_snapshot(&snap_files[0]).expect("Snapshot file corruption");
    assert_eq!(header.item_count, 15);
    assert_eq!(items.len(), 15);

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);

    running.store(false, Ordering::Relaxed);
    let _ = server_handle.join();
}

#[test]
fn test_sharded_server_pipelined_and_multi_client() {
    let test_dir = std::env::temp_dir().join(format!("val_sharded_pipeline_{}", std::process::id()));
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    let mut engine = ShardedServerEngine::bind_with_data_dir("127.0.0.1:0", 4, &test_dir)
        .expect("Failed to bind 4-shard server");
    let addr = engine.local_addr();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let server_handle = thread::spawn(move || {
        while running_clone.load(Ordering::Relaxed) {
            let events = engine.tick();
            if events == 0 {
                thread::sleep(Duration::from_micros(50));
            }
        }
    });

    let num_clients = 4;
    let ops_per_client = 100;
    let mut client_handles = Vec::new();

    for client_id in 0..num_clients {
        let handle = thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).expect("Client connect failed");
            stream.set_nodelay(true).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();

            let mut buf = [0u8; 1024];

            for op in 0..ops_per_client {
                let key = format!("c{}:k{}", client_id, op);
                let val = format!("v{}", op);

                // SET
                let set_cmd = format!(
                    "*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
                    key.len(),
                    key,
                    val.len(),
                    val
                );
                stream.write_all(set_cmd.as_bytes()).unwrap();
                let n = stream.read(&mut buf).unwrap();
                assert_eq!(&buf[..n], b"+OK\r\n");

                // GET
                let get_cmd = format!("*2\r\n$3\r\nGET\r\n${}\r\n{}\r\n", key.len(), key);
                stream.write_all(get_cmd.as_bytes()).unwrap();
                let n = stream.read(&mut buf).unwrap();
                let expected_get = format!("${}\r\n{}\r\n", val.len(), val);
                assert_eq!(&buf[..n], expected_get.as_bytes());
            }
        });
        client_handles.push(handle);
    }

    for handle in client_handles {
        handle.join().unwrap();
    }

    // Connect admin client to check DBSIZE
    let mut admin_stream = TcpStream::connect(addr).unwrap();
    let mut buf = [0u8; 1024];
    admin_stream.write_all(b"*1\r\n$6\r\nDBSIZE\r\n").unwrap();
    let n = admin_stream.read(&mut buf).unwrap();
    let expected_dbsize = format!(":{}\r\n", num_clients * ops_per_client);
    assert_eq!(&buf[..n], expected_dbsize.as_bytes());

    running.store(false, Ordering::Relaxed);
    let _ = server_handle.join();
    let _ = fs::remove_dir_all(&test_dir);
}
