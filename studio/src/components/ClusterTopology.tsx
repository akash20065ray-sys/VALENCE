import React from 'react';
import {
  Cpu,
  Layers,
  ShieldCheck,
  Zap,
  Activity,
  HardDrive,
  CheckCircle2,
} from 'lucide-react';
import type { ShardData, KeyspaceItem } from '../types';

interface ClusterTopologyProps {
  shards: ShardData[];
  keyspace: KeyspaceItem[];
}

export const ClusterTopology: React.FC<ClusterTopologyProps> = ({
  shards,
  keyspace,
}) => {
  const totalKeys = keyspace.length;
  const ssoKeys = keyspace.filter((k) => k.isSso).length;
  const ssoRatio = totalKeys > 0 ? Math.round((ssoKeys / totalKeys) * 100) : 0;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Cluster Architecture Summary Bar */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
          gap: '14px',
        }}
      >
        <div className="card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-muted)', fontSize: '11.5px' }}>
            <Cpu size={14} color="var(--accent-primary)" />
            <span>THREAD-PER-CORE TOPOLOGY</span>
          </div>
          <div style={{ fontSize: '20px', fontWeight: 700, marginTop: '6px', color: 'var(--text-primary)' }}>
            {shards.length} Cores Pinned
          </div>
          <div style={{ fontSize: '11px', color: 'var(--accent-green)', marginTop: '4px', display: 'flex', alignItems: 'center', gap: '4px' }}>
            <CheckCircle2 size={12} />
            <span>Hardware Affinity Bound (CPU_SET)</span>
          </div>
        </div>

        <div className="card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-muted)', fontSize: '11.5px' }}>
            <Zap size={14} color="var(--accent-cyan)" />
            <span>ZERO-LOCK IPC RINGS</span>
          </div>
          <div style={{ fontSize: '20px', fontWeight: 700, marginTop: '6px', color: 'var(--text-primary)' }}>
            64B SPSC Ring
          </div>
          <div style={{ fontSize: '11px', color: 'var(--text-secondary)', marginTop: '4px' }}>
            Cache-Line Padded &bull; Mutex-Free
          </div>
        </div>

        <div className="card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-muted)', fontSize: '11.5px' }}>
            <Layers size={14} color="var(--accent-green)" />
            <span>ZERO-HEAP SSO EFFICIENCY</span>
          </div>
          <div style={{ fontSize: '20px', fontWeight: 700, marginTop: '6px', color: 'var(--text-primary)' }}>
            {ssoRatio}% Inlined
          </div>
          <div style={{ fontSize: '11px', color: 'var(--text-secondary)', marginTop: '4px' }}>
            {ssoKeys} of {totalKeys} keys bypass malloc
          </div>
        </div>

        <div className="card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-muted)', fontSize: '11.5px' }}>
            <HardDrive size={14} color="var(--accent-amber)" />
            <span>KEYSPACE DISTRIBUTION</span>
          </div>
          <div style={{ fontSize: '20px', fontWeight: 700, marginTop: '6px', color: 'var(--text-primary)' }}>
            Uniform Jump Hash
          </div>
          <div style={{ fontSize: '11px', color: 'var(--text-secondary)', marginTop: '4px' }}>
            Minimal shard migration on scale
          </div>
        </div>
      </div>

      {/* Shard Cores Grid */}
      <div>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px' }}>
          <div style={{ fontSize: '13px', fontWeight: 600, color: 'var(--text-primary)', display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Activity size={15} color="var(--accent-primary)" />
            <span>Hardware Shard Workers & Isolated Ring Buffers</span>
          </div>
          <span style={{ fontSize: '11.5px', color: 'var(--text-muted)', fontFamily: 'JetBrains Mono, monospace' }}>
            Shared-Nothing Architecture &bull; 0 Lock Contention
          </span>
        </div>

        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))',
            gap: '14px',
          }}
        >
          {shards.map((shard) => {
            const shardKeys = keyspace.filter((k) => k.targetShard === shard.id);
            const keySharePct = totalKeys > 0 ? Math.round((shardKeys.length / totalKeys) * 100) : 0;
            const maxDib = shardKeys.reduce((m, k) => Math.max(m, k.dib), 0);

            return (
              <div key={shard.id} className="card" style={{ padding: '16px' }}>
                {/* Header */}
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <div
                      style={{
                        background: 'var(--accent-primary-bg)',
                        color: 'var(--accent-primary)',
                        padding: '4px 8px',
                        borderRadius: '5px',
                        fontFamily: 'JetBrains Mono, monospace',
                        fontWeight: 700,
                        fontSize: '12px',
                      }}
                    >
                      SHARD #{shard.id}
                    </div>
                    <span style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>
                      CPU Core {shard.coreId}
                    </span>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span className="health-dot" />
                    <span style={{ fontSize: '11px', color: 'var(--accent-green)', fontWeight: 600 }}>
                      PINNED
                    </span>
                  </div>
                </div>

                {/* Shard Key Distribution */}
                <div style={{ marginBottom: '14px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '11.5px', marginBottom: '4px' }}>
                    <span style={{ color: 'var(--text-secondary)' }}>Assigned Keys:</span>
                    <span style={{ color: 'var(--text-primary)', fontWeight: 600, fontFamily: 'JetBrains Mono, monospace' }}>
                      {shardKeys.length} keys ({keySharePct}%)
                    </span>
                  </div>
                  <div style={{ height: '6px', background: 'var(--bg-elevated)', borderRadius: '3px', overflow: 'hidden' }}>
                    <div
                      style={{
                        height: '100%',
                        width: `${Math.max(5, keySharePct)}%`,
                        background: 'var(--accent-primary)',
                        borderRadius: '3px',
                      }}
                    />
                  </div>
                </div>

                {/* SPSC Ring Queue Monitor */}
                <div
                  style={{
                    background: 'var(--bg-elevated)',
                    border: '1px solid var(--border-subtle)',
                    borderRadius: '6px',
                    padding: '10px 12px',
                    marginBottom: '12px',
                  }}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '11px', color: 'var(--text-muted)', marginBottom: '4px' }}>
                    <span>SPSC RING QUEUE</span>
                    <span style={{ color: 'var(--accent-green)' }}>STANDBY (0 INGESTION)</span>
                  </div>
                  <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '12.5px', fontWeight: 600, color: 'var(--text-primary)', fontFamily: 'JetBrains Mono, monospace' }}>
                    <span>Depth: {shard.queueDepth} / {shard.queueCapacity}</span>
                    <span style={{ color: 'var(--text-secondary)', fontSize: '11px' }}>64B Cache Line Aligned</span>
                  </div>
                </div>

                {/* Memory & Hash Table Internals */}
                <div
                  style={{
                    display: 'grid',
                    gridTemplateColumns: 'repeat(2, 1fr)',
                    gap: '8px',
                    fontSize: '11.5px',
                  }}
                >
                  <div style={{ background: 'var(--bg-surface)', padding: '8px 10px', borderRadius: '5px', border: '1px solid var(--border-subtle)' }}>
                    <div style={{ color: 'var(--text-dim)', fontSize: '10.5px' }}>MAX DIB PROBE</div>
                    <div style={{ fontWeight: 600, color: 'var(--accent-cyan)', fontFamily: 'JetBrains Mono, monospace', marginTop: '2px' }}>
                      {maxDib} slots
                    </div>
                  </div>
                  <div style={{ background: 'var(--bg-surface)', padding: '8px 10px', borderRadius: '5px', border: '1px solid var(--border-subtle)' }}>
                    <div style={{ color: 'var(--text-dim)', fontSize: '10.5px' }}>AFFINITY MASK</div>
                    <div style={{ fontWeight: 600, color: 'var(--text-primary)', fontFamily: 'JetBrains Mono, monospace', marginTop: '2px' }}>
                      0x{(1 << shard.coreId).toString(16).toUpperCase()}
                    </div>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Architectural Guarantee Notes */}
      <div className="card" style={{ background: 'var(--bg-surface)' }}>
        <div className="card-header">
          <div className="card-title">
            <ShieldCheck size={16} color="var(--accent-green)" />
            <span>Valence Engine Architectural Guarantees</span>
          </div>
          <span style={{ fontSize: '11.5px', color: 'var(--text-dim)', fontFamily: 'JetBrains Mono, monospace' }}>
            Zero-Cost Abstractions in Pure Rust
          </span>
        </div>
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
            gap: '16px',
            fontSize: '12px',
            color: 'var(--text-secondary)',
            lineHeight: 1.6,
          }}
        >
          <div>
            <span style={{ fontWeight: 600, color: 'var(--text-primary)' }}>1. Shared-Nothing Thread Isolation:</span>
            <p style={{ marginTop: '4px' }}>
              Each shard runs strictly on a pinned CPU core without cross-thread mutexes. Keys allocated to a shard never lock or block other worker cores.
            </p>
          </div>
          <div>
            <span style={{ fontWeight: 600, color: 'var(--text-primary)' }}>2. 64-Byte Cache Line Padding:</span>
            <p style={{ marginTop: '4px' }}>
              SPSC lock-free head/tail pointers reside on isolated 64-byte hardware cache lines, eliminating multi-core false sharing on modern x86_64 / ARM architectures.
            </p>
          </div>
          <div>
            <span style={{ fontWeight: 600, color: 'var(--text-primary)' }}>3. SSO 23-Byte Inlined Keys:</span>
            <p style={{ marginTop: '4px' }}>
              Keys up to 15B and values up to 23B are packed directly within Robin Hood slots. Bypasses standard malloc overhead entirely.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
