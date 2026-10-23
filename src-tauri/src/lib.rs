mod cleaner;

use cleaner::{CleanReport, ScanResult, DEFAULT_MIN_AGE};
use std::time::SystemTime;

/// Scans every allow-listed directory and reports what a clean would remove.
#[tauri::command]
async fn scan_targets() -> Result<Vec<ScanResult>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let now = SystemTime::now();
        cleaner::default_targets()
            .iter()
            .map(|t| cleaner::scan(t, now, DEFAULT_MIN_AGE))
            .collect()
    })
    .await
    .map_err(|e| e.to_string())
}

/// Cleans the targets with the given ids. Paths are never accepted from the UI.
#[tauri::command]
async fn clean_targets(ids: Vec<String>) -> Result<Vec<CleanReport>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let targets = cleaner::default_targets();
        let now = SystemTime::now();
        ids.iter()
            .map(|id| {
                cleaner::find_target(&targets, id)
                    .map(|t| cleaner::clean(t, now, DEFAULT_MIN_AGE))
                    .ok_or_else(|| format!("Unknown target: {id}"))
            })
            .collect()
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![scan_targets, clean_targets])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
