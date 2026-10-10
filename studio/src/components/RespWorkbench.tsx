import React, { useState, useRef, useEffect } from 'react';
import {
  Terminal,
  Trash2,
  Copy,
  Check,
  CornerDownLeft,
  Clock,
  Sparkles,
} from 'lucide-react';
import type { CommandLog } from '../types';

interface RespWorkbenchProps {
  logs: CommandLog[];
  onExecuteCommand: (command: string) => void;
  onClearLogs: () => void;
}

export const RespWorkbench: React.FC<RespWorkbenchProps> = ({
  logs,
  onExecuteCommand,
  onClearLogs,
}) => {
  const [inputVal, setInputVal] = useState('');
  const [copied, setCopied] = useState(false);
  const [historyIndex, setHistoryIndex] = useState<number | null>(null);

  const inputRef = useRef<HTMLInputElement>(null);
  const outputEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll output to bottom
  useEffect(() => {
    outputEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  // Extract command history
  const commandHistory = logs.map((l) => l.command);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!inputVal.trim()) return;
    onExecuteCommand(inputVal.trim());
    setInputVal('');
    setHistoryIndex(null);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (commandHistory.length === 0) return;
      const nextIndex =
        historyIndex === null
          ? commandHistory.length - 1
          : Math.max(0, historyIndex - 1);
      setHistoryIndex(nextIndex);
      setInputVal(commandHistory[nextIndex] || '');
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (historyIndex === null) return;
      const nextIndex = historyIndex + 1;
      if (nextIndex >= commandHistory.length) {
        setHistoryIndex(null);
        setInputVal('');
      } else {
        setHistoryIndex(nextIndex);
        setInputVal(commandHistory[nextIndex] || '');
      }
    }
  };

  const handleRunSample = (cmd: string) => {
    setInputVal(cmd);
    inputRef.current?.focus();
  };

  const handleCopyLogs = () => {
    const text = logs
      .map((l) => `> ${l.command}\n${l.response}\n[${l.durationUs.toFixed(1)} µs]`)
      .join('\n\n');
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
      {/* Quick Run Command Chips */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          overflowX: 'auto',
          paddingBottom: '2px',
        }}
      >
        <span style={{ fontSize: '11.5px', color: 'var(--text-muted)', display: 'flex', alignItems: 'center', gap: '4px', whiteSpace: 'nowrap' }}>
          <Sparkles size={13} color="var(--accent-primary)" /> Quick Run:
        </span>
        {[
          'PING',
          'INFO',
          'DBSIZE',
          'BGSAVE',
          'LASTSAVE',
          'GET auth:jwt:session_991',
          'SET cache:demo "zero_copy" EX 60',
          'MGET ratelimit:api:10.0.1.5 lock:distributed:leader_node',
          'EXISTS lock:distributed:leader_node',
          'SAVE',
        ].map((cmd) => (
          <button
            key={cmd}
            onClick={() => handleRunSample(cmd)}
            style={{
              background: 'var(--bg-elevated)',
              color: 'var(--text-secondary)',
              border: '1px solid var(--border-subtle)',
              fontSize: '11px',
              padding: '4px 9px',
              borderRadius: '4px',
              cursor: 'pointer',
              fontFamily: 'JetBrains Mono, monospace',
              whiteSpace: 'nowrap',
              transition: 'all 0.12s ease',
            }}
          >
            {cmd}
          </button>
        ))}
      </div>

      {/* Terminal Console Window */}
      <div className="terminal-window">
        {/* Terminal Header Bar */}
        <div className="terminal-header">
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Terminal size={14} color="var(--accent-primary)" />
            <span style={{ fontWeight: 600, color: 'var(--text-primary)' }}>
              RESP3 Interactive Console &bull; localhost:6379
            </span>
          </div>

          <div style={{ display: 'flex', gap: '8px' }}>
            <button
              onClick={handleCopyLogs}
              style={{
                background: 'transparent',
                border: 'none',
                color: copied ? 'var(--accent-green)' : 'var(--text-secondary)',
                fontSize: '11px',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: '4px',
              }}
            >
              {copied ? <Check size={12} /> : <Copy size={12} />} Copy
            </button>
            <button
              onClick={onClearLogs}
              style={{
                background: 'transparent',
                border: 'none',
                color: 'var(--text-muted)',
                fontSize: '11px',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: '4px',
              }}
            >
              <Trash2 size={12} /> Clear
            </button>
          </div>
        </div>

        {/* Scrollable Terminal Output */}
        <div className="terminal-body">
          <div style={{ color: 'var(--text-dim)', fontSize: '11.5px', marginBottom: '8px' }}>
            Valence Engine v0.1.0 &bull; RESP3 Wire Protocol &bull; Microsecond Engine Latency
            <br />
            Type commands below or click Quick Run chips above (Up/Down arrow for history).
          </div>

          {logs.map((log) => {
            const isError = log.isError;
            return (
              <div key={log.id} style={{ display: 'flex', flexDirection: 'column', gap: '3px' }}>
                {/* Command Input Prompt */}
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <span style={{ color: 'var(--accent-primary)', fontWeight: 700 }}>valence&gt;</span>
                  <span style={{ color: 'var(--text-primary)' }}>{log.command}</span>
                  <span style={{ fontSize: '10.5px', color: 'var(--text-dim)', marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: '3px' }}>
                    <Clock size={10} />
                    {log.durationUs.toFixed(1)} µs
                  </span>
                </div>

                {/* Command Result Output */}
                <pre
                  style={{
                    margin: 0,
                    padding: '6px 10px',
                    borderRadius: '4px',
                    background: 'rgba(0, 0, 0, 0.25)',
                    color: isError ? 'var(--accent-red)' : log.response.startsWith('+') ? 'var(--accent-green)' : log.response.startsWith(':') ? 'var(--accent-cyan)' : 'var(--terminal-text)',
                    fontSize: '12px',
                    whiteSpace: 'pre-wrap',
                    wordBreak: 'break-all',
                    fontFamily: 'JetBrains Mono, monospace',
                  }}
                >
                  {log.response}
                </pre>
              </div>
            );
          })}
          <div ref={outputEndRef} />
        </div>

        {/* Command Input Line */}
        <form onSubmit={handleSubmit} className="terminal-input-bar">
          <span className="terminal-prompt">valence&gt;</span>
          <input
            ref={inputRef}
            type="text"
            placeholder="Enter RESP command (e.g. SET auth:token '123' EX 3600, GET, DBSIZE, INFO)..."
            value={inputVal}
            onChange={(e) => setInputVal(e.target.value)}
            onKeyDown={handleKeyDown}
            className="terminal-input"
            autoFocus
          />
          <button type="submit" className="btn-primary" style={{ padding: '6px 12px' }}>
            <CornerDownLeft size={13} /> Run
          </button>
        </form>
      </div>
    </div>
  );
};
