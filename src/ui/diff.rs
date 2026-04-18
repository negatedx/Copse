use crate::{git::DiffLineKind, state::AppState};
use egui::{Color32, RichText, ScrollArea, Ui};

/// Renders the diff view for the currently selected file.
pub fn show(ui: &mut Ui, state: &AppState) {
    let dark = ui.visuals().dark_mode;
    let add_bg = if dark { Color32::from_rgb(30, 55, 30) } else { Color32::from_rgb(215, 242, 215) };
    let del_bg = if dark { Color32::from_rgb(60, 25, 25) } else { Color32::from_rgb(250, 220, 220) };
    let add_fg = if dark { Color32::from_rgb(130, 190, 100) } else { Color32::from_rgb(25, 105, 25) };
    let del_fg = if dark { Color32::from_rgb(200, 100, 100) } else { Color32::from_rgb(155, 30, 30) };
    let ctx_fg = ui.visuals().text_color();
    let hunk_fg = Color32::GRAY;
    let ln_fg = if dark { Color32::from_rgb(80, 80, 80) } else { Color32::from_rgb(160, 160, 160) };

    // Header: breadcrumb
    if let Some(file) = state.selected_file() {
        let wt_name = state
            .selected_worktree()
            .map(|w| w.name.as_str())
            .unwrap_or("");
        let repo_name = state
            .selected_repo()
            .map(|r| r.name.as_str())
            .unwrap_or("");
        let file_name = file.path.to_string_lossy();

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{repo_name} / {wt_name}"))
                    .size(11.0)
                    .color(Color32::GRAY),
            );
            ui.label(RichText::new("—").size(11.0).color(Color32::GRAY));
            ui.label(RichText::new(file_name.as_ref()).size(11.0).monospace());
        });
        ui.separator();
    } else {
        ui.add_space(8.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("select a file to view diff")
                    .size(12.0)
                    .color(Color32::GRAY),
            );
        });
        return;
    }

    if state.ui.diff_hunks.is_empty() {
        ui.add_space(8.0);
        ui.label(
            RichText::new("  no diff available (untracked or binary file)")
                .size(11.0)
                .color(Color32::GRAY),
        );
        return;
    }

    ScrollArea::both()
        .id_source("diff_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::new(0.0, 0.0);

            for (hunk_i, hunk) in state.ui.diff_hunks.iter().enumerate() {
                ui.push_id(hunk_i, |ui| {
                    ui.label(
                        RichText::new(&hunk.header)
                            .size(10.0)
                            .monospace()
                            .color(hunk_fg),
                    );

                    for (line_i, line) in hunk.lines.iter().enumerate() {
                        let (bg, fg, prefix) = match line.kind {
                            DiffLineKind::Added => (add_bg, add_fg, "+"),
                            DiffLineKind::Deleted => (del_bg, del_fg, "-"),
                            DiffLineKind::Context => (Color32::TRANSPARENT, ctx_fg, " "),
                        };

                        ui.push_id(line_i, |ui| {
                            egui::Frame::none().fill(bg).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let ln = line
                                        .new_lineno
                                        .or(line.old_lineno)
                                        .map(|n| format!("{n:>4}"))
                                        .unwrap_or_else(|| "    ".to_string());
                                    ui.label(
                                        RichText::new(&ln).size(10.0).monospace().color(ln_fg),
                                    );
                                    ui.add_space(4.0);
                                    let text = format!("{prefix}{}", line.content);
                                    ui.label(RichText::new(&text).size(11.0).monospace().color(fg));
                                });
                            });
                        });
                    }
                });
            }
        });
}
