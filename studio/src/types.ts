export interface ShardData {
  id: number;
  coreId: number;
  keyCount: number;
  opsPerSec: number;
  queueDepth: number;
  queueCapacity: number;
  skewPercent: number;
  isPinned: boolean;
}

export interface TimingWheelTier {
  tier: number;
  name: string;
  slotCount: number;
  resolution: string;
  activeTimers: number;
  currentSlot: number;
}

export interface KeyspaceItem {
  key: string;
  value: string;
  sizeBytes: number;
  isSso: boolean;
  dib: number;
  epoch: number;
  ttlMs: number | null;
  expiresAt: number | null;
  targetShard: number;
}

export interface LatencyMetric {
  timestamp: number;
  valenceUs: number;
  redisUs: number;
  p50: number;
  p90: number;
  p99: number;
}

export interface CommandLog {
  id: string;
  timestamp: string;
  command: string;
  response: string;
  durationUs: number;
  isError: boolean;
}
