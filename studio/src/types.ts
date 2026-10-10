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

export interface WalState {
  currentLsn: number;
  syncPolicy: 'Always' | 'GroupCommit' | 'Periodic';
  activeSegment: string;
  totalSegments: number;
  segmentSizeBytes: number;
  status: 'active' | 'recovering';
  pendingFsyncCount: number;
  groupCommitBatches: number;
  avgFlushLatencyUs: number;
}

export interface WalRecordView {
  lsn: number;
  epoch: number;
  op: 'SET' | 'DEL' | 'EXPIRE' | 'CHECKPOINT';
  key: string;
  valPreview: string;
  sizeBytes: number;
  crc64Hex: string;
  timestamp: string;
}

export interface WalSegmentView {
  id: number;
  filename: string;
  startLsn: number;
  endLsn: number;
  sizeBytes: number;
  maxSizeBytes: number;
  status: 'active' | 'sealed';
  createdAt: string;
}

export interface WalRecoveryReportView {
  recordsReplayed: number;
  maxLsn: number;
  truncatedBytes: number;
  corruptedSegments: number;
  elapsedMs: number;
  timestamp: string;
}

export type WorkloadMode = 'idle' | 'oltp' | 'stress';


