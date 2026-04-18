use crate::state::{AppState, Selection};
use egui::{Color32, Id, Rect, RichText, ScrollArea, Sense, Ui, Vec2, pos2};

pub enum SidebarAction {
    Select(Selection),
    RemoveRepo(usize),
}

pub fn show(ui: &mut Ui, state: &mut AppState) -> Option<SidebarAction> {
    let mut action: Option<SidebarAction> = None;
    let text_color = ui.visuals().text_color();
    let sel_color = ui.visuals().strong_text_color();

    // ── Header ─────────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.label(RichText::new("REPOS").size(10.0).color(Color32::GRAY));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(egui::Label::new(
                RichText::new("⚙").size(14.0).color(Color32::GRAY)
            ).sense(Sense::click()))
                .on_hover_text("Settings")
                .clicked()
            {
                state.ui.show_settings = !state.ui.show_settings;
            }
            ui.add_space(4.0);
            if ui.small_button("+").on_hover_text("Add repository").clicked() {
                state.ui.show_add_dialog = true;
            }
        });
    });

    ui.add_space(4.0);

    // ── Search box ─────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.label(RichText::new("⌕").size(14.0).color(Color32::GRAY));
        ui.text_edit_singleline(&mut state.ui.worktree_filter)
            .on_hover_text("Filter worktrees by name or branch");
    });

    ui.add_space(6.0);
    ui.separator();

    // ── Repo tree ──────────────────────────────────────────────────────────────
    ScrollArea::vertical()
        .id_source("sidebar_scroll")
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(0.0, 4.0);

            let repo_count = state.repos.len();
            for repo_idx in 0..repo_count {
                if !state.repo_matches_filter(repo_idx) {
                    continue;
                }

                let repo = &state.repos[repo_idx];
                let repo_name = repo.name.clone();
                let linked_count = repo.worktrees.iter().filter(|w| !w.is_main).count();
                let total_changes: usize = repo.worktrees.iter().map(|w| w.change_count()).sum();
                let is_collapsed = state.ui.collapsed_repos.contains(&repo_idx);
                let force_expand = !state.ui.worktree_filter.is_empty();
                let main_wt_idx = repo.worktrees.iter().position(|w| w.is_main).unwrap_or(0);
                let repo_selected = state.selection.repo_idx == Some(repo_idx)
                    && state.selection.worktree_idx == Some(main_wt_idx);
                let chevron = if is_collapsed && !force_expand { "▶" } else { "▼" };

                // ── Repo row ───────────────────────────────────────────────────
                let row_y = ui.cursor().min.y;
                let row_x = ui.cursor().min.x;
                let row_w = ui.available_width();

                ui.push_id(("repo", repo_idx), |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(6.0);
                        if ui.add(egui::Label::new(
                            RichText::new(chevron).size(11.0).color(Color32::GRAY)
                        ).sense(Sense::click())).clicked() {
                            if is_collapsed {
                                state.ui.collapsed_repos.remove(&repo_idx);
                            } else {
                                state.ui.collapsed_repos.insert(repo_idx);
                            }
                        }
                        ui.label(
                            RichText::new(&repo_name).size(14.0).strong()
                                .color(if repo_selected { sel_color } else { text_color }),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(4.0);
                            let remove_resp = ui.add(egui::Label::new(
                                RichText::new("×").size(14.0).color(Color32::DARK_GRAY)
                            ).sense(Sense::click()))
                                .on_hover_text("Remove repository");
                            if remove_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if remove_resp.clicked() {
                                action = Some(SidebarAction::RemoveRepo(repo_idx));
                            }
                            ui.label(RichText::new(format!("{linked_count}")).size(12.0).color(Color32::DARK_GRAY));
                            if total_changes > 0 {
                                ui.label(RichText::new("●").size(8.0).color(Color32::from_rgb(217, 90, 48)));
                            }
                        });
                    });
                });

                // Leave the rightmost 36px uncovered so the × button can receive clicks.
                let row_rect = Rect::from_min_max(
                    pos2(row_x, row_y),
                    pos2(row_x + row_w - 36.0, ui.cursor().min.y),
                );
                let row_resp = ui.interact(row_rect, Id::new(("repo_click", repo_idx)), Sense::click());
                if row_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if row_resp.clicked() {
                    action = Some(SidebarAction::Select(Selection::worktree(repo_idx, main_wt_idx)));
                }

                // ── Linked worktree rows ───────────────────────────────────────
                if !is_collapsed || force_expand {
                    let matching = state.filtered_worktrees(repo_idx);
                    for wt_idx in matching {
                        let wt = &state.repos[repo_idx].worktrees[wt_idx];
                        if wt.is_main { continue; }

                        let wt_name = wt.name.clone();
                        let changes = wt.change_count();
                        let is_active = state.selection.repo_idx == Some(repo_idx)
                            && state.selection.worktree_idx == Some(wt_idx);
                        let is_highlighted = !state.ui.worktree_filter.is_empty();

                        let wt_y = ui.cursor().min.y;
                        let wt_x = ui.cursor().min.x;
                        let wt_w = ui.available_width();

                        ui.push_id(("wt", repo_idx, wt_idx), |ui| {
                            ui.horizontal(|ui| {
                                ui.add_space(26.0);
                                ui.label(RichText::new("–").size(12.0).color(Color32::DARK_GRAY));
                                ui.label(
                                    RichText::new(&wt_name).size(13.0).color(
                                        if is_active { sel_color }
                                        else if is_highlighted { Color32::from_rgb(55, 138, 221) }
                                        else { text_color }
                                    ),
                                );
                                if changes > 0 {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add_space(6.0);
                                        ui.label(
                                            RichText::new(format!("{changes}"))
                                                .size(12.0)
                                                .color(Color32::from_rgb(153, 60, 29)),
                                        );
                                    });
                                }
                            });
                        });

                        let wt_rect = Rect::from_min_max(
                            pos2(wt_x, wt_y),
                            pos2(wt_x + wt_w, ui.cursor().min.y),
                        );
                        if ui.interact(wt_rect, Id::new(("wt_click", repo_idx, wt_idx)), Sense::click()).clicked() {
                            action = Some(SidebarAction::Select(Selection::worktree(repo_idx, wt_idx)));
                        }
                    }
                }
            }
        });

    action
}
