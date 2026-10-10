export type ThemeMode = 'slate' | 'charcoal' | 'platinum';

export interface ShardData {
  id: number;
  coreId: number;
  keyCount: number;
  opsPerSec: number;
  queueDepth: number;
  queueCapacity: number;
  skewPercent: number;
  isPinned: boolean;
  memoryAllocatedKb: number;
  loadFactorPct: number;
  maxDib: number;
}

export interface KeyspaceItem {
  key: string;
  value: string;
  dataType: 'string' | 'hash' | 'list' | 'set' | 'zset';
  sizeBytes: number;
  isSso: boolean;
  dib: number;
  epoch: number;
  ttlMs: number | null;
  expiresAt: number | null;
  targetShard: number;
}

export interface CommandLog {
  id: string;
  timestamp: string;
  command: string;
  response: string;
  durationUs: number;
  isError: boolean;
  shardId?: number;
}

export interface SnapshotRecord {
  epoch: number;
  timestamp: string;
  itemCount: number;
  crc64Hex: string;
  sizeBytes: number;
  status: 'saved' | 'in_progress';
}
