//! Integration Tests for Binary WAL Durability & Crash Recovery

use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use valence::{
    parse_segment_id, ShardedServerEngine, SyncPolicy, WalOp, WalRecovery, WalWriter,
};

#[test]
fn test_wal_end_to_end_crash_recovery_and_replay() {
    let test_dir = std::env::temp_dir().join(format!("val_wal_crash_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    let _addr = {
        // --- 1. First Server Lifetime (Simulating Normal Operations) ---
        let mut engine = ShardedServerEngine::bind_with_data_dir("127.0.0.1:0", 4, &test_dir)
            .expect("Failed to bind server")
            .with_sync_policy(SyncPolicy::Always);

        let local_addr = engine.local_addr();
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

        let mut stream = TcpStream::connect(local_addr).expect("Client connect failed");
        stream.set_nodelay(true).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        let mut buf = [0u8; 1024];

        // SET users 1..10
        for i in 1..=10 {
            let key = format!("user:{}", i);
            let val = format!("profile_data_{}", i);
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

        // MSET across multiple shards
        stream
            .write_all(b"*5\r\n$4\r\nMSET\r\n$7\r\nshard:a\r\n$5\r\nval_a\r\n$7\r\nshard:b\r\n$5\r\nval_b\r\n")
            .unwrap();
        let n = stream.read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"+OK\r\n");

        // DEL user:5
        stream.write_all(b"*2\r\n$3\r\nDEL\r\n$6\r\nuser:5\r\n").unwrap();
        let n = stream.read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b":1\r\n");

        // Verify DBSIZE before simulated crash is 11 (10 - 1 + 2)
        stream.write_all(b"*1\r\n$6\r\nDBSIZE\r\n").unwrap();
        let n = stream.read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b":11\r\n");

        // Simulate crash: stop server thread without taking a snapshot
        running.store(false, Ordering::Relaxed);
        let _ = server_handle.join();

        local_addr
    };

    // --- 2. Second Server Lifetime (Boot from disk and replay WAL) ---
    {
        // Re-bind engine with the exact same data directory
        let mut recovered_engine = ShardedServerEngine::bind_with_data_dir("127.0.0.1:0", 4, &test_dir)
            .expect("Failed to bind recovered server")
            .with_sync_policy(SyncPolicy::Always);

        let recovered_addr = recovered_engine.local_addr();
        let running_rec = Arc::new(AtomicBool::new(true));
        let running_rec_clone = running_rec.clone();

        let rec_server_handle = thread::spawn(move || {
            while running_rec_clone.load(Ordering::Relaxed) {
                let events = recovered_engine.tick();
                if events == 0 {
                    thread::sleep(Duration::from_micros(100));
                }
            }
        });

        let mut stream = TcpStream::connect(recovered_addr).expect("Client connect failed");
        stream.set_nodelay(true).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        let mut buf = [0u8; 1024];

        // Verify DBSIZE is 11 after WAL recovery
        stream.write_all(b"*1\r\n$6\r\nDBSIZE\r\n").unwrap();
        let n = stream.read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b":11\r\n");

        // Verify user:1 through user:10 (except user:5 which was deleted)
        for i in 1..=10 {
            let key = format!("user:{}", i);
            let cmd = format!("*2\r\n$3\r\nGET\r\n${}\r\n{}\r\n", key.len(), key);
            stream.write_all(cmd.as_bytes()).unwrap();
            let n = stream.read(&mut buf).unwrap();

            if i == 5 {
                assert_eq!(&buf[..n], b"$-1\r\n", "user:5 should be absent (deleted)");
            } else {
                let expected_val = format!("profile_data_{}", i);
                let expected_resp = format!("${}\r\n{}\r\n", expected_val.len(), expected_val);
                assert_eq!(&buf[..n], expected_resp.as_bytes());
            }
        }

        // Verify MSET keys
        stream.write_all(b"*2\r\n$3\r\nGET\r\n$7\r\nshard:a\r\n").unwrap();
        let n = stream.read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"$5\r\nval_a\r\n");

        stream.write_all(b"*2\r\n$3\r\nGET\r\n$7\r\nshard:b\r\n").unwrap();
        let n = stream.read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"$5\r\nval_b\r\n");

        // Verify INFO reports active WAL
        stream.write_all(b"*1\r\n$4\r\nINFO\r\n").unwrap();
        let n = stream.read(&mut buf).unwrap();
        let info_str = String::from_utf8_lossy(&buf[..n]);
        assert!(info_str.contains("wal_enabled:1"));
        assert!(info_str.contains("wal_current_lsn:13")); // 10 SETs + 2 (from MSET) + 1 DEL = 13

        running_rec.store(false, Ordering::Relaxed);
        let _ = rec_server_handle.join();
    }

    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_wal_torn_write_tail_truncation() {
    let test_dir = std::env::temp_dir().join(format!("val_wal_torn_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    let wal_dir = test_dir.join("wal");

    // Write 20 valid records
    {
        let writer = WalWriter::new(&wal_dir, 64 * 1024 * 1024, SyncPolicy::Always).unwrap();
        for i in 1..=20 {
            writer.append(1, WalOp::Set {
                key: format!("k_{}", i).into_bytes(),
                value: format!("v_{}", i).into_bytes(),
                ttl_ms: None,
            }).unwrap();
        }
        writer.flush_and_sync().unwrap();
    }

    // Append 5 bytes of garbage simulating power loss mid-write
    {
        let seg_path = wal_dir.join(valence::format_segment_filename(1));
        let mut file = fs::OpenOptions::new().append(true).open(&seg_path).unwrap();
        file.write_all(&[0x56, 0x4C, 0x01, 0xFF, 0xAA]).unwrap();
        file.sync_all().unwrap();
    }

    // Recover from disk
    let (records, report) = WalRecovery::read_all_records(&wal_dir).unwrap();
    assert_eq!(records.len(), 20);
    assert_eq!(report.records_replayed, 20);
    assert!(report.torn_write_detected, "Torn write at tail should be detected and isolated");

    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_wal_segment_rotation_and_checkpoint_pruning() {
    let test_dir = std::env::temp_dir().join(format!("val_wal_rotate_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    let wal_dir = test_dir.join("wal");

    // Configure tiny 256-byte segments to force rapid rotations
    let writer = WalWriter::new(&wal_dir, 256, SyncPolicy::Always).unwrap();

    for i in 1..=15 {
        writer.append(i as u64, WalOp::Set {
            key: format!("long_key_name_number_{}", i).into_bytes(),
            value: format!("long_value_data_payload_number_{}", i).into_bytes(),
            ttl_ms: None,
        }).unwrap();
    }
    writer.flush_and_sync().unwrap();

    // Verify multiple segments were created
    let mut segment_ids: Vec<u64> = fs::read_dir(&wal_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|e| parse_segment_id(&e.path()))
        .collect();
    segment_ids.sort();

    assert!(segment_ids.len() >= 3, "Expected at least 3 rotated segments, got: {:?}", segment_ids);

    // Prune segments older than the last segment
    let last_seg = *segment_ids.last().unwrap();
    let pruned = writer.prune_segments_before(last_seg).unwrap();
    assert!(pruned >= 2, "Expected at least 2 pruned segments, got: {}", pruned);

    let remaining_ids: Vec<u64> = fs::read_dir(&wal_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|e| parse_segment_id(&e.path()))
        .collect();
    assert_eq!(remaining_ids, vec![last_seg]);

    let _ = fs::remove_dir_all(&test_dir);
}
