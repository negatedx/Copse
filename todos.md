# gitwatcher — Todo List

---

## Some icons are not rendering correctly

**Problem:** Several unicode glyphs used as icons (e.g. ⌕, ⚙, chevrons) may not render correctly depending on the system font — showing as boxes or missing entirely.

**Acceptance criteria:**
- All icon glyphs render visibly on Windows with the default egui font
- Any broken glyph is replaced with one that renders correctly, or with plain ASCII fallback text

**Notes:** egui bundles a subset of unicode. Glyphs outside that range need a custom font loaded that covers them.

---

## UI is too small and tight generally

**Problem:** Row heights, padding, and hit targets are too small. Text is cramped and the UI feels dense and hard to use.

**Acceptance criteria:**
- Repo rows, worktree rows, commit rows, and file rows all have comfortable padding
- Minimum row height feels consistent across all panels
- No text or elements feel clipped or crowded

**Notes:** `item_spacing`, `ui.add_space()`, and explicit row height minimums in the interact rect are the levers here.

---

## Show hand cursor when hovering over clickable elements

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

## Fix commit graph rendering: disconnected nodes and missing spacing

**Problem:** In the HISTORY panel, commit graph nodes are not connected by lines between them. Additionally, the commit hash and date are rendered with no space between them, making the text run together and hard to read.

**Acceptance criteria:**
- Graph nodes are connected by vertical (and branch) lines forming a visible commit graph
- A clear space separates the commit hash from the date in each row
- Layout remains consistent across varying numbers of branches

**Notes:** Graph drawing is in `src/ui/graph.rs`. Check the node connector painting logic for missing line segments between rows, and the commit row label layout for the hash/date spacing.

---

## Repos and worktrees should be listed alphabetically

**Problem:** The sidebar lists repos in the order they were added or discovered, and worktrees in the order libgit2 returns them. With many repos, finding a specific one requires scanning the whole list rather than knowing roughly where to look.

**Acceptance criteria:**
- Repos are sorted alphabetically by directory name in the sidebar
- Worktrees within each repo are sorted alphabetically by path
- Sort is applied on initial load, after adding a repo, and after a watcher-triggered reload

**Notes:** Sorting belongs in `state/mod.rs` or at the call sites in `ui/mod.rs` (after `load_repos_parallel`, after `load_repo` pushes a new entry), not in `ui/sidebar.rs`. A simple `sort_by` on the repo name and worktree path is sufficient.

---

## Scrollbar not pinned to right edge in CHANGES and HISTORY panels

**Problem:** The vertical scrollbar in both the CHANGES and HISTORY panels does not appear flush against the right edge of the middle panel — it floats inside the content area instead.

**Acceptance criteria:**
- Scrollbar appears at the right edge of the middle panel in both the CHANGES and HISTORY panels
- File list and commit list content fills the full available width up to the scrollbar
- Behaviour is consistent whether the panel is at default width or resized

**Notes:** CHANGES scroll area is in `src/ui/pending.rs` (inside a `child_ui`); HISTORY scroll area is in `src/ui/graph.rs`. Both use `ScrollArea::vertical()` — the scrollbar anchor or available-width calculation likely needs fixing in both.

---

## Selected repo is not clearly highlighted in the sidebar

**Problem:** When a repo is selected in the sidebar, it is not visually distinct enough from unselected repos — it's hard to tell at a glance which repo is active.

**Acceptance criteria:**
- The selected repo row has a clearly visible highlight (background colour, left accent bar, or similar)
- The highlight is distinct in both Dark and Light themes
- Selecting a different repo moves the highlight immediately

**Notes:** Sidebar rendering is in `src/ui/sidebar.rs`. Check how the selected state is currently rendered for repo rows vs worktree rows — worktree rows may already have a highlight to use as reference.

---

## Show branch names on commit graph

**Problem:** The HISTORY panel shows commits but gives no indication of which commit each branch (local or remote) points to. It's impossible to see branch positions at a glance.

**Acceptance criteria:**
- Each commit row that is the tip of one or more branches shows the branch name(s) as a pill/label beside the commit message
- Local and remote-tracking branches (e.g. `main`, `origin/main`) are both shown
- HEAD branch is visually distinct (e.g. bold or different colour)
- Labels don't overflow or obscure the commit message — truncate or wrap if needed

**Notes:** `CommitInfo.branches` in `git/mod.rs` exists but is always `vec![]`. Populate it in `get_commits` by building a map of OID → branch names from `repo.branches(None)` before the revwalk, then assign matches per commit. Rendering belongs in `src/ui/graph.rs`.

---
