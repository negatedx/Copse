use crate::state::AppState;
use egui::{Color32, Id, Rect, RichText, ScrollArea, Sense, Ui, Vec2, pos2};

const NODE_COLOR: Color32 = Color32::from_rgb(55, 138, 221);
const LINE_COLOR: Color32 = Color32::from_rgb(80, 80, 80);

pub enum GraphAction {
    SelectPending,
    SelectCommit(usize), // index into state.ui.commits
}

pub fn show(ui: &mut Ui, state: &AppState) -> Option<GraphAction> {
    let mut action: Option<GraphAction> = None;
    let dark = ui.visuals().dark_mode;
    let text_color = ui.visuals().text_color();
    let sel_color = ui.visuals().strong_text_color();
    let branch_bg = if dark { Color32::from_rgb(20, 40, 70) } else { Color32::from_rgb(210, 225, 255) };
    let branch_fg = if dark { Color32::from_rgb(100, 160, 240) } else { Color32::from_rgb(20, 80, 180) };

    ui.add_space(4.0);
    ui.label(RichText::new("HISTORY").size(10.0).color(Color32::GRAY));
    ui.add_space(2.0);

    if state.selected_worktree().is_none() {
        ui.label(
            RichText::new("  no worktree selected")
                .size(11.0)
                .color(Color32::GRAY),
        );
        return None;
    }

    let commit_count = state.ui.commits.len();
    let pending_count = state.selected_worktree()
        .map(|wt| wt.pending_changes.len())
        .unwrap_or(0);
    let pending_selected = state.ui.viewing_pending;

    ScrollArea::vertical()
        .id_source("graph_scroll")
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(0.0, 4.0);

            // ── Working Tree item (always first) ───────────────────────────────
            let wt_y = ui.cursor().min.y;
            let wt_x = ui.cursor().min.x;
            let wt_w = ui.available_width();

            ui.push_id(("graph", "wt"), |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.add_space(6.0);
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), Sense::hover());
                        let painter = ui.painter();
                        painter.circle_stroke(rect.center(), 5.5, (2.0, NODE_COLOR));
                        if commit_count > 0 {
                            let line_start = rect.center_bottom();
                            let line_end = line_start + Vec2::new(0.0, 10.0);
                            painter.line_segment([line_start, line_end], (1.5, LINE_COLOR));
                        }
                    });
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.add_space(4.0);
                        let color = if pending_selected { sel_color } else { text_color };
                        ui.label(RichText::new("Working Tree").size(13.0).color(color));
                        ui.push_id("wt_meta", |ui| { ui.horizontal(|ui| {
                            let change_text = if pending_count == 0 {
                                "clean".to_string()
                            } else {
                                format!("{pending_count} change{}", if pending_count == 1 { "" } else { "s" })
                            };
                            ui.label(RichText::new(change_text).size(12.0).color(
                                if pending_count > 0 { Color32::from_rgb(217, 90, 48) } else { Color32::GRAY }
                            ));
                        }); });
                        ui.add_space(6.0);
                    });
                });
            });

            let wt_rect = Rect::from_min_max(pos2(wt_x, wt_y), pos2(wt_x + wt_w, ui.cursor().min.y));
            if ui.interact(wt_rect, Id::new("graph_wt_click"), Sense::click()).clicked() {
                action = Some(GraphAction::SelectPending);
            }

            // ── Commit rows ────────────────────────────────────────────────────
            for (i, commit) in state.ui.commits.iter().enumerate() {
                let is_sel = !pending_selected && state.selection.commit_idx == Some(i);

                let row_y = ui.cursor().min.y;
                let row_x = ui.cursor().min.x;
                let row_w = ui.available_width();

                ui.push_id(("graph", i), |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.add_space(6.0);
                            let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), Sense::hover());
                            let painter = ui.painter();
                            painter.circle_filled(rect.center(), 5.5, NODE_COLOR);
                            if !commit.is_head {
                                painter.circle_stroke(rect.center(), 5.5, (2.0, NODE_COLOR));
                            }
                            if i < commit_count - 1 {
                                let line_start = rect.center_bottom();
                                let line_end = line_start + Vec2::new(0.0, 10.0);
                                painter.line_segment([line_start, line_end], (1.5, LINE_COLOR));
                            }
                        });
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.add_space(4.0);
                            let msg_color = if is_sel { sel_color } else { text_color };
                            let msg = if commit.message.chars().count() > 52 {
                                let end = commit.message.char_indices().nth(51).map(|(i, _)| i).unwrap_or(commit.message.len());
                                format!("{}…", &commit.message[..end])
                            } else {
                                commit.message.clone()
                            };
                            ui.label(RichText::new(&msg).size(13.0).color(msg_color));
                            ui.push_id("meta", |ui| { ui.horizontal(|ui| {
                                ui.label(RichText::new(&commit.short_id).size(12.0).monospace()
                                    .color(branch_fg));
                                ui.label(RichText::new(relative_time(&commit.time)).size(11.0).color(Color32::GRAY));
                            }); });
                            for branch in &commit.branches {
                                ui.label(RichText::new(branch).size(11.0)
                                    .color(branch_fg)
                                    .background_color(branch_bg));
                            }
                            ui.add_space(6.0);
                        });
                    });
                });

                let row_rect = Rect::from_min_max(pos2(row_x, row_y), pos2(row_x + row_w, ui.cursor().min.y));
                if ui.interact(row_rect, Id::new(("graph_commit_click", i)), Sense::click()).clicked() {
                    action = Some(GraphAction::SelectCommit(i));
                }
            }
        });

    action
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
