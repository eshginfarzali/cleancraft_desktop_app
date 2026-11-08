mod cleaner;
mod history;
mod schedule;
mod settings;
mod targets;

use cleaner::{CleanReport, Preview, ScanResult, DEFAULT_MIN_AGE};
use history::{HistoryEntry, Trigger};
use serde::Serialize;
use settings::{Language, Settings};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use targets::{Category, Env, Granularity, Os, TargetDef};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent, Wry};

/// Emitted after any clean (UI, tray or schedule) so the UI can refresh.
const HISTORY_UPDATED: &str = "history-updated";
const TRAY_ID: &str = "main";
const SCHEDULE_POLL: Duration = Duration::from_secs(60);

struct AppState {
    data_dir: PathBuf,
    /// Serialises cleans so the schedule, the tray and the UI never overlap.
    cleaning: Mutex<()>,
}

impl AppState {
    fn settings_path(&self) -> PathBuf {
        self.data_dir.join("settings.json")
    }
    fn history_path(&self) -> PathBuf {
        self.data_dir.join("history.json")
    }
    fn settings(&self, catalog: &[TargetDef]) -> Settings {
        settings::load(&self.settings_path()).normalized(catalog)
    }
}

fn catalog() -> Vec<TargetDef> {
    targets::catalog(Os::current(), &Env::current())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TargetInfo {
    id: &'static str,
    category: Category,
    granularity: Granularity,
    roots: Vec<PathBuf>,
    /// At least one of its folders exists on this computer.
    available: bool,
    enabled: bool,
}

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

/// Looks up an enabled target by id. The UI never sends paths for targets.
fn enabled_target<'a>(
    catalog: &'a [TargetDef],
    settings: &Settings,
    id: &str,
) -> Result<&'a TargetDef, String> {
    let target = targets::find(catalog, id).ok_or_else(|| format!("Unknown target: {id}"))?;
    if settings.is_enabled(target) {
        Ok(target)
    } else {
        Err(format!("Target is turned off: {id}"))
    }
}

/// Cleans the given targets (or every enabled one) and records it in the history.
fn run_clean(
    app: &AppHandle,
    ids: Option<Vec<String>>,
    trigger: Trigger,
) -> Result<Vec<CleanReport>, String> {
    let state = app.state::<AppState>();
    let _guard = state.cleaning.lock().map_err(|e| e.to_string())?;
    let catalog = catalog();
    let settings = state.settings(&catalog);
    let chosen: Vec<&TargetDef> = match ids {
        Some(ids) => ids
            .iter()
            .map(|id| enabled_target(&catalog, &settings, id))
            .collect::<Result<_, _>>()?,
        None => catalog.iter().filter(|t| settings.is_enabled(t)).collect(),
    };
    let now = SystemTime::now();
    let reports: Vec<CleanReport> = chosen
        .iter()
        .map(|t| cleaner::clean(t, &catalog, now, DEFAULT_MIN_AGE))
        .collect();
    record(app, &state, trigger, reports.clone())?;
    Ok(reports)
}

fn record(
    app: &AppHandle,
    state: &AppState,
    trigger: Trigger,
    reports: Vec<CleanReport>,
) -> Result<(), String> {
    history::append(
        &state.history_path(),
        HistoryEntry::new(now_ms(), trigger, reports),
    )
    .map_err(|e| e.to_string())?;
    let _ = app.emit(HISTORY_UPDATED, ());
    Ok(())
}

#[tauri::command]
async fn get_targets(state: State<'_, AppState>) -> Result<Vec<TargetInfo>, String> {
    let catalog = catalog();
    let settings = state.settings(&catalog);
    Ok(catalog
        .iter()
        .map(|t| TargetInfo {
            id: t.id,
            category: t.category,
            granularity: t.granularity,
            roots: t.roots.clone(),
            available: !t.existing_roots().is_empty(),
            enabled: settings.is_enabled(t),
        })
        .collect())
}

/// Scans every enabled target that exists on this computer.
#[tauri::command]
async fn scan_targets(state: State<'_, AppState>) -> Result<Vec<ScanResult>, String> {
    let settings_path = state.settings_path();
    blocking(move || {
        let catalog = catalog();
        let settings = settings::load(&settings_path).normalized(&catalog);
        let now = SystemTime::now();
        Ok(catalog
            .iter()
            .filter(|t| settings.is_enabled(t) && !t.existing_roots().is_empty())
            .map(|t| cleaner::scan(t, &catalog, now, DEFAULT_MIN_AGE))
            .collect())
    })
    .await
}

/// Dry run for one target: the files (or project folders) a clean would delete.
#[tauri::command]
async fn preview_target(state: State<'_, AppState>, id: String) -> Result<Preview, String> {
    let settings_path = state.settings_path();
    blocking(move || {
        let catalog = catalog();
        let settings = settings::load(&settings_path).normalized(&catalog);
        let target = enabled_target(&catalog, &settings, &id)?;
        Ok(cleaner::preview(
            target,
            &catalog,
            SystemTime::now(),
            DEFAULT_MIN_AGE,
        ))
    })
    .await
}

/// Cleans whole targets by id.
#[tauri::command]
async fn clean_targets(app: AppHandle, ids: Vec<String>) -> Result<Vec<CleanReport>, String> {
    blocking(move || run_clean(&app, Some(ids), Trigger::Manual)).await
}

/// Deletes only the entries picked in a preview. Each path is re-validated in Rust.
#[tauri::command]
async fn clean_selected(
    app: AppHandle,
    id: String,
    paths: Vec<PathBuf>,
) -> Result<CleanReport, String> {
    blocking(move || {
        let state = app.state::<AppState>();
        let _guard = state.cleaning.lock().map_err(|e| e.to_string())?;
        let catalog = catalog();
        let settings = state.settings(&catalog);
        let target = enabled_target(&catalog, &settings, &id)?;
        let report =
            cleaner::clean_selected(target, &catalog, &paths, SystemTime::now(), DEFAULT_MIN_AGE);
        record(&app, &state, Trigger::Selective, vec![report.clone()])?;
        Ok(report)
    })
    .await
}

#[tauri::command]
async fn get_history(state: State<'_, AppState>) -> Result<Vec<HistoryEntry>, String> {
    Ok(history::load(&state.history_path()))
}

#[tauri::command]
async fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    history::clear(&state.history_path()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    Ok(state.settings(&catalog()))
}

#[tauri::command]
async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Settings, String> {
    let normalized = settings.normalized(&catalog());
    settings::save(&state.settings_path(), &normalized).map_err(|e| e.to_string())?;
    let _ = refresh_tray_menu(&app, normalized.language);
    Ok(normalized)
}

struct TrayLabels {
    open: &'static str,
    clean: &'static str,
    quit: &'static str,
}

fn tray_labels(language: Language) -> TrayLabels {
    match language {
        Language::En => TrayLabels {
            open: "Open CleanCraft",
            clean: "Clean enabled targets now",
            quit: "Quit",
        },
        Language::Az => TrayLabels {
            open: "CleanCraft-ı aç",
            clean: "Aktiv hədəfləri indi təmizlə",
            quit: "Çıxış",
        },
    }
}

fn tray_menu(app: &AppHandle, language: Language) -> tauri::Result<Menu<Wry>> {
    let labels = tray_labels(language);
    let open = MenuItem::with_id(app, "open", labels.open, true, None::<&str>)?;
    let clean = MenuItem::with_id(app, "clean", labels.clean, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", labels.quit, true, None::<&str>)?;
    Menu::with_items(app, &[&open, &clean, &quit])
}

fn refresh_tray_menu(app: &AppHandle, language: Language) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(tray_menu(app, language)?))?;
    }
    Ok(())
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn setup_tray(app: &AppHandle, language: Language) -> tauri::Result<()> {
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("CleanCraft")
        .menu(&tray_menu(app, language)?)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main_window(app),
            "clean" => {
                let app = app.clone();
                std::thread::spawn(move || {
                    let _ = run_clean(&app, None, Trigger::Manual);
                });
            }
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

/// Checks once a minute whether the weekly clean is due. Runs while the app is
/// open or minimised to the tray.
fn start_scheduler(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(SCHEDULE_POLL);
        let state = app.state::<AppState>();
        let schedule = state.settings(&catalog()).schedule;
        let last_run = history::last_scheduled_at_ms(&history::load(&state.history_path()))
            .and_then(local_time_from_ms);
        let now = chrono::Local::now().naive_local();
        if schedule::is_due(now, last_run, &schedule) {
            let _ = run_clean(&app, None, Trigger::Scheduled);
        }
    });
}

fn local_time_from_ms(ms: u64) -> Option<chrono::NaiveDateTime> {
    chrono::DateTime::from_timestamp_millis(ms as i64)
        .map(|utc| utc.with_timezone(&chrono::Local).naive_local())
}

fn data_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir().join("cleancraft"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let state = AppState {
                data_dir: data_dir(&handle),
                cleaning: Mutex::new(()),
            };
            let language = state.settings(&catalog()).language;
            app.manage(state);
            setup_tray(&handle, language)?;
            start_scheduler(handle);
            Ok(())
        })
        // Closing the window keeps CleanCraft in the tray so the schedule can run.
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_targets,
            scan_targets,
            preview_target,
            clean_targets,
            clean_selected,
            get_history,
            clear_history,
            get_settings,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_labels_exist_for_every_language() {
        for language in [Language::En, Language::Az] {
            let labels = tray_labels(language);
            assert!(!labels.open.is_empty() && !labels.clean.is_empty() && !labels.quit.is_empty());
        }
        assert_ne!(
            tray_labels(Language::En).quit,
            tray_labels(Language::Az).quit
        );
    }

    #[test]
    fn enabled_target_rejects_unknown_and_disabled_ids() {
        let catalog = targets::catalog(Os::Mac, &Env::default());
        let settings = Settings::default();
        assert!(enabled_target(&catalog, &settings, "temp").is_ok());
        assert!(enabled_target(&catalog, &settings, "npm")
            .unwrap_err()
            .contains("turned off"));
        assert!(enabled_target(&catalog, &settings, "../etc")
            .unwrap_err()
            .contains("Unknown"));
    }

    #[test]
    fn converts_history_timestamps_to_local_time() {
        assert!(local_time_from_ms(0).is_some());
    }
}
