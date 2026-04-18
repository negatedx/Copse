use crate::state::{AppState, Selection};
use egui::{Color32, RichText, ScrollArea, Sense, Ui, Vec2};

/// Renders the sidebar. Returns a new Selection if the user clicked something.
pub fn show(ui: &mut Ui, state: &mut AppState) -> Option<Selection> {
    let mut new_selection: Option<Selection> = None;

    // ── Header ─────────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.label(RichText::new("REPOS").size(10.0).color(Color32::GRAY));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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
            ui.spacing_mut().item_spacing = Vec2::new(0.0, 1.0);

            let repo_count = state.repos.len();
            for repo_idx in 0..repo_count {
                if !state.repo_matches_filter(repo_idx) {
                    continue;
                }

                let repo = &state.repos[repo_idx];
                let repo_name = repo.name.clone();
                let wt_count = repo.worktrees.len();
                let total_changes: usize =
                    repo.worktrees.iter().map(|w| w.change_count()).sum();
                let is_collapsed = state.ui.collapsed_repos.contains(&repo_idx);
                let force_expand = !state.ui.worktree_filter.is_empty();

                // ── Repo row ───────────────────────────────────────────────────
                let chevron = if is_collapsed && !force_expand { "▶" } else { "▼" };
                let selected_repo = state.selection.repo_idx == Some(repo_idx);

                let repo_resp = ui
                    .horizontal(|ui| {
                        ui.add_space(4.0);
                        ui.label(RichText::new(chevron).size(9.0).color(Color32::GRAY));
                        ui.label(RichText::new(&repo_name).size(12.0).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(4.0);
                            let count_label = RichText::new(format!("{wt_count}")).size(10.0);
                            ui.label(count_label);
                            if total_changes > 0 {
                                ui.label(
                                    RichText::new(format!("●"))
                                        .size(8.0)
                                        .color(Color32::from_rgb(217, 90, 48)),
                                );
                            }
                        });
                    })
                    .response
                    .interact(Sense::click());

                if repo_resp.clicked() {
                    if is_collapsed {
                        state.ui.collapsed_repos.remove(&repo_idx);
                    } else {
                        state.ui.collapsed_repos.insert(repo_idx);
                    }
                    new_selection = Some(Selection::repo(repo_idx));
                }

                // ── Worktree rows ──────────────────────────────────────────────
                if !is_collapsed || force_expand {
                    let matching = state.filtered_worktrees(repo_idx);
                    for wt_idx in matching {
                        let repo = &state.repos[repo_idx];
                        let wt = &repo.worktrees[wt_idx];
                        let wt_name = wt.name.clone();
                        let changes = wt.change_count();
                        let is_active = state.selection.repo_idx == Some(repo_idx)
                            && state.selection.worktree_idx == Some(wt_idx);
                        let is_highlighted = !state.ui.worktree_filter.is_empty();

                        let wt_resp = ui
                            .horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.label(
                                    RichText::new("–")
                                        .size(10.0)
                                        .color(Color32::GRAY),
                                );
                                ui.label(
                                    RichText::new(&wt_name)
                                        .size(11.0)
                                        .color(if is_active {
                                            Color32::WHITE
                                        } else if is_highlighted {
                                            Color32::from_rgb(55, 138, 221)
                                        } else {
                                            Color32::GRAY
                                        }),
                                );
                                if changes > 0 {
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.add_space(4.0);
                                            ui.label(
                                                RichText::new(format!("{changes}"))
                                                    .size(10.0)
                                                    .color(Color32::from_rgb(153, 60, 29)),
                                            );
                                        },
                                    );
                                }
                            })
                            .response
                            .interact(Sense::click());

                        if wt_resp.clicked() {
                            new_selection = Some(Selection::worktree(repo_idx, wt_idx));
                        }
                    }
                }
            }
        });

    new_selection
}
