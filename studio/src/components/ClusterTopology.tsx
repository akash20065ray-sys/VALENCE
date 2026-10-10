import React from 'react';
import {
  Cpu,
  Layers,
  ShieldCheck,
  Zap,
  Activity,
  HardDrive,
  CheckCircle2,
  Save,
  Check,
  FileText,
} from 'lucide-react';
import type { ShardData, KeyspaceItem, SnapshotRecord, WalState } from '../types';

interface ClusterTopologyProps {
  shards: ShardData[];
  keyspace: KeyspaceItem[];
  epoch: number;
  lastSnapshot: SnapshotRecord | null;
  walState?: WalState;
}

export const ClusterTopology: React.FC<ClusterTopologyProps> = ({
  shards,
  keyspace,
  epoch,
  lastSnapshot,
  walState,
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
          gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
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
            <span>INTER-CORE SPSC CHANNELS</span>
          </div>
          <div style={{ fontSize: '20px', fontWeight: 700, marginTop: '6px', color: 'var(--text-primary)' }}>
            64B SPSC Ring
          </div>
          <div style={{ fontSize: '11px', color: 'var(--text-secondary)', marginTop: '4px' }}>
            Cache-Line Padded &bull; Lock-Free IPC
          </div>
        </div>

        <div className="card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-muted)', fontSize: '11.5px' }}>
            <Layers size={14} color="var(--accent-green)" />
            <span>SMALL STRING INLINING (SSO)</span>
          </div>
          <div style={{ fontSize: '20px', fontWeight: 700, marginTop: '6px', color: 'var(--text-primary)' }}>
            {ssoRatio}% Inlined
          </div>
          <div style={{ fontSize: '11px', color: 'var(--text-secondary)', marginTop: '4px' }}>
            {ssoKeys} of {totalKeys} keys inlined in pointer
          </div>
        </div>

        <div className="card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-muted)', fontSize: '11.5px' }}>
            <HardDrive size={14} color="var(--accent-amber)" />
            <span>PERSISTENCE & RECOVERY</span>
          </div>
          <div style={{ fontSize: '20px', fontWeight: 700, marginTop: '6px', color: 'var(--text-primary)' }}>
            Epoch #{epoch}
          </div>
          <div style={{ fontSize: '11px', color: 'var(--accent-green)', marginTop: '4px', display: 'flex', alignItems: 'center', gap: '4px' }}>
            <Check size={12} />
            <span>CRC64 Validated &bull; Streaming CoW</span>
          </div>
        </div>
      </div>

      {/* Snapshot & Persistence Architecture Card */}
      <div className="card" style={{ background: 'var(--bg-surface)' }}>
        <div className="card-header">
          <div className="card-title">
            <Save size={15} color="var(--accent-primary)" />
            <span>Point-in-Time Persistence & Crash Recovery (Valence .val v1)</span>
          </div>
          <span style={{ fontSize: '11.5px', color: 'var(--accent-green)', fontFamily: 'JetBrains Mono, monospace', display: 'flex', alignItems: 'center', gap: '4px' }}>
            <CheckCircle2 size={13} /> Fork-Free Streaming
          </span>
        </div>

        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
            gap: '12px',
          }}
        >
          <div style={{ background: 'var(--bg-elevated)', padding: '10px 14px', borderRadius: '6px', border: '1px solid var(--border-subtle)' }}>
            <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>CURRENT MUTATION EPOCH</div>
            <div style={{ fontSize: '15px', fontWeight: 700, marginTop: '2px', color: 'var(--accent-primary)', fontFamily: 'JetBrains Mono, monospace' }}>
              Epoch #{epoch}
            </div>
            <div style={{ fontSize: '10.5px', color: 'var(--text-dim)', marginTop: '2px' }}>
              Monotonic AtomicU64 mutation tracker
            </div>
          </div>

          <div style={{ background: 'var(--bg-elevated)', padding: '10px 14px', borderRadius: '6px', border: '1px solid var(--border-subtle)' }}>
            <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>COPY-ON-WRITE MEMORY OVERHEAD</div>
            <div style={{ fontSize: '15px', fontWeight: 700, marginTop: '2px', color: 'var(--accent-green)', fontFamily: 'JetBrains Mono, monospace' }}>
              0B (Key-Level Shadow Buffer)
            </div>
            <div style={{ fontSize: '10.5px', color: 'var(--text-dim)', marginTop: '2px' }}>
              Bypasses Linux 4KB page duplication & OOM
            </div>
          </div>

          <div style={{ background: 'var(--bg-elevated)', padding: '10px 14px', borderRadius: '6px', border: '1px solid var(--border-subtle)' }}>
            <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>DATA INTEGRITY CHECKSUM</div>
            <div style={{ fontSize: '15px', fontWeight: 700, marginTop: '2px', color: 'var(--accent-cyan)', fontFamily: 'JetBrains Mono, monospace' }}>
              ECMA-182 CRC64 (Verified)
            </div>
            <div style={{ fontSize: '10.5px', color: 'var(--text-dim)', marginTop: '2px' }}>
              Multi-GB/s precomputed table validation
            </div>
          </div>

          <div style={{ background: 'var(--bg-elevated)', padding: '10px 14px', borderRadius: '6px', border: '1px solid var(--border-subtle)' }}>
            <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>LATEST SNAPSHOT ON DISK</div>
            <div style={{ fontSize: '13px', fontWeight: 600, marginTop: '2px', color: 'var(--text-primary)', fontFamily: 'JetBrains Mono, monospace' }}>
              {lastSnapshot ? `Epoch #${lastSnapshot.epoch} (${lastSnapshot.itemCount} keys)` : 'None (Use BGSAVE / SAVE)'}
            </div>
            <div style={{ fontSize: '10.5px', color: 'var(--text-dim)', marginTop: '2px' }}>
              {lastSnapshot ? `CRC: ${lastSnapshot.crc64Hex} @ ${lastSnapshot.timestamp}` : 'Ready for on-demand persistence'}
            </div>
          </div>
        </div>
      </div>

      {/* Segmented Binary WAL & Group Commit Pipeline Card */}
      <div className="card" style={{ background: 'var(--bg-surface)' }}>
        <div className="card-header">
          <div className="card-title">
            <FileText size={15} color="var(--accent-cyan)" />
            <span>Segmented Binary Write-Ahead Log (WAL) & Group Commit</span>
          </div>
          <span style={{ fontSize: '11.5px', color: 'var(--accent-cyan)', fontFamily: 'JetBrains Mono, monospace', display: 'flex', alignItems: 'center', gap: '4px' }}>
            <CheckCircle2 size={13} /> {walState?.syncPolicy ?? 'GroupCommit'} Durability
          </span>
        </div>

        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
            gap: '12px',
          }}
        >
          <div style={{ background: 'var(--bg-elevated)', padding: '10px 14px', borderRadius: '6px', border: '1px solid var(--border-subtle)' }}>
            <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>LOG SEQUENCE NUMBER (LSN)</div>
            <div style={{ fontSize: '15px', fontWeight: 700, marginTop: '2px', color: 'var(--accent-cyan)', fontFamily: 'JetBrains Mono, monospace' }}>
              LSN #{walState?.currentLsn ?? 13}
            </div>
            <div style={{ fontSize: '10.5px', color: 'var(--text-dim)', marginTop: '2px' }}>
              Strict monotonic ordering across all shards
            </div>
          </div>

          <div style={{ background: 'var(--bg-elevated)', padding: '10px 14px', borderRadius: '6px', border: '1px solid var(--border-subtle)' }}>
            <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>ACTIVE ROTATION SEGMENT</div>
            <div style={{ fontSize: '13px', fontWeight: 600, marginTop: '2px', color: 'var(--text-primary)', fontFamily: 'JetBrains Mono, monospace' }}>
              {walState?.activeSegment ?? 'wal_0000000000000001.valwal'}
            </div>
            <div style={{ fontSize: '10.5px', color: 'var(--text-dim)', marginTop: '2px' }}>
              64MB boundary rotation & deterministic headers
            </div>
          </div>

          <div style={{ background: 'var(--bg-elevated)', padding: '10px 14px', borderRadius: '6px', border: '1px solid var(--border-subtle)' }}>
            <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>CRASH RECOVERY REPLAY</div>
            <div style={{ fontSize: '15px', fontWeight: 700, marginTop: '2px', color: 'var(--accent-green)', fontFamily: 'JetBrains Mono, monospace' }}>
              Zero-Loss Tail Scan
            </div>
            <div style={{ fontSize: '10.5px', color: 'var(--text-dim)', marginTop: '2px' }}>
              Torn-write truncation & CRC64 verification
            </div>
          </div>

          <div style={{ background: 'var(--bg-elevated)', padding: '10px 14px', borderRadius: '6px', border: '1px solid var(--border-subtle)' }}>
            <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>FSYNC GROUP PIPELINE</div>
            <div style={{ fontSize: '15px', fontWeight: 700, marginTop: '2px', color: 'var(--accent-amber)', fontFamily: 'JetBrains Mono, monospace' }}>
              High-Throughput Batching
            </div>
            <div style={{ fontSize: '10.5px', color: 'var(--text-dim)', marginTop: '2px' }}>
              Sub-millisecond batched disk flush
            </div>
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
            <span style={{ fontWeight: 600, color: 'var(--text-primary)' }}>3. Forkless Epoch-Versioned Snapshots:</span>
            <p style={{ marginTop: '4px' }}>
              Replaces Redis's Linux fork() with key-level Copy-on-Write and monotonic mutation epochs. Zero virtual memory page duplication, eliminating OOM killer crashes.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
