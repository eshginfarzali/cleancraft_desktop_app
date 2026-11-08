//! The allow-list: every folder CleanCraft may ever touch, per operating system.
//!
//! Targets are defined here and only here. The UI refers to them by id and can
//! never add a path. Each target can be switched on or off in Settings.

use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Category {
    System,
    Browser,
    Developer,
}

/// How a target is cleaned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Granularity {
    /// Individual stale files are deleted; folders are kept.
    File,
    /// Each direct sub-folder is one unit (e.g. one Xcode project's build data).
    /// A unit is removed as a whole, and only when nothing inside it is recent,
    /// so a project that is being built is never left half-deleted.
    TopLevelDir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Mac,
    Linux,
    Windows,
}

impl Os {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Os::Mac
        } else if cfg!(target_os = "windows") {
            Os::Windows
        } else {
            Os::Linux
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetDef {
    pub id: &'static str,
    pub category: Category,
    pub granularity: Granularity,
    /// Absolute folders. Some targets have several (one per browser profile).
    pub roots: Vec<PathBuf>,
    /// Off by default: the user opts in from Settings.
    pub default_enabled: bool,
}

/// The parts of the environment the catalog depends on, so tests can point
/// everything at a temporary directory.
#[derive(Debug, Clone, Default)]
pub struct Env {
    pub home: Option<PathBuf>,
    pub temp: Option<PathBuf>,
    pub local_app_data: Option<PathBuf>,
    pub xdg_cache_home: Option<PathBuf>,
    pub xdg_data_home: Option<PathBuf>,
}

impl Env {
    pub fn current() -> Self {
        let abs = |name: &str| {
            std::env::var_os(name)
                .map(PathBuf::from)
                .filter(|p| p.is_absolute())
        };
        Env {
            home: abs("HOME").or_else(|| abs("USERPROFILE")),
            temp: Some(std::env::temp_dir()).filter(|p| p.is_absolute()),
            local_app_data: abs("LOCALAPPDATA"),
            xdg_cache_home: abs("XDG_CACHE_HOME"),
            xdg_data_home: abs("XDG_DATA_HOME"),
        }
    }
}

pub const TEMP: &str = "temp";
pub const USER_CACHE: &str = "user-cache";

/// Every target defined for `os`, including ones whose folders do not exist
/// on this machine (the UI shows them as unavailable).
pub fn catalog(os: Os, env: &Env) -> Vec<TargetDef> {
    let mut defs = Vec::new();
    let mut add = |id, category, granularity, roots: Vec<PathBuf>, default_enabled| {
        defs.push(TargetDef {
            id,
            category,
            granularity,
            roots,
            default_enabled,
        })
    };
    use Category::*;
    use Granularity::*;

    add(TEMP, System, File, env.temp.iter().cloned().collect(), true);

    let home = env.home.as_deref();
    let under_home = |rel: &[&str]| home.map(|h| join(h, rel)).into_iter().collect::<Vec<_>>();

    match os {
        Os::Mac => {
            let caches = home.map(|h| h.join("Library").join("Caches"));
            let in_caches = |rel: &[&str]| caches.iter().map(|c| join(c, rel)).collect::<Vec<_>>();
            add(
                USER_CACHE,
                System,
                File,
                caches.iter().cloned().collect(),
                true,
            );
            add(
                "chrome",
                Browser,
                File,
                in_caches(&["Google", "Chrome"]),
                false,
            );
            add("edge", Browser, File, in_caches(&["Microsoft Edge"]), false);
            add(
                "firefox",
                Browser,
                File,
                profiles(caches.as_deref(), &["Firefox", "Profiles"], "cache2"),
                false,
            );
            add(
                "npm",
                Developer,
                File,
                under_home(&[".npm", "_cacache"]),
                false,
            );
            add("yarn", Developer, File, in_caches(&["Yarn"]), false);
            add(
                "pnpm",
                Developer,
                File,
                under_home(&["Library", "pnpm", "store"]),
                false,
            );
            add(
                "xcode-derived-data",
                Developer,
                TopLevelDir,
                under_home(&["Library", "Developer", "Xcode", "DerivedData"]),
                false,
            );
        }
        Os::Linux => {
            let cache = env
                .xdg_cache_home
                .clone()
                .or_else(|| home.map(|h| h.join(".cache")));
            let data = env
                .xdg_data_home
                .clone()
                .or_else(|| home.map(|h| join(h, &[".local", "share"])));
            let in_cache = |rel: &[&str]| cache.iter().map(|c| join(c, rel)).collect::<Vec<_>>();
            add(
                USER_CACHE,
                System,
                File,
                cache.iter().cloned().collect(),
                true,
            );
            add("chrome", Browser, File, in_cache(&["google-chrome"]), false);
            add("edge", Browser, File, in_cache(&["microsoft-edge"]), false);
            add(
                "firefox",
                Browser,
                File,
                profiles(cache.as_deref(), &["mozilla", "firefox"], "cache2"),
                false,
            );
            add(
                "npm",
                Developer,
                File,
                under_home(&[".npm", "_cacache"]),
                false,
            );
            add("yarn", Developer, File, in_cache(&["yarn"]), false);
            add(
                "pnpm",
                Developer,
                File,
                data.iter().map(|d| join(d, &["pnpm", "store"])).collect(),
                false,
            );
        }
        Os::Windows => {
            let lad = env.local_app_data.as_deref();
            let in_lad = |rel: &[&str]| lad.map(|l| join(l, rel)).into_iter().collect::<Vec<_>>();
            // %LOCALAPPDATA% as a whole holds application data, not just caches,
            // so only these well-known cache folders inside it are cleaned.
            let mut user_cache = in_lad(&["Microsoft", "Windows", "INetCache"]);
            user_cache.extend(in_lad(&["CrashDumps"]));
            user_cache.extend(in_lad(&["D3DSCache"]));
            add(USER_CACHE, System, File, user_cache, true);
            add(
                "chrome",
                Browser,
                File,
                profiles(lad, &["Google", "Chrome", "User Data"], "Cache"),
                false,
            );
            add(
                "edge",
                Browser,
                File,
                profiles(lad, &["Microsoft", "Edge", "User Data"], "Cache"),
                false,
            );
            add(
                "firefox",
                Browser,
                File,
                profiles(lad, &["Mozilla", "Firefox", "Profiles"], "cache2"),
                false,
            );
            add(
                "npm",
                Developer,
                File,
                in_lad(&["npm-cache", "_cacache"]),
                false,
            );
            add("yarn", Developer, File, in_lad(&["Yarn", "Cache"]), false);
            add("pnpm", Developer, File, in_lad(&["pnpm", "store"]), false);
        }
    }
    defs
}

fn join(base: &Path, rel: &[&str]) -> PathBuf {
    rel.iter().fold(base.to_path_buf(), |p, part| p.join(part))
}

/// `<base>/<rel>/<each profile>/<leaf>` for every existing profile folder.
/// Symlinked profile folders are ignored.
fn profiles(base: Option<&Path>, rel: &[&str], leaf: &str) -> Vec<PathBuf> {
    let Some(base) = base else { return Vec::new() };
    let dir = join(base, rel);
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut roots: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path().join(leaf))
        .filter(|p| is_real_dir(p))
        .collect();
    roots.sort();
    roots
}

/// A directory that is not a symlink.
pub fn is_real_dir(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|m| m.file_type().is_dir())
        .unwrap_or(false)
}

impl TargetDef {
    /// Roots that exist on this machine.
    pub fn existing_roots(&self) -> Vec<&Path> {
        self.roots
            .iter()
            .map(PathBuf::as_path)
            .filter(|p| is_real_dir(p))
            .collect()
    }

    /// Folders inside this target that belong to another, more specific target
    /// (e.g. Chrome's cache inside `~/Library/Caches`). They are skipped so
    /// every file is counted and cleaned under exactly one target, and turning
    /// a browser off really keeps its cache.
    pub fn exclusions(&self, all: &[TargetDef]) -> Vec<PathBuf> {
        all.iter()
            .filter(|other| other.id != self.id)
            .flat_map(|other| other.roots.iter())
            .filter(|root| {
                self.roots
                    .iter()
                    .any(|mine| root.starts_with(mine) && root.as_path() != mine.as_path())
            })
            .cloned()
            .collect()
    }
}

pub fn find<'a>(all: &'a [TargetDef], id: &str) -> Option<&'a TargetDef> {
    all.iter().find(|t| t.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_in(dir: &Path) -> Env {
        Env {
            home: Some(dir.join("home")),
            temp: Some(dir.join("tmp")),
            local_app_data: Some(dir.join("lad")),
            xdg_cache_home: None,
            xdg_data_home: None,
        }
    }

    fn ids(defs: &[TargetDef]) -> Vec<&str> {
        defs.iter().map(|d| d.id).collect()
    }

    #[test]
    fn every_os_has_temp_and_user_cache_enabled_by_default() {
        let dir = tempfile::tempdir().unwrap();
        for os in [Os::Mac, Os::Linux, Os::Windows] {
            let defs = catalog(os, &env_in(dir.path()));
            for id in [TEMP, USER_CACHE] {
                let def = find(&defs, id).unwrap();
                assert!(def.default_enabled, "{os:?} {id}");
            }
            assert!(
                defs.iter()
                    .filter(|d| d.id != TEMP && d.id != USER_CACHE)
                    .all(|d| !d.default_enabled),
                "extra targets must be opt-in on {os:?}"
            );
        }
    }

    #[test]
    fn target_ids_are_unique_and_paths_absolute() {
        let dir = tempfile::tempdir().unwrap();
        for os in [Os::Mac, Os::Linux, Os::Windows] {
            let defs = catalog(os, &env_in(dir.path()));
            let mut seen = ids(&defs);
            seen.sort();
            seen.dedup();
            assert_eq!(seen.len(), defs.len());
            for root in defs.iter().flat_map(|d| d.roots.iter()) {
                assert!(root.is_absolute(), "{root:?}");
            }
        }
    }

    #[test]
    fn xcode_derived_data_is_macos_only_and_cleaned_per_project() {
        let dir = tempfile::tempdir().unwrap();
        let mac = catalog(Os::Mac, &env_in(dir.path()));
        let xcode = find(&mac, "xcode-derived-data").unwrap();
        assert_eq!(xcode.granularity, Granularity::TopLevelDir);
        assert!(xcode.roots[0].ends_with("Library/Developer/Xcode/DerivedData"));
        for os in [Os::Linux, Os::Windows] {
            assert!(find(&catalog(os, &env_in(dir.path())), "xcode-derived-data").is_none());
        }
    }

    #[test]
    fn windows_user_cache_uses_known_folders_not_all_of_localappdata() {
        let dir = tempfile::tempdir().unwrap();
        let env = env_in(dir.path());
        let defs = catalog(Os::Windows, &env);
        let roots = &find(&defs, USER_CACHE).unwrap().roots;
        let lad = env.local_app_data.unwrap();
        assert!(roots.iter().all(|r| r.starts_with(&lad) && r != &lad));
        assert!(roots.iter().any(|r| r.ends_with("INetCache")));
    }

    #[test]
    fn browser_profiles_are_discovered_and_symlinked_profiles_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let env = env_in(dir.path());
        let lad = env.local_app_data.clone().unwrap();
        let user_data = lad.join("Google").join("Chrome").join("User Data");
        fs::create_dir_all(user_data.join("Default").join("Cache")).unwrap();
        fs::create_dir_all(user_data.join("Profile 1").join("Cache")).unwrap();
        fs::create_dir_all(user_data.join("NoCache")).unwrap();
        #[cfg(unix)]
        {
            let outside = dir.path().join("outside");
            fs::create_dir_all(outside.join("Cache")).unwrap();
            std::os::unix::fs::symlink(&outside, user_data.join("Linked")).unwrap();
        }

        let defs = catalog(Os::Windows, &env);
        let chrome = &find(&defs, "chrome").unwrap().roots;
        assert_eq!(
            chrome,
            &vec![
                user_data.join("Default").join("Cache"),
                user_data.join("Profile 1").join("Cache"),
            ]
        );
    }

    #[test]
    fn linux_honours_xdg_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let mut env = env_in(dir.path());
        env.xdg_cache_home = Some(dir.path().join("xc"));
        env.xdg_data_home = Some(dir.path().join("xd"));
        let defs = catalog(Os::Linux, &env);
        assert_eq!(
            find(&defs, USER_CACHE).unwrap().roots,
            vec![dir.path().join("xc")]
        );
        assert_eq!(
            find(&defs, "pnpm").unwrap().roots,
            vec![dir.path().join("xd").join("pnpm").join("store")]
        );
    }

    #[test]
    fn nested_targets_are_excluded_from_the_enclosing_target() {
        let dir = tempfile::tempdir().unwrap();
        let defs = catalog(Os::Mac, &env_in(dir.path()));
        let user_cache = find(&defs, USER_CACHE).unwrap();
        let excluded = user_cache.exclusions(&defs);
        let caches = dir.path().join("home").join("Library").join("Caches");
        assert!(excluded.contains(&caches.join("Google").join("Chrome")));
        assert!(excluded.contains(&caches.join("Yarn")));
        // Targets outside ~/Library/Caches are not exclusions of user-cache.
        assert!(!excluded.iter().any(|p| p.ends_with("DerivedData")));
        // A leaf target excludes nothing.
        assert!(find(&defs, "chrome").unwrap().exclusions(&defs).is_empty());
    }

    #[test]
    fn missing_home_yields_targets_without_roots() {
        let defs = catalog(Os::Mac, &Env::default());
        assert!(find(&defs, "npm").unwrap().roots.is_empty());
        assert!(find(&defs, TEMP).unwrap().roots.is_empty());
    }
}
