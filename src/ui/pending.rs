use crate::{
    git::ChangeStatus,
    state::AppState,
};
use egui::{Color32, Id, Layout, Rect, RichText, ScrollArea, Sense, Ui, Vec2, pos2};

/// Renders the pending changes section. Returns the selected file index if clicked.
pub fn show(ui: &mut Ui, state: &mut AppState) -> Option<usize> {
    let mut clicked_idx: Option<usize> = None;

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

    // Allocate a fixed-height rect so the divider below never moves regardless
    // of how many files are in the list.
    let avail_w = ui.available_width();
    let (fixed_rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, panel_h), Sense::hover());
    let mut child = ui.child_ui(fixed_rect, Layout::top_down(egui::Align::LEFT), None);

    if changes.is_empty() {
        child.add_space(6.0);
        child.label(RichText::new("  no changes").size(11.0).color(Color32::GRAY));
    } else {
        ScrollArea::vertical()
            .id_source("pending_scroll")
            .max_height(panel_h)
            .show(&mut child, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);

                for (i, change) in changes.iter().enumerate() {
                    let is_sel = state.selection.file_idx == Some(i);
                    let label_color = status_color(&change.status);
                    let name = change
                        .path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();

                    let row_y = ui.cursor().min.y;
                    let row_x = ui.cursor().min.x;
                    let row_w = ui.available_width();

                    ui.push_id(("pending", i), |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            ui.label(RichText::new(change.status.label()).size(12.0).strong().color(label_color));
                            ui.add_space(6.0);
                            ui.label(RichText::new(&name).size(13.0)
                                .color(if is_sel { Color32::WHITE } else { Color32::LIGHT_GRAY }));
                        });
                    });

                    let row_rect = Rect::from_min_max(pos2(row_x, row_y), pos2(row_x + row_w, ui.cursor().min.y));
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

fn status_color(status: &ChangeStatus) -> Color32 {
    match status {
        ChangeStatus::Modified => Color32::from_rgb(55, 138, 221),
        ChangeStatus::Added => Color32::from_rgb(99, 153, 34),
        ChangeStatus::Deleted => Color32::from_rgb(163, 45, 45),
        ChangeStatus::Untracked => Color32::from_rgb(133, 79, 11),
        ChangeStatus::Renamed => Color32::from_rgb(83, 74, 183),
        ChangeStatus::Conflicted => Color32::from_rgb(211, 75, 78),
    }
}
