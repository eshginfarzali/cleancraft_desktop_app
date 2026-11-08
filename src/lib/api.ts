import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// Types mirror the Rust structs in src-tauri/src (serde camelCase).

export type Category = 'system' | 'browser' | 'developer';
export type Granularity = 'file' | 'topLevelDir';
export type Theme = 'system' | 'light' | 'dark';
export type Language = 'en' | 'az';
export type Trigger = 'manual' | 'selective' | 'scheduled';

export interface TargetInfo {
  id: string;
  category: Category;
  granularity: Granularity;
  roots: string[];
  available: boolean;
  enabled: boolean;
}

export interface ScanResult {
  id: string;
  roots: string[];
  fileCount: number;
  totalBytes: number;
  skipped: number;
}

export interface PreviewEntry {
  path: string;
  size: number;
  files: number;
  modifiedMs: number;
}

export interface Preview {
  id: string;
  entries: PreviewEntry[];
  totalBytes: number;
  totalFiles: number;
  truncated: boolean;
}

export interface CleanReport {
  id: string;
  deletedFiles: number;
  freedBytes: number;
  failed: number;
  rejected: number;
}

export interface HistoryEntry {
  atMs: number;
  trigger: Trigger;
  reports: CleanReport[];
  freedBytes: number;
  deletedFiles: number;
  failed: number;
}

export interface Schedule {
  enabled: boolean;
  /** 0 = Monday … 6 = Sunday */
  weekday: number;
  hour: number;
}

export interface Settings {
  /** null until the user changes anything: the built-in defaults apply. */
  enabledTargets: string[] | null;
  schedule: Schedule;
  theme: Theme;
  language: Language;
}

export const getTargets = () => invoke<TargetInfo[]>('get_targets');
export const scanTargets = () => invoke<ScanResult[]>('scan_targets');
export const previewTarget = (id: string) =>
  invoke<Preview>('preview_target', { id });
export const cleanTargets = (ids: string[]) =>
  invoke<CleanReport[]>('clean_targets', { ids });
/** Paths come from a preview; the backend validates each one again. */
export const cleanSelected = (id: string, paths: string[]) =>
  invoke<CleanReport>('clean_selected', { id, paths });
export const getHistory = () => invoke<HistoryEntry[]>('get_history');
export const clearHistory = () => invoke<void>('clear_history');
export const getSettings = () => invoke<Settings>('get_settings');
export const saveSettings = (settings: Settings) =>
  invoke<Settings>('save_settings', { settings });

/** Fired after any clean, including the weekly schedule and the tray menu. */
export const onHistoryUpdated = (handler: () => void): Promise<UnlistenFn> =>
  listen('history-updated', handler);
