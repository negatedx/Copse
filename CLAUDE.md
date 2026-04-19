# CLAUDE.md — gitrove

This file tells Claude Code how to work effectively on this project. Read it before making changes.

## What this project is

gitrove is a read-only Rust/egui desktop app for visualising Git state across multiple repos and worktrees. It is a developer tool built for a specific workflow: many microservice repos open at once, each with one or more worktrees, with commits being driven by Claude Code rather than by the GUI itself.

**v1 is intentionally read-only.** Do not add staging, committing, pushing, or branch management unless explicitly asked. Keep the surface area small.

## Architecture in one paragraph

`AppState` (in `state/mod.rs`) is the single source of truth. The `git` module talks to libgit2 and returns plain data structs. The `watcher` module fires path events on a `mpsc` channel when files change. The `ui` module reads from `AppState` to draw panels; user interactions return new `Selection` values back up to `App::update`, which updates state and triggers data refreshes. There is no async in the hot path — all git calls are synchronous and cheap enough for interactive use. Heavy operations (initial load, forced full refresh) use rayon for parallelism.

## File map

```
src/main.rs          Entry point. Init logging, create eframe window, run App.
src/git/mod.rs       All libgit2 interaction. No egui here.
src/state/mod.rs     AppState, Settings, Selection, UiState. No egui, no git2 here.
src/watcher/mod.rs   notify watcher. Returns mpsc::Receiver<PathBuf> to App.
src/ui/mod.rs        App struct + eframe::App impl. Orchestrates panels. Add-repo dialog.
src/ui/sidebar.rs    Left panel. Repo tree, worktree rows, search box.
src/ui/pending.rs    Top of middle panel. File list with status badges.
src/ui/graph.rs      Bottom of middle panel. Commit history with graph line.
src/ui/diff.rs       Right panel. Unified diff with line numbers.
```

## Conventions

- **No business logic in `ui/`** — UI files only read from `AppState` and return user actions as data (a new `Selection`, a clicked index). All state mutation happens in `ui/mod.rs` or `state/mod.rs`.
- **No egui imports in `git/` or `state/`** — keep the git and state layers pure.
- **Errors are non-fatal** — use `anyhow::Result` everywhere in `git/`. If a repo fails to load, skip it and log a warning. Never `unwrap()` or `panic!()` in production paths.
- **Clone freely for simplicity** — data structs are small (a few hundred bytes per worktree). Premature optimisation with lifetimes will complicate the UI layer for negligible gain at this scale.
- **`tracing` for all logging** — use `tracing::info!`, `warn!`, `debug!`. Never `println!` or `eprintln!` in library code.

## Common tasks

### Adding a new panel or sidebar section

1. Create `src/ui/<name>.rs` with a `pub fn show(ui: &mut Ui, state: &...) -> Option<...>` signature.
2. Add `mod <name>;` in `src/ui/mod.rs`.
3. Call it from `App::update` inside the appropriate `SidePanel` or `CentralPanel` closure.
4. If it needs new state, add fields to `UiState` in `state/mod.rs`.

### Adding a new git data field

1. Add the field to the appropriate struct in `git/mod.rs` (`RepoInfo`, `WorktreeInfo`, `CommitInfo`, etc.).
2. Populate it in the loader function (`load_worktree`, `get_commits`, etc.).
3. Add `#[serde(default)]` if the struct is serialised and the field may be absent in saved data.

### Adding a new setting

1. Add the field with a sensible default to `Settings` in `state/mod.rs`.
2. Annotate with `#[serde(default = "...")]` so old settings files don't break on load.
3. Expose a UI control in an appropriate panel. Settings are saved by calling `state.settings.save()` after mutation.

### Refreshing data after a watcher event

The watcher fires `PathBuf` on the `reload_rx` channel. `App::poll_watcher()` reads it each frame and calls `git::load_repo` for the affected repo, replacing the stale entry in `state.repos`. If you add new git data (e.g., stash list), ensure `load_repo` fetches it so watcher-driven refreshes stay complete.

## egui reference

**`egui-patterns.md`** in the repo root is the living reference for non-obvious egui behaviour. Read it before tackling a tricky layout problem, and add to it whenever a new pattern is worked out.

## What to avoid

- **Do not shell out to `git`** — use `git2` for everything. Parsing CLI output is fragile and slow.
- **Do not block the UI thread** — `git2` calls in `App::update` should be limited to incremental refreshes of one repo at a time. For full rescans, move to a background thread and send results back via channel.
- **Do not add dependencies without a clear reason** — the binary size and compile time are worth protecting.
- **Do not use `egui::Window` for primary UI** — panels only. `egui::Window` is for dialogs like the add-repo prompt.
- **Do not hardcode paths** — always use `dirs::config_dir()` for config, relative paths for assets.

## Testing approach

There are no UI tests. Test the `git` module against real repos:

```bash
cargo test
```

For manual testing, point the app at a directory containing multiple repos with multiple worktrees. The dev's own `~/projects` directory is usually sufficient. Log output at `RUST_LOG=gitrove=debug` shows all repo loads and watcher events.

## Build

**Working directory for all cargo commands is `src/` — the `Cargo.toml` lives there, not at the repo root.**

```bash
cd src
cargo build           # debug
cargo build --release # optimised, stripped binary (~5MB typical)
cargo clippy          # lint — fix all warnings before committing
cargo fmt             # format — always run before committing
```

## Dependencies — do not change without discussion

| Crate | Reason pinned |
|-------|--------------|
| `git2` with `vendored-libgit2` | Ensures consistent libgit2 version; avoids system library mismatch |
| `egui 0.28` / `eframe 0.28` | egui has breaking changes between minor versions; pin and upgrade deliberately |
| `notify-debouncer-mini 0.4` | API changed significantly in 0.5; evaluate before upgrading |
