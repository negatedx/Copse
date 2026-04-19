use crate::{
    git::ChangeStatus,
    state::AppState,
};
use egui::{Color32, Id, Rect, RichText, ScrollArea, Sense, Ui, Vec2, pos2};

/// Renders the pending changes section. Returns the selected file index if clicked.
pub fn show(ui: &mut Ui, state: &mut AppState) -> Option<usize> {
    let mut clicked_idx: Option<usize> = None;
    let dark = ui.visuals().dark_mode;
    let text_color = ui.visuals().text_color();
    let sel_color = ui.visuals().strong_text_color();

    let changes = state.ui.files_view.clone();
    let count = changes.len();
    let panel_h = state.settings.changes_panel_height;

    ui.horizontal(|ui| {
        ui.label(RichText::new("CHANGES").size(10.0).color(Color32::GRAY));
        if count > 0 {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{count}"))
                        .size(10.0)
                        .color(Color32::from_rgb(153, 60, 29)),
                );
            });
        }
    });

    ui.add_space(2.0);

    if changes.is_empty() {
        // Reserve the same fixed height so the divider below stays stable.
        let avail_w = ui.available_width();
        let (fixed_rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, panel_h), Sense::hover());
        ui.painter().text(
            fixed_rect.left_top() + egui::vec2(8.0, 6.0),
            egui::Align2::LEFT_TOP,
            "no changes",
            egui::TextStyle::Small.resolve(ui.style()),
            Color32::GRAY,
        );
    } else {
        ScrollArea::vertical()
            .id_source("pending_scroll")
            .max_height(panel_h)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);
                ui.set_min_height(panel_h);

                for (i, change) in changes.iter().enumerate() {
                    let is_sel = state.selection.file_idx == Some(i);
                    let label_color = status_color(&change.status, dark);
                    let name = change
                        .path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();

                    let row_y = ui.cursor().min.y;
                    let row_x = ui.cursor().min.x;
                    let row_w = ui.available_width();

                    let bg_idx = ui.painter().add(egui::Shape::Noop);

                    ui.push_id(("pending", i), |ui| {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            ui.label(RichText::new(change.status.label()).size(12.0).strong().color(label_color));
                            ui.add_space(6.0);
                            ui.label(RichText::new(&name).size(13.0)
                                .color(if is_sel { sel_color } else { text_color }));
                        });
                        ui.add_space(4.0);
                    });

                    let row_rect = Rect::from_min_max(pos2(row_x, row_y), pos2(row_x + row_w, ui.cursor().min.y));
                    if is_sel {
                        let bg_fill = ui.visuals().selection.bg_fill.gamma_multiply(0.4);
                        ui.painter().set(bg_idx, egui::Shape::rect_filled(row_rect, 0.0, bg_fill));
                        ui.painter().vline(row_x, row_y..=row_rect.max.y, egui::Stroke::new(3.0, sel_color));
                    }
                    let resp = ui.interact(row_rect, Id::new(("pending_click", i)), Sense::click());

                    if resp.clicked() {
                        clicked_idx = Some(i);
                    }
                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                }
            });
    }

    clicked_idx
}

fn status_color(status: &ChangeStatus, dark: bool) -> Color32 {
    if dark {
        match status {
            ChangeStatus::Modified => Color32::from_rgb(55, 138, 221),
            ChangeStatus::Added => Color32::from_rgb(99, 153, 34),
            ChangeStatus::Deleted => Color32::from_rgb(200, 65, 65),
            ChangeStatus::Untracked => Color32::from_rgb(180, 120, 40),
            ChangeStatus::Renamed => Color32::from_rgb(120, 110, 220),
            ChangeStatus::Conflicted => Color32::from_rgb(211, 75, 78),
        }
    } else {
        match status {
            ChangeStatus::Modified => Color32::from_rgb(20, 100, 190),
            ChangeStatus::Added => Color32::from_rgb(40, 120, 10),
            ChangeStatus::Deleted => Color32::from_rgb(170, 30, 30),
            ChangeStatus::Untracked => Color32::from_rgb(140, 80, 0),
            ChangeStatus::Renamed => Color32::from_rgb(70, 60, 180),
            ChangeStatus::Conflicted => Color32::from_rgb(180, 40, 40),
        }
    }
}
