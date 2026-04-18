use crate::state::AppState;
use egui::{Color32, RichText, ScrollArea, Sense, Ui, Vec2};

const NODE_COLOR: Color32 = Color32::from_rgb(55, 138, 221);
const HEAD_COLOR: Color32 = Color32::from_rgb(55, 138, 221);
const LINE_COLOR: Color32 = Color32::from_rgb(80, 80, 80);

/// Renders the commit history graph. Clicking a commit updates selection.
pub fn show(ui: &mut Ui, state: &mut AppState) {
    ui.add_space(4.0);
    ui.label(RichText::new("HISTORY").size(10.0).color(Color32::GRAY));
    ui.add_space(2.0);

    if state.ui.commits.is_empty() {
        ui.label(
            RichText::new("  no worktree selected")
                .size(11.0)
                .color(Color32::GRAY),
        );
        return;
    }

    let commit_count = state.ui.commits.len();
    let sel_commit = state.selection.commit_idx.unwrap_or(0);

    ScrollArea::vertical()
        .id_source("graph_scroll")
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);

            for (i, commit) in state.ui.commits.iter().enumerate() {
                let is_sel = i == sel_commit;

                let resp = ui
                    .horizontal(|ui| {
                        ui.add_space(8.0);

                        // Graph column: node + vertical line
                        ui.vertical(|ui| {
                            ui.add_space(4.0);
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::new(10.0, 10.0),
                                Sense::hover(),
                            );
                            let painter = ui.painter();
                            painter.circle_filled(rect.center(), 4.5, NODE_COLOR);
                            if !commit.is_head {
                                painter.circle_stroke(
                                    rect.center(),
                                    4.5,
                                    (1.5, NODE_COLOR),
                                );
                            }
                            if i < commit_count - 1 {
                                let line_start = rect.center_bottom();
                                let line_end = line_start + Vec2::new(0.0, 8.0);
                                painter.line_segment([line_start, line_end], (1.5, LINE_COLOR));
                            }
                        });

                        ui.add_space(6.0);

                        // Commit info
                        ui.vertical(|ui| {
                            ui.add_space(2.0);
                            let msg_color = if is_sel {
                                Color32::WHITE
                            } else {
                                Color32::LIGHT_GRAY
                            };
                            // Truncate long messages
                            let msg = if commit.message.len() > 42 {
                                format!("{}…", &commit.message[..41])
                            } else {
                                commit.message.clone()
                            };
                            ui.label(RichText::new(&msg).size(11.0).color(msg_color));

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(&commit.short_id)
                                        .size(10.0)
                                        .monospace()
                                        .color(Color32::from_rgb(55, 138, 221)),
                                );
                                ui.label(
                                    RichText::new(relative_time(&commit.time))
                                        .size(10.0)
                                        .color(Color32::GRAY),
                                );
                            });

                            if !commit.branches.is_empty() {
                                for branch in &commit.branches {
                                    ui.label(
                                        RichText::new(branch)
                                            .size(9.0)
                                            .color(Color32::from_rgb(55, 138, 221))
                                            .background_color(Color32::from_rgb(20, 40, 70)),
                                    );
                                }
                            }
                            ui.add_space(4.0);
                        });
                    })
                    .response
                    .interact(Sense::click());

                if resp.clicked() {
                    state.selection.commit_idx = Some(i);
                }
            }
        });
}

fn relative_time(dt: &chrono::DateTime<chrono::Local>) -> String {
    let now = chrono::Local::now();
    let secs = (now - *dt).num_seconds();
    match secs {
        s if s < 60 => "just now".to_string(),
        s if s < 3600 => format!("{}m ago", s / 60),
        s if s < 86400 => format!("{}h ago", s / 3600),
        s if s < 86400 * 7 => format!("{}d ago", s / 86400),
        _ => dt.format("%b %d").to_string(),
    }
}
