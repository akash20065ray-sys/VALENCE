import React, { useState } from 'react';
import {
  FileText,
  ShieldCheck,
  Zap,
  RotateCw,
  AlertTriangle,
  Play,
  CheckCircle2,
  HardDrive,
  RefreshCw,
  Search,
} from 'lucide-react';
import type { WalState, WalRecordView, WalSegmentView, WalRecoveryReportView } from '../types';

interface DurabilityConsoleProps {
  walState: WalState;
  walRecords: WalRecordView[];
  walSegments: WalSegmentView[];
  lastRecoveryReport: WalRecoveryReportView | null;
  onChangeSyncPolicy: (policy: 'Always' | 'GroupCommit' | 'Periodic') => void;
  onSimulateCrashAndReplay: (simulateTornWrite: boolean) => void;
  onTriggerCheckpoint: () => void;
}

export const DurabilityConsole: React.FC<DurabilityConsoleProps> = ({
  walState,
  walRecords,
  walSegments,
  lastRecoveryReport,
  onChangeSyncPolicy,
  onSimulateCrashAndReplay,
  onTriggerCheckpoint,
}) => {
  const [filterOp, setFilterOp] = useState<string>('ALL');
  const [searchKey, setSearchKey] = useState<string>('');
  const [includeTornWrite, setIncludeTornWrite] = useState<boolean>(true);
  const [isSimulating, setIsSimulating] = useState<boolean>(false);

  const filteredRecords = walRecords.filter((rec) => {
    if (filterOp !== 'ALL' && rec.op !== filterOp) return false;
    if (searchKey && !rec.key.toLowerCase().includes(searchKey.toLowerCase())) return false;
    return true;
  });

  const handleRunCrashSimulation = () => {
    setIsSimulating(true);
    setTimeout(() => {
      onSimulateCrashAndReplay(includeTornWrite);
      setIsSimulating(false);
    }, 600);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Top Durability Overview Cards */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
          gap: '14px',
        }}
      >
        {/* Card 1: Active LSN */}
        <div className="card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-muted)', fontSize: '11.5px' }}>
            <FileText size={14} color="var(--accent-cyan)" />
            <span>CURRENT LOG SEQUENCE NUMBER</span>
          </div>
          <div style={{ fontSize: '22px', fontWeight: 700, marginTop: '6px', color: 'var(--accent-cyan)', fontFamily: 'JetBrains Mono, monospace' }}>
            LSN #{walState.currentLsn}
          </div>
          <div style={{ fontSize: '11px', color: 'var(--text-secondary)', marginTop: '4px' }}>
            Monotonic 64-bit atomic sequencing
          </div>
        </div>

        {/* Card 2: Durability Policy */}
        <div className="card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-muted)', fontSize: '11.5px' }}>
            <Zap size={14} color="var(--accent-primary)" />
            <span>SYNC POLICY & PIPELINE</span>
          </div>
          <div style={{ fontSize: '20px', fontWeight: 700, marginTop: '6px', color: 'var(--text-primary)' }}>
            {walState.syncPolicy}
          </div>
          <div style={{ fontSize: '11px', color: 'var(--accent-green)', marginTop: '4px', display: 'flex', alignItems: 'center', gap: '4px' }}>
            <CheckCircle2 size={12} />
            <span>{walState.avgFlushLatencyUs} µs Avg Fsync Latency</span>
          </div>
        </div>

        {/* Card 3: Active Segment */}
        <div className="card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-muted)', fontSize: '11.5px' }}>
            <HardDrive size={14} color="var(--accent-amber)" />
            <span>ACTIVE 64MB WAL SEGMENT</span>
          </div>
          <div style={{ fontSize: '13.5px', fontWeight: 700, marginTop: '8px', color: 'var(--text-primary)', fontFamily: 'JetBrains Mono, monospace' }}>
            {walState.activeSegment}
          </div>
          <div style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '4px' }}>
            Fixed 64MB Boundary &bull; Magic `VALENCE_WAL_V1`
          </div>
        </div>

        {/* Card 4: Checksum & Framing */}
        <div className="card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', color: 'var(--text-muted)', fontSize: '11.5px' }}>
            <ShieldCheck size={14} color="var(--accent-green)" />
            <span>FRAME INTEGRITY VERIFICATION</span>
          </div>
          <div style={{ fontSize: '18px', fontWeight: 700, marginTop: '6px', color: 'var(--text-primary)', fontFamily: 'JetBrains Mono, monospace' }}>
            ECMA-182 CRC64
          </div>
          <div style={{ fontSize: '11px', color: 'var(--text-secondary)', marginTop: '4px' }}>
            Multi-GB/s Hardware Sliced Tables
          </div>
        </div>
      </div>

      {/* Sync Policy Switcher & Actions Strip */}
      <div
        className="card"
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: '14px',
          background: 'var(--bg-surface)',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <span style={{ fontSize: '12px', fontWeight: 600, color: 'var(--text-secondary)' }}>
            DURABILITY MODE:
          </span>
          <div style={{ display: 'flex', background: 'var(--bg-elevated)', borderRadius: '6px', padding: '3px', border: '1px solid var(--border-subtle)' }}>
            {(['GroupCommit', 'Always', 'Periodic'] as const).map((policy) => (
              <button
                key={policy}
                onClick={() => onChangeSyncPolicy(policy)}
                style={{
                  background: walState.syncPolicy === policy ? 'var(--accent-primary)' : 'transparent',
                  color: walState.syncPolicy === policy ? '#FFF' : 'var(--text-muted)',
                  border: 'none',
                  padding: '5px 12px',
                  borderRadius: '4px',
                  fontSize: '11.5px',
                  fontWeight: 600,
                  cursor: 'pointer',
                  transition: 'all 0.15s ease',
                }}
              >
                {policy === 'GroupCommit' && 'Group Commit (Batched)'}
                {policy === 'Always' && 'Always (Per-Write Fsync)'}
                {policy === 'Periodic' && 'Periodic (OS Writeback)'}
              </button>
            ))}
          </div>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
          <button
            onClick={onTriggerCheckpoint}
            className="action-btn"
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '6px 14px',
              fontSize: '11.5px',
              background: 'var(--bg-elevated)',
              color: 'var(--accent-cyan)',
              border: '1px solid var(--border-default)',
              borderRadius: '5px',
              cursor: 'pointer',
            }}
          >
            <RotateCw size={12} />
            <span>Flush Checkpoint & Prune</span>
          </button>
        </div>
      </div>

      {/* Crash Simulation & Recovery Scanner Banner */}
      <div
        className="card"
        style={{
          borderLeft: '4px solid var(--accent-amber)',
          background: 'var(--bg-surface)',
          display: 'flex',
          flexDirection: 'column',
          gap: '14px',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '12px' }}>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <AlertTriangle size={16} color="var(--accent-amber)" />
              <span style={{ fontSize: '13.5px', fontWeight: 700, color: 'var(--text-primary)' }}>
                Crash Recovery & Tail-Truncation Simulator
              </span>
            </div>
            <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginTop: '4px' }}>
              Simulates immediate SIGKILL / power disruption, parses on-disk binary segments, isolates corrupt/truncated frames, and replays valid state.
            </div>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
            <label style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '11.5px', color: 'var(--text-secondary)', cursor: 'pointer' }}>
              <input
                type="checkbox"
                checked={includeTornWrite}
                onChange={(e) => setIncludeTornWrite(e.target.checked)}
                style={{ cursor: 'pointer' }}
              />
              <span>Inject Torn Tail Write (Partial Frame)</span>
            </label>

            <button
              onClick={handleRunCrashSimulation}
              disabled={isSimulating}
              className="action-btn"
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
                padding: '7px 16px',
                fontSize: '12px',
                fontWeight: 600,
                background: 'var(--accent-amber)',
                color: '#12151B',
                border: 'none',
                borderRadius: '5px',
                cursor: isSimulating ? 'wait' : 'pointer',
              }}
            >
              {isSimulating ? <RefreshCw size={13} className="spin" /> : <Play size={13} />}
              <span>{isSimulating ? 'Scanning & Replaying...' : 'Simulate Crash & Replay'}</span>
            </button>
          </div>
        </div>

        {/* Recovery Report Display */}
        {lastRecoveryReport && (
          <div
            style={{
              background: 'var(--bg-elevated)',
              padding: '12px 16px',
              borderRadius: '6px',
              border: '1px solid var(--border-subtle)',
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))',
              gap: '12px',
            }}
          >
            <div>
              <div style={{ fontSize: '10.5px', color: 'var(--text-muted)' }}>REPLAY STATUS</div>
              <div style={{ fontSize: '14px', fontWeight: 700, color: 'var(--accent-green)', marginTop: '2px', display: 'flex', alignItems: 'center', gap: '4px' }}>
                <CheckCircle2 size={13} /> 100% Consistent
              </div>
            </div>

            <div>
              <div style={{ fontSize: '10.5px', color: 'var(--text-muted)' }}>RECORDS REPLAYED</div>
              <div style={{ fontSize: '14px', fontWeight: 700, color: 'var(--text-primary)', marginTop: '2px', fontFamily: 'JetBrains Mono, monospace' }}>
                {lastRecoveryReport.recordsReplayed} Mutations
              </div>
            </div>

            <div>
              <div style={{ fontSize: '10.5px', color: 'var(--text-muted)' }}>RECOVERED MAX LSN</div>
              <div style={{ fontSize: '14px', fontWeight: 700, color: 'var(--accent-cyan)', marginTop: '2px', fontFamily: 'JetBrains Mono, monospace' }}>
                LSN #{lastRecoveryReport.maxLsn}
              </div>
            </div>

            <div>
              <div style={{ fontSize: '10.5px', color: 'var(--text-muted)' }}>TORN WRITE TRUNCATED</div>
              <div style={{ fontSize: '14px', fontWeight: 700, color: lastRecoveryReport.truncatedBytes > 0 ? 'var(--accent-amber)' : 'var(--text-muted)', marginTop: '2px', fontFamily: 'JetBrains Mono, monospace' }}>
                {lastRecoveryReport.truncatedBytes} Bytes Discarded
              </div>
            </div>

            <div>
              <div style={{ fontSize: '10.5px', color: 'var(--text-muted)' }}>SCAN & REPLAY DURATION</div>
              <div style={{ fontSize: '14px', fontWeight: 700, color: 'var(--accent-primary)', marginTop: '2px', fontFamily: 'JetBrains Mono, monospace' }}>
                {lastRecoveryReport.elapsedMs.toFixed(2)} ms
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Segment File Layout Strip */}
      <div className="card">
        <div className="card-header">
          <div className="card-title">
            <HardDrive size={15} color="var(--accent-primary)" />
            <span>On-Disk WAL Segment Catalog</span>
          </div>
          <span style={{ fontSize: '11px', color: 'var(--text-muted)', fontFamily: 'JetBrains Mono, monospace' }}>
            Directory: ./data/wal/ &bull; 64MB Boundary
          </span>
        </div>

        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
            gap: '12px',
          }}
        >
          {walSegments.map((seg) => {
            const usagePct = Math.min(100, Math.round((seg.sizeBytes / seg.maxSizeBytes) * 100));
            return (
              <div
                key={seg.id}
                style={{
                  background: 'var(--bg-elevated)',
                  padding: '12px 14px',
                  borderRadius: '6px',
                  border: '1px solid var(--border-subtle)',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                  <span style={{ fontSize: '12.5px', fontWeight: 700, color: 'var(--text-primary)', fontFamily: 'JetBrains Mono, monospace' }}>
                    {seg.filename}
                  </span>
                  <span
                    style={{
                      fontSize: '10px',
                      fontWeight: 700,
                      padding: '2px 6px',
                      borderRadius: '4px',
                      background: seg.status === 'active' ? 'var(--accent-green-bg)' : 'var(--bg-surface)',
                      color: seg.status === 'active' ? 'var(--accent-green)' : 'var(--text-muted)',
                      textTransform: 'uppercase',
                    }}
                  >
                    {seg.status}
                  </span>
                </div>

                <div style={{ marginTop: '8px', fontSize: '11px', color: 'var(--text-secondary)', display: 'flex', justifyContent: 'space-between' }}>
                  <span>LSN Range: #{seg.startLsn} &rarr; #{seg.endLsn}</span>
                  <span>{(seg.sizeBytes / 1024).toFixed(1)} KB / 64MB</span>
                </div>

                <div style={{ width: '100%', height: '5px', background: 'var(--bg-input)', borderRadius: '3px', marginTop: '6px', overflow: 'hidden' }}>
                  <div
                    style={{
                      width: `${Math.max(2, usagePct)}%`,
                      height: '100%',
                      background: seg.status === 'active' ? 'var(--accent-cyan)' : 'var(--text-muted)',
                      borderRadius: '3px',
                    }}
                  />
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Live Binary WAL Frame Stream Table */}
      <div className="card">
        <div className="card-header">
          <div className="card-title">
            <FileText size={15} color="var(--accent-cyan)" />
            <span>Monotonic Binary WAL Stream (Live Appends)</span>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <div style={{ display: 'flex', alignItems: 'center', background: 'var(--bg-input)', padding: '4px 8px', borderRadius: '4px', border: '1px solid var(--border-subtle)' }}>
              <Search size={12} color="var(--text-muted)" />
              <input
                type="text"
                placeholder="Filter by key..."
                value={searchKey}
                onChange={(e) => setSearchKey(e.target.value)}
                style={{ background: 'transparent', border: 'none', color: 'var(--text-primary)', fontSize: '11px', marginLeft: '6px', outline: 'none', width: '110px' }}
              />
            </div>

            <div style={{ display: 'flex', background: 'var(--bg-input)', borderRadius: '4px', border: '1px solid var(--border-subtle)' }}>
              {['ALL', 'SET', 'DEL', 'EXPIRE', 'CHECKPOINT'].map((op) => (
                <button
                  key={op}
                  onClick={() => setFilterOp(op)}
                  style={{
                    background: filterOp === op ? 'var(--accent-primary)' : 'transparent',
                    color: filterOp === op ? '#FFF' : 'var(--text-muted)',
                    border: 'none',
                    padding: '3px 8px',
                    fontSize: '10.5px',
                    cursor: 'pointer',
                    borderRadius: '3px',
                  }}
                >
                  {op}
                </button>
              ))}
            </div>
          </div>
        </div>

        {/* Records Table */}
        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '11.5px', textAlign: 'left' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid var(--border-subtle)', color: 'var(--text-muted)', fontSize: '10.5px' }}>
                <th style={{ padding: '8px 10px' }}>LSN</th>
                <th style={{ padding: '8px 10px' }}>EPOCH</th>
                <th style={{ padding: '8px 10px' }}>OPCODE</th>
                <th style={{ padding: '8px 10px' }}>TARGET KEY</th>
                <th style={{ padding: '8px 10px' }}>PAYLOAD PREVIEW</th>
                <th style={{ padding: '8px 10px' }}>SIZE</th>
                <th style={{ padding: '8px 10px' }}>ECMA-182 CRC64</th>
                <th style={{ padding: '8px 10px' }}>COMMITTED AT</th>
              </tr>
            </thead>
            <tbody>
              {filteredRecords.map((rec) => {
                const opColor =
                  rec.op === 'SET'
                    ? 'var(--accent-green)'
                    : rec.op === 'DEL'
                    ? 'var(--accent-red)'
                    : rec.op === 'EXPIRE'
                    ? 'var(--accent-amber)'
                    : 'var(--accent-cyan)';

                return (
                  <tr
                    key={rec.lsn}
                    style={{
                      borderBottom: '1px solid var(--border-subtle)',
                      fontFamily: 'JetBrains Mono, monospace',
                    }}
                  >
                    <td style={{ padding: '8px 10px', color: 'var(--accent-cyan)', fontWeight: 600 }}>
                      #{rec.lsn}
                    </td>
                    <td style={{ padding: '8px 10px', color: 'var(--text-muted)' }}>
                      #{rec.epoch}
                    </td>
                    <td style={{ padding: '8px 10px' }}>
                      <span
                        style={{
                          color: opColor,
                          fontWeight: 700,
                          fontSize: '10.5px',
                          background: `${opColor}22`,
                          padding: '2px 6px',
                          borderRadius: '3px',
                        }}
                      >
                        {rec.op}
                      </span>
                    </td>
                    <td style={{ padding: '8px 10px', color: 'var(--text-primary)', fontWeight: 500 }}>
                      {rec.key}
                    </td>
                    <td style={{ padding: '8px 10px', color: 'var(--text-secondary)', maxWidth: '240px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                      {rec.valPreview}
                    </td>
                    <td style={{ padding: '8px 10px', color: 'var(--text-muted)' }}>
                      {rec.sizeBytes} B
                    </td>
                    <td style={{ padding: '8px 10px', color: 'var(--accent-cyan)', fontSize: '11px' }}>
                      {rec.crc64Hex}
                    </td>
                    <td style={{ padding: '8px 10px', color: 'var(--text-dim)', fontSize: '10.5px' }}>
                      {rec.timestamp}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
