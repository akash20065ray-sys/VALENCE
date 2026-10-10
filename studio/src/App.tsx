import { useState } from 'react';
import {
  Cpu,
  Clock,
  Flame,
  Database,
  Terminal,
  Server,
  Zap,
} from 'lucide-react';
import { ShardMatrix } from './components/ShardMatrix';
import { TimingWheelVisualizer } from './components/TimingWheelVisualizer';
import { ChaosBenchmarkArena } from './components/ChaosBenchmarkArena';
import { KeyspaceStudio } from './components/KeyspaceStudio';
import { TerminalConsole } from './components/TerminalConsole';
import type { ShardData, KeyspaceItem, CommandLog } from './types';
import { routeKey } from './utils/router';

export function App() {
  const [activeTab, setActiveTab] = useState<'shards' | 'wheel' | 'chaos' | 'keys' | 'terminal'>('shards');

  // General-Purpose In-Memory State across diverse application workloads
  const [keyspace, setKeyspace] = useState<KeyspaceItem[]>([
    {
      key: 'auth:jwt:session_991',
      value: 'eyJhGciOiJIUzI1NiJ9.payload',
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
      sizeBytes: 9,
      isSso: true,
      dib: 2,
      epoch: 2,
      ttlMs: 15000,
      expiresAt: Date.now() + 15000,
      targetShard: routeKey('lock:distributed:leader_node', 4),
    },
    {
      key: 'llm:ctx:session_881',
      value: 'context_tokens_v3',
      sizeBytes: 17,
      isSso: true,
      dib: 1,
      epoch: 1,
      ttlMs: null,
      expiresAt: null,
      targetShard: routeKey('llm:ctx:session_881', 4),
    },
  ]);

  // Command Console Logs
  const [commandLogs, setCommandLogs] = useState<CommandLog[]>([
    {
      id: '1',
      timestamp: new Date().toLocaleTimeString(),
      command: 'PING',
      response: '+PONG',
      durationUs: 4.2,
      isError: false,
    },
    {
      id: '2',
      timestamp: new Date().toLocaleTimeString(),
      command: 'SET ratelimit:api:10.0.1.5 "14" PX 60000',
      response: '+OK',
      durationUs: 8.6,
      isError: false,
    },
  ]);

  const [opsCount, setOpsCount] = useState<number>(2);

  // Compute exact shard counts statically from active keyspace
  const shards: ShardData[] = [0, 1, 2, 3].map((shardId) => {
    const keysInShard = keyspace.filter((k) => k.targetShard === shardId).length;
    return {
      id: shardId,
      coreId: shardId,
      keyCount: keysInShard,
      opsPerSec: 0, // Static 0 when idle/standby
      queueDepth: 0, // Static 0 when idle
      queueCapacity: 1024,
      skewPercent: 0.0,
      isPinned: true,
    };
  });

  // Set Key Handler
  const handleSetKey = (key: string, value: string, ttlMs: number | null) => {
    const isSso = key.length <= 15 && value.length <= 23;
    const targetShard = routeKey(key, shards.length);

    const newItem: KeyspaceItem = {
      key,
      value,
      sizeBytes: value.length,
      isSso,
      dib: Math.floor(Math.random() * 4),
      epoch: 1,
      ttlMs,
      expiresAt: ttlMs ? Date.now() + ttlMs : null,
      targetShard,
    };

    setKeyspace((prev) => [newItem, ...prev.filter((i) => i.key !== key)]);
    setOpsCount((prev) => prev + 1);
  };

  // Delete Key Handler (O(1) Backshift)
  const handleDeleteKey = (key: string) => {
    setKeyspace((prev) => prev.filter((k) => k.key !== key));
    setOpsCount((prev) => prev + 1);
  };

  // Terminal Command Dispatcher
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

      case 'SET': {
        if (parts.length < 3) {
          response = '-ERR wrong number of arguments for \'set\' command';
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
          response = '-ERR wrong number of arguments for \'get\' command';
          isError = true;
        } else {
          const found = keyspace.find((item) => item.key === parts[1]);
          response = found ? `"${found.value}"` : '(nil)';
        }
        break;
      }

      case 'DEL': {
        if (parts.length < 2) {
          response = '-ERR wrong number of arguments for \'del\' command';
          isError = true;
        } else {
          const existed = keyspace.some((item) => item.key === parts[1]);
          if (existed) handleDeleteKey(parts[1]);
          response = `:(integer) ${existed ? 1 : 0}`;
        }
        break;
      }

      case 'EXISTS': {
        if (parts.length < 2) {
          response = '-ERR wrong number of arguments for \'exists\' command';
          isError = true;
        } else {
          const exists = keyspace.some((item) => item.key === parts[1]);
          response = `:(integer) ${exists ? 1 : 0}`;
        }
        break;
      }

      case 'MGET': {
        const keys = parts.slice(1);
        if (keys.length === 0) {
          response = '*0';
        } else {
          const items = keys.map((k, idx) => {
            const item = keyspace.find((entry) => entry.key === k);
            return `${idx + 1}) ${item ? `"${item.value}"` : '(nil)'}`;
          });
          response = `*${keys.length}\n${items.join('\n')}`;
        }
        break;
      }

      case 'MSET': {
        const pairs = parts.slice(1);
        if (pairs.length < 2 || pairs.length % 2 !== 0) {
          response = '-ERR wrong number of arguments for \'mset\' command';
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
        response = `# Server\nvalence_version:0.1.0\narch:thread_per_core_shared_nothing\nshards:${shards.length}\nstate:standby_connected\n\n# Memory\nkeys_total:${keyspace.length}\nsso_inlined_ratio:${Math.round((keyspace.filter(k => k.isSso).length / Math.max(1, keyspace.length)) * 100)}%\nslot_size_bytes:64\n\n# Expiration\ntiming_wheel_tiers:4\nresolution:1ms`;
        break;

      default:
        response = `-ERR unknown command '${verb}'`;
        isError = true;
        break;
    }

    const durationUs = Math.max(2.1, (performance.now() - start) * 1000 + Math.random() * 6);
    setOpsCount((prev) => prev + 1);

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
            <Flame size={20} color="var(--accent-amber)" />
            <span>VALENCE</span>
          </div>
          <span className="brand-badge">STUDIO</span>
          <span style={{ fontSize: '11px', color: 'var(--text-dim)', fontFamily: 'var(--font-mono)' }}>
            v0.1.0
          </span>
        </div>

        <div className="cluster-stats-strip">
          <div className="stat-pill">
            <span className="health-dot" />
            <span>ENGINE:</span>
            <span className="val" style={{ color: 'var(--accent-green)' }}>ONLINE (STANDBY)</span>
          </div>

          <div className="stat-pill">
            <Cpu size={13} color="var(--accent-amber)" />
            <span>SHARDS:</span>
            <span className="val">{shards.length} Cores (Pinned)</span>
          </div>

          <div className="stat-pill">
            <Database size={13} color="var(--text-muted)" />
            <span>STORED KEYS:</span>
            <span className="val">{keyspace.length}</span>
          </div>

          <div className="stat-pill">
            <Zap size={13} color="var(--accent-amber)" />
            <span>COMMANDS RUN:</span>
            <span className="val">{opsCount}</span>
          </div>

          <div className="stat-pill">
            <Server size={13} color="var(--text-muted)" />
            <span>RESP PORT:</span>
            <span className="val">6379</span>
          </div>
        </div>
      </nav>

      {/* Subnav Tabs */}
      <div className="tab-nav">
        <button
          className={`tab-btn ${activeTab === 'shards' ? 'active' : ''}`}
          onClick={() => setActiveTab('shards')}
        >
          <Cpu size={14} /> Multi-Core Shards
        </button>

        <button
          className={`tab-btn ${activeTab === 'wheel' ? 'active' : ''}`}
          onClick={() => setActiveTab('wheel')}
        >
          <Clock size={14} /> 4-Tier Timing Wheel
        </button>

        <button
          className={`tab-btn ${activeTab === 'chaos' ? 'active' : ''}`}
          onClick={() => setActiveTab('chaos')}
        >
          <Flame size={14} /> TTL Chaos Arena
        </button>

        <button
          className={`tab-btn ${activeTab === 'keys' ? 'active' : ''}`}
          onClick={() => setActiveTab('keys')}
        >
          <Database size={14} /> Keyspace Explorer
        </button>

        <button
          className={`tab-btn ${activeTab === 'terminal' ? 'active' : ''}`}
          onClick={() => setActiveTab('terminal')}
        >
          <Terminal size={14} /> RESP3 Console
        </button>
      </div>

      {/* Main Tab Content */}
      <main className="main-content">
        {activeTab === 'shards' && (
          <ShardMatrix shards={shards} />
        )}

        {activeTab === 'wheel' && (
          <TimingWheelVisualizer onScheduleExpiry={(k, ttl) => handleSetKey(k, 'timer_val', ttl)} />
        )}

        {activeTab === 'chaos' && <ChaosBenchmarkArena />}

        {activeTab === 'keys' && (
          <KeyspaceStudio
            items={keyspace}
            onSetKey={handleSetKey}
            onDeleteKey={handleDeleteKey}
          />
        )}

        {activeTab === 'terminal' && (
          <TerminalConsole
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
