import React, { useState, useMemo } from 'react';
import {
  Activity,
  Search,
  Filter,
  CheckCircle,
  AlertTriangle,
  Zap,
  BarChart3,
  Clock,
  Trash2,
} from 'lucide-react';
import type { CommandLog, KeyspaceItem, WorkloadMode } from '../types';

interface ActivityProfilerProps {
  logs: CommandLog[];
  keyspace: KeyspaceItem[];
  onClearLogs: () => void;
  workloadMode: WorkloadMode;
  opsPerSec: number;
  onSetWorkloadMode: (mode: WorkloadMode) => void;
}

export const ActivityProfiler: React.FC<ActivityProfilerProps> = ({
  logs,
  keyspace,
  onClearLogs,
  workloadMode,
  opsPerSec,
  onSetWorkloadMode,
}) => {
  const [filterVerb, setFilterVerb] = useState<string>('ALL');
  const [searchQuery, setSearchQuery] = useState('');

  // Calculate Namespace Breakdown directly from keyspace
  const namespaceStats = useMemo(() => {
    const stats: Record<string, number> = {};
    keyspace.forEach((item) => {
      const parts = item.key.split(':');
      const ns = parts.length > 1 ? parts[0] + ':*' : 'general';
      stats[ns] = (stats[ns] || 0) + 1;
    });
    return Object.entries(stats).map(([namespace, count]) => ({
      namespace,
      count,
      pct: keyspace.length > 0 ? Math.round((count / keyspace.length) * 100) : 0,
    }));
  }, [keyspace]);

  // Filter command logs
  const filteredLogs = useMemo(() => {
    return logs.filter((log) => {
      const verb = log.command.split(' ')[0].toUpperCase();
      if (filterVerb !== 'ALL' && verb !== filterVerb) return false;
      if (searchQuery && !log.command.toLowerCase().includes(searchQuery.toLowerCase())) return false;
      return true;
    });
  }, [logs, filterVerb, searchQuery]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Operational Status & Workload Injector Banner */}
      <div
        className="card"
        style={{
          background: 'var(--bg-card)',
          borderLeft: `4px solid ${workloadMode === 'idle' ? 'var(--accent-primary)' : 'var(--accent-green)'}`,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: '14px',
        }}
      >
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span
              className="health-dot"
              style={{
                background: workloadMode === 'idle' ? 'var(--text-muted)' : 'var(--accent-green)',
                boxShadow: workloadMode === 'idle' ? 'none' : '0 0 8px var(--accent-green)',
              }}
            />
            <span style={{ fontSize: '13.5px', fontWeight: 700, color: 'var(--text-primary)' }}>
              {workloadMode === 'idle'
                ? 'LIVE PROFILER • STANDBY MODE'
                : `LIVE INGESTION PROFILER • ${workloadMode.toUpperCase()} BENCHMARK ACTIVE`}
            </span>
          </div>
          <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginTop: '4px' }}>
            Ingestion rate:{' '}
            <strong style={{ color: workloadMode === 'idle' ? 'var(--text-muted)' : 'var(--accent-green)' }}>
              {opsPerSec.toLocaleString()} ops/sec ({workloadMode === 'idle' ? 'Idle baseline' : 'Active workload'})
            </strong>{' '}
            &bull; Client connections: <strong style={{ color: 'var(--text-primary)' }}>{workloadMode === 'idle' ? '1 (Studio Console)' : '16 Pipelined Sessions'}</strong>
          </div>
        </div>

        {/* Traffic Generator & Latency Percentiles */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
          <span style={{ fontSize: '11px', color: 'var(--text-muted)', fontWeight: 600, letterSpacing: '0.5px' }}>
            LOAD GENERATOR:
          </span>
          <div style={{ display: 'flex', background: 'var(--bg-elevated)', borderRadius: '6px', padding: '3px', border: '1px solid var(--border-subtle)' }}>
            {(
              [
                { id: 'idle', label: 'Idle' },
                { id: 'oltp', label: 'OLTP (1.2k ops/s)' },
                { id: 'stress', label: 'Stress (8.5k ops/s)' },
              ] as const
            ).map((w) => (
              <button
                key={w.id}
                onClick={() => onSetWorkloadMode(w.id)}
                style={{
                  background: workloadMode === w.id ? 'var(--accent-primary)' : 'transparent',
                  color: workloadMode === w.id ? '#FFF' : 'var(--text-muted)',
                  border: 'none',
                  padding: '4px 10px',
                  borderRadius: '4px',
                  fontSize: '11px',
                  fontWeight: 600,
                  cursor: 'pointer',
                  transition: 'all 0.15s ease',
                }}
              >
                {w.label}
              </button>
            ))}
          </div>

          <div className="stat-pill">
            <CheckCircle size={12} color="var(--accent-green)" />
            <span>HIT RATE:</span>
            <span className="val" style={{ color: 'var(--accent-green)' }}>
              {workloadMode === 'idle' ? '100%' : '99.4%'}
            </span>
          </div>

          <div className="stat-pill">
            <Clock size={12} color="var(--accent-primary)" />
            <span>P50:</span>
            <span className="val" style={{ color: 'var(--accent-green)' }}>
              {workloadMode === 'idle' ? '3.4 µs' : workloadMode === 'oltp' ? '2.8 µs' : '4.6 µs'}
            </span>
          </div>
          <div className="stat-pill">
            <Zap size={12} color="var(--accent-cyan)" />
            <span>P99:</span>
            <span className="val" style={{ color: 'var(--accent-cyan)' }}>
              {workloadMode === 'idle' ? '8.2 µs' : workloadMode === 'oltp' ? '6.9 µs' : '11.8 µs'}
            </span>
          </div>
        </div>
      </div>

      {/* Namespace Distribution Analysis */}
      <div className="card">
        <div className="card-header">
          <div className="card-title">
            <BarChart3 size={15} color="var(--accent-cyan)" />
            <span>State Namespace Distribution</span>
          </div>
          <span style={{ fontSize: '11.5px', color: 'var(--text-muted)', fontFamily: 'JetBrains Mono, monospace' }}>
            {keyspace.length} Keys Partitioned Across {namespaceStats.length} Domains
          </span>
        </div>

        {/* Proportional distribution bar */}
        <div style={{ height: '8px', background: 'var(--bg-elevated)', borderRadius: '4px', overflow: 'hidden', display: 'flex', marginBottom: '14px' }}>
          {namespaceStats.map((item, idx) => {
            const colors = [
              'var(--accent-primary)',
              'var(--accent-cyan)',
              'var(--accent-green)',
              'var(--accent-amber)',
              '#A855F7',
              '#EC4899',
            ];
            const color = colors[idx % colors.length];
            return (
              <div
                key={item.namespace}
                style={{
                  width: `${item.pct}%`,
                  background: color,
                  height: '100%',
                }}
                title={`${item.namespace}: ${item.count} keys (${item.pct}%)`}
              />
            );
          })}
        </div>

        {/* Namespace List */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
            gap: '10px',
          }}
        >
          {namespaceStats.map((item, idx) => {
            const colors = [
              'var(--accent-primary)',
              'var(--accent-cyan)',
              'var(--accent-green)',
              'var(--accent-amber)',
              '#A855F7',
              '#EC4899',
            ];
            const color = colors[idx % colors.length];
            return (
              <div
                key={item.namespace}
                style={{
                  background: 'var(--bg-surface)',
                  padding: '8px 12px',
                  borderRadius: '6px',
                  border: '1px solid var(--border-subtle)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <div style={{ width: '8px', height: '8px', borderRadius: '2px', background: color }} />
                  <span style={{ fontFamily: 'JetBrains Mono, monospace', fontSize: '12px', fontWeight: 600, color: 'var(--text-primary)' }}>
                    {item.namespace}
                  </span>
                </div>
                <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
                  <strong>{item.count}</strong> ({item.pct}%)
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Live Operations Stream Table */}
      <div className="card">
        <div className="card-header">
          <div className="card-title">
            <Activity size={15} color="var(--accent-primary)" />
            <span>Command Stream & Execution Audit ({filteredLogs.length} events)</span>
          </div>

          <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
            {logs.length > 0 && (
              <button className="btn-secondary" onClick={onClearLogs} style={{ padding: '4px 8px', fontSize: '11.5px' }}>
                <Trash2 size={12} /> Clear Stream
              </button>
            )}
          </div>
        </div>

        {/* Filters */}
        <div style={{ display: 'flex', gap: '10px', marginBottom: '14px', flexWrap: 'wrap' }}>
          <div style={{ position: 'relative', flex: '1', minWidth: '220px' }}>
            <Search size={13} color="var(--text-dim)" style={{ position: 'absolute', left: '10px', top: '9px' }} />
            <input
              type="text"
              placeholder="Search command stream..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="input-field"
              style={{ paddingLeft: '30px' }}
            />
          </div>

          <div style={{ display: 'flex', gap: '4px', alignItems: 'center' }}>
            <Filter size={12} color="var(--text-muted)" />
            {['ALL', 'GET', 'SET', 'DEL', 'PING', 'MGET', 'INFO'].map((verb) => (
              <button
                key={verb}
                onClick={() => setFilterVerb(verb)}
                style={{
                  background: filterVerb === verb ? 'var(--accent-primary-bg)' : 'var(--bg-elevated)',
                  color: filterVerb === verb ? 'var(--accent-primary)' : 'var(--text-secondary)',
                  border: filterVerb === verb ? '1px solid var(--accent-primary)' : '1px solid var(--border-subtle)',
                  padding: '3px 8px',
                  borderRadius: '4px',
                  fontSize: '11px',
                  fontWeight: 500,
                  cursor: 'pointer',
                  fontFamily: 'JetBrains Mono, monospace',
                }}
              >
                {verb}
              </button>
            ))}
          </div>
        </div>

        {/* Stream Table */}
        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontFamily: 'JetBrains Mono, monospace', fontSize: '12px' }}>
            <thead>
              <tr style={{ background: 'var(--bg-surface)', textAlign: 'left', borderBottom: '1px solid var(--border-default)' }}>
                <th style={{ padding: '8px 12px', color: 'var(--text-muted)', fontWeight: 600, fontSize: '11px' }}>TIMESTAMP</th>
                <th style={{ padding: '8px 12px', color: 'var(--text-muted)', fontWeight: 600, fontSize: '11px' }}>COMMAND VERB</th>
                <th style={{ padding: '8px 12px', color: 'var(--text-muted)', fontWeight: 600, fontSize: '11px' }}>STATEMENT / ARGS</th>
                <th style={{ padding: '8px 12px', color: 'var(--text-muted)', fontWeight: 600, fontSize: '11px' }}>DURATION</th>
                <th style={{ padding: '8px 12px', color: 'var(--text-muted)', fontWeight: 600, fontSize: '11px' }}>STATUS</th>
              </tr>
            </thead>
            <tbody>
              {filteredLogs.length === 0 ? (
                <tr>
                  <td colSpan={5} style={{ padding: '24px', textAlign: 'center', color: 'var(--text-muted)' }}>
                    No recorded operations matching filter. Execute queries in the RESP Workbench to see live profiling.
                  </td>
                </tr>
              ) : (
                filteredLogs.slice(-20).reverse().map((log) => {
                  const verb = log.command.split(' ')[0].toUpperCase();
                  return (
                    <tr
                      key={log.id}
                      style={{
                        borderBottom: '1px solid var(--border-subtle)',
                        transition: 'background-color 0.1s ease',
                      }}
                    >
                      <td style={{ padding: '8px 12px', color: 'var(--text-dim)', whiteSpace: 'nowrap' }}>
                        {log.timestamp}
                      </td>
                      <td style={{ padding: '8px 12px' }}>
                        <span
                          style={{
                            background: 'var(--bg-elevated)',
                            color: 'var(--accent-primary)',
                            padding: '2px 6px',
                            borderRadius: '3px',
                            fontWeight: 700,
                          }}
                        >
                          {verb}
                        </span>
                      </td>
                      <td style={{ padding: '8px 12px', color: 'var(--text-primary)', wordBreak: 'break-all' }}>
                        {log.command}
                      </td>
                      <td style={{ padding: '8px 12px', color: 'var(--accent-green)', whiteSpace: 'nowrap' }}>
                        {log.durationUs.toFixed(1)} µs
                      </td>
                      <td style={{ padding: '8px 12px', whiteSpace: 'nowrap' }}>
                        {log.isError ? (
                          <span style={{ color: 'var(--accent-red)', display: 'inline-flex', alignItems: 'center', gap: '4px' }}>
                            <AlertTriangle size={12} /> ERR
                          </span>
                        ) : (
                          <span style={{ color: 'var(--accent-green)', display: 'inline-flex', alignItems: 'center', gap: '4px' }}>
                            <CheckCircle size={12} /> OK
                          </span>
                        )}
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
