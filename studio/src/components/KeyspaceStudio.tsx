import React, { useState } from 'react';
import { Database, Search, Plus, Trash2, ShieldCheck, HardDrive, Clock } from 'lucide-react';
import type { KeyspaceItem } from '../types';

interface KeyspaceStudioProps {
  items: KeyspaceItem[];
  onSetKey: (key: string, value: string, ttlMs: number | null) => void;
  onDeleteKey: (key: string) => void;
}

export const KeyspaceStudio: React.FC<KeyspaceStudioProps> = ({ items, onSetKey, onDeleteKey }) => {
  const [search, setSearch] = useState('');
  const [newKey, setNewKey] = useState('');
  const [newValue, setNewValue] = useState('');
  const [newTtl, setNewTtl] = useState<number | ''>('');
  const [showAddForm, setShowAddForm] = useState(false);

  const filteredItems = items.filter(
    (item) => item.key.toLowerCase().includes(search.toLowerCase()) || item.value.toLowerCase().includes(search.toLowerCase())
  );

  const handleAddSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newKey) return;
    onSetKey(newKey, newValue, newTtl === '' ? null : Number(newTtl));
    setNewKey('');
    setNewValue('');
    setNewTtl('');
    setShowAddForm(false);
  };

  const ssoCount = items.filter((i) => i.isSso).length;
  const ssoPercent = items.length > 0 ? Math.round((ssoCount / items.length) * 100) : 0;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Telemetry Strip */}
      <div className="grid-3">
        <div className="card">
          <div className="card-subtitle">TOTAL STORED KEYS</div>
          <div style={{ fontSize: '24px', fontWeight: 700, marginTop: '4px', color: 'var(--text-primary)' }}>
            {items.length.toLocaleString()} <span style={{ fontSize: '13px', color: 'var(--text-muted)' }}>Entries</span>
          </div>
          <div style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '4px' }}>
            Contiguous Robin Hood Flat Array
          </div>
        </div>

        <div className="card">
          <div className="card-subtitle">SMALL STRING OPTIMIZATION (SSO)</div>
          <div style={{ fontSize: '24px', fontWeight: 700, marginTop: '4px', color: 'var(--accent-green)' }}>
            {ssoPercent}% <span style={{ fontSize: '13px', color: 'var(--text-muted)' }}>({ssoCount} Keys)</span>
          </div>
          <div style={{ fontSize: '11px', color: 'var(--accent-green)', marginTop: '4px', display: 'flex', alignItems: 'center', gap: '4px' }}>
            <ShieldCheck size={13} /> Zero Heap Allocation (Inlined in 64B Slot)
          </div>
        </div>

        <div className="card">
          <div className="card-subtitle">MAX DISTANCE FROM IDEAL BUCKET (DIB)</div>
          <div style={{ fontSize: '24px', fontWeight: 700, marginTop: '4px', color: 'var(--accent-amber)' }}>
            DIB &le; 18 <span style={{ fontSize: '13px', color: 'var(--text-muted)' }}>(Max Limit 25)</span>
          </div>
          <div style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '4px' }}>
            O(1) Backshift Deletion (0 Dead Tombstones)
          </div>
        </div>
      </div>

      {/* Main Keyspace Table Card */}
      <div className="card">
        <div className="card-header">
          <div style={{ display: 'flex', alignItems: 'center', gap: '16px', flex: 1 }}>
            <div className="card-title">
              <Database size={16} color="var(--accent-amber)" /> Flat Robin Hood Keyspace Explorer
            </div>

            {/* Search Input */}
            <div style={{ position: 'relative', width: '280px' }}>
              <Search size={14} style={{ position: 'absolute', left: '10px', top: '10px', color: 'var(--text-muted)' }} />
              <input
                type="text"
                placeholder="Filter keys or values..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                style={{
                  width: '100%',
                  background: 'var(--bg-core)',
                  border: '1px solid var(--border-default)',
                  borderRadius: '6px',
                  padding: '6px 12px 6px 30px',
                  color: 'var(--text-primary)',
                  fontSize: '12px',
                  outline: 'none',
                }}
              />
            </div>
          </div>

          <button className="btn-primary" onClick={() => setShowAddForm(!showAddForm)}>
            <Plus size={14} /> Add New Key
          </button>
        </div>

        {/* Add Key Form Dropdown */}
        {showAddForm && (
          <form
            onSubmit={handleAddSubmit}
            style={{
              background: 'var(--bg-elevated)',
              border: '1px solid var(--border-focus)',
              borderRadius: '8px',
              padding: '16px',
              marginBottom: '16px',
              display: 'flex',
              gap: '12px',
              alignItems: 'flex-end',
              flexWrap: 'wrap',
            }}
          >
            <div style={{ flex: 1, minWidth: '160px' }}>
              <label style={{ display: 'block', fontSize: '11px', color: 'var(--text-muted)', marginBottom: '4px' }}>
                Key Name
              </label>
              <input
                type="text"
                className="mono-text"
                placeholder="e.g. user:session:101"
                value={newKey}
                onChange={(e) => setNewKey(e.target.value)}
                style={{
                  width: '100%',
                  background: 'var(--bg-core)',
                  border: '1px solid var(--border-default)',
                  color: 'var(--text-primary)',
                  padding: '7px 10px',
                  borderRadius: '6px',
                  fontSize: '12px',
                  outline: 'none',
                }}
              />
            </div>

            <div style={{ flex: 1, minWidth: '160px' }}>
              <label style={{ display: 'block', fontSize: '11px', color: 'var(--text-muted)', marginBottom: '4px' }}>
                Value Payload
              </label>
              <input
                type="text"
                className="mono-text"
                placeholder="Payload string"
                value={newValue}
                onChange={(e) => setNewValue(e.target.value)}
                style={{
                  width: '100%',
                  background: 'var(--bg-core)',
                  border: '1px solid var(--border-default)',
                  color: 'var(--text-primary)',
                  padding: '7px 10px',
                  borderRadius: '6px',
                  fontSize: '12px',
                  outline: 'none',
                }}
              />
            </div>

            <div style={{ width: '130px' }}>
              <label style={{ display: 'block', fontSize: '11px', color: 'var(--text-muted)', marginBottom: '4px' }}>
                TTL (ms, optional)
              </label>
              <input
                type="number"
                placeholder="e.g. 5000"
                value={newTtl}
                onChange={(e) => setNewTtl(e.target.value === '' ? '' : Number(e.target.value))}
                style={{
                  width: '100%',
                  background: 'var(--bg-core)',
                  border: '1px solid var(--border-default)',
                  color: 'var(--text-primary)',
                  padding: '7px 10px',
                  borderRadius: '6px',
                  fontSize: '12px',
                  outline: 'none',
                }}
              />
            </div>

            <button type="submit" className="btn-primary" style={{ height: '34px' }}>
              Save Slot
            </button>
            <button
              type="button"
              className="btn-secondary"
              style={{ height: '34px' }}
              onClick={() => setShowAddForm(false)}
            >
              Cancel
            </button>
          </form>
        )}

        {/* Data Table */}
        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', fontSize: '12px' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid var(--border-default)', color: 'var(--text-muted)' }}>
                <th style={{ padding: '10px 12px' }}>Key Identifier</th>
                <th style={{ padding: '10px 12px' }}>Value Payload</th>
                <th style={{ padding: '10px 12px' }}>Memory Allocation</th>
                <th style={{ padding: '10px 12px' }}>DIB</th>
                <th style={{ padding: '10px 12px' }}>Epoch</th>
                <th style={{ padding: '10px 12px' }}>Shard</th>
                <th style={{ padding: '10px 12px' }}>TTL Status</th>
                <th style={{ padding: '10px 12px', textAlign: 'right' }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {filteredItems.map((item) => (
                <tr
                  key={item.key}
                  style={{
                    borderBottom: '1px solid var(--border-subtle)',
                    transition: 'background-color 0.15s ease',
                  }}
                  onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-card-hover)')}
                  onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
                >
                  <td style={{ padding: '10px 12px' }} className="mono-text">
                    <span style={{ color: 'var(--text-primary)', fontWeight: 600 }}>{item.key}</span>
                  </td>

                  <td style={{ padding: '10px 12px' }} className="mono-text">
                    <span style={{ color: 'var(--text-secondary)' }}>
                      {item.value.length > 28 ? item.value.slice(0, 28) + '...' : item.value}
                    </span>
                  </td>

                  <td style={{ padding: '10px 12px' }}>
                    {item.isSso ? (
                      <span
                        style={{
                          background: 'rgba(16, 185, 129, 0.12)',
                          color: 'var(--accent-green)',
                          border: '1px solid rgba(16, 185, 129, 0.3)',
                          padding: '2px 8px',
                          borderRadius: '4px',
                          fontSize: '11px',
                          fontWeight: 600,
                          display: 'inline-flex',
                          alignItems: 'center',
                          gap: '4px',
                        }}
                      >
                        <ShieldCheck size={12} /> SSO (0 Heap)
                      </span>
                    ) : (
                      <span
                        style={{
                          background: 'rgba(148, 163, 184, 0.1)',
                          color: 'var(--text-secondary)',
                          border: '1px solid var(--border-default)',
                          padding: '2px 8px',
                          borderRadius: '4px',
                          fontSize: '11px',
                          display: 'inline-flex',
                          alignItems: 'center',
                          gap: '4px',
                        }}
                      >
                        <HardDrive size={12} /> Heap Slab
                      </span>
                    )}
                  </td>

                  <td style={{ padding: '10px 12px' }} className="mono-text">
                    <span
                      style={{
                        color: item.dib > 15 ? 'var(--accent-red)' : item.dib > 8 ? 'var(--accent-amber)' : 'var(--text-muted)',
                        fontWeight: 600,
                      }}
                    >
                      {item.dib}
                    </span>
                  </td>

                  <td style={{ padding: '10px 12px' }} className="mono-text">
                    <span style={{ color: 'var(--text-dim)' }}>ep-{item.epoch}</span>
                  </td>

                  <td style={{ padding: '10px 12px' }} className="mono-text">
                    <span style={{ color: 'var(--accent-amber)' }}>S{item.targetShard}</span>
                  </td>

                  <td style={{ padding: '10px 12px' }}>
                    {item.ttlMs ? (
                      <span
                        className="mono-text"
                        style={{
                          fontSize: '11px',
                          color: 'var(--accent-cyan)',
                          display: 'inline-flex',
                          alignItems: 'center',
                          gap: '4px',
                        }}
                      >
                        <Clock size={11} /> {item.ttlMs} ms
                      </span>
                    ) : (
                      <span style={{ color: 'var(--text-dim)', fontSize: '11px' }}>Persistent</span>
                    )}
                  </td>

                  <td style={{ padding: '10px 12px', textAlign: 'right' }}>
                    <button
                      onClick={() => onDeleteKey(item.key)}
                      style={{
                        background: 'transparent',
                        border: 'none',
                        color: 'var(--text-muted)',
                        cursor: 'pointer',
                        padding: '4px',
                        borderRadius: '4px',
                        transition: 'color 0.15s ease',
                      }}
                      onMouseEnter={(e) => (e.currentTarget.style.color = 'var(--accent-red)')}
                      onMouseLeave={(e) => (e.currentTarget.style.color = 'var(--text-muted)')}
                      title="Backshift Delete (0 Tombstones)"
                    >
                      <Trash2 size={14} />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
