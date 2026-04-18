use crate::state::AppState;
use egui::{Color32, Id, Pos2, Rect, RichText, ScrollArea, Sense, Ui, Vec2, pos2};

const NODE_COLOR: Color32 = Color32::from_rgb(55, 138, 221);
const LINE_COLOR: Color32 = Color32::from_rgb(80, 80, 80);
const NODE_RADIUS: f32 = 5.5;

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
    let head_branch_bg = if dark { Color32::from_rgb(15, 55, 25) } else { Color32::from_rgb(210, 242, 215) };
    let head_branch_fg = if dark { Color32::from_rgb(80, 200, 100) } else { Color32::from_rgb(15, 120, 40) };

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

            // Collect node centers during layout; draw connecting lines after all rows.
            let mut node_centers: Vec<Pos2> = Vec::new();

            // ── Working Tree item (always first) ───────────────────────────────
            let wt_y = ui.cursor().min.y;
            let wt_x = ui.cursor().min.x;
            let wt_w = ui.available_width();

            let wt_bg_idx = ui.painter().add(egui::Shape::Noop);

            ui.push_id(("graph", "wt"), |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.add_space(6.0);
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), Sense::hover());
                        node_centers.push(rect.center());
                        ui.painter().circle_stroke(rect.center(), NODE_RADIUS, (2.0, NODE_COLOR));
                    });
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.add_space(4.0);
                        let color = if pending_selected { sel_color } else { text_color };
                        ui.label(RichText::new("Working Tree").size(13.0).color(color));
                        ui.push_id("wt_meta", |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 6.0;
                                let change_text = if pending_count == 0 {
                                    "clean".to_string()
                                } else {
                                    format!("{pending_count} change{}", if pending_count == 1 { "" } else { "s" })
                                };
                                ui.label(RichText::new(change_text).size(12.0).color(
                                    if pending_count > 0 { Color32::from_rgb(217, 90, 48) } else { Color32::GRAY }
                                ));
                            });
                        });
                        ui.add_space(6.0);
                    });
                });
            });

            let wt_rect = Rect::from_min_max(pos2(wt_x, wt_y), pos2(wt_x + wt_w, ui.cursor().min.y));
            if pending_selected {
                let bg_fill = ui.visuals().selection.bg_fill.gamma_multiply(0.4);
                ui.painter().set(wt_bg_idx, egui::Shape::rect_filled(wt_rect, 0.0, bg_fill));
                ui.painter().vline(wt_x, wt_y..=wt_rect.max.y, egui::Stroke::new(3.0, sel_color));
            }
            if ui.interact(wt_rect, Id::new("graph_wt_click"), Sense::click()).clicked() {
                action = Some(GraphAction::SelectPending);
            }

            // ── Commit rows ────────────────────────────────────────────────────
            for (i, commit) in state.ui.commits.iter().enumerate() {
                let is_sel = !pending_selected && state.selection.commit_idx == Some(i);

                let row_y = ui.cursor().min.y;
                let row_x = ui.cursor().min.x;
                let row_w = ui.available_width();

                let row_bg_idx = ui.painter().add(egui::Shape::Noop);

                ui.push_id(("graph", i), |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.add_space(6.0);
                            let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), Sense::hover());
                            node_centers.push(rect.center());
                            let painter = ui.painter();
                            painter.circle_filled(rect.center(), NODE_RADIUS, NODE_COLOR);
                            if !commit.is_head {
                                painter.circle_stroke(rect.center(), NODE_RADIUS, (2.0, NODE_COLOR));
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
                            if !commit.branches.is_empty() {
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing.x = 4.0;
                                    for branch in &commit.branches {
                                        let is_hb = commit.head_branch.as_deref() == Some(branch.as_str());
                                        let (fg, bg) = if is_hb { (head_branch_fg, head_branch_bg) } else { (branch_fg, branch_bg) };
                                        let display = if branch.chars().count() > 24 {
                                            let end = branch.char_indices().nth(23).map(|(i, _)| i).unwrap_or(branch.len());
                                            format!("{}…", &branch[..end])
                                        } else {
                                            branch.clone()
                                        };
                                        let label = RichText::new(display).size(11.0).color(fg).background_color(bg);
                                        ui.label(if is_hb { label.strong() } else { label });
                                    }
                                });
                            }
                            ui.label(RichText::new(&msg).size(13.0).color(msg_color));
                            ui.push_id("meta", |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing.x = 6.0;
                                    ui.label(RichText::new(&commit.short_id).size(12.0).monospace()
                                        .color(branch_fg));
                                    ui.label(RichText::new(&commit.author).size(11.0).color(Color32::GRAY));
                                    ui.label(RichText::new(commit.time.format("%d/%m/%y\u{00A0}%H:%M").to_string()).size(11.0).color(Color32::GRAY));
                                });
                            });
                            ui.add_space(6.0);
                        });
                    });
                });

                let row_rect = Rect::from_min_max(pos2(row_x, row_y), pos2(row_x + row_w, ui.cursor().min.y));
                if is_sel {
                    let bg_fill = ui.visuals().selection.bg_fill.gamma_multiply(0.4);
                    ui.painter().set(row_bg_idx, egui::Shape::rect_filled(row_rect, 0.0, bg_fill));
                    ui.painter().vline(row_x, row_y..=row_rect.max.y, egui::Stroke::new(3.0, sel_color));
                }
                if ui.interact(row_rect, Id::new(("graph_commit_click", i)), Sense::click()).clicked() {
                    action = Some(GraphAction::SelectCommit(i));
                }
            }

            // Draw connecting lines between consecutive node centers, clear of the circles.
            if node_centers.len() > 1 {
                let painter = ui.painter();
                for window in node_centers.windows(2) {
                    let from = window[0] + Vec2::new(0.0, NODE_RADIUS);
                    let to   = window[1] - Vec2::new(0.0, NODE_RADIUS);
                    painter.line_segment([from, to], (1.5, LINE_COLOR));
                }
            }

            // Dummy allocation ensures commit_count is referenced to suppress unused warning.
            let _ = commit_count;
        });

    action
}
