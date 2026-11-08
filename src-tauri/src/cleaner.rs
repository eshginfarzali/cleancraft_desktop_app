//! Finds and deletes stale files inside the allow-listed targets.
//!
//! Safety rules, enforced here rather than in the UI:
//! - only folders from [`crate::targets::catalog`] are touched; the UI sends
//!   target ids, and the paths it sends back from a preview are re-validated;
//! - only regular files whose modification time is older than `min_age` are
//!   deleted (for per-project targets: only units with nothing recent inside);
//! - symlinks are never followed or deleted;
//! - folders that belong to a more specific target are skipped;
//! - files that cannot be read or deleted are skipped and counted.

use crate::targets::{is_real_dir, Granularity, TargetDef};
use serde::Serialize;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Files modified within this window are considered in use and kept.
pub const DEFAULT_MIN_AGE: Duration = Duration::from_secs(24 * 60 * 60);

/// The preview lists at most this many entries (largest first); totals still
/// cover everything.
pub const MAX_PREVIEW_ENTRIES: usize = 2000;

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub id: String,
    pub roots: Vec<PathBuf>,
    /// Files that would be deleted.
    pub file_count: u64,
    /// Total size of those files in bytes.
    pub total_bytes: u64,
    /// Folders or files that could not be read.
    pub skipped: u64,
}

/// One deletable thing: a file, or a whole unit folder for per-project targets.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PreviewEntry {
    pub path: PathBuf,
    pub size: u64,
    /// Number of files (1 for a file entry).
    pub files: u64,
    /// Last modification, milliseconds since the Unix epoch.
    pub modified_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub id: String,
    pub entries: Vec<PreviewEntry>,
    pub total_bytes: u64,
    pub total_files: u64,
    pub truncated: bool,
}

#[derive(Debug, Clone, Default, Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CleanReport {
    pub id: String,
    pub deleted_files: u64,
    pub freed_bytes: u64,
    /// Entries that matched but could not be deleted.
    pub failed: u64,
    /// Selected paths that failed validation (outside the target, recent, symlinked…).
    #[serde(default)]
    pub rejected: u64,
}

/// What a walk over a target found.
#[derive(Debug, Default)]
struct Collected {
    entries: Vec<PreviewEntry>,
    skipped: u64,
}

fn collect(
    target: &TargetDef,
    exclusions: &[PathBuf],
    now: SystemTime,
    min_age: Duration,
) -> Collected {
    let mut out = Collected::default();
    for root in target.existing_roots() {
        match target.granularity {
            Granularity::File => walk_files(root, exclusions, now, min_age, &mut out),
            Granularity::TopLevelDir => collect_units(root, exclusions, now, min_age, &mut out),
        }
    }
    out
}

fn walk_files(
    dir: &Path,
    exclusions: &[PathBuf],
    now: SystemTime,
    min_age: Duration,
    out: &mut Collected,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        out.skipped += 1;
        return;
    };
    for entry in entries {
        let Ok(entry) = entry else {
            out.skipped += 1;
            continue;
        };
        let path = entry.path();
        // symlink_metadata does not follow links.
        let Ok(meta) = fs::symlink_metadata(&path) else {
            out.skipped += 1;
            continue;
        };
        let kind = meta.file_type();
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            if !exclusions.iter().any(|e| e == &path) {
                walk_files(&path, exclusions, now, min_age, out);
            }
        } else if kind.is_file() && is_stale(&meta, now, min_age) {
            out.entries.push(PreviewEntry {
                size: meta.len(),
                files: 1,
                modified_ms: modified_ms(&meta),
                path,
            });
        }
        // Sockets, FIFOs and devices are ignored.
    }
}

/// Totals for a unit folder: (bytes, files, newest modification).
fn unit_stats(dir: &Path) -> Option<(u64, u64, SystemTime)> {
    let mut bytes = 0;
    let mut files = 0;
    let mut newest = fs::symlink_metadata(dir).ok()?.modified().ok()?;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).ok()?.flatten() {
            let meta = fs::symlink_metadata(entry.path()).ok()?;
            if let Ok(m) = meta.modified() {
                newest = newest.max(m);
            }
            let kind = meta.file_type();
            if kind.is_dir() {
                stack.push(entry.path());
            } else if kind.is_file() {
                bytes += meta.len();
                files += 1;
            }
        }
    }
    Some((bytes, files, newest))
}

fn collect_units(
    root: &Path,
    exclusions: &[PathBuf],
    now: SystemTime,
    min_age: Duration,
    out: &mut Collected,
) {
    let Ok(entries) = fs::read_dir(root) else {
        out.skipped += 1;
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !is_real_dir(&path) || exclusions.iter().any(|e| e == &path) {
            continue;
        }
        match unit_stats(&path) {
            Some((bytes, files, newest)) if age_ok(newest, now, min_age) => {
                out.entries.push(PreviewEntry {
                    path,
                    size: bytes,
                    files,
                    modified_ms: system_time_ms(newest),
                })
            }
            Some(_) => {}
            None => out.skipped += 1,
        }
    }
}

pub fn scan(
    target: &TargetDef,
    all: &[TargetDef],
    now: SystemTime,
    min_age: Duration,
) -> ScanResult {
    let collected = collect(target, &target.exclusions(all), now, min_age);
    ScanResult {
        id: target.id.to_string(),
        roots: target.roots.clone(),
        file_count: collected.entries.iter().map(|e| e.files).sum(),
        total_bytes: collected.entries.iter().map(|e| e.size).sum(),
        skipped: collected.skipped,
    }
}

/// Dry run: what a clean would delete, largest first.
pub fn preview(
    target: &TargetDef,
    all: &[TargetDef],
    now: SystemTime,
    min_age: Duration,
) -> Preview {
    let mut collected = collect(target, &target.exclusions(all), now, min_age);
    let total_bytes = collected.entries.iter().map(|e| e.size).sum();
    let total_files = collected.entries.iter().map(|e| e.files).sum();
    collected
        .entries
        .sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.path.cmp(&b.path)));
    let truncated = collected.entries.len() > MAX_PREVIEW_ENTRIES;
    collected.entries.truncate(MAX_PREVIEW_ENTRIES);
    Preview {
        id: target.id.to_string(),
        entries: collected.entries,
        total_bytes,
        total_files,
        truncated,
    }
}

/// Deletes everything a preview of this target would list.
pub fn clean(
    target: &TargetDef,
    all: &[TargetDef],
    now: SystemTime,
    min_age: Duration,
) -> CleanReport {
    let collected = collect(target, &target.exclusions(all), now, min_age);
    let mut report = CleanReport {
        id: target.id.to_string(),
        ..Default::default()
    };
    for entry in &collected.entries {
        delete_entry(target.granularity, entry, &mut report);
    }
    report
}

/// Deletes only the given paths (picked from a preview). Every path is checked
/// again here; anything that does not pass is counted as rejected, not deleted.
pub fn clean_selected(
    target: &TargetDef,
    all: &[TargetDef],
    paths: &[PathBuf],
    now: SystemTime,
    min_age: Duration,
) -> CleanReport {
    let exclusions = target.exclusions(all);
    let mut report = CleanReport {
        id: target.id.to_string(),
        ..Default::default()
    };
    for path in paths {
        match validate(target, &exclusions, path, now, min_age) {
            Some(entry) => delete_entry(target.granularity, &entry, &mut report),
            None => report.rejected += 1,
        }
    }
    report
}

fn delete_entry(granularity: Granularity, entry: &PreviewEntry, report: &mut CleanReport) {
    let result = match granularity {
        Granularity::File => fs::remove_file(&entry.path),
        // remove_dir_all does not follow symlinks inside the tree.
        Granularity::TopLevelDir => fs::remove_dir_all(&entry.path),
    };
    match result {
        Ok(()) => {
            report.deleted_files += entry.files;
            report.freed_bytes += entry.size;
        }
        Err(_) => report.failed += 1,
    }
}

/// Re-checks a path that came from the UI.
fn validate(
    target: &TargetDef,
    exclusions: &[PathBuf],
    path: &Path,
    now: SystemTime,
    min_age: Duration,
) -> Option<PreviewEntry> {
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return None;
    }
    let root = target
        .existing_roots()
        .into_iter()
        .find(|r| path.starts_with(r))?;
    if path == root || exclusions.iter().any(|e| path.starts_with(e)) {
        return None;
    }
    // Every folder between the root and the path must be a real folder, so a
    // symlink inside the target cannot lead outside it.
    let mut ancestor = path.parent()?;
    while ancestor != root {
        if !is_real_dir(ancestor) {
            return None;
        }
        ancestor = ancestor.parent()?;
    }
    match target.granularity {
        Granularity::File => {
            let meta = fs::symlink_metadata(path).ok()?;
            (meta.file_type().is_file() && is_stale(&meta, now, min_age)).then(|| PreviewEntry {
                path: path.to_path_buf(),
                size: meta.len(),
                files: 1,
                modified_ms: modified_ms(&meta),
            })
        }
        Granularity::TopLevelDir => {
            if path.parent() != Some(root) || !is_real_dir(path) {
                return None;
            }
            let (size, files, newest) = unit_stats(path)?;
            age_ok(newest, now, min_age).then(|| PreviewEntry {
                path: path.to_path_buf(),
                size,
                files,
                modified_ms: system_time_ms(newest),
            })
        }
    }
}

fn is_stale(meta: &fs::Metadata, now: SystemTime, min_age: Duration) -> bool {
    meta.modified().is_ok_and(|m| age_ok(m, now, min_age))
}

fn age_ok(modified: SystemTime, now: SystemTime, min_age: Duration) -> bool {
    now.duration_since(modified).is_ok_and(|age| age >= min_age)
}

fn modified_ms(meta: &fs::Metadata) -> u64 {
    meta.modified().map(system_time_ms).unwrap_or(0)
}

fn system_time_ms(t: SystemTime) -> u64 {
    t.duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::targets::Category;
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
        touch(path, age);
    }

    fn touch(path: &Path, age: Duration) {
        let mtime = SystemTime::now() - age;
        set_file_mtime(path, FileTime::from_system_time(mtime)).unwrap();
    }

    fn target(id: &'static str, roots: Vec<PathBuf>, granularity: Granularity) -> TargetDef {
        TargetDef {
            id,
            category: Category::System,
            granularity,
            roots,
            default_enabled: true,
        }
    }

    fn file_target(dir: &Path) -> TargetDef {
        target("test", vec![dir.to_path_buf()], Granularity::File)
    }

    fn now() -> SystemTime {
        SystemTime::now()
    }

    #[test]
    fn scan_counts_only_files_older_than_min_age() {
        let dir = tempfile::tempdir().unwrap();
        write(&dir.path().join("old.log"), 100, 2 * DAY);
        write(&dir.path().join("nested/deep/old.bin"), 50, 3 * DAY);
        write(&dir.path().join("fresh.tmp"), 999, Duration::from_secs(60));
        let t = file_target(dir.path());

        let result = scan(&t, std::slice::from_ref(&t), now(), DAY);

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
        let t = file_target(dir.path());

        let report = clean(&t, std::slice::from_ref(&t), now(), DAY);

        assert_eq!(
            (report.deleted_files, report.freed_bytes, report.failed),
            (1, 10, 0)
        );
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
        std::os::unix::fs::symlink(outside.path(), dir.path().join("link-to-dir")).unwrap();
        std::os::unix::fs::symlink(&precious, dir.path().join("link-to-file")).unwrap();
        let t = file_target(dir.path());

        assert_eq!(scan(&t, std::slice::from_ref(&t), now(), DAY).file_count, 0);
        assert_eq!(
            clean(&t, std::slice::from_ref(&t), now(), DAY).deleted_files,
            0
        );
        assert!(precious.exists());
    }

    #[test]
    fn missing_root_is_simply_empty() {
        let dir = tempfile::tempdir().unwrap();
        let t = file_target(&dir.path().join("does-not-exist"));
        let result = scan(&t, std::slice::from_ref(&t), now(), DAY);
        assert_eq!((result.file_count, result.skipped), (0, 0));
    }

    #[test]
    fn nested_target_is_excluded_from_the_outer_one() {
        let dir = tempfile::tempdir().unwrap();
        let outer_root = dir.path().join("Caches");
        let inner_root = outer_root.join("Google").join("Chrome");
        write(&outer_root.join("app/old.db"), 10, 2 * DAY);
        write(&inner_root.join("Cache/old.blob"), 20, 2 * DAY);
        let outer = target("user-cache", vec![outer_root.clone()], Granularity::File);
        let inner = target("chrome", vec![inner_root.clone()], Granularity::File);
        let all = vec![outer.clone(), inner.clone()];

        assert_eq!(scan(&outer, &all, now(), DAY).total_bytes, 10);
        assert_eq!(scan(&inner, &all, now(), DAY).total_bytes, 20);

        clean(&outer, &all, now(), DAY);
        assert!(
            inner_root.join("Cache/old.blob").exists(),
            "chrome cache kept"
        );
    }

    #[test]
    fn preview_lists_largest_first_with_totals_and_truncation_flag() {
        let dir = tempfile::tempdir().unwrap();
        write(&dir.path().join("small"), 5, 2 * DAY);
        write(&dir.path().join("big"), 50, 2 * DAY);
        write(&dir.path().join("mid"), 20, 2 * DAY);
        let t = file_target(dir.path());

        let p = preview(&t, std::slice::from_ref(&t), now(), DAY);

        let names: Vec<_> = p
            .entries
            .iter()
            .map(|e| e.path.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["big", "mid", "small"]);
        assert_eq!((p.total_bytes, p.total_files, p.truncated), (75, 3, false));
        assert!(p.entries.iter().all(|e| e.modified_ms > 0));
    }

    #[test]
    fn preview_caps_the_list_but_not_the_totals() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..(MAX_PREVIEW_ENTRIES + 5) {
            write(&dir.path().join(format!("f{i}")), 1, 2 * DAY);
        }
        let t = file_target(dir.path());
        let p = preview(&t, std::slice::from_ref(&t), now(), DAY);
        assert_eq!(p.entries.len(), MAX_PREVIEW_ENTRIES);
        assert_eq!(p.total_files, (MAX_PREVIEW_ENTRIES + 5) as u64);
        assert!(p.truncated);
    }

    #[test]
    fn clean_selected_deletes_only_valid_picks() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let keep = root.join("keep.log");
        let pick = root.join("sub/pick.log");
        let fresh = root.join("fresh.log");
        write(&keep, 1, 2 * DAY);
        write(&pick, 7, 2 * DAY);
        write(&fresh, 3, Duration::ZERO);
        let outside = dir.path().join("outside.txt");
        write(&outside, 9, 2 * DAY);
        let t = file_target(&root);

        let report = clean_selected(
            &t,
            std::slice::from_ref(&t),
            &[
                pick.clone(),
                fresh.clone(),
                outside.clone(),
                root.join("sub/../keep.log"),
                PathBuf::from("relative.log"),
                root.clone(),
                root.join("missing.log"),
            ],
            now(),
            DAY,
        );

        assert_eq!(
            (report.deleted_files, report.freed_bytes, report.rejected),
            (1, 7, 6)
        );
        assert!(!pick.exists());
        assert!(keep.exists() && fresh.exists() && outside.exists());
    }

    #[cfg(unix)]
    #[test]
    fn clean_selected_rejects_paths_through_a_symlinked_folder() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        fs::create_dir_all(&root).unwrap();
        let outside = dir.path().join("outside");
        let precious = outside.join("precious.txt");
        write(&precious, 4, 10 * DAY);
        std::os::unix::fs::symlink(&outside, root.join("link")).unwrap();
        let t = file_target(&root);

        let report = clean_selected(
            &t,
            std::slice::from_ref(&t),
            &[root.join("link/precious.txt")],
            now(),
            DAY,
        );

        assert_eq!((report.deleted_files, report.rejected), (0, 1));
        assert!(precious.exists());
    }

    #[test]
    fn clean_selected_rejects_paths_inside_another_target() {
        let dir = tempfile::tempdir().unwrap();
        let outer_root = dir.path().join("Caches");
        let inner_root = outer_root.join("Yarn");
        let blob = inner_root.join("old.tgz");
        write(&blob, 4, 2 * DAY);
        let outer = target("user-cache", vec![outer_root], Granularity::File);
        let inner = target("yarn", vec![inner_root], Granularity::File);
        let all = vec![outer.clone(), inner];

        let report = clean_selected(&outer, &all, std::slice::from_ref(&blob), now(), DAY);

        assert_eq!(report.rejected, 1);
        assert!(blob.exists());
    }

    #[test]
    fn per_project_target_removes_only_fully_stale_projects() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("DerivedData");
        let old_project = root.join("OldApp-abc");
        let active_project = root.join("ActiveApp-def");
        write(&old_project.join("Build/Products/a.o"), 30, 5 * DAY);
        write(&old_project.join("Index/b.idx"), 12, 5 * DAY);
        touch(&old_project.join("Build/Products"), 5 * DAY);
        touch(&old_project.join("Build"), 5 * DAY);
        touch(&old_project.join("Index"), 5 * DAY);
        touch(&old_project, 5 * DAY);
        write(&active_project.join("Build/old.o"), 100, 5 * DAY);
        write(&active_project.join("Build/new.o"), 1, Duration::ZERO);
        let t = target(
            "xcode-derived-data",
            vec![root.clone()],
            Granularity::TopLevelDir,
        );

        let p = preview(&t, std::slice::from_ref(&t), now(), DAY);
        assert_eq!(p.entries.len(), 1);
        assert_eq!(p.entries[0].path, old_project);
        assert_eq!((p.entries[0].size, p.entries[0].files), (42, 2));

        let report = clean(&t, std::slice::from_ref(&t), now(), DAY);
        assert_eq!((report.deleted_files, report.freed_bytes), (2, 42));
        assert!(!old_project.exists());
        assert!(
            active_project.join("Build/old.o").exists(),
            "active project untouched"
        );
    }

    #[test]
    fn per_project_selection_must_be_a_direct_child_of_the_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("DerivedData");
        let project = root.join("App-abc");
        write(&project.join("Build/a.o"), 3, 5 * DAY);
        touch(&project.join("Build"), 5 * DAY);
        touch(&project, 5 * DAY);
        let t = target(
            "xcode-derived-data",
            vec![root.clone()],
            Granularity::TopLevelDir,
        );

        let nested = clean_selected(
            &t,
            std::slice::from_ref(&t),
            &[project.join("Build")],
            now(),
            DAY,
        );
        assert_eq!(nested.rejected, 1);
        assert!(project.exists());

        let ok = clean_selected(
            &t,
            std::slice::from_ref(&t),
            std::slice::from_ref(&project),
            now(),
            DAY,
        );
        assert_eq!((ok.deleted_files, ok.rejected), (1, 0));
        assert!(!project.exists());
    }
}
