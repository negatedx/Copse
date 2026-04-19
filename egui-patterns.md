# egui 0.28 patterns

Hard-won patterns from building this app. Consult before reaching for an unfamiliar egui API.

---

## Label wrapping

`Label::wrap()` takes no arguments — it's a flag method that enables wrapping. Without it, a label inherits `ui.wrap_text()`, which returns `true` whenever `available_width` is finite. This means labels inside columns, frames, or any bounded container will wrap by default.

To explicitly disable wrapping:
```rust
ui.style_mut().wrap = Some(false);
ui.add(Label::new(...));
```

To explicitly enable it:
```rust
ui.add(Label::new(...).wrap());
```

---

## Full-width backgrounds

`egui::Frame::fill` only covers the frame's actual layout width — it won't extend past content into a scroll area. For backgrounds that must span the full scrollable width (e.g. diff line highlights), use the shape-placeholder pattern:

```rust
let bg_id = ui.painter().add(egui::Shape::Noop); // reserve slot
// ... lay out row content ...
let row_bottom = ui.cursor().min.y;
ui.painter().set(bg_id, egui::Shape::rect_filled(
    egui::Rect::from_min_max(
        egui::pos2(row_x, row_top),
        egui::pos2(row_x + 32_768.0, row_bottom), // painter clips automatically
    ),
    0.0, bg_color,
));
```

The 32 768 px right edge extends well past any realistic viewport; the painter clips it to the visible area.

---

## `auto_shrink` in stacked scroll areas

`auto_shrink([h, v])` controls whether the scroll area shrinks to fit content when content is smaller than the viewport.

- `auto_shrink([false, false])` — fills ALL available width and height. Use for a single full-panel scroll area.
- `auto_shrink([false, true])` — fills available width, shrinks to content height. Required when stacking multiple horizontal scroll areas vertically (e.g. per-hunk columns); without `true` on the vertical axis the first scroll area consumes all remaining height and subsequent ones never render.

---

## Split-view horizontal scroll sync (pointer-side driver)

Two `ScrollArea::horizontal()` areas side by side need their offsets kept in sync. Naively forcing both to a shared offset causes "bounce" when the two sides have different content widths — the shorter side clamps its forced offset and then wins the "which side moved?" check, pulling the longer side back.

Fix: use the pointer position to decide which side drives. The driver renders free; the follower has `scroll_offset` forced to match the driver's output.

```rust
let ptr_x = ui.ctx().pointer_hover_pos().map_or(f32::NEG_INFINITY, |p| p.x);
let mid_x = ui.cursor().min.x + ui.available_width() / 2.0;
let right_drives = ptr_x >= mid_x;

ui.columns(2, |cols| {
    let left_sa = ScrollArea::horizontal().id_source("left")...;
    let left_sa = if right_drives { left_sa.scroll_offset(stored_h_vec) } else { left_sa };
    let lo = left_sa.show(&mut cols[0], |ui| { ... }).state.offset.x;

    let right_sa = ScrollArea::horizontal().id_source("right")...;
    let right_sa = if !right_drives { right_sa.scroll_offset(Vec2::new(lo, 0.0)) } else { right_sa };
    let ro = right_sa.show(&mut cols[1], |ui| { ... }).state.offset.x;

    new_h = if right_drives { ro } else { lo };
});
```

---

## Per-widget persistent state via egui temp data

For state that belongs to a specific widget instance (not app-level), store it in egui's temp data keyed by a unique `Id`. It persists across frames and is automatically cleaned up when the widget stops rendering.

```rust
let id = egui::Id::new(("my_widget_state", index));
let stored: f32 = ui.ctx().data(|d| d.get_temp(id).unwrap_or(0.0));
// ... compute new_value ...
ui.ctx().data_mut(|d| d.insert_temp(id, new_value));
```

Used in the diff view to give each hunk its own independent horizontal scroll position, avoiding cross-hunk sync bugs that arise from a single shared scroll offset.

---

## Full-width headers in a split column layout

To render a full-width element (e.g. a section header) between split columns without duplicating it in each column, wrap everything in an outer `ScrollArea::vertical()` and render the header at that level. The per-column content goes in a nested `ui.columns(2)` block below each header.

```
ScrollArea::vertical() {
    for each section:
        full_width_header(ui, ...)      // spans entire width
        ui.columns(2, |cols| {
            ScrollArea::horizontal() { left content }
            ScrollArea::horizontal() { right content }
        })
}
```

Vertical scroll is handled by the outer area; horizontal scroll by the inner per-column areas.
