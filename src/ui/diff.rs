use crate::{git::DiffLineKind, state::AppState};
use egui::{Color32, RichText, ScrollArea, Ui};

const ADD_BG: Color32 = Color32::from_rgb(30, 55, 30);
const DEL_BG: Color32 = Color32::from_rgb(60, 25, 25);
const ADD_FG: Color32 = Color32::from_rgb(150, 210, 120);
const DEL_FG: Color32 = Color32::from_rgb(210, 120, 120);
const CTX_FG: Color32 = Color32::LIGHT_GRAY;
const HUNK_FG: Color32 = Color32::GRAY;
const LN_FG: Color32 = Color32::from_rgb(80, 80, 80);

/// Renders the diff view for the currently selected file.
pub fn show(ui: &mut Ui, state: &AppState) {
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

            for hunk in &state.ui.diff_hunks {
                // Hunk header
                ui.label(
                    RichText::new(&hunk.header)
                        .size(10.0)
                        .monospace()
                        .color(HUNK_FG),
                );

                for line in &hunk.lines {
                    let (bg, fg, prefix) = match line.kind {
                        DiffLineKind::Added => (ADD_BG, ADD_FG, "+"),
                        DiffLineKind::Deleted => (DEL_BG, DEL_FG, "-"),
                        DiffLineKind::Context => (Color32::TRANSPARENT, CTX_FG, " "),
                    };

                    egui::Frame::none().fill(bg).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Line number gutter
                            let ln = line
                                .new_lineno
                                .or(line.old_lineno)
                                .map(|n| format!("{n:>4}"))
                                .unwrap_or_else(|| "    ".to_string());
                            ui.label(
                                RichText::new(&ln).size(10.0).monospace().color(LN_FG),
                            );
                            ui.add_space(4.0);

                            // Prefix + content
                            let text = format!("{prefix}{}", line.content);
                            ui.label(RichText::new(&text).size(11.0).monospace().color(fg));
                        });
                    });
                }
            }
        });
}
