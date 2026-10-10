import React, { useState, useMemo } from 'react';
import {
  Database,
  Search,
  Plus,
  Trash2,
  Clock,
  Copy,
  Check,
  Edit2,
  Save,
  X,
  FileCode,
  Binary,
  AlignLeft,
} from 'lucide-react';
import type { KeyspaceItem } from '../types';

interface KeyspaceBrowserProps {
  items: KeyspaceItem[];
  onSetKey: (key: string, value: string, ttlMs: number | null) => void;
  onDeleteKey: (key: string) => void;
}

export const KeyspaceBrowser: React.FC<KeyspaceBrowserProps> = ({
  items,
  onSetKey,
  onDeleteKey,
}) => {
  const [selectedKeyStr, setSelectedKeyStr] = useState<string>(items[0]?.key || '');
  const [searchQuery, setSearchQuery] = useState('');
  const [activeNamespace, setActiveNamespace] = useState<string>('all');
  const [viewTab, setViewTab] = useState<'raw' | 'json' | 'hex'>('raw');
  const [copiedKey, setCopiedKey] = useState(false);

  // Modals & Inline Edit State
  const [showAddModal, setShowAddModal] = useState(false);
  const [newKey, setNewKey] = useState('');
  const [newValue, setNewValue] = useState('');
  const [newTtlSec, setNewTtlSec] = useState('');

  const [isEditing, setIsEditing] = useState(false);
  const [editedValue, setEditedValue] = useState('');

  // Extract unique namespaces
  const namespaces = useMemo(() => {
    const set = new Set<string>();
    items.forEach((item) => {
      const idx = item.key.indexOf(':');
      if (idx !== -1) {
        set.add(item.key.slice(0, idx) + ':*');
      }
    });
    return ['all', ...Array.from(set).sort()];
  }, [items]);

  // Filtered keys
  const filteredItems = useMemo(() => {
    return items.filter((item) => {
      const matchesSearch = item.key.toLowerCase().includes(searchQuery.toLowerCase());
      if (!matchesSearch) return false;
      if (activeNamespace === 'all') return true;
      const prefix = activeNamespace.replace('*', '');
      return item.key.startsWith(prefix);
    });
  }, [items, searchQuery, activeNamespace]);

  const selectedItem = items.find((i) => i.key === selectedKeyStr) || filteredItems[0] || null;

  const handleCopyKey = () => {
    if (selectedItem) {
      navigator.clipboard.writeText(selectedItem.key);
      setCopiedKey(true);
      setTimeout(() => setCopiedKey(false), 1500);
    }
  };

  const handleSaveEdit = () => {
    if (selectedItem) {
      onSetKey(selectedItem.key, editedValue, selectedItem.ttlMs);
      setIsEditing(false);
    }
  };

  const handleCreateKey = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newKey.trim()) return;
    const ttl = newTtlSec ? Number(newTtlSec) * 1000 : null;
    onSetKey(newKey.trim(), newValue, ttl);
    setSelectedKeyStr(newKey.trim());
    setNewKey('');
    setNewValue('');
    setNewTtlSec('');
    setShowAddModal(false);
  };

  // Generate Hex Dump representation of value
  const renderHexDump = (str: string) => {
    const bytes = new TextEncoder().encode(str);
    const rows = [];
    const chunkSize = 16;
    for (let i = 0; i < bytes.length; i += chunkSize) {
      const chunk = bytes.slice(i, i + chunkSize);
      const hexPart = Array.from(chunk)
        .map((b) => b.toString(16).padStart(2, '0'))
        .join(' ');
      const asciiPart = Array.from(chunk)
        .map((b) => (b >= 32 && b <= 126 ? String.fromCharCode(b) : '.'))
        .join('');
      const offset = i.toString(16).padStart(4, '0');
      rows.push({ offset, hexPart, asciiPart });
    }
    if (rows.length === 0) {
      rows.push({ offset: '0000', hexPart: '00', asciiPart: '.' });
    }
    return rows;
  };

  // Try parsing JSON
  const isJson = (str: string) => {
    try {
      JSON.parse(str);
      return true;
    } catch {
      return false;
    }
  };

  return (
    <div className="two-pane-container">
      {/* Left Pane: Key List & Filters */}
      <div className="pane-card">
        <div className="pane-header">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Database size={15} color="var(--accent-primary)" />
            <span style={{ fontSize: '13px', fontWeight: 600, color: 'var(--text-primary)' }}>
              Keyspace ({filteredItems.length})
            </span>
          </div>
          <button className="btn-primary" onClick={() => setShowAddModal(true)}>
            <Plus size={13} /> New Key
          </button>
        </div>

        {/* Search & Namespace Filters */}
        <div style={{ padding: '12px 14px', borderBottom: '1px solid var(--border-subtle)', background: 'var(--bg-card)' }}>
          <div style={{ position: 'relative', marginBottom: '10px' }}>
            <Search
              size={14}
              color="var(--text-dim)"
              style={{ position: 'absolute', left: '10px', top: '9px' }}
            />
            <input
              type="text"
              placeholder="Filter keys (e.g. auth:*)..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="input-field"
              style={{ paddingLeft: '32px' }}
            />
          </div>

          {/* Namespace Filter Chips */}
          <div style={{ display: 'flex', gap: '5px', flexWrap: 'wrap' }}>
            {namespaces.map((ns) => (
              <button
                key={ns}
                onClick={() => setActiveNamespace(ns)}
                style={{
                  background: activeNamespace === ns ? 'var(--accent-primary-bg)' : 'var(--bg-elevated)',
                  color: activeNamespace === ns ? 'var(--accent-primary)' : 'var(--text-secondary)',
                  border: activeNamespace === ns ? '1px solid var(--accent-primary)' : '1px solid var(--border-subtle)',
                  fontSize: '11px',
                  fontWeight: 500,
                  padding: '3px 8px',
                  borderRadius: '4px',
                  cursor: 'pointer',
                  fontFamily: 'var(--font-mono, monospace)',
                }}
              >
                {ns}
              </button>
            ))}
          </div>
        </div>

        {/* Scrollable Key Items */}
        <div className="pane-content">
          {filteredItems.length === 0 ? (
            <div style={{ textAlign: 'center', padding: '30px 10px', color: 'var(--text-muted)', fontSize: '12.5px' }}>
              No keys matching filter
            </div>
          ) : (
            filteredItems.map((item) => {
              const isSelected = selectedItem?.key === item.key;
              return (
                <div
                  key={item.key}
                  className={`key-item ${isSelected ? 'selected' : ''}`}
                  onClick={() => {
                    setSelectedKeyStr(item.key);
                    setIsEditing(false);
                  }}
                >
                  <div style={{ flex: 1, minWidth: 0, paddingRight: '8px' }}>
                    <div className="key-item-title">{item.key}</div>
                    <div style={{ display: 'flex', gap: '6px', marginTop: '4px', alignItems: 'center' }}>
                      <span className={`badge ${item.isSso ? 'badge-sso' : 'badge-slab'}`}>
                        {item.isSso ? 'SSO 23B' : 'SLAB'}
                      </span>
                      <span className="badge badge-shard">S{item.targetShard}</span>
                      <span style={{ fontSize: '11px', color: 'var(--text-dim)', fontFamily: 'JetBrains Mono, monospace' }}>
                        {item.sizeBytes}B
                      </span>
                    </div>
                  </div>
                  {item.ttlMs ? (
                    <span className="badge badge-ttl">
                      <Clock size={10} />
                      {Math.ceil(item.ttlMs / 1000)}s
                    </span>
                  ) : (
                    <span className="badge badge-persist">PERM</span>
                  )}
                </div>
              );
            })
          )}
        </div>
      </div>

      {/* Right Pane: Key Detail Inspector */}
      <div className="pane-card">
        {selectedItem ? (
          <>
            <div className="pane-header">
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', minWidth: 0 }}>
                <span
                  style={{
                    fontFamily: 'JetBrains Mono, monospace',
                    fontWeight: 700,
                    fontSize: '14px',
                    color: 'var(--text-primary)',
                    wordBreak: 'break-all',
                  }}
                >
                  {selectedItem.key}
                </span>
                <button
                  onClick={handleCopyKey}
                  style={{
                    background: 'transparent',
                    border: 'none',
                    cursor: 'pointer',
                    color: copiedKey ? 'var(--accent-green)' : 'var(--text-muted)',
                    display: 'flex',
                    alignItems: 'center',
                  }}
                  title="Copy Key"
                >
                  {copiedKey ? <Check size={14} /> : <Copy size={14} />}
                </button>
              </div>

              <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                {!isEditing ? (
                  <button
                    className="btn-secondary"
                    onClick={() => {
                      setEditedValue(selectedItem.value);
                      setIsEditing(true);
                    }}
                  >
                    <Edit2 size={13} /> Edit
                  </button>
                ) : (
                  <>
                    <button className="btn-primary" onClick={handleSaveEdit}>
                      <Save size={13} /> Save
                    </button>
                    <button className="btn-secondary" onClick={() => setIsEditing(false)}>
                      <X size={13} /> Cancel
                    </button>
                  </>
                )}

                <button
                  className="btn-danger"
                  onClick={() => onDeleteKey(selectedItem.key)}
                  title="Delete key (O(1) backward-shift)"
                >
                  <Trash2 size={13} /> Delete
                </button>
              </div>
            </div>

            <div className="pane-content" style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
              {/* Architecture & Memory Stats Cards */}
              <div
                style={{
                  display: 'grid',
                  gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))',
                  gap: '10px',
                }}
              >
                <div style={{ background: 'var(--bg-elevated)', padding: '10px 12px', borderRadius: '6px', border: '1px solid var(--border-subtle)' }}>
                  <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>ALLOCATION MODE</div>
                  <div style={{ fontSize: '12.5px', fontWeight: 600, marginTop: '2px', color: selectedItem.isSso ? 'var(--accent-green)' : 'var(--text-primary)' }}>
                    {selectedItem.isSso ? 'SSO Inlined (0-Heap)' : 'Heap Slab Buffer'}
                  </div>
                  <div style={{ fontSize: '10px', color: 'var(--text-dim)', marginTop: '2px' }}>
                    {selectedItem.isSso ? 'Stored inside 64B slot' : 'External slab block'}
                  </div>
                </div>

                <div style={{ background: 'var(--bg-elevated)', padding: '10px 12px', borderRadius: '6px', border: '1px solid var(--border-subtle)' }}>
                  <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>SIZE IN MEMORY</div>
                  <div style={{ fontSize: '12.5px', fontWeight: 600, marginTop: '2px', color: 'var(--text-primary)', fontFamily: 'JetBrains Mono, monospace' }}>
                    {selectedItem.sizeBytes} Bytes
                  </div>
                  <div style={{ fontSize: '10px', color: 'var(--text-dim)', marginTop: '2px' }}>
                    Raw UTF-8 byte length
                  </div>
                </div>

                <div style={{ background: 'var(--bg-elevated)', padding: '10px 12px', borderRadius: '6px', border: '1px solid var(--border-subtle)' }}>
                  <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>ROBIN HOOD DIB</div>
                  <div style={{ fontSize: '12.5px', fontWeight: 600, marginTop: '2px', color: 'var(--accent-cyan)', fontFamily: 'JetBrains Mono, monospace' }}>
                    DIB = {selectedItem.dib}
                  </div>
                  <div style={{ fontSize: '10px', color: 'var(--text-dim)', marginTop: '2px' }}>
                    Distance to initial slot
                  </div>
                </div>

                <div style={{ background: 'var(--bg-elevated)', padding: '10px 12px', borderRadius: '6px', border: '1px solid var(--border-subtle)' }}>
                  <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>HARDWARE ROUTE</div>
                  <div style={{ fontSize: '12.5px', fontWeight: 600, marginTop: '2px', color: 'var(--accent-primary)', fontFamily: 'JetBrains Mono, monospace' }}>
                    Shard {selectedItem.targetShard} (Core {selectedItem.targetShard})
                  </div>
                  <div style={{ fontSize: '10px', color: 'var(--text-dim)', marginTop: '2px' }}>
                    Murmur3 Consistent Hash
                  </div>
                </div>

                <div style={{ background: 'var(--bg-elevated)', padding: '10px 12px', borderRadius: '6px', border: '1px solid var(--border-subtle)' }}>
                  <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>EXPIRATION</div>
                  <div style={{ fontSize: '12.5px', fontWeight: 600, marginTop: '2px', color: selectedItem.ttlMs ? 'var(--accent-amber)' : 'var(--text-muted)' }}>
                    {selectedItem.ttlMs ? `${Math.ceil(selectedItem.ttlMs / 1000)}s (Timing Wheel)` : 'Persistent'}
                  </div>
                  <div style={{ fontSize: '10px', color: 'var(--text-dim)', marginTop: '2px' }}>
                    O(1) cascaded expiration
                  </div>
                </div>
              </div>

              {/* Value Viewer with Mode Switcher */}
              <div style={{ background: 'var(--bg-surface)', border: '1px solid var(--border-subtle)', borderRadius: '6px', overflow: 'hidden' }}>
                <div
                  style={{
                    padding: '8px 12px',
                    borderBottom: '1px solid var(--border-subtle)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    background: 'var(--bg-card)',
                  }}
                >
                  <div style={{ display: 'flex', gap: '4px' }}>
                    <button
                      onClick={() => setViewTab('raw')}
                      style={{
                        background: viewTab === 'raw' ? 'var(--bg-elevated)' : 'transparent',
                        color: viewTab === 'raw' ? 'var(--text-primary)' : 'var(--text-secondary)',
                        border: 'none',
                        padding: '4px 10px',
                        borderRadius: '4px',
                        cursor: 'pointer',
                        fontSize: '11.5px',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '5px',
                      }}
                    >
                      <AlignLeft size={12} /> Raw String
                    </button>
                    <button
                      onClick={() => setViewTab('json')}
                      style={{
                        background: viewTab === 'json' ? 'var(--bg-elevated)' : 'transparent',
                        color: viewTab === 'json' ? 'var(--text-primary)' : 'var(--text-secondary)',
                        border: 'none',
                        padding: '4px 10px',
                        borderRadius: '4px',
                        cursor: 'pointer',
                        fontSize: '11.5px',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '5px',
                      }}
                    >
                      <FileCode size={12} /> JSON {isJson(selectedItem.value) && '✓'}
                    </button>
                    <button
                      onClick={() => setViewTab('hex')}
                      style={{
                        background: viewTab === 'hex' ? 'var(--bg-elevated)' : 'transparent',
                        color: viewTab === 'hex' ? 'var(--text-primary)' : 'var(--text-secondary)',
                        border: 'none',
                        padding: '4px 10px',
                        borderRadius: '4px',
                        cursor: 'pointer',
                        fontSize: '11.5px',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '5px',
                      }}
                    >
                      <Binary size={12} /> Hex Memory Layout
                    </button>
                  </div>
                </div>

                <div style={{ padding: '14px' }}>
                  {isEditing ? (
                    <div>
                      <textarea
                        value={editedValue}
                        onChange={(e) => setEditedValue(e.target.value)}
                        className="input-field"
                        style={{ height: '140px', resize: 'vertical' }}
                      />
                      <div style={{ marginTop: '6px', fontSize: '11px', color: 'var(--text-muted)' }}>
                        Byte size: {editedValue.length}B &bull;{' '}
                        {editedValue.length <= 23 ? (
                          <span style={{ color: 'var(--accent-green)' }}>Eligible for SSO 23B Inlined Zero-Heap Storage</span>
                        ) : (
                          <span style={{ color: 'var(--text-dim)' }}>Will be allocated in dynamic heap slab</span>
                        )}
                      </div>
                    </div>
                  ) : viewTab === 'raw' ? (
                    <pre
                      style={{
                        margin: 0,
                        fontFamily: 'JetBrains Mono, monospace',
                        fontSize: '12.5px',
                        color: 'var(--text-primary)',
                        whiteSpace: 'pre-wrap',
                        wordBreak: 'break-all',
                      }}
                    >
                      {selectedItem.value}
                    </pre>
                  ) : viewTab === 'json' ? (
                    <pre
                      style={{
                        margin: 0,
                        fontFamily: 'JetBrains Mono, monospace',
                        fontSize: '12px',
                        color: 'var(--accent-cyan)',
                        whiteSpace: 'pre-wrap',
                      }}
                    >
                      {isJson(selectedItem.value)
                        ? JSON.stringify(JSON.parse(selectedItem.value), null, 2)
                        : '// Value is not valid JSON:\n' + selectedItem.value}
                    </pre>
                  ) : (
                    <div style={{ overflowX: 'auto' }}>
                      <table className="hexdump-table">
                        <tbody>
                          {renderHexDump(selectedItem.value).map((row, idx) => (
                            <tr key={idx}>
                              <td className="hexdump-offset">{row.offset}</td>
                              <td className="hexdump-bytes">{row.hexPart}</td>
                              <td className="hexdump-ascii">{row.asciiPart}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  )}
                </div>
              </div>
            </div>
          </>
        ) : (
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: 'var(--text-muted)' }}>
            Select a key from the list to inspect memory layout
          </div>
        )}
      </div>

      {/* Add Key Modal */}
      {showAddModal && (
        <div className="modal-overlay">
          <div className="modal-card">
            <div className="modal-header">
              <span style={{ fontWeight: 600, fontSize: '13.5px', color: 'var(--text-primary)' }}>
                Create New State Key
              </span>
              <button
                onClick={() => setShowAddModal(false)}
                style={{ background: 'transparent', border: 'none', cursor: 'pointer', color: 'var(--text-muted)' }}
              >
                <X size={15} />
              </button>
            </div>
            <form onSubmit={handleCreateKey}>
              <div className="modal-body">
                <div>
                  <label style={{ display: 'block', fontSize: '11.5px', color: 'var(--text-secondary)', marginBottom: '5px' }}>
                    KEY NAME (Supports namespace prefixes: auth:*, ratelimit:*, cache:*)
                  </label>
                  <input
                    type="text"
                    required
                    placeholder="e.g. auth:jwt:session_992"
                    value={newKey}
                    onChange={(e) => setNewKey(e.target.value)}
                    className="input-field"
                  />
                </div>

                <div>
                  <label style={{ display: 'block', fontSize: '11.5px', color: 'var(--text-secondary)', marginBottom: '5px' }}>
                    VALUE
                  </label>
                  <textarea
                    required
                    placeholder="String or JSON payload..."
                    value={newValue}
                    onChange={(e) => setNewValue(e.target.value)}
                    className="input-field"
                    style={{ height: '80px', resize: 'vertical' }}
                  />
                  <div style={{ marginTop: '5px', fontSize: '11px' }}>
                    {newValue.length <= 23 ? (
                      <span style={{ color: 'var(--accent-green)' }}>
                        ✓ SSO 23-byte threshold satisfied: Zero-heap allocation guaranteed
                      </span>
                    ) : (
                      <span style={{ color: 'var(--text-dim)' }}>
                        Exceeds 23 bytes ({newValue.length}B) — stored in external memory slab
                      </span>
                    )}
                  </div>
                </div>

                <div>
                  <label style={{ display: 'block', fontSize: '11.5px', color: 'var(--text-secondary)', marginBottom: '5px' }}>
                    TTL EXPIRY (SECONDS, LEAVE EMPTY FOR PERSISTENT)
                  </label>
                  <input
                    type="number"
                    min="1"
                    placeholder="e.g. 60"
                    value={newTtlSec}
                    onChange={(e) => setNewTtlSec(e.target.value)}
                    className="input-field"
                  />
                </div>
              </div>

              <div className="modal-footer">
                <button type="button" className="btn-secondary" onClick={() => setShowAddModal(false)}>
                  Cancel
                </button>
                <button type="submit" className="btn-primary">
                  Store Key
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
