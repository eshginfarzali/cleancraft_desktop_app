//! Local clean history (`history.json` in the app data folder).

use crate::cleaner::CleanReport;
use crate::settings::write_json_atomic;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Older entries are dropped beyond this many.
pub const MAX_ENTRIES: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Trigger {
    /// "Clean selected" on whole targets.
    Manual,
    /// Individual files picked in the preview.
    Selective,
    /// The weekly schedule.
    Scheduled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    /// Milliseconds since the Unix epoch.
    pub at_ms: u64,
    pub trigger: Trigger,
    pub reports: Vec<CleanReport>,
    pub freed_bytes: u64,
    pub deleted_files: u64,
    pub failed: u64,
}

impl HistoryEntry {
    pub fn new(at_ms: u64, trigger: Trigger, reports: Vec<CleanReport>) -> Self {
        HistoryEntry {
            at_ms,
            trigger,
            freed_bytes: reports.iter().map(|r| r.freed_bytes).sum(),
            deleted_files: reports.iter().map(|r| r.deleted_files).sum(),
            failed: reports.iter().map(|r| r.failed).sum(),
            reports,
        }
    }
}

/// Newest last. A missing or corrupt file reads as empty history.
pub fn load(path: &Path) -> Vec<HistoryEntry> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn append(path: &Path, entry: HistoryEntry) -> std::io::Result<Vec<HistoryEntry>> {
    let mut entries = load(path);
    entries.push(entry);
    if entries.len() > MAX_ENTRIES {
        entries.drain(..entries.len() - MAX_ENTRIES);
    }
    write_json_atomic(path, &entries)?;
    Ok(entries)
}

pub fn clear(path: &Path) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

/// When the weekly schedule last ran, if ever.
pub fn last_scheduled_at_ms(entries: &[HistoryEntry]) -> Option<u64> {
    entries
        .iter()
        .filter(|e| e.trigger == Trigger::Scheduled)
        .map(|e| e.at_ms)
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(id: &str, files: u64, bytes: u64, failed: u64) -> CleanReport {
        CleanReport {
            id: id.into(),
            deleted_files: files,
            freed_bytes: bytes,
            failed,
            rejected: 0,
        }
    }

    #[test]
    fn entry_totals_sum_its_reports() {
        let e = HistoryEntry::new(
            1,
            Trigger::Manual,
            vec![report("temp", 2, 100, 1), report("npm", 3, 50, 0)],
        );
        assert_eq!((e.deleted_files, e.freed_bytes, e.failed), (5, 150, 1));
    }

    #[test]
    fn append_persists_and_caps_the_log() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.json");
        for i in 0..(MAX_ENTRIES as u64 + 3) {
            append(&path, HistoryEntry::new(i, Trigger::Manual, vec![])).unwrap();
        }
        let entries = load(&path);
        assert_eq!(entries.len(), MAX_ENTRIES);
        assert_eq!(entries.first().unwrap().at_ms, 3, "oldest entries dropped");
        assert_eq!(entries.last().unwrap().at_ms, MAX_ENTRIES as u64 + 2);
    }

    #[test]
    fn corrupt_or_missing_history_reads_as_empty_and_clear_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.json");
        assert!(load(&path).is_empty());
        fs::write(&path, "[{").unwrap();
        assert!(load(&path).is_empty());
        clear(&path).unwrap();
        clear(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn finds_the_last_scheduled_run() {
        let entries = vec![
            HistoryEntry::new(10, Trigger::Scheduled, vec![]),
            HistoryEntry::new(20, Trigger::Manual, vec![]),
            HistoryEntry::new(15, Trigger::Scheduled, vec![]),
        ];
        assert_eq!(last_scheduled_at_ms(&entries), Some(15));
        assert_eq!(last_scheduled_at_ms(&entries[1..2]), None);
    }
}
