use crate::{
    git::ChangeStatus,
    state::AppState,
};
use egui::{Color32, RichText, ScrollArea, Sense, Ui, Vec2};

/// Renders the pending changes section. Returns the selected file index if clicked.
pub fn show(ui: &mut Ui, state: &mut AppState) -> Option<usize> {
    let mut clicked_idx: Option<usize> = None;

    let changes = state
        .selected_worktree()
        .map(|wt| wt.pending_changes.clone())
        .unwrap_or_default();

    let count = changes.len();

    ui.horizontal(|ui| {
        ui.label(RichText::new("PENDING CHANGES").size(10.0).color(Color32::GRAY));
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
        ui.add_space(6.0);
        ui.label(
            RichText::new("  working tree clean")
                .size(11.0)
                .color(Color32::GRAY),
        );
        ui.add_space(6.0);
        return None;
    }

    ScrollArea::vertical()
        .id_source("pending_scroll")
        .max_height(140.0)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);

            for (i, change) in changes.iter().enumerate() {
                let is_sel = state.selection.file_idx == Some(i);
                let label_color = status_color(&change.status);
                let name = change
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();

                let resp = ui
                    .horizontal(|ui| {
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(change.status.label())
                                .size(10.0)
                                .strong()
                                .color(label_color),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(&name)
                                .size(11.0)
                                .color(if is_sel { Color32::WHITE } else { Color32::LIGHT_GRAY }),
                        );
                    })
                    .response
                    .interact(Sense::click());

                if resp.clicked() {
                    clicked_idx = Some(i);
                }

                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
            }
        });

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
