mod diff;
mod graph;
mod pending;
mod sidebar;

use crate::{
    git::{discover_repos_in_dir, get_commits, get_file_diff, load_repos_parallel},
    state::{AppState, Selection},
    watcher::{all_watch_paths, spawn_watcher},
};
use eframe::CreationContext;
use egui::{CentralPanel, Context, SidePanel, TopBottomPanel};
use std::sync::mpsc;
use tracing::info;

pub struct App {
    state: AppState,
}

impl App {
    pub fn new(_cc: &CreationContext) -> Self {
        let (tx, rx) = mpsc::channel();
        // Dummy sender — watcher will be re-spawned after initial load
        drop(tx);

        let mut state = AppState::new(rx);
        let paths = state
            .settings
            .repo_paths
            .iter()
            .cloned()
            .chain(
                state
                    .settings
                    .scan_dirs
                    .iter()
                    .flat_map(|d| discover_repos_in_dir(d)),
            )
            .collect::<Vec<_>>();

        if !paths.is_empty() {
            state.repos = load_repos_parallel(&paths);
            info!("loaded {} repos", state.repos.len());
        }

        Self { state }
    }

    /// Poll the watcher channel and reload any changed repos.
    fn poll_watcher(&mut self) {
        let changed: Vec<_> = self
            .state
            .reload_rx
            .as_ref()
            .map(|rx| rx.try_iter().collect())
            .unwrap_or_default();

        for path in changed {
            if let Some(repo) = self.state.repos.iter_mut().find(|r| {
                r.worktrees
                    .iter()
                    .any(|wt| wt.path == path || path.starts_with(&wt.path))
            }) {
                if let Ok(fresh) = crate::git::load_repo(&repo.path) {
                    *repo = fresh;
                }
            }
        }
    }

    fn refresh_commits(&mut self) {
        if let Some(wt) = self.state.selected_worktree() {
            let path = wt.path.clone();
            let limit = self.state.settings.history_limit;
            self.state.ui.commits = get_commits(&path, limit).unwrap_or_default();
        }
    }

    fn refresh_diff(&mut self) {
        if let (Some(wt), Some(file)) = (
            self.state.selected_worktree(),
            self.state.selected_file(),
        ) {
            let repo_path = wt.path.clone();
            let file_path = file.path.clone();
            self.state.ui.diff_hunks =
                get_file_diff(&repo_path, &file_path).unwrap_or_default();
        } else {
            self.state.ui.diff_hunks.clear();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.poll_watcher();

        // Request a repaint shortly so we catch file-watcher events promptly
        ctx.request_repaint_after(std::time::Duration::from_millis(500));

        // ── Left sidebar: repo / worktree tree ─────────────────────────────────
        SidePanel::left("sidebar")
            .resizable(true)
            .min_width(160.0)
            .default_width(210.0)
            .show(ctx, |ui| {
                let action = sidebar::show(ui, &mut self.state);
                if let Some(sel) = action {
                    let changed = sel != self.state.selection;
                    self.state.selection = sel;
                    if changed {
                        self.state.ui.diff_hunks.clear();
                        self.refresh_commits();
                    }
                }
            });

        // ── Middle panel: pending changes + commit graph ────────────────────────
        SidePanel::left("middle")
            .resizable(true)
            .min_width(160.0)
            .default_width(220.0)
            .show(ctx, |ui| {
                let file_sel = pending::show(ui, &mut self.state);
                if let Some(idx) = file_sel {
                    self.state.selection.file_idx = Some(idx);
                    self.refresh_diff();
                }
                ui.separator();
                graph::show(ui, &mut self.state);
            });

        // ── Right panel: diff view ─────────────────────────────────────────────
        CentralPanel::default().show(ctx, |ui| {
            diff::show(ui, &self.state);
        });

        // ── Add-repo dialog ────────────────────────────────────────────────────
        if self.state.ui.show_add_dialog {
            egui::Window::new("Add repository")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Repository path:");
                    ui.text_edit_singleline(&mut self.state.ui.add_path_input);
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.button("Add single repo").clicked() {
                            let p = std::path::PathBuf::from(&self.state.ui.add_path_input);
                            if p.exists() {
                                self.state.settings.repo_paths.push(p.clone());
                                self.state.settings.save();
                                if let Ok(r) = crate::git::load_repo(&p) {
                                    self.state.repos.push(r);
                                }
                            }
                            self.state.ui.show_add_dialog = false;
                        }
                        if ui.button("Add all repos in dir").clicked() {
                            let p = std::path::PathBuf::from(&self.state.ui.add_path_input);
                            let discovered = discover_repos_in_dir(&p);
                            if !discovered.is_empty() {
                                self.state.settings.scan_dirs.push(p);
                                self.state.settings.save();
                                let mut new_repos = load_repos_parallel(&discovered);
                                self.state.repos.append(&mut new_repos);
                            }
                            self.state.ui.show_add_dialog = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.state.ui.show_add_dialog = false;
                        }
                    });
                });
        }
    }
}
