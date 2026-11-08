//! User preferences, stored as JSON in the app data folder.

use crate::targets::TargetDef;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Language {
    #[default]
    En,
    Az,
}

/// Weekly automatic clean of the enabled targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Schedule {
    pub enabled: bool,
    /// 0 = Monday … 6 = Sunday.
    pub weekday: u8,
    /// Local hour, 0–23.
    pub hour: u8,
}

impl Default for Schedule {
    fn default() -> Self {
        Schedule {
            enabled: false,
            weekday: 6,
            hour: 10,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// `None` until the user changes anything: the catalog defaults apply.
    pub enabled_targets: Option<Vec<String>>,
    pub schedule: Schedule,
    pub theme: Theme,
    pub language: Language,
}

impl Settings {
    /// Drops unknown target ids and clamps the schedule into range.
    pub fn normalized(mut self, catalog: &[TargetDef]) -> Self {
        if let Some(ids) = self.enabled_targets.as_mut() {
            ids.retain(|id| catalog.iter().any(|t| t.id == id));
            ids.sort();
            ids.dedup();
        }
        self.schedule.weekday = self.schedule.weekday.min(6);
        self.schedule.hour = self.schedule.hour.min(23);
        self
    }

    pub fn is_enabled(&self, target: &TargetDef) -> bool {
        match &self.enabled_targets {
            Some(ids) => ids.iter().any(|id| id == target.id),
            None => target.default_enabled,
        }
    }
}

/// Missing or unreadable settings fall back to the defaults.
pub fn load(path: &Path) -> Settings {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Writes through a temporary file so a crash never leaves half a file.
pub fn save(path: &Path, settings: &Settings) -> std::io::Result<()> {
    write_json_atomic(path, settings)
}

pub fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(value)?)?;
    fs::rename(tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::targets::{catalog, Env, Os};

    fn defs() -> Vec<TargetDef> {
        catalog(Os::Mac, &Env::default())
    }

    #[test]
    fn defaults_enable_only_default_targets() {
        let s = Settings::default();
        let defs = defs();
        let enabled: Vec<_> = defs
            .iter()
            .filter(|t| s.is_enabled(t))
            .map(|t| t.id)
            .collect();
        assert_eq!(enabled, ["temp", "user-cache"]);
    }

    #[test]
    fn explicit_list_overrides_defaults() {
        let s = Settings {
            enabled_targets: Some(vec!["npm".into()]),
            ..Settings::default()
        };
        let defs = defs();
        let enabled: Vec<_> = defs
            .iter()
            .filter(|t| s.is_enabled(t))
            .map(|t| t.id)
            .collect();
        assert_eq!(enabled, ["npm"]);
    }

    #[test]
    fn normalization_drops_unknown_ids_and_clamps_schedule() {
        let s = Settings {
            enabled_targets: Some(vec!["npm".into(), "/etc".into(), "npm".into()]),
            schedule: Schedule {
                enabled: true,
                weekday: 9,
                hour: 99,
            },
            ..Settings::default()
        }
        .normalized(&defs());
        assert_eq!(s.enabled_targets, Some(vec!["npm".to_string()]));
        assert_eq!((s.schedule.weekday, s.schedule.hour), (6, 23));
    }

    #[test]
    fn round_trips_and_tolerates_missing_or_corrupt_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("settings.json");
        assert_eq!(load(&path), Settings::default());

        let s = Settings {
            enabled_targets: Some(vec!["temp".into()]),
            theme: Theme::Dark,
            language: Language::Az,
            schedule: Schedule {
                enabled: true,
                weekday: 2,
                hour: 18,
            },
        };
        save(&path, &s).unwrap();
        assert_eq!(load(&path), s);
        assert!(!path.with_extension("json.tmp").exists());

        fs::write(&path, "{ not json").unwrap();
        assert_eq!(load(&path), Settings::default());
    }

    #[test]
    fn partial_files_keep_defaults_for_missing_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, r#"{ "theme": "light" }"#).unwrap();
        let s = load(&path);
        assert_eq!(s.theme, Theme::Light);
        assert_eq!(s.schedule, Schedule::default());
        assert_eq!(s.enabled_targets, None);
    }
}
