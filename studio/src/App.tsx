import { useState, useEffect } from 'react';
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

  // Initial Shard Data (4 Cores)
  const [shards, setShards] = useState<ShardData[]>([
    { id: 0, coreId: 0, keyCount: 2540, opsPerSec: 38200, queueDepth: 8, queueCapacity: 1024, skewPercent: 0.2, isPinned: true },
    { id: 1, coreId: 1, keyCount: 2510, opsPerSec: 39100, queueDepth: 12, queueCapacity: 1024, skewPercent: 0.4, isPinned: true },
    { id: 2, coreId: 2, keyCount: 2580, opsPerSec: 37900, queueDepth: 4, queueCapacity: 1024, skewPercent: 0.3, isPinned: true },
    { id: 3, coreId: 3, keyCount: 2490, opsPerSec: 40100, queueDepth: 15, queueCapacity: 1024, skewPercent: 0.6, isPinned: true },
  ]);

  // Initial Keyspace
  const [keyspace, setKeyspace] = useState<KeyspaceItem[]>([
    {
      key: 'user:session:991',
      value: 'active_auth_tk',
      sizeBytes: 14,
      isSso: true,
      dib: 2,
      epoch: 1,
      ttlMs: 45000,
      expiresAt: Date.now() + 45000,
      targetShard: routeKey('user:session:991', 4),
    },
    {
      key: 'ip:limit:10.0.1.5',
      value: '42',
      sizeBytes: 2,
      isSso: true,
      dib: 0,
      epoch: 1,
      ttlMs: 60000,
      expiresAt: Date.now() + 60000,
      targetShard: routeKey('ip:limit:10.0.1.5', 4),
    },
    {
      key: 'app:config:flags',
      value: '{"debug":false,"tracing":true,"zero_fork":true}',
      sizeBytes: 48,
      isSso: false,
      dib: 4,
      epoch: 2,
      ttlMs: null,
      expiresAt: null,
      targetShard: routeKey('app:config:flags', 4),
    },
    {
      key: 'dev:sensor:node_3',
      value: 'temp_22.4c',
      sizeBytes: 10,
      isSso: true,
      dib: 1,
      epoch: 1,
      ttlMs: 12000,
      expiresAt: Date.now() + 12000,
      targetShard: routeKey('dev:sensor:node_3', 4),
    },
    {
      key: 'cart:checkout:902',
      value: 'item_list_payload',
      sizeBytes: 17,
      isSso: true,
      dib: 3,
      epoch: 3,
      ttlMs: null,
      expiresAt: null,
      targetShard: routeKey('cart:checkout:902', 4),
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
      command: 'SET user:session:991 "active_auth_tk" PX 45000',
      response: '+OK',
      durationUs: 12.8,
      isError: false,
    },
  ]);

  // Periodic Telemetry Simulation
  useEffect(() => {
    const timer = setInterval(() => {
      setShards((prev) =>
        prev.map((s) => ({
          ...s,
          opsPerSec: Math.round(36000 + Math.random() * 6000),
          queueDepth: Math.max(0, Math.min(1024, s.queueDepth + Math.floor(Math.random() * 5) - 2)),
        }))
      );
    }, 1500);
    return () => clearInterval(timer);
  }, []);

  // Set Key Handler
  const handleSetKey = (key: string, value: string, ttlMs: number | null) => {
    const isSso = key.length <= 15 && value.length <= 23;
    const targetShard = routeKey(key, shards.length);

    const newItem: KeyspaceItem = {
      key,
      value,
      sizeBytes: value.length,
      isSso,
      dib: Math.floor(Math.random() * 6),
      epoch: 1,
      ttlMs,
      expiresAt: ttlMs ? Date.now() + ttlMs : null,
      targetShard,
    };

    setKeyspace((prev) => [newItem, ...prev.filter((i) => i.key !== key)]);

    setShards((prev) =>
      prev.map((s) => (s.id === targetShard ? { ...s, keyCount: s.keyCount + 1 } : s))
    );
  };

  // Delete Key Handler (O(1) Backshift)
  const handleDeleteKey = (key: string) => {
    const target = keyspace.find((k) => k.key === key);
    if (!target) return;

    setKeyspace((prev) => prev.filter((k) => k.key !== key));
    setShards((prev) =>
      prev.map((s) => (s.id === target.targetShard ? { ...s, keyCount: Math.max(0, s.keyCount - 1) } : s))
    );
  };

  // Bulk 10k Keys Distribution Simulation
  const handleAddSampleKeys = () => {
    setShards((prev) =>
      prev.map((s) => ({
        ...s,
        keyCount: s.keyCount + 2500,
        opsPerSec: s.opsPerSec + 12000,
      }))
    );
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
        response = `# Server\nvalence_version:0.1.0\narch:thread_per_core_shared_nothing\nshards:${shards.length}\n\n# Memory\nsso_inlined_ratio:${Math.round((keyspace.filter(k => k.isSso).length / Math.max(1, keyspace.length)) * 100)}%\nslot_size_bytes:64\nfalse_sharing_eliminated:true\n\n# Expiration\ntiming_wheel_tiers:4\navalanche_immunity:active`;
        break;

      default:
        response = `-ERR unknown command '${verb}'`;
        isError = true;
        break;
    }

    const durationUs = Math.max(1.8, (performance.now() - start) * 1000 + Math.random() * 8);

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

  const totalOps = shards.reduce((acc, s) => acc + s.opsPerSec, 0);

  return (
    <div className="app-container">
      {/* Top Industrial Navbar */}
      <nav className="top-nav">
        <div className="brand-section">
          <div className="brand-logo">
            <Flame size={20} color="var(--accent-amber)" />
            <span>VALENCE</span>
          </div>
          <span className="brand-badge">STUDIO</span>
          <span style={{ fontSize: '11px', color: 'var(--text-dim)', fontFamily: 'var(--font-mono)' }}>
            v0.1.0-alpha
          </span>
        </div>

        <div className="cluster-stats-strip">
          <div className="stat-pill">
            <span className="health-dot" />
            <span>STATUS:</span>
            <span className="val" style={{ color: 'var(--accent-green)' }}>ONLINE</span>
          </div>

          <div className="stat-pill">
            <Cpu size={13} color="var(--accent-amber)" />
            <span>SHARDS:</span>
            <span className="val">{shards.length} Pinned Cores</span>
          </div>

          <div className="stat-pill">
            <Zap size={13} color="var(--accent-amber)" />
            <span>THROUGHPUT:</span>
            <span className="val">{totalOps.toLocaleString()} ops/s</span>
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
          <ShardMatrix shards={shards} onAddSampleKeys={handleAddSampleKeys} />
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
