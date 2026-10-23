//! Scans and cleans a fixed allow-list of cache and temp directories.
//!
//! Safety rules, enforced here rather than in the UI:
//! - only the directories returned by [`default_targets`] can be touched;
//!   the frontend sends target ids, never paths;
//! - only regular files whose modification time is older than `min_age`
//!   are deleted, so caches and temp files that are in use are left alone;
//! - symlinks are never followed or deleted, and directories are never removed;
//! - files that cannot be read or deleted (permissions, locks) are skipped and counted.

use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Files modified within this window are considered in use and kept.
pub const DEFAULT_MIN_AGE: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub id: String,
    pub label: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub id: String,
    pub label: String,
    pub path: PathBuf,
    /// Files that would be deleted.
    pub file_count: u64,
    /// Total size of those files in bytes.
    pub total_bytes: u64,
    /// Entries that could not be read.
    pub skipped: u64,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CleanReport {
    pub id: String,
    pub deleted_files: u64,
    pub freed_bytes: u64,
    /// Files that matched but could not be deleted.
    pub failed: u64,
}

/// The allow-list for the current OS. Only directories that exist are returned.
pub fn default_targets() -> Vec<Target> {
    let mut targets = vec![Target {
        id: "temp".into(),
        label: "Temporary files".into(),
        path: std::env::temp_dir(),
    }];
    if let Some(cache) = user_cache_dir() {
        targets.push(Target {
            id: "user-cache".into(),
            label: "Application caches".into(),
            path: cache,
        });
    }
    targets.retain(|t| t.path.is_dir());
    targets
}

/// `~/Library/Caches` on macOS, `$XDG_CACHE_HOME` or `~/.cache` on Linux.
/// Windows has no single per-user cache folder that is safe to empty, so only
/// `%TEMP%` is offered there.
fn user_cache_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    if cfg!(target_os = "macos") {
        home.map(|h| h.join("Library").join("Caches"))
    } else if cfg!(target_os = "linux") {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| home.map(|h| h.join(".cache")))
    } else {
        None
    }
}

pub fn find_target<'a>(targets: &'a [Target], id: &str) -> Option<&'a Target> {
    targets.iter().find(|t| t.id == id)
}

pub fn scan(target: &Target, now: SystemTime, min_age: Duration) -> ScanResult {
    let mut result = ScanResult {
        id: target.id.clone(),
        label: target.label.clone(),
        path: target.path.clone(),
        ..Default::default()
    };
    walk(
        &target.path,
        &mut |entry| match entry {
            Entry::Stale { size, .. } => {
                result.file_count += 1;
                result.total_bytes += size;
            }
            Entry::Unreadable => result.skipped += 1,
        },
        now,
        min_age,
    );
    result
}

pub fn clean(target: &Target, now: SystemTime, min_age: Duration) -> CleanReport {
    let mut report = CleanReport {
        id: target.id.clone(),
        ..Default::default()
    };
    walk(
        &target.path,
        &mut |entry| {
            if let Entry::Stale { path, size } = entry {
                match fs::remove_file(path) {
                    Ok(()) => {
                        report.deleted_files += 1;
                        report.freed_bytes += size;
                    }
                    Err(_) => report.failed += 1,
                }
            }
        },
        now,
        min_age,
    );
    report
}

enum Entry<'a> {
    /// A regular file older than `min_age`.
    Stale {
        path: &'a Path,
        size: u64,
    },
    Unreadable,
}

fn walk(dir: &Path, visit: &mut dyn FnMut(Entry), now: SystemTime, min_age: Duration) {
    let Ok(entries) = fs::read_dir(dir) else {
        visit(Entry::Unreadable);
        return;
    };
    for entry in entries {
        let Ok(entry) = entry else {
            visit(Entry::Unreadable);
            continue;
        };
        let path = entry.path();
        // symlink_metadata does not follow links.
        let Ok(meta) = fs::symlink_metadata(&path) else {
            visit(Entry::Unreadable);
            continue;
        };
        let kind = meta.file_type();
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            walk(&path, visit, now, min_age);
        } else if kind.is_file() && is_stale(&meta, now, min_age) {
            visit(Entry::Stale {
                path: &path,
                size: meta.len(),
            });
        }
        // Sockets, FIFOs and devices are ignored.
    }
}

fn is_stale(meta: &fs::Metadata, now: SystemTime, min_age: Duration) -> bool {
    meta.modified()
        .ok()
        .and_then(|modified| now.duration_since(modified).ok())
        .is_some_and(|age| age >= min_age)
}

#[cfg(test)]
mod tests {
    use super::*;
    use filetime::{set_file_mtime, FileTime};
    use std::fs::File;
    use std::io::Write;

    const DAY: Duration = Duration::from_secs(24 * 60 * 60);

    fn write(path: &Path, bytes: usize, age: Duration) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        File::create(path)
            .unwrap()
            .write_all(&vec![0u8; bytes])
            .unwrap();
        let mtime = SystemTime::now() - age;
        set_file_mtime(path, FileTime::from_system_time(mtime)).unwrap();
    }

    fn target(dir: &Path) -> Target {
        Target {
            id: "test".into(),
            label: "Test".into(),
            path: dir.to_path_buf(),
        }
    }

    #[test]
    fn scan_counts_only_files_older_than_min_age() {
        let dir = tempfile::tempdir().unwrap();
        write(&dir.path().join("old.log"), 100, 2 * DAY);
        write(&dir.path().join("nested/deep/old.bin"), 50, 3 * DAY);
        write(&dir.path().join("fresh.tmp"), 999, Duration::from_secs(60));

        let result = scan(&target(dir.path()), SystemTime::now(), DAY);

        assert_eq!(result.file_count, 2);
        assert_eq!(result.total_bytes, 150);
        assert_eq!(result.skipped, 0);
    }

    #[test]
    fn clean_deletes_stale_files_and_keeps_fresh_files_and_directories() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("a/old.cache");
        let fresh = dir.path().join("a/fresh.cache");
        write(&old, 10, 2 * DAY);
        write(&fresh, 20, Duration::ZERO);

        let report = clean(&target(dir.path()), SystemTime::now(), DAY);

        assert_eq!(report.deleted_files, 1);
        assert_eq!(report.freed_bytes, 10);
        assert_eq!(report.failed, 0);
        assert!(!old.exists());
        assert!(fresh.exists());
        assert!(dir.path().join("a").is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_never_followed_or_deleted() {
        let outside = tempfile::tempdir().unwrap();
        let precious = outside.path().join("precious.txt");
        write(&precious, 5, 10 * DAY);

        let dir = tempfile::tempdir().unwrap();
        let dir_link = dir.path().join("link-to-dir");
        let file_link = dir.path().join("link-to-file");
        std::os::unix::fs::symlink(outside.path(), &dir_link).unwrap();
        std::os::unix::fs::symlink(&precious, &file_link).unwrap();

        let scanned = scan(&target(dir.path()), SystemTime::now(), DAY);
        let report = clean(&target(dir.path()), SystemTime::now(), DAY);

        assert_eq!(scanned.file_count, 0);
        assert_eq!(report.deleted_files, 0);
        assert!(precious.exists());
        assert!(fs::symlink_metadata(&dir_link).is_ok());
        assert!(fs::symlink_metadata(&file_link).is_ok());
    }

    #[test]
    fn missing_directory_is_reported_as_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let gone = dir.path().join("does-not-exist");

        let result = scan(&target(&gone), SystemTime::now(), DAY);

        assert_eq!(result.file_count, 0);
        assert_eq!(result.skipped, 1);
    }

    #[test]
    fn default_targets_are_absolute_existing_directories() {
        let targets = default_targets();
        assert!(targets.iter().any(|t| t.id == "temp"));
        for t in &targets {
            assert!(t.path.is_absolute(), "{:?}", t.path);
            assert!(t.path.is_dir(), "{:?}", t.path);
        }
    }

    #[test]
    fn find_target_only_matches_allow_listed_ids() {
        let targets = vec![target(Path::new("/tmp"))];
        assert!(find_target(&targets, "test").is_some());
        assert!(find_target(&targets, "/etc").is_none());
    }
}
