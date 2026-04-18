# gitwatcher — Todo List

---

## Some icons are not rendering correctly

**Priority:** Medium

**Problem:** Several unicode glyphs used as icons (e.g. ⌕, ⚙, chevrons) may not render correctly depending on the system font — showing as boxes or missing entirely.

**Acceptance criteria:**
- All icon glyphs render visibly on Windows with the default egui font
- Any broken glyph is replaced with one that renders correctly, or with plain ASCII fallback text

**Notes:** egui bundles a subset of unicode. Glyphs outside that range need a custom font loaded that covers them.

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

## Show hand cursor when hovering over clickable elements

**Priority:** Medium

**Problem:** The cursor stays as an arrow over all clickable rows and buttons, giving no affordance that something is interactive.

**Acceptance criteria:**
- Cursor changes to a pointing hand over all clickable rows (repo, worktree, commit, file rows)
- Cursor changes over all buttons (add, remove, settings gear, theme toggles)

**Notes:** `ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand)` on the hovered response. Already used in pending.rs — needs applying consistently everywhere.

---

## Changes panel height should be resizable and persist

**Problem:** The CHANGES panel has a fixed `max_height(200)`. When switching between pending changes and a commit with more/fewer files, the history panel jumps around.

**Acceptance criteria:**
- User can drag to resize the boundary between CHANGES and HISTORY panels
- The chosen height persists across sessions (saved to settings)
- History panel fills remaining space below without jumping

**Notes:** Could use a stored `changes_panel_height: f32` in `Settings` and a drag handle between the two sections.

---


## Scrollbar not pinned to right edge in CHANGES and HISTORY panels

**Priority:** Medium

**Problem:** The vertical scrollbar in both the CHANGES and HISTORY panels does not appear flush against the right edge of the middle panel — it floats inside the content area instead.

**Acceptance criteria:**
- Scrollbar appears at the right edge of the middle panel in both the CHANGES and HISTORY panels
- File list and commit list content fills the full available width up to the scrollbar
- Behaviour is consistent whether the panel is at default width or resized

**Notes:** CHANGES scroll area is in `src/ui/pending.rs` (inside a `child_ui`); HISTORY scroll area is in `src/ui/graph.rs`. Both use `ScrollArea::vertical()` — the scrollbar anchor or available-width calculation likely needs fixing in both.

---

## Show branch names on commit graph

**Priority:** High

**Problem:** The HISTORY panel shows commits but gives no indication of which commit each branch (local or remote) points to. It's impossible to see branch positions at a glance.

**Acceptance criteria:**
- Each commit row that is the tip of one or more branches shows the branch name(s) as a pill/label beside the commit message
- Local and remote-tracking branches (e.g. `main`, `origin/main`) are both shown
- HEAD branch is visually distinct (e.g. bold or different colour)
- Labels don't overflow or obscure the commit message — truncate or wrap if needed

**Notes:** `CommitInfo.branches` in `git/mod.rs` exists but is always `vec![]`. Populate it in `get_commits` by building a map of OID → branch names from `repo.branches(None)` before the revwalk, then assign matches per commit. Rendering belongs in `src/ui/graph.rs`.

---

## Repo persistence is broken for scanned directories

**Priority:** High

**Problem:** Repos discovered via directory scan are re-discovered from scratch on every launch. This means: (1) repos explicitly removed by the user reappear after restart because the scan dir is still in settings, and (2) the app's repo list is unstable across sessions.

**Acceptance criteria:**
- Repos added via directory scan persist individually across restarts
- A repo explicitly removed by the user does not reappear after restart, even if its parent scan dir is still in settings
- Manually added repos are unaffected

**Notes:** On startup, `scan_dirs` are re-scanned via `discover_repos_in_dir`, overriding any user removals. Simplest fix: after the initial scan, convert discovered paths into explicit `repo_paths` entries and drop the `scan_dir` from settings (one-shot expansion). Alternatively, add a `removed_repo_paths: Vec<PathBuf>` blocklist to `Settings` as a filter. The one-shot expansion is simpler and avoids the blocklist growing unboundedly.

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
