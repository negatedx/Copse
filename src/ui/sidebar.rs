use crate::state::{AppState, Selection};
use egui::{Color32, Id, Rect, RichText, ScrollArea, Sense, TextEdit, Ui, Vec2, pos2};
use egui_phosphor::regular as ph;

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
            let gear_resp = ui.add(egui::Label::new(
                RichText::new(ph::GEAR_SIX).size(14.0).color(Color32::GRAY)
            ).sense(Sense::click()))
                .on_hover_text("Settings");
            if gear_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if gear_resp.clicked() {
                state.ui.show_settings = !state.ui.show_settings;
            }
            ui.add_space(4.0);
            let add_resp = ui.add(egui::Label::new(
                RichText::new(ph::PLUS).size(14.0).color(Color32::GRAY)
            ).sense(Sense::click()))
                .on_hover_text("Add repository");
            if add_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if add_resp.clicked() {
                state.ui.show_add_dialog = true;
            }
        });
    });

    ui.add_space(4.0);

    // ── Update banner ──────────────────────────────────────────────────────────
    if let Some(ref tag) = state.ui.update_available.clone() {
        if !state.ui.update_dismissed {
            let banner_color = Color32::from_rgb(40, 80, 40);
            let text_color_update = Color32::from_rgb(140, 220, 140);
            egui::Frame::none()
                .fill(banner_color)
                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                .rounding(egui::Rounding::same(4.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("{} Update available: {tag}", egui_phosphor::regular::ARROW_CIRCLE_UP))
                                .size(12.0)
                                .color(text_color_update),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let dismiss = ui.add(
                                egui::Label::new(RichText::new(ph::X).size(12.0).color(Color32::GRAY))
                                    .sense(Sense::click()),
                            );
                            if dismiss.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if dismiss.clicked() {
                                state.ui.update_dismissed = true;
                            }
                            let link = ui.add(
                                egui::Label::new(
                                    RichText::new("Download")
                                        .size(12.0)
                                        .color(Color32::from_rgb(100, 180, 255))
                                        .underline(),
                                )
                                .sense(Sense::click()),
                            );
                            if link.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if link.clicked() {
                                ui.ctx().open_url(egui::OpenUrl::new_tab(
                                    "https://github.com/negatedx/gitrove/releases",
                                ));
                            }
                        });
                    });
                });
            ui.add_space(4.0);
        }
    }

    // ── Search box ─────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.label(RichText::new(ph::MAGNIFYING_GLASS).size(14.0).color(Color32::GRAY));
        ui.add(
            TextEdit::singleline(&mut state.ui.worktree_filter)
                .hint_text("Filter repos…")
                .desired_width(f32::INFINITY),
        );
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
                let chevron = if is_collapsed && !force_expand { ph::CARET_RIGHT } else { ph::CARET_DOWN };

                // ── Repo row ───────────────────────────────────────────────────
                let row_y = ui.cursor().min.y;
                let row_x = ui.cursor().min.x;
                let row_w = ui.available_width();

                // Reserve a background shape slot; filled in once we know the row height.
                let bg_idx = ui.painter().add(egui::Shape::Noop);

                ui.push_id(("repo", repo_idx), |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(6.0);
                        let chevron_resp = ui.add(egui::Label::new(
                            RichText::new(chevron).size(11.0).color(Color32::GRAY)
                        ).sense(Sense::click()));
                        if chevron_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if chevron_resp.clicked() {
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
                                RichText::new(ph::X).size(14.0).color(Color32::DARK_GRAY)
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
                                ui.label(RichText::new("●").size(12.0).color(Color32::from_rgb(217, 90, 48)));
                            }
                        });
                    });
                });

                // Leave the rightmost 36px uncovered so the × button can receive clicks.
                let row_rect = Rect::from_min_max(
                    pos2(row_x, row_y),
                    pos2(row_x + row_w - 36.0, ui.cursor().min.y),
                );
                let full_row_rect = Rect::from_min_max(
                    pos2(row_x, row_y),
                    pos2(row_x + row_w, ui.cursor().min.y),
                );
                if repo_selected {
                    let bg_fill = ui.visuals().selection.bg_fill.gamma_multiply(0.4);
                    ui.painter().set(bg_idx, egui::Shape::rect_filled(full_row_rect, 0.0, bg_fill));
                    ui.painter().vline(row_x, row_y..=full_row_rect.max.y, egui::Stroke::new(3.0, sel_color));
                }
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

                        let wt_bg_idx = ui.painter().add(egui::Shape::Noop);

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
                        if is_active {
                            let bg_fill = ui.visuals().selection.bg_fill.gamma_multiply(0.4);
                            ui.painter().set(wt_bg_idx, egui::Shape::rect_filled(wt_rect, 0.0, bg_fill));
                            ui.painter().vline(wt_x, wt_y..=wt_rect.max.y, egui::Stroke::new(3.0, sel_color));
                        }
                        let wt_resp = ui.interact(wt_rect, Id::new(("wt_click", repo_idx, wt_idx)), Sense::click());
                        if wt_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if wt_resp.clicked() {
                            action = Some(SidebarAction::Select(Selection::worktree(repo_idx, wt_idx)));
                        }
                    }
                }
            }
        });

    action
}
