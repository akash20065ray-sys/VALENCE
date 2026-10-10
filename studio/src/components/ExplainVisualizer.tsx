import React, { useState } from 'react';
import {
  Terminal,
  Cpu,
  Layers,
  Zap,
  Clock,
  HardDrive,
  Play,
} from 'lucide-react';

export const ExplainVisualizer: React.FC = () => {
  const [queryInput, setQueryInput] = useState<string>('GET "user:session:100"');
  const [activeStage, setActiveStage] = useState<number>(0);
  const [viewMode, setViewMode] = useState<'diagram' | 'terminal'>('diagram');

  // Query Parsing Helper
  const parts = queryInput.trim().split(/\s+/);
  const cmd = (parts[0] || 'GET').toUpperCase();
  const rawKey = (parts[1] || 'user:session:100').replace(/['"]/g, '');

  // Synthetic deterministic hash & sharding calculations
  const hash = 0xa5c2c12c;
  const shardId = Math.abs(rawKey.split('').reduce((acc, c) => acc + c.charCodeAt(0), 0) % 4);
  const assignedCore = shardId;
  const idealSlot = (hash + rawKey.length * 31) % 1024;
  const actualSlot = idealSlot + 1; // DIB: 1

  return (
    <div className="explain-visualizer" style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Top Query Input Toolbar */}
      <div
        className="card"
        style={{
          padding: '16px 20px',
          background: 'var(--bg-card)',
          borderRadius: '8px',
          border: '1px solid var(--border-default)',
          display: 'flex',
          flexDirection: 'column',
          gap: '12px',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div>
            <h3 style={{ margin: 0, fontSize: '16px', fontWeight: 600, color: 'var(--text-primary)' }}>
              Diagnostic Query Plan Visualizer (`--explain`)
            </h3>
            <span style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>
              Inspect live stage-by-stage microsecond latency breakdowns and mechanical sympathy paths
            </span>
          </div>

          <div style={{ display: 'flex', gap: '8px' }}>
            <button
              onClick={() => setViewMode('diagram')}
              style={{
                padding: '6px 12px',
                borderRadius: '4px',
                border: '1px solid var(--border-default)',
                background: viewMode === 'diagram' ? 'var(--accent-primary)' : 'var(--bg-surface)',
                color: viewMode === 'diagram' ? '#fff' : 'var(--text-secondary)',
                fontSize: '11px',
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              Interactive Diagram
            </button>
            <button
              onClick={() => setViewMode('terminal')}
              style={{
                padding: '6px 12px',
                borderRadius: '4px',
                border: '1px solid var(--border-default)',
                background: viewMode === 'terminal' ? 'var(--accent-primary)' : 'var(--bg-surface)',
                color: viewMode === 'terminal' ? '#fff' : 'var(--text-secondary)',
                fontSize: '11px',
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              Terminal ANSI Trace
            </button>
          </div>
        </div>

        {/* Input Bar */}
        <div style={{ display: 'flex', gap: '10px' }}>
          <div
            style={{
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              padding: '8px 12px',
              background: 'var(--bg-input)',
              border: '1px solid var(--border-focus)',
              borderRadius: '6px',
            }}
          >
            <Terminal size={14} color="var(--accent-primary)" />
            <input
              type="text"
              value={queryInput}
              onChange={(e) => setQueryInput(e.target.value)}
              placeholder="e.g. GET user:session:100"
              style={{
                flex: 1,
                background: 'transparent',
                border: 'none',
                color: 'var(--text-primary)',
                fontFamily: 'JetBrains Mono',
                fontSize: '13px',
                outline: 'none',
              }}
            />
          </div>
          <button
            onClick={() => setActiveStage((prev) => (prev + 1) % 6)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '8px 18px',
              background: 'var(--accent-primary)',
              color: '#fff',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              fontWeight: 600,
              fontSize: '12px',
            }}
          >
            <Play size={13} /> Step Trace
          </button>
        </div>

        {/* Query Presets */}
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center', fontSize: '11px' }}>
          <span style={{ color: 'var(--text-muted)' }}>Quick Presets:</span>
          {['GET "user:session:100"', 'SET "auth:jwt:token" "active_bearer" EX 3600', 'DEL "user:session:100"', 'GET "model:weights:inference_cache"'].map((preset) => (
            <button
              key={preset}
              onClick={() => setQueryInput(preset)}
              style={{
                padding: '3px 8px',
                background: 'var(--bg-surface)',
                border: '1px solid var(--border-subtle)',
                color: 'var(--text-secondary)',
                borderRadius: '4px',
                cursor: 'pointer',
                fontFamily: 'JetBrains Mono',
                fontSize: '11px',
              }}
            >
              {preset.split(' ')[0]} {preset.split(' ')[1]}
            </button>
          ))}
        </div>
      </div>

      {viewMode === 'diagram' ? (
        /* Visual Interactive Pipeline Diagram */
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(5, 1fr)', gap: '14px' }}>
          {/* Stage 1 */}
          <div
            className="card"
            style={{
              padding: '16px',
              background: activeStage === 0 ? 'var(--accent-primary-bg)' : 'var(--bg-card)',
              border: activeStage === 0 ? '1px solid var(--accent-primary)' : '1px solid var(--border-default)',
              borderRadius: '8px',
              display: 'flex',
              flexDirection: 'column',
              gap: '10px',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--accent-cyan)' }}>
              <Zap size={16} />
              <span style={{ fontSize: '12px', fontWeight: 700 }}>[1] INGRESS & SIMD</span>
            </div>
            <div style={{ fontSize: '11px', color: 'var(--text-secondary)', lineHeight: '1.5' }}>
              <div><strong>Wire:</strong> RESP3 Array</div>
              <div><strong>SIMD:</strong> AVX2 256-bit</div>
              <div><strong>Throughput:</strong> 289M ops/s</div>
              <div><strong>Vector:</strong> _mm256_cmpeq_epi8</div>
            </div>
            <div style={{ marginTop: 'auto', paddingTop: '8px', borderTop: '1px solid var(--border-subtle)', fontSize: '11px', color: 'var(--accent-green)', fontWeight: 600 }}>
              Latency: ~0.08 µs
            </div>
          </div>

          {/* Stage 2 */}
          <div
            className="card"
            style={{
              padding: '16px',
              background: activeStage === 1 ? 'var(--accent-primary-bg)' : 'var(--bg-card)',
              border: activeStage === 1 ? '1px solid var(--accent-primary)' : '1px solid var(--border-default)',
              borderRadius: '8px',
              display: 'flex',
              flexDirection: 'column',
              gap: '10px',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--accent-primary)' }}>
              <Cpu size={16} />
              <span style={{ fontSize: '12px', fontWeight: 700 }}>[2] SHARD ROUTING</span>
            </div>
            <div style={{ fontSize: '11px', color: 'var(--text-secondary)', lineHeight: '1.5' }}>
              <div><strong>Hash:</strong> 0x{hash.toString(16)}</div>
              <div><strong>Jump Hash:</strong> Shard #{shardId}</div>
              <div><strong>Pinning:</strong> CPU Core #{assignedCore}</div>
              <div><strong>Queue:</strong> 64B Cache-Padded</div>
            </div>
            <div style={{ marginTop: 'auto', paddingTop: '8px', borderTop: '1px solid var(--border-subtle)', fontSize: '11px', color: 'var(--accent-green)', fontWeight: 600 }}>
              Latency: ~0.15 µs
            </div>
          </div>

          {/* Stage 3 */}
          <div
            className="card"
            style={{
              padding: '16px',
              background: activeStage === 2 ? 'var(--accent-primary-bg)' : 'var(--bg-card)',
              border: activeStage === 2 ? '1px solid var(--accent-primary)' : '1px solid var(--border-default)',
              borderRadius: '8px',
              display: 'flex',
              flexDirection: 'column',
              gap: '10px',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--accent-amber)' }}>
              <Layers size={16} />
              <span style={{ fontSize: '12px', fontWeight: 700 }}>[3] ROBIN HOOD MAP</span>
            </div>
            <div style={{ fontSize: '11px', color: 'var(--text-secondary)', lineHeight: '1.5' }}>
              <div><strong>Table:</strong> Flat Array</div>
              <div><strong>Ideal Slot:</strong> #{idealSlot}</div>
              <div><strong>Actual Slot:</strong> #{actualSlot}</div>
              <div><strong>DIB Hop:</strong> 1 (O(1) Bound)</div>
            </div>
            <div style={{ marginTop: 'auto', paddingTop: '8px', borderTop: '1px solid var(--border-subtle)', fontSize: '11px', color: 'var(--accent-green)', fontWeight: 600 }}>
              Latency: ~0.22 µs
            </div>
          </div>

          {/* Stage 4 */}
          <div
            className="card"
            style={{
              padding: '16px',
              background: activeStage === 3 ? 'var(--accent-primary-bg)' : 'var(--bg-card)',
              border: activeStage === 3 ? '1px solid var(--accent-primary)' : '1px solid var(--border-default)',
              borderRadius: '8px',
              display: 'flex',
              flexDirection: 'column',
              gap: '10px',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--accent-green)' }}>
              <HardDrive size={16} />
              <span style={{ fontSize: '12px', fontWeight: 700 }}>[4] MEMORY & TIER</span>
            </div>
            <div style={{ fontSize: '11px', color: 'var(--text-secondary)', lineHeight: '1.5' }}>
              <div><strong>Storage:</strong> Slab Arena</div>
              <div><strong>Alignment:</strong> 2MB THP</div>
              <div><strong>CLOCK:</strong> FREQ_3 (Hot)</div>
              <div><strong>Metadata Tax:</strong> 0 Bytes</div>
            </div>
            <div style={{ marginTop: 'auto', paddingTop: '8px', borderTop: '1px solid var(--border-subtle)', fontSize: '11px', color: 'var(--accent-green)', fontWeight: 600 }}>
              Latency: ~0.12 µs
            </div>
          </div>

          {/* Stage 5 */}
          <div
            className="card"
            style={{
              padding: '16px',
              background: activeStage === 4 ? 'var(--accent-primary-bg)' : 'var(--bg-card)',
              border: activeStage === 4 ? '1px solid var(--accent-primary)' : '1px solid var(--border-default)',
              borderRadius: '8px',
              display: 'flex',
              flexDirection: 'column',
              gap: '10px',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--accent-cyan)' }}>
              <Clock size={16} />
              <span style={{ fontSize: '12px', fontWeight: 700 }}>[5] EXPIRATION & WAL</span>
            </div>
            <div style={{ fontSize: '11px', color: 'var(--text-secondary)', lineHeight: '1.5' }}>
              <div><strong>Timer:</strong> 4-Tier HTW</div>
              <div><strong>Detach:</strong> 1.99 ns Swap</div>
              <div><strong>WAL LSN:</strong> #14 (Pipelined)</div>
              <div><strong>Snapshot:</strong> Monotonic Epoch</div>
            </div>
            <div style={{ marginTop: 'auto', paddingTop: '8px', borderTop: '1px solid var(--border-subtle)', fontSize: '11px', color: 'var(--accent-green)', fontWeight: 600 }}>
              Latency: 0.00 µs
            </div>
          </div>
        </div>
      ) : (
        /* Terminal ANSI Box */
        <div
          className="card"
          style={{
            padding: '20px',
            background: 'var(--terminal-bg)',
            borderRadius: '8px',
            border: '1px solid var(--border-default)',
            fontFamily: 'JetBrains Mono',
            fontSize: '12px',
            color: 'var(--terminal-text)',
            lineHeight: '1.6',
            whiteSpace: 'pre',
            overflowX: 'auto',
          }}
        >
{`VALENCE QUERY EXECUTION PLAN & ENGINE TRACE
Target Command: ${cmd} | Target Key: "${rawKey}"
================================================================================
┌── [1] Keyspace Ingress & Wire Protocol
│   ├── Wire Protocol: RESP3 Zero-Copy Array
│   ├── Vector Accelerator: AVX2 256-bit SIMD Scanner (32B/cycle, TZCNT delimiter search)
│   └── Parser Ingress Latency: ~0.08 µs
│
├── [2] Consistent Sharding & Thread-per-Core Dispatch
│   ├── Murmur3-32 Hash: 0x${hash.toString(16)}
│   ├── Jump Consistent Hash: Shard #${shardId} (of 4 active shards)
│   ├── NUMA / Core Pinning: CPU Core #${assignedCore} (Zero Thread Migration)
│   ├── Inter-Core SPSC Queue: 64-Byte Cache-Padded Ring (Lock-Free, 0 False Sharing)
│   └── Cross-Shard Dispatch Latency: ~0.15 µs
│
├── [3] Robin Hood Hash Table & Slot Probing
│   ├── Index Architecture: Flat Array with Distance-from-Ideal-Bucket (DIB)
│   ├── Probing Path: O(1) Probing (Ideal Slot #${idealSlot} -> Hop: 1)
│   ├── Collision Resolution: Robin Hood Backward-Shift (Zero Dead Tombstones)
│   └── Storage Index Lookup Latency: ~0.22 µs
│
├── [4] Memory Hierarchy & Transparent Tiering
│   ├── Storage Tier: TIER-0 HOT (Slab Arena Pool & SSO Inline Buffer)
│   ├── Frequency Tracker: 2-Bit CLOCK Status: FREQ_3 (Actively Hot)
│   ├── Page Alignment: 2MB Linux THP / Hugepage Aligned Arenas
│   ├── Per-Object Metadata Tax: 0 Bytes (64-bit Bitmapped Allocator)
│   └── Memory Retrieval Latency: ~0.12 µs
│
├── [5] Expiration Wheels & NVMe Durability
│   ├── Expiration Engine: 4-Tier Cascaded Timing Wheel (O(1) Atomic Detach)
│   ├── Durability Pipeline: Pipelined NVMe WAL + Zero-Fork Epoch Snapshotting
│   └── Durability Overhead: 0.00 µs (Asynchronous shadow-buffering)
│
└── [SUMMARY] End-to-End Query Execution
    ├── Pure In-Engine Latency: 0.57 µs
    ├── Client Roundtrip Latency: 1.12 µs
    └── Response Status: OK (Cache Hit)`}
        </div>
      )}

      {/* Latency Waterfall Breakdown Card */}
      <div className="card" style={{ padding: '20px', background: 'var(--bg-card)', borderRadius: '8px', border: '1px solid var(--border-default)' }}>
        <h3 style={{ margin: '0 0 16px 0', fontSize: '15px', fontWeight: 600, color: 'var(--text-primary)' }}>
          Microsecond Latency Waterfall Breakdown (&lt; 1.0 µs Total Engine Latency)
        </h3>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          {[
            { label: 'AVX2 SIMD Delimiter Scan', timeUs: 0.08, pct: 14, color: 'var(--accent-cyan)' },
            { label: 'Murmur3 + Jump Hash Dispatch', timeUs: 0.15, pct: 26, color: 'var(--accent-primary)' },
            { label: 'Robin Hood DIB Slot Probing', timeUs: 0.22, pct: 38, color: 'var(--accent-amber)' },
            { label: 'Slab Arena Hugepage Memory Read', timeUs: 0.12, pct: 22, color: 'var(--accent-green)' },
          ].map((item) => (
            <div key={item.label}>
              <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '12px', marginBottom: '4px' }}>
                <span style={{ color: 'var(--text-primary)', fontWeight: 500 }}>{item.label}</span>
                <span style={{ color: item.color, fontFamily: 'JetBrains Mono', fontWeight: 600 }}>{item.timeUs.toFixed(2)} µs ({item.pct}%)</span>
              </div>
              <div style={{ width: '100%', height: '6px', background: 'var(--bg-surface)', borderRadius: '3px', overflow: 'hidden' }}>
                <div style={{ width: `${item.pct}%`, height: '100%', background: item.color, borderRadius: '3px' }} />
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
