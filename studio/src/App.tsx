import { useState, useEffect } from 'react';
import {
  Cpu,
  Database,
  Terminal,
  Activity,
  Flame,
  Moon,
  Sun,
  Layers,
  Save,
  FileText,
  ShieldCheck,
  HardDrive,
  Zap,
} from 'lucide-react';
import { KeyspaceBrowser } from './components/KeyspaceBrowser';
import { ClusterTopology } from './components/ClusterTopology';
import { DurabilityConsole } from './components/DurabilityConsole';
import { ActivityProfiler } from './components/ActivityProfiler';
import { RespWorkbench } from './components/RespWorkbench';
import { TieringConsole } from './components/TieringConsole';
import { ExplainVisualizer } from './components/ExplainVisualizer';
import type {
  ShardData,
  KeyspaceItem,
  CommandLog,
  ThemeMode,
  SnapshotRecord,
  WalState,
  WalRecordView,
  WalSegmentView,
  WalRecoveryReportView,
  WorkloadMode,
} from './types';
import { routeKey, murmur3_32 } from './utils/router';

export function App() {
  // Navigation: 7 Core Operational Pillars of Enterprise In-Memory Database Console
  const [activeTab, setActiveTab] = useState<
    'keyspace' | 'topology' | 'tiering' | 'explain' | 'durability' | 'profiler' | 'workbench'
  >('tiering');

  // Theme Mode: Default is 'charcoal' (Deep dark mode for ultra-high contrast)
  const [theme, setTheme] = useState<ThemeMode>(() => {
    return (localStorage.getItem('valence_theme') as ThemeMode) || 'charcoal';
  });

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('valence_theme', theme);
  }, [theme]);

  // Monotonic Mutation Epoch (Phase 4 Forkless Persistence)
  const [epoch, setEpoch] = useState<number>(1);

  // Latest Snapshot Status
  const [lastSnapshot, setLastSnapshot] = useState<SnapshotRecord | null>({
    epoch: 1,
    timestamp: new Date().toLocaleTimeString(),
    itemCount: 6,
    crc64Hex: '0x9E4B8F21A7C3D015',
    sizeBytes: 254,
    status: 'saved',
  });

  // Binary Write-Ahead Log (WAL) State
  const [walState, setWalState] = useState<WalState>({
    currentLsn: 13,
    syncPolicy: 'GroupCommit',
    activeSegment: 'wal_0000000000000001.valwal',
    totalSegments: 1,
    segmentSizeBytes: 67108864, // 64MB boundary
    status: 'active',
    pendingFsyncCount: 0,
    groupCommitBatches: 84,
    avgFlushLatencyUs: 48,
  });

  // On-disk WAL Segments Catalog
  const [walSegments, setWalSegments] = useState<WalSegmentView[]>([
    {
      id: 1,
      filename: 'wal_0000000000000001.valwal',
      startLsn: 1,
      endLsn: 13,
      sizeBytes: 1420,
      maxSizeBytes: 67108864,
      status: 'active',
      createdAt: '11:15:00',
    },
  ]);

  // Initial Seed Binary WAL Records
  const [walRecords, setWalRecords] = useState<WalRecordView[]>([
    {
      lsn: 13,
      epoch: 1,
      op: 'CHECKPOINT',
      key: 'system:checkpoint',
      valPreview: 'lsn_cutoff: 12',
      sizeBytes: 40,
      crc64Hex: '0x9E4B8F21A7C3D015',
      timestamp: '11:15:26',
    },
    {
      lsn: 12,
      epoch: 1,
      op: 'DEL',
      key: 'user:5',
      valPreview: 'tombstone: true',
      sizeBytes: 22,
      crc64Hex: '0x1C5A8F90E2D3B748',
      timestamp: '11:15:24',
    },
    {
      lsn: 11,
      epoch: 1,
      op: 'EXPIRE',
      key: 'ratelimit:api:10.0.1.5',
      valPreview: 'ttl_ms: 60000',
      sizeBytes: 26,
      crc64Hex: '0xDF810B2A3E7C9451',
      timestamp: '11:15:22',
    },
    {
      lsn: 10,
      epoch: 1,
      op: 'SET',
      key: 'shard:b',
      valPreview: 'val_b',
      sizeBytes: 24,
      crc64Hex: '0x2B4A7C91EF380D65',
      timestamp: '11:15:20',
    },
    {
      lsn: 9,
      epoch: 1,
      op: 'SET',
      key: 'shard:a',
      valPreview: 'val_a',
      sizeBytes: 24,
      crc64Hex: '0x5C1F8E3A9D20B746',
      timestamp: '11:15:18',
    },
    {
      lsn: 8,
      epoch: 1,
      op: 'SET',
      key: 'user:2',
      valPreview: 'profile_data_2',
      sizeBytes: 32,
      crc64Hex: '0x9E70C58B3A1D4F28',
      timestamp: '11:15:16',
    },
    {
      lsn: 7,
      epoch: 1,
      op: 'SET',
      key: 'user:1',
      valPreview: 'profile_data_1',
      sizeBytes: 32,
      crc64Hex: '0x4D8E2A1B7F0C3956',
      timestamp: '11:15:14',
    },
    {
      lsn: 6,
      epoch: 1,
      op: 'SET',
      key: 'telemetry:node:mem_pct',
      valPreview: '42.8',
      sizeBytes: 30,
      crc64Hex: '0x6A3B91EF02C7845D',
      timestamp: '11:15:12',
    },
    {
      lsn: 5,
      epoch: 1,
      op: 'SET',
      key: 'llm:ctx:session_881',
      valPreview: 'context_tokens_v3',
      sizeBytes: 38,
      crc64Hex: '0x19DF7B0C5E832A41',
      timestamp: '11:15:10',
    },
    {
      lsn: 4,
      epoch: 1,
      op: 'SET',
      key: 'lock:distributed:leader_node',
      valPreview: 'worker_04',
      sizeBytes: 34,
      crc64Hex: '0x7C50BA3E2F814D90',
      timestamp: '11:15:08',
    },
    {
      lsn: 3,
      epoch: 1,
      op: 'SET',
      key: 'cache:query:digest_fa7',
      valPreview: 'res_hash_89a1b',
      sizeBytes: 36,
      crc64Hex: '0xA1E4D078B5C92314',
      timestamp: '11:15:06',
    },
    {
      lsn: 2,
      epoch: 1,
      op: 'SET',
      key: 'ratelimit:api:10.0.1.5',
      valPreview: '14',
      sizeBytes: 28,
      crc64Hex: '0x8B2C4F1A9D3E7105',
      timestamp: '11:15:04',
    },
    {
      lsn: 1,
      epoch: 1,
      op: 'SET',
      key: 'auth:jwt:session_991',
      valPreview: 'eyJhGciOiJIUzI1NiJ9.payload',
      sizeBytes: 42,
      crc64Hex: '0x3F81C92B4A1D8E70',
      timestamp: '11:15:02',
    },
  ]);

  // Last Crash Recovery Report (simulated or initial state)
  const [lastRecoveryReport, setLastRecoveryReport] = useState<WalRecoveryReportView | null>({
    recordsReplayed: 13,
    maxLsn: 13,
    truncatedBytes: 0,
    corruptedSegments: 0,
    elapsedMs: 1.15,
    timestamp: new Date().toLocaleTimeString(),
  });

  // Workload Generator Mode & Dynamic Telemetry
  const [workloadMode, setWorkloadMode] = useState<WorkloadMode>('idle');
  const [opsPerSec, setOpsPerSec] = useState<number>(0);

  // General-Purpose In-Memory State across diverse application domains
  const [keyspace, setKeyspace] = useState<KeyspaceItem[]>([
    {
      key: 'auth:jwt:session_991',
      value: 'eyJhGciOiJIUzI1NiJ9.payload',
      dataType: 'string',
      sizeBytes: 28,
      isSso: false,
      dib: 1,
      epoch: 1,
      ttlMs: 3600000,
      expiresAt: Date.now() + 3600000,
      targetShard: routeKey('auth:jwt:session_991', 4),
    },
    {
      key: 'ratelimit:api:10.0.1.5',
      value: '14',
      dataType: 'string',
      sizeBytes: 2,
      isSso: true,
      dib: 0,
      epoch: 1,
      ttlMs: 60000,
      expiresAt: Date.now() + 60000,
      targetShard: routeKey('ratelimit:api:10.0.1.5', 4),
    },
    {
      key: 'cache:query:digest_fa7',
      value: 'res_hash_89a1b',
      dataType: 'string',
      sizeBytes: 14,
      isSso: true,
      dib: 3,
      epoch: 1,
      ttlMs: null,
      expiresAt: null,
      targetShard: routeKey('cache:query:digest_fa7', 4),
    },
    {
      key: 'lock:distributed:leader_node',
      value: 'worker_04',
      dataType: 'string',
      sizeBytes: 9,
      isSso: true,
      dib: 2,
      epoch: 1,
      ttlMs: 15000,
      expiresAt: Date.now() + 15000,
      targetShard: routeKey('lock:distributed:leader_node', 4),
    },
    {
      key: 'llm:ctx:session_881',
      value: 'context_tokens_v3',
      dataType: 'string',
      sizeBytes: 17,
      isSso: true,
      dib: 1,
      epoch: 1,
      ttlMs: null,
      expiresAt: null,
      targetShard: routeKey('llm:ctx:session_881', 4),
    },
    {
      key: 'telemetry:node:mem_pct',
      value: '42.8',
      dataType: 'string',
      sizeBytes: 4,
      isSso: true,
      dib: 0,
      epoch: 1,
      ttlMs: 30000,
      expiresAt: Date.now() + 30000,
      targetShard: routeKey('telemetry:node:mem_pct', 4),
    },
  ]);

  // Live Synchronization with Valence Rust Engine (Port 6379 via /api/sync)
  useEffect(() => {
    const syncWithEngine = async () => {
      try {
        const res = await fetch('/api/sync');
        const data = await res.json();
        if (data.success && Array.isArray(data.keys) && data.keys.length > 0) {
          setKeyspace((prev) => {
            const updated = [...prev];
            for (const serverKey of data.keys) {
              const existingIndex = updated.findIndex((k) => k.key === serverKey.key);
              const mappedItem: KeyspaceItem = {
                key: serverKey.key,
                value: serverKey.value || '(nil)',
                dataType: 'string',
                sizeBytes: serverKey.sizeBytes || 16,
                isSso: (serverKey.sizeBytes || 16) <= 24,
                dib: (murmur3_32(serverKey.key) % 3),
                epoch: epoch,
                ttlMs: null,
                expiresAt: null,
                targetShard: routeKey(serverKey.key, 4),
              };
              if (existingIndex >= 0) {
                updated[existingIndex] = { ...updated[existingIndex], value: mappedItem.value };
              } else {
                updated.unshift(mappedItem);
              }
            }
            return updated;
          });
        }
      } catch {
        // standalone mode fallback
      }
    };

    const interval = setInterval(syncWithEngine, 1500);
    syncWithEngine();
    return () => clearInterval(interval);
  }, [epoch]);

  // Initial Command Audit Logs
  const [commandLogs, setCommandLogs] = useState<CommandLog[]>([
    {
      id: '1',
      timestamp: new Date().toLocaleTimeString(),
      command: 'PING',
      response: '+PONG',
      durationUs: 3.2,
      isError: false,
    },
    {
      id: '2',
      timestamp: new Date().toLocaleTimeString(),
      command: 'SET ratelimit:api:10.0.1.5 "14" PX 60000',
      response: '+OK',
      durationUs: 6.8,
      isError: false,
    },
    {
      id: '3',
      timestamp: new Date().toLocaleTimeString(),
      command: 'GET auth:jwt:session_991',
      response: '"eyJhGciOiJIUzI1NiJ9.payload"',
      durationUs: 4.1,
      isError: false,
    },
  ]);

  // Shards computed dynamically from active keyspace
  const shards: ShardData[] = [0, 1, 2, 3].map((shardId) => {
    const keysInShard = keyspace.filter((k) => k.targetShard === shardId).length;
    return {
      id: shardId,
      coreId: shardId,
      keyCount: keysInShard,
      opsPerSec: workloadMode === 'idle' ? 0 : Math.round(opsPerSec / 4),
      queueDepth: workloadMode === 'idle' ? 0 : Math.floor(Math.random() * 8),
      queueCapacity: 1024,
      skewPercent: 0.0,
      isPinned: true,
      memoryAllocatedKb: 64,
      loadFactorPct: Math.round((keysInShard / 16) * 100),
      maxDib: 2,
    };
  });

  // Key Mutation Handlers (Advances Monotonic Epoch & WAL LSN)
  const handleSetKey = (key: string, value: string, ttlMs: number | null) => {
    const isSso = key.length <= 15 && value.length <= 23;
    const targetShard = routeKey(key, shards.length);
    const nextEpoch = epoch + 1;
    const nextLsn = walState.currentLsn + 1;

    setEpoch(nextEpoch);
    setWalState((prev) => ({
      ...prev,
      currentLsn: nextLsn,
      groupCommitBatches: prev.groupCommitBatches + 1,
    }));

    const newItem: KeyspaceItem = {
      key,
      value,
      dataType: 'string',
      sizeBytes: value.length,
      isSso,
      dib: Math.floor(Math.random() * 3),
      epoch: nextEpoch,
      ttlMs,
      expiresAt: ttlMs ? Date.now() + ttlMs : null,
      targetShard,
    };

    setKeyspace((prev) => [newItem, ...prev.filter((i) => i.key !== key)]);

    // Append to live WAL binary frame stream
    const walRec: WalRecordView = {
      lsn: nextLsn,
      epoch: nextEpoch,
      op: 'SET',
      key,
      valPreview: value,
      sizeBytes: 25 + key.length + value.length,
      crc64Hex:
        '0x' +
        Math.floor(Math.random() * 0xffffffffff)
          .toString(16)
          .padStart(16, 'A')
          .toUpperCase(),
      timestamp: new Date().toLocaleTimeString(),
    };
    setWalRecords((prev) => [walRec, ...prev.slice(0, 49)]);

    // Update active segment size
    setWalSegments((prev) =>
      prev.map((s) =>
        s.status === 'active'
          ? { ...s, endLsn: nextLsn, sizeBytes: s.sizeBytes + walRec.sizeBytes }
          : s
      )
    );

    // Record mutation in command log
    setCommandLogs((prev) => [
      ...prev,
      {
        id: Math.random().toString(),
        timestamp: new Date().toLocaleTimeString(),
        command: ttlMs ? `SET ${key} "${value}" PX ${ttlMs}` : `SET ${key} "${value}"`,
        response: '+OK',
        durationUs: isSso ? 3.4 : 7.2,
        isError: false,
      },
    ]);
  };

  const handleDeleteKey = (key: string) => {
    const nextEpoch = epoch + 1;
    const nextLsn = walState.currentLsn + 1;

    setEpoch(nextEpoch);
    setWalState((prev) => ({
      ...prev,
      currentLsn: nextLsn,
      groupCommitBatches: prev.groupCommitBatches + 1,
    }));
    setKeyspace((prev) => prev.filter((k) => k.key !== key));

    // Append to live WAL binary frame stream
    const walRec: WalRecordView = {
      lsn: nextLsn,
      epoch: nextEpoch,
      op: 'DEL',
      key,
      valPreview: 'tombstone: true',
      sizeBytes: 25 + key.length,
      crc64Hex:
        '0x' +
        Math.floor(Math.random() * 0xffffffffff)
          .toString(16)
          .padStart(16, 'C')
          .toUpperCase(),
      timestamp: new Date().toLocaleTimeString(),
    };
    setWalRecords((prev) => [walRec, ...prev.slice(0, 49)]);

    setWalSegments((prev) =>
      prev.map((s) =>
        s.status === 'active'
          ? { ...s, endLsn: nextLsn, sizeBytes: s.sizeBytes + walRec.sizeBytes }
          : s
      )
    );

    setCommandLogs((prev) => [
      ...prev,
      {
        id: Math.random().toString(),
        timestamp: new Date().toLocaleTimeString(),
        command: `DEL ${key}`,
        response: ':(integer) 1',
        durationUs: 3.1,
        isError: false,
      },
    ]);
  };

  // Change WAL Durability Sync Policy
  const handleChangeSyncPolicy = (policy: 'Always' | 'GroupCommit' | 'Periodic') => {
    const avgLatency = policy === 'Always' ? 385 : policy === 'GroupCommit' ? 48 : 4;
    setWalState((prev) => ({
      ...prev,
      syncPolicy: policy,
      avgFlushLatencyUs: avgLatency,
    }));
    setCommandLogs((prev) => [
      ...prev,
      {
        id: Math.random().toString(),
        timestamp: new Date().toLocaleTimeString(),
        command: `CONFIG SET wal_sync_policy ${policy}`,
        response: `+OK [Durability policy changed to ${policy}]`,
        durationUs: 12.4,
        isError: false,
      },
    ]);
  };

  // Trigger Point-in-Time Checkpoint and WAL pruning
  const handleTriggerCheckpoint = () => {
    const nextEpoch = epoch + 1;
    const nextLsn = walState.currentLsn + 1;

    setEpoch(nextEpoch);
    setWalState((prev) => ({ ...prev, currentLsn: nextLsn }));

    const checkpointRecord: WalRecordView = {
      lsn: nextLsn,
      epoch: nextEpoch,
      op: 'CHECKPOINT',
      key: 'system:checkpoint',
      valPreview: `cutoff_lsn: ${walState.currentLsn}`,
      sizeBytes: 40,
      crc64Hex:
        '0x' +
        Math.floor(Math.random() * 0xffffffffff)
          .toString(16)
          .padStart(16, 'F')
          .toUpperCase(),
      timestamp: new Date().toLocaleTimeString(),
    };

    setWalRecords((prev) => [checkpointRecord, ...prev.slice(0, 49)]);

    const newSnap: SnapshotRecord = {
      epoch: nextEpoch,
      timestamp: new Date().toLocaleTimeString(),
      itemCount: keyspace.length,
      crc64Hex: checkpointRecord.crc64Hex,
      sizeBytes: 48 + keyspace.reduce((acc, k) => acc + k.sizeBytes + k.key.length + 17, 0) + 16,
      status: 'saved',
    };
    setLastSnapshot(newSnap);

    setCommandLogs((prev) => [
      ...prev,
      {
        id: Math.random().toString(),
        timestamp: new Date().toLocaleTimeString(),
        command: 'CHECKPOINT',
        response: `+OK Checkpoint committed at LSN #${nextLsn}, Epoch #${nextEpoch}`,
        durationUs: 45.2,
        isError: false,
      },
    ]);
  };

  // Simulate Crash & Replay Engine
  const handleSimulateCrashAndReplay = (simulateTornWrite: boolean) => {
    const truncatedBytes = simulateTornWrite ? 48 : 0;
    const report: WalRecoveryReportView = {
      recordsReplayed: walRecords.length,
      maxLsn: walState.currentLsn,
      truncatedBytes,
      corruptedSegments: 0,
      elapsedMs: 1.18 + Math.random() * 0.4,
      timestamp: new Date().toLocaleTimeString(),
    };
    setLastRecoveryReport(report);

    setCommandLogs((prev) => [
      ...prev,
      {
        id: Math.random().toString(),
        timestamp: new Date().toLocaleTimeString(),
        command: simulateTornWrite
          ? 'VALENCE CRASH_REPLAY --torn-tail'
          : 'VALENCE CRASH_REPLAY',
        response: `+OK Replayed ${report.recordsReplayed} WAL records up to LSN #${report.maxLsn} (${report.truncatedBytes} torn bytes discarded) in ${report.elapsedMs.toFixed(2)}ms`,
        durationUs: report.elapsedMs * 1000,
        isError: false,
      },
    ]);
  };

  // Background Workload Generator Simulator
  useEffect(() => {
    if (workloadMode === 'idle') {
      setOpsPerSec(0);
      return;
    }

    const interval = setInterval(() => {
      if (workloadMode === 'oltp') {
        const rate = 1180 + Math.floor(Math.random() * 140);
        setOpsPerSec(rate);
        const randomId = Math.floor(Math.random() * 60);
        const k = `session:user_${randomId}`;
        const v = `token_${Math.random().toString(36).substring(7)}`;
        handleSetKey(k, v, 30000);
      } else if (workloadMode === 'stress') {
        const rate = 8300 + Math.floor(Math.random() * 700);
        setOpsPerSec(rate);
        const batchId = Math.floor(Math.random() * 120);
        handleSetKey(`stress:node_${batchId}`, `val_${Date.now()}`, 15000);
      }
    }, 1200);

    return () => clearInterval(interval);
  }, [workloadMode, epoch, walState.currentLsn]);

  // RESP Terminal Command Dispatcher
  const handleExecuteCommand = (raw: string) => {
    const start = performance.now();
    const parts = raw.trim().split(/\s+/);
    const verb = parts[0].toUpperCase();
    let response = '';
    let isError = false;

    switch (verb) {
      case 'PING':
        response = '+PONG';
        break;

      case 'BGSAVE': {
        const snapEpoch = epoch;
        const newSnap: SnapshotRecord = {
          epoch: snapEpoch,
          timestamp: new Date().toLocaleTimeString(),
          itemCount: keyspace.length,
          crc64Hex:
            '0x' +
            Math.floor(Math.random() * 0xffffffffff)
              .toString(16)
              .padStart(16, 'A')
              .toUpperCase(),
          sizeBytes:
            48 + keyspace.reduce((acc, k) => acc + k.sizeBytes + k.key.length + 17, 0) + 16,
          status: 'saved',
        };
        setLastSnapshot(newSnap);
        response = `+Background saving started at Epoch #${snapEpoch} (Zero-Fork streaming with Key-Level CoW)`;
        break;
      }

      case 'SAVE': {
        const snapEpoch = epoch;
        const newSnap: SnapshotRecord = {
          epoch: snapEpoch,
          timestamp: new Date().toLocaleTimeString(),
          itemCount: keyspace.length,
          crc64Hex:
            '0x' +
            Math.floor(Math.random() * 0xffffffffff)
              .toString(16)
              .padStart(16, 'E')
              .toUpperCase(),
          sizeBytes:
            48 + keyspace.reduce((acc, k) => acc + k.sizeBytes + k.key.length + 17, 0) + 16,
          status: 'saved',
        };
        setLastSnapshot(newSnap);
        response = `+OK [Snapshot saved to disk: ${keyspace.length} keys, CRC64 verified]`;
        break;
      }

      case 'LASTSAVE': {
        response = `:(integer) ${Math.floor(Date.now() / 1000)}`;
        break;
      }

      case 'CHECKPOINT': {
        handleTriggerCheckpoint();
        response = `+OK [WAL Checkpoint synchronized, LSN #${walState.currentLsn}]`;
        break;
      }

      case 'SET': {
        if (parts.length < 3) {
          response = "-ERR wrong number of arguments for 'set' command";
          isError = true;
        } else {
          const k = parts[1];
          const v = parts[2].replace(/^["']|["']$/g, '');
          let ttl: number | null = null;
          if (parts.length >= 5) {
            const opt = parts[3].toUpperCase();
            const val = Number(parts[4]);
            if (opt === 'EX') ttl = val * 1000;
            if (opt === 'PX') ttl = val;
          }
          handleSetKey(k, v, ttl);
          response = '+OK';
        }
        break;
      }

      case 'GET': {
        if (parts.length < 2) {
          response = "-ERR wrong number of arguments for 'get' command";
          isError = true;
        } else {
          const k = parts[1];
          const found = keyspace.find((item) => item.key === k);
          if (found) {
            response = `"${found.value}"`;
          } else {
            response = '$-1';
          }
        }
        break;
      }

      case 'DEL': {
        if (parts.length < 2) {
          response = "-ERR wrong number of arguments for 'del' command";
          isError = true;
        } else {
          const k = parts[1];
          const found = keyspace.find((item) => item.key === k);
          if (found) {
            handleDeleteKey(k);
            response = ':(integer) 1';
          } else {
            response = ':(integer) 0';
          }
        }
        break;
      }

      case 'EXISTS': {
        if (parts.length < 2) {
          response = "-ERR wrong number of arguments for 'exists' command";
          isError = true;
        } else {
          const k = parts[1];
          const exists = keyspace.some((item) => item.key === k);
          response = `:(integer) ${exists ? 1 : 0}`;
        }
        break;
      }

      case 'TTL': {
        if (parts.length < 2) {
          response = "-ERR wrong number of arguments for 'ttl' command";
          isError = true;
        } else {
          const k = parts[1];
          const found = keyspace.find((item) => item.key === k);
          if (!found) {
            response = ':(integer) -2';
          } else if (!found.expiresAt) {
            response = ':(integer) -1';
          } else {
            const remSec = Math.max(0, Math.floor((found.expiresAt - Date.now()) / 1000));
            response = `:(integer) ${remSec}`;
          }
        }
        break;
      }

      case 'MGET': {
        const keys = parts.slice(1);
        if (keys.length === 0) {
          response = "-ERR wrong number of arguments for 'mget' command";
          isError = true;
        } else {
          const vals = keys.map((k) => {
            const item = keyspace.find((i) => i.key === k);
            return item ? `"${item.value}"` : 'nil';
          });
          response = `*${vals.length}\n${vals.map((v, idx) => ` ${idx + 1}) ${v}`).join('\n')}`;
        }
        break;
      }

      case 'MSET': {
        const pairs = parts.slice(1);
        if (pairs.length < 2 || pairs.length % 2 !== 0) {
          response = "-ERR wrong number of arguments for 'mset' command";
          isError = true;
        } else {
          for (let i = 0; i < pairs.length; i += 2) {
            handleSetKey(pairs[i], pairs[i + 1].replace(/^["']|["']$/g, ''), null);
          }
          response = '+OK';
        }
        break;
      }

      case 'DBSIZE':
        response = `:(integer) ${keyspace.length}`;
        break;

      case 'INFO':
        response = `# Server\nvalence_version:0.1.0\narch:thread_per_core_shared_nothing\nshards:${shards.length}\nstate:standby_ready\n\n# Persistence\nepoch:${epoch}\nforkless_snapshots:active\nformat:valence_snap_v1\nwal_enabled:1\nwal_current_lsn:${walState.currentLsn}\nwal_sync_policy:${walState.syncPolicy}\nwal_active_segment:${walState.activeSegment}\nchecksum:ecma182_crc64\n\n# Memory\nkeys_total:${keyspace.length}\nsso_inlined_ratio:${Math.round(
          (keyspace.filter((k) => k.isSso).length / Math.max(1, keyspace.length)) * 100
        )}%\nslot_size_bytes:64\n\n# Expiration\ntiming_wheel_tiers:4\nresolution:1ms`;
        break;

      default:
        response = `-ERR unknown command '${verb}'`;
        isError = true;
        break;
    }

    const durationUs = Math.max(2.4, (performance.now() - start) * 1000 + Math.random() * 4);

    setCommandLogs((prev) => [
      ...prev,
      {
        id: Math.random().toString(),
        timestamp: new Date().toLocaleTimeString(),
        command: raw,
        response,
        durationUs,
        isError,
      },
    ]);
  };

  return (
    <div className="app-container">
      {/* Top Navbar */}
      <nav className="top-nav">
        <div className="brand-section">
          <div className="brand-logo">
            <Flame size={18} color="var(--accent-primary)" />
            <span>VALENCE</span>
          </div>
          <span className="brand-badge">STUDIO</span>
          <span className="version-tag">v0.1.0</span>
        </div>

        <div className="top-nav-right">
          {/* Cluster Status Pills */}
          <div className="cluster-stats-strip">
            <div className="stat-pill">
              <span
                className="health-dot"
                style={{
                  background:
                    workloadMode === 'idle' ? 'var(--accent-green)' : 'var(--accent-amber)',
                  boxShadow:
                    workloadMode === 'idle' ? 'none' : '0 0 6px var(--accent-amber)',
                }}
              />
              <span>NODE:</span>
              <span
                className="val"
                style={{
                  color:
                    workloadMode === 'idle' ? 'var(--accent-green)' : 'var(--accent-amber)',
                }}
              >
                127.0.0.1:6379 (ONLINE)
              </span>
            </div>

            <div className="stat-pill">
              <Cpu size={12} color="var(--accent-primary)" />
              <span>CORES:</span>
              <span className="val">{shards.length} Pinned</span>
            </div>

            <div className="stat-pill">
              <Database size={12} color="var(--text-muted)" />
              <span>KEYS:</span>
              <span className="val">{keyspace.length}</span>
            </div>

            <div className="stat-pill">
              <HardDrive size={12} color="var(--accent-cyan)" />
              <span>MEMORY:</span>
              <span className="val" style={{ color: 'var(--accent-cyan)' }}>
                4.8 MB (RSS: 5.1 MB)
              </span>
            </div>

            <div className="stat-pill">
              <Save size={12} color="var(--text-muted)" />
              <span>EPOCH:</span>
              <span className="val">
                #{epoch}
              </span>
            </div>

            <div className="stat-pill">
              <FileText size={12} color="var(--accent-cyan)" />
              <span>WAL LSN:</span>
              <span className="val" style={{ color: 'var(--accent-cyan)' }}>
                #{walState.currentLsn}
              </span>
            </div>
          </div>

          {/* Theme Switcher Segmented Control */}
          <div className="theme-switcher">
            <button
              className={`theme-btn ${theme === 'slate' ? 'active' : ''}`}
              onClick={() => setTheme('slate')}
              title="Industrial Slate (Balanced neutral - neither dark nor white)"
            >
              <Layers size={11} /> Slate (Mid)
            </button>
            <button
              className={`theme-btn ${theme === 'charcoal' ? 'active' : ''}`}
              onClick={() => setTheme('charcoal')}
              title="Deep Charcoal (Terminal contrast)"
            >
              <Moon size={11} /> Charcoal
            </button>
            <button
              className={`theme-btn ${theme === 'platinum' ? 'active' : ''}`}
              onClick={() => setTheme('platinum')}
              title="Platinum Mist (Clean daylight)"
            >
              <Sun size={11} /> Platinum
            </button>
          </div>
        </div>
      </nav>

      {/* Subnav Tabs: Core Architectural Pillars of Enterprise In-Memory Database */}
      <div className="tab-nav">
        <button
          className={`tab-btn ${activeTab === 'keyspace' ? 'active' : ''}`}
          onClick={() => setActiveTab('keyspace')}
        >
          <Database size={14} /> Keyspace Browser
        </button>

        <button
          className={`tab-btn ${activeTab === 'topology' ? 'active' : ''}`}
          onClick={() => setActiveTab('topology')}
        >
          <Cpu size={14} /> Cluster Topology & Memory
        </button>

        <button
          className={`tab-btn ${activeTab === 'tiering' ? 'active' : ''}`}
          onClick={() => setActiveTab('tiering')}
        >
          <HardDrive size={14} /> Storage Tiering
        </button>

        <button
          className={`tab-btn ${activeTab === 'explain' ? 'active' : ''}`}
          onClick={() => setActiveTab('explain')}
        >
          <Zap size={14} /> Query Plan (--explain)
        </button>

        <button
          className={`tab-btn ${activeTab === 'durability' ? 'active' : ''}`}
          onClick={() => setActiveTab('durability')}
        >
          <ShieldCheck size={14} /> WAL Durability & Snapshots
        </button>

        <button
          className={`tab-btn ${activeTab === 'profiler' ? 'active' : ''}`}
          onClick={() => setActiveTab('profiler')}
        >
          <Activity size={14} /> Live Activity & Profiler
        </button>

        <button
          className={`tab-btn ${activeTab === 'workbench' ? 'active' : ''}`}
          onClick={() => setActiveTab('workbench')}
        >
          <Terminal size={14} /> RESP3 Workbench
        </button>
      </div>

      {/* Main Tab Content */}
      <main className="main-content">
        {activeTab === 'keyspace' && (
          <KeyspaceBrowser
            items={keyspace}
            onSetKey={handleSetKey}
            onDeleteKey={handleDeleteKey}
          />
        )}

        {activeTab === 'topology' && (
          <ClusterTopology
            shards={shards}
            keyspace={keyspace}
            epoch={epoch}
            lastSnapshot={lastSnapshot}
            walState={walState}
          />
        )}

        {activeTab === 'tiering' && <TieringConsole />}

        {activeTab === 'explain' && <ExplainVisualizer />}

        {activeTab === 'durability' && (
          <DurabilityConsole
            walState={walState}
            walRecords={walRecords}
            walSegments={walSegments}
            lastRecoveryReport={lastRecoveryReport}
            onChangeSyncPolicy={handleChangeSyncPolicy}
            onSimulateCrashAndReplay={handleSimulateCrashAndReplay}
            onTriggerCheckpoint={handleTriggerCheckpoint}
          />
        )}

        {activeTab === 'profiler' && (
          <ActivityProfiler
            logs={commandLogs}
            keyspace={keyspace}
            onClearLogs={() => setCommandLogs([])}
            workloadMode={workloadMode}
            opsPerSec={opsPerSec}
            onSetWorkloadMode={setWorkloadMode}
          />
        )}

        {activeTab === 'workbench' && (
          <RespWorkbench
            logs={commandLogs}
            onExecuteCommand={handleExecuteCommand}
            onClearLogs={() => setCommandLogs([])}
          />
        )}
      </main>
    </div>
  );
}

export default App;
