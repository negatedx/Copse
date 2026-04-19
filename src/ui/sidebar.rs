use crate::git::FileChange;
use crate::state::{AppState, Selection};
use egui::{Color32, Id, Rect, RichText, ScrollArea, Sense, TextEdit, Ui, Vec2, pos2};
use egui_phosphor::regular as ph;

pub enum SidebarAction {
    Select(Selection),
    SelectFile { repo_idx: usize, wt_idx: usize, file_idx: usize },
    RemoveRepo(usize),
}

pub fn show(ui: &mut Ui, state: &mut AppState) -> Option<SidebarAction> {
    let mut action: Option<SidebarAction> = None;
    let text_color = ui.visuals().text_color();
    let sel_color = ui.visuals().strong_text_color();
    let dark = ui.visuals().dark_mode;

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

    // ── Search box ─────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.label(RichText::new(ph::MAGNIFYING_GLASS).size(14.0).color(Color32::GRAY));
        // X is always laid out so the text field width stays stable and focus is kept.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let has_text = !state.ui.worktree_filter.is_empty();
            let x_color = if has_text { Color32::GRAY } else { Color32::TRANSPARENT };
            let clear = ui.add(egui::Label::new(
                RichText::new(ph::X).size(13.0).color(x_color)
            ).sense(if has_text { Sense::click() } else { Sense::hover() }));
            if has_text {
                if clear.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if clear.clicked() {
                    state.ui.worktree_filter.clear();
                }
            }
            ui.add(
                TextEdit::singleline(&mut state.ui.worktree_filter)
                    .hint_text("Filter repos…")
                    .desired_width(f32::INFINITY),
            );
        });
    });

    ui.add_space(6.0);
    ui.separator();

    // ── Repo tree ──────────────────────────────────────────────────────────────
    ScrollArea::vertical()
        .id_source("sidebar_scroll")
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);

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
                let chevron = if is_collapsed && !force_expand { ph::CARET_RIGHT } else { ph::CARET_DOWN };

                // ── Repo header row ────────────────────────────────────────────
                let row_y = ui.cursor().min.y;
                let row_x = ui.cursor().min.x;
                let row_w = ui.available_width();
                let bg_idx = ui.painter().add(egui::Shape::Noop);

                ui.push_id(("repo", repo_idx), |ui| {
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.add_space(6.0);
                        let (chev_rect, chev_resp) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::click());
                        ui.painter().text(
                            chev_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            chevron,
                            egui::FontId::proportional(14.0),
                            if chev_resp.hovered() { text_color } else { Color32::GRAY },
                        );
                        if chev_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if chev_resp.clicked() {
                            if is_collapsed {
                                state.ui.collapsed_repos.remove(&repo_idx);
                            } else {
                                state.ui.collapsed_repos.insert(repo_idx);
                            }
                        }
                        ui.add_space(4.0);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
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
                            if linked_count > 0 {
                                ui.label(RichText::new(format!("{linked_count}")).size(12.0).color(Color32::DARK_GRAY));
                            }
                            if total_changes > 0 {
                                ui.label(RichText::new("●").size(12.0).color(Color32::from_rgb(217, 90, 48)));
                            }
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                ui.add(egui::Label::new(
                                    RichText::new(&repo_name).size(14.0).strong().color(text_color),
                                ).truncate());
                            });
                        });
                    });
                    ui.add_space(5.0);
                });

                // Draw background shape slot (no selection highlight on header row).
                let _ = (bg_idx, Rect::from_min_max(pos2(row_x, row_y), pos2(row_x + row_w, ui.cursor().min.y)));

                // Re-read after push_id closure may have toggled via chevron click.
                if state.ui.collapsed_repos.contains(&repo_idx) && !force_expand {
                    continue;
                }

                // ── Worktree child rows ────────────────────────────────────────
                // Main worktree first, then linked worktrees alphabetically.
                let mut matching = state.filtered_worktrees(repo_idx);
                matching.sort_by_key(|&i| {
                    let wt = &state.repos[repo_idx].worktrees[i];
                    (!wt.is_main, wt.name.to_lowercase())
                });

                for wt_idx in matching {
                    let wt = &state.repos[repo_idx].worktrees[wt_idx];
                    let display_name = wt.branch.clone().unwrap_or_else(|| wt.name.clone());
                    let is_main = wt.is_main;
                    let changes = wt.change_count();
                    let pending_files: Vec<FileChange> = wt.pending_changes.clone();

                    let is_active = state.selection.repo_idx == Some(repo_idx)
                        && state.selection.worktree_idx == Some(wt_idx)
                        && !state.ui.hide_middle_panel;
                    let is_highlighted = !state.ui.worktree_filter.is_empty();
                    let wt_expanded = state.ui.expanded_worktrees.contains(&(repo_idx, wt_idx));
                    let wt_chevron = if wt_expanded { ph::CARET_DOWN } else { ph::CARET_RIGHT };

                    let wt_y = ui.cursor().min.y;
                    let wt_x = ui.cursor().min.x;
                    let wt_w = ui.available_width();
                    let wt_bg_idx = ui.painter().add(egui::Shape::Noop);

                    ui.push_id(("wt", repo_idx, wt_idx), |ui| {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.add_space(20.0);
                            let (chev_rect, chev_resp) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::click());
                            ui.painter().text(
                                chev_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                wt_chevron,
                                egui::FontId::proportional(13.0),
                                if chev_resp.hovered() { text_color } else { Color32::GRAY },
                            );
                            if chev_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if chev_resp.clicked() {
                                if wt_expanded {
                                    state.ui.expanded_worktrees.remove(&(repo_idx, wt_idx));
                                } else {
                                    state.ui.expanded_worktrees.insert((repo_idx, wt_idx));
                                }
                            }
                            ui.add_space(3.0);
                            if is_main {
                                ui.label(RichText::new(ph::HOUSE).size(11.0).color(Color32::GRAY));
                                ui.add_space(2.0);
                            }
                            ui.label(
                                RichText::new(&display_name).size(13.0).color(
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
                        ui.add_space(4.0);
                    });

                    // Worktree click zone starts after the chevron to avoid double-trigger.
                    let wt_rect = Rect::from_min_max(
                        pos2(wt_x + 44.0, wt_y),
                        pos2(wt_x + wt_w, ui.cursor().min.y),
                    );
                    if is_active {
                        let full_wt_rect = Rect::from_min_max(pos2(wt_x, wt_y), pos2(wt_x + wt_w, ui.cursor().min.y));
                        let bg_fill = ui.visuals().selection.bg_fill.gamma_multiply(0.4);
                        ui.painter().set(wt_bg_idx, egui::Shape::rect_filled(full_wt_rect, 0.0, bg_fill));
                        ui.painter().vline(wt_x, wt_y..=full_wt_rect.max.y, egui::Stroke::new(3.0, sel_color));
                    }
                    let wt_resp = ui.interact(wt_rect, Id::new(("wt_click", repo_idx, wt_idx)), Sense::click());
                    if wt_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if wt_resp.clicked() {
                        action = Some(SidebarAction::Select(Selection::worktree(repo_idx, wt_idx)));
                    }

                    // Re-read after chevron click may have toggled it.
                    if state.ui.expanded_worktrees.contains(&(repo_idx, wt_idx)) {
                        show_file_children(
                            ui, &mut action, state, repo_idx, wt_idx,
                            &pending_files, sel_color, text_color, dark,
                            44.0,
                        );
                    }
                }
            }
        });

    action
}

fn show_file_children(
    ui: &mut Ui,
    action: &mut Option<SidebarAction>,
    state: &AppState,
    repo_idx: usize,
    wt_idx: usize,
    files: &[FileChange],
    sel_color: Color32,
    text_color: Color32,
    dark: bool,
    indent: f32,
) {
    for (file_idx, file) in files.iter().enumerate() {
        let fname = file.path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let file_status = file.status.clone();

        let is_file_sel = state.ui.hide_middle_panel
            && state.selection.repo_idx == Some(repo_idx)
            && state.selection.worktree_idx == Some(wt_idx)
            && state.selection.file_idx == Some(file_idx);

        let f_y = ui.cursor().min.y;
        let f_x = ui.cursor().min.x;
        let f_w = ui.available_width();
        let f_bg_idx = ui.painter().add(egui::Shape::Noop);

        ui.push_id(("sf", repo_idx, wt_idx, file_idx), |ui| {
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                ui.add_space(indent);
                let label_color = super::pending::status_color(&file_status, dark);
                ui.label(RichText::new(file_status.label()).size(11.0).strong().color(label_color));
                ui.add_space(4.0);
                ui.label(RichText::new(&fname).size(12.0)
                    .color(if is_file_sel { sel_color } else { text_color }));
            });
            ui.add_space(3.0);
        });

        let f_rect = Rect::from_min_max(pos2(f_x, f_y), pos2(f_x + f_w, ui.cursor().min.y));
        if is_file_sel {
            let bg_fill = ui.visuals().selection.bg_fill.gamma_multiply(0.4);
            ui.painter().set(f_bg_idx, egui::Shape::rect_filled(f_rect, 0.0, bg_fill));
            ui.painter().vline(f_x, f_y..=f_rect.max.y, egui::Stroke::new(3.0, sel_color));
        }
        let f_resp = ui.interact(f_rect, Id::new(("sfc", repo_idx, wt_idx, file_idx)), Sense::click());
        if f_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if f_resp.clicked() {
            *action = Some(SidebarAction::SelectFile { repo_idx, wt_idx, file_idx });
        }
    }
}
