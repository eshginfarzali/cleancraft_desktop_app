import type {
  HistoryEntry,
  Preview,
  ScanResult,
  Settings,
  TargetInfo,
} from '../lib/api';

export const settings = (over: Partial<Settings> = {}): Settings => ({
  enabledTargets: null,
  schedule: { enabled: false, weekday: 6, hour: 10 },
  theme: 'system',
  language: 'en',
  ...over,
});

export const targets: TargetInfo[] = [
  {
    id: 'temp',
    category: 'system',
    granularity: 'file',
    roots: ['/tmp'],
    available: true,
    enabled: true,
  },
  {
    id: 'user-cache',
    category: 'system',
    granularity: 'file',
    roots: ['/home/me/.cache'],
    available: true,
    enabled: true,
  },
  {
    id: 'npm',
    category: 'developer',
    granularity: 'file',
    roots: ['/home/me/.npm/_cacache'],
    available: true,
    enabled: false,
  },
  {
    id: 'firefox',
    category: 'browser',
    granularity: 'file',
    roots: [],
    available: false,
    enabled: false,
  },
];

export const scans: ScanResult[] = [
  {
    id: 'temp',
    roots: ['/tmp'],
    fileCount: 3,
    totalBytes: 3 * 1024 ** 2,
    skipped: 0,
  },
  {
    id: 'user-cache',
    roots: ['/home/me/.cache'],
    fileCount: 0,
    totalBytes: 0,
    skipped: 2,
  },
];

export const preview: Preview = {
  id: 'temp',
  entries: [
    {
      path: '/tmp/big.log',
      size: 2 * 1024 ** 2,
      files: 1,
      modifiedMs: Date.UTC(2026, 0, 1),
    },
    {
      path: '/tmp/a/small.tmp',
      size: 1024 ** 2,
      files: 1,
      modifiedMs: Date.UTC(2026, 0, 2),
    },
  ],
  totalBytes: 3 * 1024 ** 2,
  totalFiles: 2,
  truncated: false,
};

export const history: HistoryEntry[] = [
  {
    atMs: Date.UTC(2026, 0, 4, 10),
    trigger: 'scheduled',
    reports: [
      { id: 'temp', deletedFiles: 4, freedBytes: 4096, failed: 0, rejected: 0 },
    ],
    freedBytes: 4096,
    deletedFiles: 4,
    failed: 0,
  },
  {
    atMs: Date.UTC(2026, 0, 6, 9),
    trigger: 'manual',
    reports: [
      {
        id: 'npm',
        deletedFiles: 10,
        freedBytes: 2 * 1024 ** 2,
        failed: 1,
        rejected: 0,
      },
    ],
    freedBytes: 2 * 1024 ** 2,
    deletedFiles: 10,
    failed: 1,
  },
];
