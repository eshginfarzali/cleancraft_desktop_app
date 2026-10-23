import { invoke } from '@tauri-apps/api/core';

/** Mirrors `cleaner::ScanResult` in src-tauri/src/cleaner.rs. */
export interface ScanResult {
  id: string;
  label: string;
  path: string;
  fileCount: number;
  totalBytes: number;
  skipped: number;
}

/** Mirrors `cleaner::CleanReport`. */
export interface CleanReport {
  id: string;
  deletedFiles: number;
  freedBytes: number;
  failed: number;
}

export const scanTargets = () => invoke<ScanResult[]>('scan_targets');

export const cleanTargets = (ids: string[]) =>
  invoke<CleanReport[]>('clean_targets', { ids });
