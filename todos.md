# gitwatcher — Todo List

---

## 1. More visual settings — font and font size

**Problem:** There is currently no way to change the font family or font size. The UI scale slider exists but font selection is missing.

**Acceptance criteria:**
- User can select from a small set of available fonts (e.g. system default, monospace)
- User can adjust font size independently of UI scale
- Settings persist across sessions

**Notes:** egui supports loading custom fonts via `egui::FontDefinitions`. Font size can be controlled via `egui::Style::text_styles`.

---

## 2. Some icons are not rendering correctly

**Problem:** Several unicode glyphs used as icons (e.g. ⌕, ⚙, chevrons) may not render correctly depending on the system font — showing as boxes or missing entirely.

**Acceptance criteria:**
- All icon glyphs render visibly on Windows with the default egui font
- Any broken glyph is replaced with one that renders correctly, or with plain ASCII fallback text

**Notes:** egui bundles a subset of unicode. Glyphs outside that range need a custom font loaded that covers them.

---

## 3. UI is too small and tight generally

**Problem:** Row heights, padding, and hit targets are too small. Text is cramped and the UI feels dense and hard to use.

**Acceptance criteria:**
- Repo rows, worktree rows, commit rows, and file rows all have comfortable padding
- Minimum row height feels consistent across all panels
- No text or elements feel clipped or crowded

**Notes:** `item_spacing`, `ui.add_space()`, and explicit row height minimums in the interact rect are the levers here.

---

## 4. Show hand cursor when hovering over clickable elements

**Problem:** The cursor stays as an arrow over all clickable rows and buttons, giving no affordance that something is interactive.

**Acceptance criteria:**
- Cursor changes to a pointing hand over all clickable rows (repo, worktree, commit, file rows)
- Cursor changes over all buttons (add, remove, settings gear, theme toggles)

**Notes:** `ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand)` on the hovered response. Already used in pending.rs — needs applying consistently everywhere.

---

## 5. Changes panel height should be resizable and persist

**Problem:** The CHANGES panel has a fixed `max_height(200)`. When switching between pending changes and a commit with more/fewer files, the history panel jumps around.

**Acceptance criteria:**
- User can drag to resize the boundary between CHANGES and HISTORY panels
- The chosen height persists across sessions (saved to settings)
- History panel fills remaining space below without jumping

**Notes:** Could use a stored `changes_panel_height: f32` in `Settings` and a drag handle between the two sections.

---

## 6. Colour themes need work

**Problem:** The Light theme has text that is very hard to read in places. Diff colours (added/removed lines) look too bold/saturated in both themes.

**Acceptance criteria:**
- All text in Light theme is legible against its background
- Diff line colours (green/red) are readable but not overwhelming in both themes
- Status badge colours (M, A, D, etc.) work in both themes

**Notes:** egui's `Visuals::light()` defaults may need overrides for specific colours. Check diff.rs and pending.rs colour constants.

---

## 7. Repo list should be deduplicated on path

**Problem:** When scanning a directory for repos, repos already in the list can be added again, creating duplicates.

**Acceptance criteria:**
- Adding a repo that is already in the list (by path) does nothing silently, or shows a brief message
- Scanning a directory skips any paths already present in `state.repos`
- Applies to both single-repo add and bulk scan

**Notes:** The scan path already filters (`!self.state.repos.iter().any(|r| &r.path == p)`) but the single-repo add path (`git2::Repository::open` branch) may not.

---

## 8. Fix commit graph rendering: disconnected nodes and missing spacing

**Problem:** In the HISTORY panel, commit graph nodes are not connected by lines between them. Additionally, the commit hash and date are rendered with no space between them, making the text run together and hard to read.

**Acceptance criteria:**
- Graph nodes are connected by vertical (and branch) lines forming a visible commit graph
- A clear space separates the commit hash from the date in each row
- Layout remains consistent across varying numbers of branches

**Notes:** Graph drawing is in `src/ui/graph.rs`. Check the node connector painting logic for missing line segments between rows, and the commit row label layout for the hash/date spacing.

---
