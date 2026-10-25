# Changelog

All notable changes to CleanCraft are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses [Semantic Versioning](https://semver.org/).

## [0.2.0] - 2026-10-25

### Added

- **Dry-run preview** for each target: the files a clean would delete, largest first, with size and last-modified date. Pick individual files and delete only those. Every picked path is validated again in Rust before deletion.
- **Freeable total** after a scan, and a **clean history** stored locally in `history.json` (last 200 cleans), with the trigger, the space freed, the files deleted, the failures and a running total.
- **New targets**, each switched on or off separately and all off by default:
  - Google Chrome, Microsoft Edge and Firefox caches
  - npm, Yarn and pnpm caches
  - Xcode DerivedData, cleaned per project folder, and only when the whole project is idle for 24 hours
- **Windows application caches**: well-known cache folders inside `%LOCALAPPDATA%` (`INetCache`, `CrashDumps`, `D3DSCache`).
- **System tray** with Open, Clean now and Quit. Closing the window keeps the app running in the tray.
- **Weekly scheduled clean** at a chosen day and hour. It catches up on a missed slot the next time the app runs.
- **Light, dark and system themes**, and an **English / Azerbaijani** UI. The tray menu follows the selected language.
- **Settings** stored in `settings.json`, normalised in Rust (unknown target ids dropped, schedule clamped).
- Rust tests for the allow-list, the preview and selective deletion, nested targets, settings, history and schedule maths. Frontend tests for the preview, history, settings, theme and language, plus a check that the two dictionaries match.

### Changed

- The allow-list moved from `cleaner.rs` to `targets.rs` and now covers all OSes from one table. Folders that belong to a more specific target are excluded from the enclosing one (for example, Chrome's cache inside `~/Library/Caches`), so nothing is counted or cleaned twice.
- Scan results now report a target's roots instead of a single path.
- Cleans from the window, the tray and the schedule are serialised, so they never overlap.

### Security

- No command accepts a user-supplied folder. `clean_selected` accepts file paths only from a preview and rejects any path that:
  - is relative or contains `..`;
  - is outside an enabled target;
  - is inside another target;
  - is reached through a symlinked folder;
  - is no longer stale.

## 0.1.0 (not tagged)

The first working version. It was never released under its own tag.

### Added

- Allow-listed cleaner for the OS temp folder and the per-user cache folder (macOS/Linux): it deletes regular files older than 24 hours, never follows symlinks and keeps folders.
- Scan → select → confirm → clean UI, CI, and `tauri-action` release builds for macOS, Windows and Linux.

[0.2.0]: https://github.com/eshginfarzali/cleancraft_desktop_app/releases/tag/v0.2.0
