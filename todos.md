# gitwatcher — Todo List

---

## Rename the app and add an app icon

**Priority:** High

**Problem:** The name "gitwatcher" undersells what the app does — it's a multi-repo, multi-worktree dashboard, not a file watcher. The app also ships with no icon, which makes it feel unfinished and hard to identify in the taskbar or a shared release.

**Acceptance criteria:**
- A new name is chosen (e.g. "multigit") and applied consistently
- `Cargo.toml` package name and binary name updated
- Window title in `main.rs` updated
- Config directory in `state/mod.rs` `settings_path()` updated to match the new name
- `CLAUDE.md` updated throughout; no remaining hardcoded references to `gitwatcher`
- An app icon (PNG, at least 256×256) is created or sourced and committed to `assets/`
- The window/taskbar icon is set via `eframe::NativeOptions::icon_data` at launch
- The `.exe` file icon is embedded for Windows using `winres` in `build.rs`

**Notes:** The settings file lives at `{config_dir}/gitwatcher/settings.json` — renaming the config dir orphans existing settings. Either silently migrate the old path on first launch, or accept the one-time reset (re-add repos) given the app is pre-release. `winres` requires a `.ico` file; convert the PNG with ImageMagick or a similar tool before embedding.

---

## GitHub Actions release pipeline and in-app update notification

**Priority:** High

**Problem:** There is no automated way to build and publish releases, and users who install the app have no way to know when a newer version is available. Both are needed before sharing the app publicly.

**Acceptance criteria:**
- A `.github/workflows/release.yml` workflow triggers on `v*` tag pushes
- The workflow builds a release binary for Windows (x86_64) and attaches it to a GitHub Release
- The app version in `Cargo.toml` is the single source of truth (read via `env!("CARGO_PKG_VERSION")`)
- On startup, the app checks the GitHub releases API for the latest published version (non-blocking, background thread)
- If a newer version is available, a dismissible banner or status indicator appears in the UI with a link to the releases page
- The update check result is cached for the session so it does not re-fire on every frame
- No auto-download or auto-install — the user opens the browser to download manually

**Notes:** Use `reqwest` (blocking, minimal features) or a plain `std::net` HTTPS call for the version check — avoid pulling in a full async runtime just for this. The GitHub releases API endpoint is `https://api.github.com/repos/{owner}/{repo}/releases/latest`; compare the `tag_name` field against the running version using semver. The update check should be on a background thread sending the result back via a `std::sync::mpsc` channel, consistent with how the watcher works. The banner can live in the sidebar header or the settings window. `self_update` crate exists but is heavyweight; a lightweight manual check is preferred given the no-auto-install requirement.

---

## UI is too small and tight generally

**Priority:** Medium

**Problem:** Row heights, padding, and hit targets are too small. Text is cramped and the UI feels dense and hard to use.

**Acceptance criteria:**
- Repo rows, worktree rows, commit rows, and file rows all have comfortable padding
- Minimum row height feels consistent across all panels
- No text or elements feel clipped or crowded

**Notes:** `item_spacing`, `ui.add_space()`, and explicit row height minimums in the interact rect are the levers here.

---

## Auto-select first file when switching worktree or commit

**Priority:** Medium

**Problem:** When a worktree or commit is selected, the CHANGES panel populates but no file is selected, leaving the diff panel blank. The user must manually click a file every time to see any diff.

**Acceptance criteria:**
- When a worktree is selected and has pending changes, the first file is auto-selected and its diff shown
- When a commit is selected, the first file of that commit is auto-selected
- If the file list is empty, the diff panel remains blank as now

**Notes:** Hook into `refresh_files_view` and `handle_graph_action` in `src/ui/mod.rs` — after populating `ui.files_view`, if non-empty set `selection.file_idx = Some(0)` and call `refresh_diff()`.

---

## Changes panel height should be resizable and persist

**Problem:** The CHANGES panel has a fixed `max_height(200)`. When switching between pending changes and a commit with more/fewer files, the history panel jumps around.

**Acceptance criteria:**
- User can drag to resize the boundary between CHANGES and HISTORY panels
- The chosen height persists across sessions (saved to settings)
- History panel fills remaining space below without jumping

**Notes:** Could use a stored `changes_panel_height: f32` in `Settings` and a drag handle between the two sections.

---

## Context menu on repo/worktree rows with "Open in VS Code"

**Priority:** Medium

**Problem:** There's no way to quickly open a repo or worktree in VS Code from the GUI — users have to navigate manually in their file system or terminal.

**Acceptance criteria:**
- Right-clicking a repo row in the sidebar shows a context menu with "Open in VS Code"
- Right-clicking a worktree row shows the same option
- Clicking the option opens the main repo root (for repo rows) or the worktree path (for worktree rows) in VS Code
- The menu closes after selection

---

## Fix all compiler warnings

**Priority:** Low

**Problem:** `cargo build` emits 7 warnings every build — unused imports (`WalkDir`, `all_watch_paths`, `spawn_watcher`) and dead code (`is_clean`, `Selection::repo`, `spawn_watcher`, `run_watcher`, `all_watch_paths` in watcher). These mask real new warnings that may appear in future.

**Acceptance criteria:**
- `cargo build` produces zero warnings
- Dead code is either removed or, if intentionally kept for future use, suppressed with `#[allow(dead_code)]` and a comment explaining why

**Notes:** Warnings are in `src/git/mod.rs` (unused `walkdir`), `src/ui/mod.rs` (unused watcher imports), `src/state/mod.rs` (`Selection::repo`), `src/watcher/mod.rs` (entire public API unused). The watcher module may be intentionally scaffolded for future use — check before deleting.

---

## Hide expand icon for single-worktree repos

**Priority:** Low

**Problem:** The sidebar renders a chevron expand/collapse icon for every repo, even when it has only one worktree (the main one). There is nothing to expand, so the icon is visual noise and implies interactivity that doesn't exist.

**Acceptance criteria:**
- Expand icon is hidden when a repo has exactly one worktree
- Repos with linked worktrees continue to show the icon as normal
- Clicking the repo row still selects it

**Notes:** Sidebar rendering is in `src/ui/sidebar.rs`. Check `repo.worktrees.len() == 1` before rendering the chevron.

---

