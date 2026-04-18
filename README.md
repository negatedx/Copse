# gitwatcher

A fast, read-only Git GUI designed for developers working across multiple repos and worktrees simultaneously. Built with Rust and [egui](https://github.com/emilk/egui).

## Why

When using Claude Code to drive development across multiple microservices, you often have several worktrees open at once — one per feature or fix, spread across several repos. Standard Git GUIs either show one repo at a time, or are too heavyweight to keep open alongside everything else. gitwatcher keeps the full picture visible at a glance.

## Features

- **Multi-repo sidebar** — all your repos in one tree view, with per-worktree change counts
- **Multi-worktree support** — all registered worktrees shown per repo, not just the main branch
- **Worktree search** — filter across all repos by name or branch prefix; useful when you use consistent naming across microservices (e.g. `feat/auth-refresh` in api-gateway, user-service, etc.)
- **Pending changes panel** — always visible at the top of the middle column, showing modified/added/deleted/untracked files
- **Commit graph** — linear history with relative timestamps, below the pending changes
- **Diff view** — unified diff for the selected file with line numbers and syntax highlighting
- **Auto-refresh** — file watcher detects changes made by Claude Code or any other tool and updates the view automatically
- **"Add all repos in dir"** — point at your microservices parent directory and all repos are discovered at once

## What it does not do (by design)

gitwatcher is intentionally read-only in v1. It does not stage, commit, push, or create branches. Claude Code handles that. This tool is purely for visibility.

## Installation

### Prerequisites

- Rust 1.78+ (`rustup` recommended)
- On Linux: `libgtk-3-dev`, `libxcb-*` packages for egui windowing

### Build

```bash
git clone <this repo>
cd gitwatcher
cargo build --release
./target/release/gitwatcher
```

### Install to PATH

```bash
cargo install --path .
gitwatcher
```

## Usage

### Adding repos

On first launch the window will be empty. Click **+** in the sidebar header to open the add dialog:

- **Add single repo** — paste an absolute path to any git repo
- **Add all repos in dir** — paste the path to a parent directory; all immediate subdirectories containing a `.git` folder are added

Repo paths are saved to `~/.config/gitwatcher/settings.json` and restored on next launch.

### Worktree search

Type in the search box at the top of the sidebar. The tree filters live as you type, matching against worktree names and branch names. Matched worktrees are highlighted in blue; repos with no matches are hidden. Clear the search to restore the full tree.

Tip: if you use a consistent prefix like `feat/TICKET-123` across repos, typing that prefix instantly shows you all the related worktrees.

### Keyboard shortcuts (planned)

| Key | Action |
|-----|--------|
| `/` | Focus the worktree search |
| `Esc` | Clear search |
| `↑` / `↓` | Navigate commits |
| `R` | Force refresh all repos |

## Project layout

```
src/
  main.rs          — entry point, logging setup
  git/mod.rs       — repo discovery, worktree loading, diff, history (libgit2)
  state/mod.rs     — AppState, Settings, Selection, UiState
  watcher/mod.rs   — file system watcher (notify + debounce)
  ui/
    mod.rs         — App struct, eframe::App impl, panel layout
    sidebar.rs     — repo/worktree tree + search
    pending.rs     — pending changes file list
    graph.rs       — commit history graph
    diff.rs        — unified diff viewer
```

## Configuration

Settings file: `~/.config/gitwatcher/settings.json`

```json
{
  "repo_paths": ["/home/user/projects/my-repo"],
  "scan_dirs": ["/home/user/projects"],
  "history_limit": 100
}
```

## Performance notes

- Repo scanning on startup is parallelised with `rayon` — 20 repos load in roughly the same time as one
- The file watcher debounces events at 300ms to avoid thrashing on rapid saves
- egui is an immediate-mode renderer; the UI only repaints when something changes
- Memory usage is proportional to the number of repos and the history limit; `history_limit` in settings caps commit list size

## Dependencies

| Crate | Purpose |
|-------|---------|
| `egui` / `eframe` | Immediate-mode UI framework |
| `git2` | libgit2 bindings — all Git operations |
| `rayon` | Data parallelism for multi-repo scanning |
| `tokio` | Async runtime (used by the watcher layer) |
| `notify` + `notify-debouncer-mini` | Cross-platform file watching |
| `serde` / `serde_json` | Settings persistence |
| `dirs` | XDG-compliant config paths |
| `chrono` | Commit timestamp formatting |
| `anyhow` | Error handling |
| `tracing` | Structured logging |

## Roadmap

- [ ] Multi-lane commit graph (parallel branches)
- [ ] Keyboard navigation throughout
- [ ] File history (commits touching a specific file)
- [ ] Word-level diff highlighting
- [ ] Optional dark/light theme toggle
- [ ] System tray / menubar mode
- [ ] Configurable repo colours in the sidebar
