# gitwatcher — Todo List

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

## Side-by-side diff view toggle

**Priority:** Low

**Problem:** The diff panel only shows unified diff. Side-by-side diff is easier to read for small changes, but requires more horizontal space — it's not always appropriate.

**Acceptance criteria:**
- A toggle button in the diff panel header switches between unified and side-by-side view
- The toggle is only shown (or enabled) when the diff panel is wide enough to render two columns legibly
- Side-by-side view splits the old and new file into left/right columns with line numbers on each side
- Removed lines appear on the left only, added lines on the right only, unchanged context on both
- The chosen mode persists across sessions (saved to settings)

**Notes:** Rendering is in `src/ui/diff.rs`. A minimum panel width threshold (e.g. ~600px) can gate the toggle. `UiState` or `Settings` can hold the `diff_side_by_side: bool` flag; `Settings` if persistence is wanted.

---
