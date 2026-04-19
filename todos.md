# copse — Todo List

---

## Vertical separator line between split-diff columns

**Priority:** Medium

**Problem:** In split (side-by-side) diff mode there is no visual separator between the left and right columns, making it hard to tell where one side ends and the other begins.

**Acceptance criteria:**
- A thin vertical line runs between the left and right columns for the full height of the diff content
- Line is visible in both dark and light themes
- Does not clip or overlap the text in either column

**Notes:** A plain `vline` painted over the existing `ui.columns(2, ...)` layout clips into the text. The correct approach is to replace `ui.columns(2, ...)` with three explicit columns: left content | narrow separator column | right content. The separator column allocates its own space so neither text column is crowded. Same fix needed for both `show_split_scroll` and `show_split_wrap`.

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


## Add Linux release build and CI job

**Priority:** Medium

**Problem:** Copse only builds and ships a Windows binary. Linux is a common platform for developers who would use this tool alongside Claude Code.

**Acceptance criteria:**
- App builds and runs on Ubuntu (latest LTS) without errors
- Release workflow produces a Linux x86_64 binary and attaches it to the GitHub Release
- README installation section covers Linux prerequisites and build instructions

**Notes:** eframe requires `libgtk-3-dev`, `libxcb-*`, and related packages on Linux. Add a `build-linux` job to `release.yml` running on `ubuntu-latest` alongside the existing Windows job. No `rust-toolchain.toml` target changes needed — default GNU toolchain works on Linux.

---

## Add macOS release build and CI job

**Priority:** Low

**Problem:** Copse has no macOS build. eframe supports macOS but the release pipeline is Windows-only.

**Acceptance criteria:**
- App builds and runs on macOS (Apple Silicon and/or Intel)
- Release workflow produces a macOS binary attached to the GitHub Release
- README covers macOS installation

**Notes:** eframe supports macOS natively. A plain binary works for dev use; a signed `.app` bundle is needed for broader distribution but can come later. Consider a universal binary (`aarch64` + `x86_64`) or separate artifacts. Add a `build-macos` job to `release.yml` on `macos-latest`.

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

