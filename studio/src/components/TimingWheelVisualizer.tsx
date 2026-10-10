import React, { useState, useEffect } from 'react';
import { Clock, Zap, ArrowDownRight, Layers } from 'lucide-react';
import type { TimingWheelTier } from '../types';

interface TimingWheelVisualizerProps {
  onScheduleExpiry: (key: string, ttlMs: number) => void;
}

export const TimingWheelVisualizer: React.FC<TimingWheelVisualizerProps> = ({ onScheduleExpiry }) => {
  const [scheduleKey, setScheduleKey] = useState('session:auth_token');
  const [scheduleTtl, setScheduleTtl] = useState(850);
  const [currentTimeMs, setCurrentTimeMs] = useState(Date.now());
  const [recentCascades, setRecentCascades] = useState<string[]>([
    'T2 (Slot 14) -> Cascaded 42 items to Tier 1',
    'T1 (Slot 850) -> Detached 18 items via atomic pointer swap (~7ns)',
  ]);

  // Live ticking simulation
  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentTimeMs(Date.now());
    }, 100);
    return () => clearInterval(timer);
  }, []);

  const tiers: TimingWheelTier[] = [
    {
      tier: 1,
      name: 'Tier 1: Milliseconds',
      slotCount: 1000,
      resolution: '1 millisecond',
      activeTimers: 412,
      currentSlot: currentTimeMs % 1000,
    },
    {
      tier: 2,
      name: 'Tier 2: Seconds',
      slotCount: 60,
      resolution: '1 second',
      activeTimers: 1240,
      currentSlot: Math.floor(currentTimeMs / 1000) % 60,
    },
    {
      tier: 3,
      name: 'Tier 3: Minutes',
      slotCount: 60,
      resolution: '1 minute',
      activeTimers: 3105,
      currentSlot: Math.floor(currentTimeMs / 60000) % 60,
    },
    {
      tier: 4,
      name: 'Tier 4: Hours',
      slotCount: 24,
      resolution: '1 hour',
      activeTimers: 8940,
      currentSlot: Math.floor(currentTimeMs / 3600000) % 24,
    },
  ];

  const handleSchedule = (e: React.FormEvent) => {
    e.preventDefault();
    if (!scheduleKey || scheduleTtl <= 0) return;

    let targetTier = 1;
    if (scheduleTtl >= 3600000) targetTier = 4;
    else if (scheduleTtl >= 60000) targetTier = 3;
    else if (scheduleTtl >= 1000) targetTier = 2;

    onScheduleExpiry(scheduleKey, scheduleTtl);
    setRecentCascades((prev) => [
      `Inserted "${scheduleKey}" (${scheduleTtl}ms) directly into Tier ${targetTier}`,
      ...prev.slice(0, 4),
    ]);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Top Banner */}
      <div className="grid-3">
        <div className="card">
          <div className="card-subtitle">HEAD ATOMIC SWAP SPEED</div>
          <div style={{ fontSize: '24px', fontWeight: 700, marginTop: '4px', color: 'var(--accent-green)' }}>
            ~5 to 10 ns <span style={{ fontSize: '13px', color: 'var(--text-muted)' }}>(AcqRel)</span>
          </div>
          <div style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '4px' }}>
            Zero foreground lock contention
          </div>
        </div>

        <div className="card">
          <div className="card-subtitle">TOTAL ACTIVE TIMERS</div>
          <div style={{ fontSize: '24px', fontWeight: 700, marginTop: '4px', color: 'var(--accent-amber)' }}>
            13,697 <span style={{ fontSize: '13px', color: 'var(--text-muted)' }}>Registered</span>
          </div>
          <div style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '4px' }}>
            Across 4 cascading resolution tiers
          </div>
        </div>

        <div className="card">
          <div className="card-subtitle">RECLAMATION PIPELINE</div>
          <div style={{ fontSize: '24px', fontWeight: 700, marginTop: '4px', color: 'var(--accent-cyan)' }}>
            Decoupled Actor
          </div>
          <div style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '4px' }}>
            Background memory deallocation
          </div>
        </div>
      </div>

      {/* 4 Cascading Tiers Grid */}
      <div className="card">
        <div className="card-header">
          <div>
            <div className="card-title">
              <Clock size={16} color="var(--accent-amber)" /> 4-Tier Hierarchical Timing Wheel Architecture
            </div>
            <div className="card-subtitle">
              Schedules millions of expirations in O(1) time without scanning hash tables. Cascades down to Tier 1 for millisecond retirement.
            </div>
          </div>
        </div>

        <div className="grid-4" style={{ gap: '16px' }}>
          {tiers.map((tier) => {
            const slotPercent = (tier.currentSlot / tier.slotCount) * 100;

            return (
              <div
                key={tier.tier}
                style={{
                  background: 'var(--bg-core)',
                  border: '1px solid var(--border-default)',
                  borderRadius: '8px',
                  padding: '16px',
                  position: 'relative',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '8px' }}>
                  <span style={{ fontWeight: 600, fontSize: '13px', color: 'var(--accent-amber)' }}>
                    Tier {tier.tier}
                  </span>
                  <span
                    className="mono-text"
                    style={{
                      fontSize: '11px',
                      background: 'rgba(255, 255, 255, 0.05)',
                      padding: '2px 6px',
                      borderRadius: '4px',
                      color: 'var(--text-secondary)',
                    }}
                  >
                    {tier.resolution}
                  </span>
                </div>

                <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '12px' }}>
                  {tier.name}
                </div>

                {/* Circular Slot Ring Visual */}
                <div style={{ marginBottom: '14px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '11px', color: 'var(--text-muted)', marginBottom: '4px' }}>
                    <span>Active Slot</span>
                    <span className="mono-text" style={{ color: 'var(--text-primary)', fontWeight: 600 }}>
                      {tier.currentSlot} / {tier.slotCount}
                    </span>
                  </div>
                  <div style={{ height: '6px', background: 'var(--bg-card)', borderRadius: '3px', overflow: 'hidden' }}>
                    <div
                      style={{
                        height: '100%',
                        width: `${slotPercent}%`,
                        background: 'var(--accent-amber)',
                        transition: 'width 0.1s linear',
                      }}
                    />
                  </div>
                </div>

                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '12px', borderTop: '1px solid var(--border-subtle)', paddingTop: '8px' }}>
                  <span style={{ color: 'var(--text-muted)' }}>Bucketed Timers</span>
                  <span className="mono-text" style={{ fontWeight: 600, color: 'var(--text-primary)' }}>
                    {tier.activeTimers.toLocaleString()}
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Schedule Expiry & Cascade Event Log */}
      <div className="grid-2">
        {/* Schedule Form */}
        <div className="card">
          <div className="card-header">
            <div>
              <div className="card-title">
                <Zap size={15} color="var(--accent-amber)" /> Schedule Test Key Expiration
              </div>
              <div className="card-subtitle">
                Inspect which tier will hold the intrusive pointer.
              </div>
            </div>
          </div>

          <form onSubmit={handleSchedule} style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
            <div>
              <label style={{ display: 'block', fontSize: '12px', color: 'var(--text-muted)', marginBottom: '6px' }}>
                Key Identifier
              </label>
              <input
                type="text"
                className="mono-text"
                value={scheduleKey}
                onChange={(e) => setScheduleKey(e.target.value)}
                style={{
                  width: '100%',
                  background: 'var(--bg-core)',
                  border: '1px solid var(--border-default)',
                  color: 'var(--text-primary)',
                  padding: '8px 12px',
                  borderRadius: '6px',
                  fontSize: '13px',
                  outline: 'none',
                }}
              />
            </div>

            <div>
              <label style={{ display: 'block', fontSize: '12px', color: 'var(--text-muted)', marginBottom: '6px' }}>
                TTL Duration (Milliseconds)
              </label>
              <div style={{ display: 'flex', gap: '8px' }}>
                <input
                  type="number"
                  className="mono-text"
                  value={scheduleTtl}
                  onChange={(e) => setScheduleTtl(Number(e.target.value))}
                  style={{
                    flex: 1,
                    background: 'var(--bg-core)',
                    border: '1px solid var(--border-default)',
                    color: 'var(--text-primary)',
                    padding: '8px 12px',
                    borderRadius: '6px',
                    fontSize: '13px',
                    outline: 'none',
                  }}
                />
                <button type="button" className="btn-secondary" onClick={() => setScheduleTtl(500)}>500ms (T1)</button>
                <button type="button" className="btn-secondary" onClick={() => setScheduleTtl(15000)}>15s (T2)</button>
                <button type="button" className="btn-secondary" onClick={() => setScheduleTtl(300000)}>5m (T3)</button>
              </div>
            </div>

            <button type="submit" className="btn-primary" style={{ marginTop: '6px' }}>
              Schedule in Timing Wheel (O(1))
            </button>
          </form>
        </div>

        {/* Live Wheel Events */}
        <div className="card">
          <div className="card-header">
            <div>
              <div className="card-title">
                <Layers size={15} color="var(--accent-cyan)" /> Live Cascading &amp; Detachment Stream
              </div>
              <div className="card-subtitle">
                Real-time intrusive pointer migrations and atomic slot pops.
              </div>
            </div>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {recentCascades.map((event, idx) => (
              <div
                key={idx}
                className="mono-text"
                style={{
                  fontSize: '12px',
                  background: 'var(--bg-core)',
                  padding: '8px 12px',
                  borderRadius: '6px',
                  border: '1px solid var(--border-subtle)',
                  color: idx === 0 ? 'var(--accent-amber)' : 'var(--text-secondary)',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                }}
              >
                <ArrowDownRight size={14} color="var(--accent-amber)" />
                {event}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
