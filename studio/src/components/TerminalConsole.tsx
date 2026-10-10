import React, { useState, useRef, useEffect } from 'react';
import { Terminal, Send, Trash2 } from 'lucide-react';
import type { CommandLog } from '../types';

interface TerminalConsoleProps {
  logs: CommandLog[];
  onExecuteCommand: (rawCmd: string) => void;
  onClearLogs: () => void;
}

export const TerminalConsole: React.FC<TerminalConsoleProps> = ({ logs, onExecuteCommand, onClearLogs }) => {
  const [inputVal, setInputVal] = useState('');
  const [historyIndex, setHistoryIndex] = useState<number | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = inputVal.trim();
    if (!trimmed) return;
    onExecuteCommand(trimmed);
    setInputVal('');
    setHistoryIndex(null);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (logs.length === 0) return;
      const nextIdx = historyIndex === null ? logs.length - 1 : Math.max(0, historyIndex - 1);
      setHistoryIndex(nextIdx);
      setInputVal(logs[nextIdx].command);
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (historyIndex === null) return;
      const nextIdx = historyIndex + 1;
      if (nextIdx >= logs.length) {
        setHistoryIndex(null);
        setInputVal('');
      } else {
        setHistoryIndex(nextIdx);
        setInputVal(logs[nextIdx].command);
      }
    }
  };

  return (
    <div className="card" style={{ display: 'flex', flexDirection: 'column', height: '600px' }}>
      <div className="card-header" style={{ marginBottom: '12px' }}>
        <div>
          <div className="card-title">
            <Terminal size={16} color="var(--accent-amber)" /> Interactive RESP3 Console
          </div>
          <div className="card-subtitle">
            Direct zero-copy protocol terminal. Supports PING, SET, GET, MSET, MGET, DEL, INFO, DBSIZE.
          </div>
        </div>

        <button className="btn-secondary" onClick={onClearLogs} style={{ padding: '4px 10px', fontSize: '11px' }}>
          <Trash2 size={12} /> Clear Screen
        </button>
      </div>

      {/* Terminal Output Area */}
      <div
        ref={scrollRef}
        className="mono-text"
        style={{
          flex: 1,
          background: 'var(--bg-core)',
          border: '1px solid var(--border-default)',
          borderRadius: '6px',
          padding: '14px',
          overflowY: 'auto',
          fontSize: '12px',
          display: 'flex',
          flexDirection: 'column',
          gap: '10px',
        }}
      >
        <div style={{ color: 'var(--text-dim)', borderBottom: '1px solid var(--border-subtle)', paddingBottom: '8px' }}>
          Valence In-Memory State Engine (Pure Rust 2021) — RESP2/RESP3 Direct Console
          <br />
          Type commands below. Example: SET session:user:42 &quot;Active&quot; EX 60
        </div>

        {logs.map((log) => (
          <div key={log.id} style={{ display: 'flex', flexDirection: 'column', gap: '3px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <span style={{ color: 'var(--accent-amber)', fontWeight: 600 }}>valence:6379&gt;</span>
              <span style={{ color: 'var(--text-primary)' }}>{log.command}</span>
              <span style={{ fontSize: '10px', color: 'var(--text-dim)', marginLeft: 'auto' }}>
                ({log.durationUs.toFixed(2)} µs)
              </span>
            </div>
            <pre
              style={{
                color: log.isError ? 'var(--accent-red)' : 'var(--accent-green)',
                whiteSpace: 'pre-wrap',
                fontFamily: 'inherit',
                margin: 0,
                paddingLeft: '16px',
              }}
            >
              {log.response}
            </pre>
          </div>
        ))}
      </div>

      {/* Command Input Bar */}
      <form onSubmit={handleSubmit} style={{ display: 'flex', gap: '10px', marginTop: '12px' }}>
        <div style={{ position: 'relative', flex: 1 }}>
          <span
            className="mono-text"
            style={{
              position: 'absolute',
              left: '12px',
              top: '9px',
              color: 'var(--accent-amber)',
              fontWeight: 600,
              fontSize: '12px',
            }}
          >
            &gt;
          </span>
          <input
            type="text"
            className="mono-text"
            value={inputVal}
            onChange={(e) => setInputVal(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Enter RESP command (e.g. MGET user:1 user:2)..."
            style={{
              width: '100%',
              background: 'var(--bg-core)',
              border: '1px solid var(--border-default)',
              color: 'var(--text-primary)',
              padding: '8px 12px 8px 28px',
              borderRadius: '6px',
              fontSize: '13px',
              outline: 'none',
            }}
            autoFocus
          />
        </div>

        <button type="submit" className="btn-primary" style={{ padding: '8px 16px' }}>
          <Send size={14} /> Send
        </button>
      </form>
    </div>
  );
};
