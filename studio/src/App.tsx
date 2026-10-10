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
} from 'lucide-react';
import { KeyspaceBrowser } from './components/KeyspaceBrowser';
import { ClusterTopology } from './components/ClusterTopology';
import { ActivityProfiler } from './components/ActivityProfiler';
import { RespWorkbench } from './components/RespWorkbench';
import type { ShardData, KeyspaceItem, CommandLog, ThemeMode } from './types';
import { routeKey } from './utils/router';

export function App() {
  // Navigation: 4 Core Pillars of Enterprise In-Memory Database
  const [activeTab, setActiveTab] = useState<'keyspace' | 'topology' | 'profiler' | 'workbench'>('keyspace');

  // Theme Mode: Default is 'slate' (balanced industrial mid-tone slate)
  const [theme, setTheme] = useState<ThemeMode>(() => {
    return (localStorage.getItem('valence_theme') as ThemeMode) || 'slate';
  });

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('valence_theme', theme);
  }, [theme]);

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
      epoch: 2,
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

  // Shards computed statically from active keyspace
  const shards: ShardData[] = [0, 1, 2, 3].map((shardId) => {
    const keysInShard = keyspace.filter((k) => k.targetShard === shardId).length;
    return {
      id: shardId,
      coreId: shardId,
      keyCount: keysInShard,
      opsPerSec: 0, // Grounded 0 when idle/standby
      queueDepth: 0, // Grounded 0 when idle
      queueCapacity: 1024,
      skewPercent: 0.0,
      isPinned: true,
      memoryAllocatedKb: 64,
      loadFactorPct: Math.round((keysInShard / 16) * 100),
      maxDib: 2,
    };
  });

  // Key Mutation Handlers
  const handleSetKey = (key: string, value: string, ttlMs: number | null) => {
    const isSso = key.length <= 15 && value.length <= 23;
    const targetShard = routeKey(key, shards.length);

    const newItem: KeyspaceItem = {
      key,
      value,
      dataType: 'string',
      sizeBytes: value.length,
      isSso,
      dib: Math.floor(Math.random() * 3),
      epoch: 1,
      ttlMs,
      expiresAt: ttlMs ? Date.now() + ttlMs : null,
      targetShard,
    };

    setKeyspace((prev) => [newItem, ...prev.filter((i) => i.key !== key)]);

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
    setKeyspace((prev) => prev.filter((k) => k.key !== key));

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
          const found = keyspace.find((item) => item.key === parts[1]);
          response = found ? `"${found.value}"` : '(nil)';
        }
        break;
      }

      case 'DEL': {
        if (parts.length < 2) {
          response = "-ERR wrong number of arguments for 'del' command";
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
          response = "-ERR wrong number of arguments for 'exists' command";
          isError = true;
        } else {
          const exists = keyspace.some((item) => item.key === parts[1]);
          response = `:(integer) ${exists ? 1 : 0}`;
        }
        break;
      }

      case 'KEYS': {
        const pattern = parts[1] || '*';
        const matched = keyspace
          .filter((item) => {
            if (pattern === '*') return true;
            const prefix = pattern.replace('*', '');
            return item.key.startsWith(prefix);
          })
          .map((item, idx) => `${idx + 1}) "${item.key}"`);
        response = `*${matched.length}\n${matched.join('\n')}`;
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
        response = `# Server\nvalence_version:0.1.0\narch:thread_per_core_shared_nothing\nshards:${shards.length}\nstate:standby_ready\n\n# Memory\nkeys_total:${keyspace.length}\nsso_inlined_ratio:${Math.round((keyspace.filter((k) => k.isSso).length / Math.max(1, keyspace.length)) * 100)}%\nslot_size_bytes:64\n\n# Expiration\ntiming_wheel_tiers:4\nresolution:1ms`;
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
              <span className="health-dot" />
              <span>ENGINE:</span>
              <span className="val" style={{ color: 'var(--accent-green)' }}>STANDBY (READY)</span>
            </div>

            <div className="stat-pill">
              <Cpu size={12} color="var(--accent-primary)" />
              <span>SHARDS:</span>
              <span className="val">{shards.length} Cores Pinned</span>
            </div>

            <div className="stat-pill">
              <Database size={12} color="var(--text-muted)" />
              <span>STORED KEYS:</span>
              <span className="val">{keyspace.length}</span>
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

      {/* Subnav Tabs: 4 Essential Enterprise Database Console Pillars */}
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
          className={`tab-btn ${activeTab === 'profiler' ? 'active' : ''}`}
          onClick={() => setActiveTab('profiler')}
        >
          <Activity size={14} /> Live Activity Profiler
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
          />
        )}

        {activeTab === 'profiler' && (
          <ActivityProfiler
            logs={commandLogs}
            keyspace={keyspace}
            onClearLogs={() => setCommandLogs([])}
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
