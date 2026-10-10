import React, { useState } from 'react';
import {
  HardDrive,
  Cpu,
  Clock,
  RotateCw,
  TrendingDown,
  Zap,
} from 'lucide-react';

interface TieredRecord {
  key: string;
  sizeBytes: number;
  tier: 'HOT' | 'COLD';
  clockFreq: number; // 0..3
  accessCount: number;
  diskPointer?: {
    extentId: number;
    blockOffset: number;
    length: number;
  };
}

export const TieringConsole: React.FC = () => {
  // Simulated tiered dataset
  const [records, setRecords] = useState<TieredRecord[]>([
    {
      key: 'user:session:active_402',
      sizeBytes: 512,
      tier: 'HOT',
      clockFreq: 3,
      accessCount: 142,
    },
    {
      key: 'auth:jwt:token_active',
      sizeBytes: 256,
      tier: 'HOT',
      clockFreq: 3,
      accessCount: 88,
    },
    {
      key: 'model:weights:inference_cache',
      sizeBytes: 1024,
      tier: 'HOT',
      clockFreq: 2,
      accessCount: 34,
    },
    {
      key: 'report:pdf:archive_2025',
      sizeBytes: 4096,
      tier: 'COLD',
      clockFreq: 0,
      accessCount: 0,
      diskPointer: { extentId: 1, blockOffset: 0, length: 4096 },
    },
    {
      key: 'user:session:dormant_891',
      sizeBytes: 512,
      tier: 'COLD',
      clockFreq: 0,
      accessCount: 0,
      diskPointer: { extentId: 1, blockOffset: 4096, length: 512 },
    },
    {
      key: 'telemetry:raw:device_node_03',
      sizeBytes: 2048,
      tier: 'COLD',
      clockFreq: 1,
      accessCount: 1,
      diskPointer: { extentId: 1, blockOffset: 8192, length: 2048 },
    },
  ]);

  const [clockHand, setClockHand] = useState<number>(0);
  const [promotionLog, setPromotionLog] = useState<string[]>([]);
  const [isSweeping, setIsSweeping] = useState<boolean>(false);

  // Compute storage footprints
  const totalLogicalBytes = records.reduce((acc, r) => acc + r.sizeBytes, 0);
  const hotBytes = records
    .filter((r) => r.tier === 'HOT')
    .reduce((acc, r) => acc + r.sizeBytes, 0);
  const coldPointerBytes = records.filter((r) => r.tier === 'COLD').length * 12; // 12-byte DiskPointer
  const actualDramBytes = hotBytes + coldPointerBytes;
  const dramReductionPct =
    totalLogicalBytes > 0
      ? ((1 - actualDramBytes / totalLogicalBytes) * 100).toFixed(1)
      : '0.0';

  // 1. Advance CLOCK sweep hand (decrements frequency, evicts to disk on 0)
  const handleClockSweep = () => {
    setIsSweeping(true);
    setTimeout(() => {
      setRecords((prev) =>
        prev.map((rec, idx) => {
          if (idx === clockHand) {
            if (rec.tier === 'HOT') {
              if (rec.clockFreq > 0) {
                return { ...rec, clockFreq: rec.clockFreq - 1 };
              } else {
                // Evict to NVMe disk!
                return {
                  ...rec,
                  tier: 'COLD',
                  diskPointer: {
                    extentId: 1,
                    blockOffset: Math.floor(Math.random() * 64) * 4096,
                    length: rec.sizeBytes,
                  },
                };
              }
            }
          }
          return rec;
        })
      );
      setClockHand((prev) => (prev + 1) % records.length);
      setIsSweeping(false);
      setPromotionLog((prev) => [
        `[CLOCK SWEEP] Hand advanced to slot #${(clockHand + 1) % records.length}`,
        ...prev.slice(0, 7),
      ]);
    }, 200);
  };

  // 2. Cold read & cache promotion simulator
  const handleQueryKey = (rec: TieredRecord) => {
    if (rec.tier === 'HOT') {
      setPromotionLog((prev) => [
        `[HOT HIT] "${rec.key}" fetched from DRAM Slab Arena (< 500ns)`,
        ...prev.slice(0, 7),
      ]);
    } else {
      // COLD read from NVMe
      const nextCount = rec.accessCount + 1;
      if (nextCount >= 2) {
        // Promoted to HOT!
        setRecords((prev) =>
          prev.map((r) =>
            r.key === rec.key
              ? {
                  ...r,
                  tier: 'HOT',
                  clockFreq: 3,
                  accessCount: nextCount,
                  diskPointer: undefined,
                }
              : r
          )
        );
        setPromotionLog((prev) => [
          `[CACHE PROMOTION] "${rec.key}" promoted from NVMe to HOT DRAM (Threshold met: ${nextCount} reads)`,
          ...prev.slice(0, 7),
        ]);
      } else {
        setRecords((prev) =>
          prev.map((r) =>
            r.key === rec.key ? { ...r, accessCount: nextCount } : r
          )
        );
        setPromotionLog((prev) => [
          `[COLD READ] "${rec.key}" asynchronously read from NVMe (16.6µs) [Access Count: ${nextCount}/2]`,
          ...prev.slice(0, 7),
        ]);
      }
    }
  };

  return (
    <div className="tiering-console" style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Top Banner KPI Cards */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
          gap: '16px',
        }}
      >
        <div className="card" style={{ padding: '18px', background: 'var(--bg-card)', borderRadius: '8px', border: '1px solid var(--border-default)' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontSize: '12px', color: 'var(--text-muted)', fontWeight: 600 }}>DRAM REDUCTION</span>
            <TrendingDown size={18} color="var(--accent-green)" />
          </div>
          <div style={{ fontSize: '28px', fontWeight: 700, color: 'var(--accent-green)', marginTop: '8px' }}>
            {dramReductionPct}%
          </div>
          <span style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>Physical DRAM savings vs 100% in-memory</span>
        </div>

        <div className="card" style={{ padding: '18px', background: 'var(--bg-card)', borderRadius: '8px', border: '1px solid var(--border-default)' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontSize: '12px', color: 'var(--text-muted)', fontWeight: 600 }}>HOT DRAM FOOTPRINT</span>
            <Cpu size={18} color="var(--accent-primary)" />
          </div>
          <div style={{ fontSize: '28px', fontWeight: 700, color: 'var(--text-primary)', marginTop: '8px' }}>
            {(actualDramBytes / 1024).toFixed(2)} KB
          </div>
          <span style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>Logical payload: {(totalLogicalBytes / 1024).toFixed(2)} KB</span>
        </div>

        <div className="card" style={{ padding: '18px', background: 'var(--bg-card)', borderRadius: '8px', border: '1px solid var(--border-default)' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontSize: '12px', color: 'var(--text-muted)', fontWeight: 600 }}>NVMe BLOCK READ LATENCY</span>
            <HardDrive size={18} color="var(--accent-cyan)" />
          </div>
          <div style={{ fontSize: '28px', fontWeight: 700, color: 'var(--accent-cyan)', marginTop: '8px' }}>
            16.62 µs
          </div>
          <span style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>Hot DRAM hit latency: &lt; 0.50 µs</span>
        </div>

        <div className="card" style={{ padding: '18px', background: 'var(--bg-card)', borderRadius: '8px', border: '1px solid var(--border-default)' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontSize: '12px', color: 'var(--text-muted)', fontWeight: 600 }}>DISK POINTER OVERHEAD</span>
            <Zap size={18} color="var(--accent-amber)" />
          </div>
          <div style={{ fontSize: '28px', fontWeight: 700, color: 'var(--accent-amber)', marginTop: '8px' }}>
            12 Bytes
          </div>
          <span style={{ fontSize: '11px', color: 'var(--text-secondary)' }}>Extent ID (4B) + Offset (4B) + Len (4B)</span>
        </div>
      </div>

      {/* Main Grid: 2-Bit CLOCK Visualizer & Tiered Table */}
      <div style={{ display: 'grid', gridTemplateColumns: '1.2fr 1fr', gap: '20px' }}>
        {/* Tiered Keys Table */}
        <div className="card" style={{ padding: '20px', background: 'var(--bg-card)', borderRadius: '8px', border: '1px solid var(--border-default)' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
            <div>
              <h3 style={{ margin: 0, fontSize: '16px', fontWeight: 600, color: 'var(--text-primary)' }}>
                Multi-Tier Storage Keyspace
              </h3>
              <p style={{ margin: 0, fontSize: '12px', color: 'var(--text-secondary)' }}>
                Click 'Query Key' to test cold read and on-demand DRAM cache promotion
              </p>
            </div>
            <button
              onClick={handleClockSweep}
              disabled={isSweeping}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                padding: '8px 14px',
                background: 'var(--accent-primary)',
                color: '#fff',
                border: 'none',
                borderRadius: '6px',
                cursor: 'pointer',
                fontWeight: 600,
                fontSize: '12px',
              }}
            >
              <RotateCw size={13} /> Sweep CLOCK Hand
            </button>
          </div>

          <div style={{ overflowX: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '12px', textAlign: 'left' }}>
              <thead>
                <tr style={{ borderBottom: '1px solid var(--border-default)', color: 'var(--text-muted)' }}>
                  <th style={{ padding: '10px 8px' }}>KEY</th>
                  <th style={{ padding: '10px 8px' }}>TIER</th>
                  <th style={{ padding: '10px 8px' }}>CLOCK FREQ</th>
                  <th style={{ padding: '10px 8px' }}>STORAGE LOCATION</th>
                  <th style={{ padding: '10px 8px' }}>ACTION</th>
                </tr>
              </thead>
              <tbody>
                {records.map((rec, idx) => (
                  <tr
                    key={rec.key}
                    style={{
                      borderBottom: '1px solid var(--border-subtle)',
                      background: idx === clockHand ? 'rgba(234, 88, 12, 0.08)' : 'transparent',
                    }}
                  >
                    <td style={{ padding: '10px 8px', fontFamily: 'JetBrains Mono', color: 'var(--text-primary)' }}>
                      {idx === clockHand && <span style={{ color: 'var(--accent-primary)', marginRight: '6px' }}>👉</span>}
                      {rec.key}
                    </td>
                    <td style={{ padding: '10px 8px' }}>
                      <span
                        style={{
                          padding: '3px 8px',
                          borderRadius: '4px',
                          fontSize: '10px',
                          fontWeight: 700,
                          background: rec.tier === 'HOT' ? 'var(--accent-green-bg)' : 'rgba(56, 189, 248, 0.15)',
                          color: rec.tier === 'HOT' ? 'var(--accent-green)' : 'var(--accent-cyan)',
                          border: `1px solid ${rec.tier === 'HOT' ? 'var(--accent-green)' : 'var(--accent-cyan)'}`,
                        }}
                      >
                        {rec.tier}
                      </span>
                    </td>
                    <td style={{ padding: '10px 8px' }}>
                      <div style={{ display: 'flex', gap: '4px' }}>
                        {[0, 1, 2, 3].map((f) => (
                          <div
                            key={f}
                            style={{
                              width: '10px',
                              height: '10px',
                              borderRadius: '2px',
                              background: f <= rec.clockFreq ? 'var(--accent-primary)' : 'var(--border-default)',
                            }}
                          />
                        ))}
                      </div>
                    </td>
                    <td style={{ padding: '10px 8px', color: 'var(--text-secondary)' }}>
                      {rec.tier === 'HOT' ? (
                        <span style={{ color: 'var(--accent-green)' }}>DRAM Slab Arena (2MB)</span>
                      ) : (
                        <span style={{ color: 'var(--accent-cyan)' }}>
                          NVMe Extent #{rec.diskPointer?.extentId} (+{rec.diskPointer?.blockOffset}B)
                        </span>
                      )}
                    </td>
                    <td style={{ padding: '10px 8px' }}>
                      <button
                        onClick={() => handleQueryKey(rec)}
                        style={{
                          padding: '4px 10px',
                          background: 'var(--bg-elevated)',
                          border: '1px solid var(--border-default)',
                          color: 'var(--text-primary)',
                          borderRadius: '4px',
                          cursor: 'pointer',
                          fontSize: '11px',
                        }}
                      >
                        Query Key
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* 2-Bit CLOCK Visual Sweep Ring & Event Stream */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          <div className="card" style={{ padding: '20px', background: 'var(--bg-card)', borderRadius: '8px', border: '1px solid var(--border-default)' }}>
            <h3 style={{ margin: '0 0 12px 0', fontSize: '15px', fontWeight: 600, color: 'var(--text-primary)' }}>
              2-Bit CLOCK Frequency State
            </h3>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '10px' }}>
              {records.map((r, i) => (
                <div
                  key={r.key}
                  style={{
                    padding: '10px',
                    borderRadius: '6px',
                    background: i === clockHand ? 'var(--accent-primary-bg)' : 'var(--bg-surface)',
                    border: i === clockHand ? '1px solid var(--accent-primary)' : '1px solid var(--border-subtle)',
                    textAlign: 'center',
                  }}
                >
                  <div style={{ fontSize: '10px', color: 'var(--text-muted)' }}>SLOT #{i}</div>
                  <div style={{ fontSize: '14px', fontWeight: 700, color: r.tier === 'HOT' ? 'var(--accent-green)' : 'var(--accent-cyan)', margin: '4px 0' }}>
                    {r.tier} (F:{r.clockFreq})
                  </div>
                  <div style={{ fontSize: '10px', color: 'var(--text-secondary)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {r.key.split(':').pop()}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Promotion & Tiering Live Log */}
          <div className="card" style={{ padding: '16px', background: 'var(--terminal-bg)', borderRadius: '8px', border: '1px solid var(--border-default)', flex: 1 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px', color: 'var(--accent-cyan)', fontSize: '12px', fontWeight: 600 }}>
              <Clock size={13} /> TIERING ENGINE EVENT STREAM
            </div>
            <div style={{ fontFamily: 'JetBrains Mono', fontSize: '11px', display: 'flex', flexDirection: 'column', gap: '6px' }}>
              {promotionLog.length === 0 ? (
                <span style={{ color: 'var(--text-dim)' }}>No events yet. Click 'Sweep CLOCK Hand' or 'Query Key'.</span>
              ) : (
                promotionLog.map((log, idx) => (
                  <div key={idx} style={{ color: log.includes('PROMOTION') ? 'var(--accent-green)' : log.includes('COLD') ? 'var(--accent-amber)' : 'var(--text-secondary)' }}>
                    {log}
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
