mod diff;
mod graph;
mod pending;
mod settings;
mod sidebar;

use crate::{
    git::{
        discover_repos_in_dir, get_commit_file_diff, get_commit_files, get_commits,
        get_file_content_as_diff, get_file_diff, load_repos_parallel, ChangeStatus,
    },
    state::{AppState, Selection, Theme},
    watcher::{all_watch_paths, spawn_watcher},
};
use eframe::CreationContext;
use egui::{CentralPanel, Context, FontId, SidePanel, TextStyle, Visuals};
use sidebar::SidebarAction;
use std::sync::mpsc;
use tracing::info;

fn apply_font_size(ctx: &Context, font_size: f32) {
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Small, FontId::new(font_size * 0.8, egui::FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(font_size, egui::FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(font_size, egui::FontFamily::Proportional)),
        (TextStyle::Heading, FontId::new(font_size * 1.4, egui::FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(font_size, egui::FontFamily::Monospace)),
    ]
    .into();
    ctx.set_style(style);
}

fn load_font(ctx: &Context, font_name: &str, available_fonts: &[(String, std::path::PathBuf)]) {
    let mut fonts = egui::FontDefinitions::default();
    if !font_name.is_empty() {
        if let Some((_, path)) = available_fonts.iter().find(|(n, _)| n == font_name) {
            if let Ok(data) = std::fs::read(path) {
                fonts.font_data.insert(font_name.to_owned(), egui::FontData::from_owned(data));
                // Apply to both families so diff/monospace text uses the same font.
                for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                    fonts.families.get_mut(&family).unwrap().insert(0, font_name.to_owned());
                }
            } else {
                tracing::warn!("could not read font file: {}", path.display());
            }
        }
    }
    ctx.set_fonts(fonts);
}

pub struct App {
    state: AppState,
    system_dark: bool,
    system_ppp: f32,
    /// Tracks which font is currently loaded so we only call set_fonts when it changes.
    loaded_font_name: String,
}

impl App {
    pub fn new(cc: &CreationContext) -> Self {
        let system_dark = cc
            .integration_info
            .system_theme
            .map(|t| t == eframe::Theme::Dark)
            .unwrap_or(true);

        let system_ppp = cc.egui_ctx.pixels_per_point();

        let (tx, rx) = mpsc::channel();
        drop(tx);

        let mut state = AppState::new(rx);
        let mut paths = state
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
        paths.sort();
        paths.dedup();

        if !paths.is_empty() {
            state.repos = load_repos_parallel(&paths);
            state.repos.sort_by(|a, b| a.name.cmp(&b.name));
            info!("loaded {} repos", state.repos.len());
        }

        // Apply persisted scale immediately
        if state.settings.ui_scale != 1.0 {
            cc.egui_ctx.set_pixels_per_point(system_ppp * state.settings.ui_scale);
        }

        // Populate font list and apply any persisted font before the first frame.
        state.ui.available_fonts = settings::enumerate_fonts();
        let loaded_font_name = state.settings.font_name.clone();
        load_font(&cc.egui_ctx, &loaded_font_name, &state.ui.available_fonts);
        apply_font_size(&cc.egui_ctx, state.settings.font_size);

        Self { state, system_dark, system_ppp, loaded_font_name }
    }

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

    /// Populate files_view from the selected worktree's pending changes.
    fn refresh_files_view(&mut self) {
        if let Some(wt) = self.state.selected_worktree() {
            self.state.ui.files_view = wt.pending_changes.clone();
        } else {
            self.state.ui.files_view.clear();
        }
        self.state.ui.viewing_pending = true;
        self.state.ui.selected_commit_id = None;
        self.state.selection.file_idx = None;
        self.state.ui.diff_hunks.clear();
    }

    fn refresh_diff(&mut self) {
        let file = match self.state.selected_file() {
            Some(f) => f.clone(),
            None => {
                self.state.ui.diff_hunks.clear();
                return;
            }
        };
        let repo_path = match self.state.selected_worktree() {
            Some(wt) => wt.path.clone(),
            None => return,
        };

        self.state.ui.diff_hunks = if self.state.ui.viewing_pending {
            match file.status {
                ChangeStatus::Added | ChangeStatus::Untracked => {
                    get_file_content_as_diff(&repo_path, &file.path).unwrap_or_default()
                }
                _ => get_file_diff(&repo_path, &file.path).unwrap_or_default(),
            }
        } else if let Some(ref commit_id) = self.state.ui.selected_commit_id.clone() {
            get_commit_file_diff(&repo_path, commit_id, &file.path).unwrap_or_default()
        } else {
            vec![]
        };
    }

    fn handle_graph_action(&mut self, action: graph::GraphAction) {
        match action {
            graph::GraphAction::SelectPending => {
                self.state.selection.commit_idx = None;
                if let Some(wt) = self.state.selected_worktree() {
                    self.state.ui.files_view = wt.pending_changes.clone();
                }
                self.state.ui.viewing_pending = true;
                self.state.ui.selected_commit_id = None;
                self.state.selection.file_idx = None;
                self.state.ui.diff_hunks.clear();
            }
            graph::GraphAction::SelectCommit(i) => {
                self.state.selection.commit_idx = Some(i);
                if let Some(commit) = self.state.ui.commits.get(i) {
                    let commit_id = commit.id.clone();
                    if let Some(wt) = self.state.selected_worktree() {
                        let repo_path = wt.path.clone();
                        self.state.ui.files_view =
                            get_commit_files(&repo_path, &commit_id).unwrap_or_default();
                        self.state.ui.selected_commit_id = Some(commit_id);
                        self.state.ui.viewing_pending = false;
                    }
                }
                self.state.selection.file_idx = None;
                self.state.ui.diff_hunks.clear();
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.poll_watcher();

        let visuals = match self.state.settings.theme {
            Theme::Dark => Visuals::dark(),
            Theme::Light => Visuals::light(),
            Theme::System => {
                if self.system_dark {
                    Visuals::dark()
                } else {
                    Visuals::light()
                }
            }
        };
        ctx.set_visuals(visuals);

        let desired_font = self.state.settings.font_name.clone();
        if desired_font != self.loaded_font_name {
            load_font(ctx, &desired_font, &self.state.ui.available_fonts);
            self.loaded_font_name = desired_font;
        }
        apply_font_size(ctx, self.state.settings.font_size);

        ctx.request_repaint_after(std::time::Duration::from_millis(500));

        // ── Left sidebar ───────────────────────────────────────────────────────
        SidePanel::left("sidebar")
            .resizable(true)
            .min_width(160.0)
            .default_width(240.0)
            .show(ctx, |ui| {
                if let Some(action) = sidebar::show(ui, &mut self.state) {
                    match action {
                        SidebarAction::Select(sel) => {
                            let changed = sel != self.state.selection;
                            self.state.selection = sel;
                            if changed {
                                self.refresh_commits();
                                self.refresh_files_view();
                            }
                        }
                        SidebarAction::RemoveRepo(repo_idx) => {
                            if repo_idx < self.state.repos.len() {
                                let path = self.state.repos[repo_idx].path.clone();
                                self.state.repos.remove(repo_idx);
                                self.state.settings.repo_paths.retain(|p| p != &path);
                                self.state.settings.save();
                                // Reset selection if it pointed at this repo
                                if self.state.selection.repo_idx == Some(repo_idx) {
                                    self.state.selection = Selection::default();
                                    self.state.ui.files_view.clear();
                                    self.state.ui.commits.clear();
                                    self.state.ui.diff_hunks.clear();
                                }
                            }
                        }
                    }
                }
            });

        // ── Middle panel: changes + graph ──────────────────────────────────────
        SidePanel::left("middle")
            .resizable(true)
            .min_width(160.0)
            .default_width(260.0)
            .show(ctx, |ui| {
                let file_sel = pending::show(ui, &mut self.state);
                if let Some(idx) = file_sel {
                    self.state.selection.file_idx = Some(idx);
                    self.refresh_diff();
                }

                // Draggable divider between CHANGES and HISTORY
                let (sep_rect, sep_resp) = ui.allocate_exact_size(
                    egui::Vec2::new(ui.available_width(), 6.0),
                    egui::Sense::drag(),
                );
                if sep_resp.hovered() || sep_resp.dragged() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                }
                if sep_resp.dragged() {
                    let new_h = self.state.settings.changes_panel_height + sep_resp.drag_delta().y;
                    self.state.settings.changes_panel_height = new_h.max(60.0);
                }
                if sep_resp.drag_stopped() {
                    self.state.settings.save();
                }
                ui.painter().hline(
                    sep_rect.x_range(),
                    sep_rect.center().y,
                    egui::Stroke::new(1.0, egui::Color32::from_gray(80)),
                );

                if let Some(graph_action) = graph::show(ui, &self.state) {
                    self.handle_graph_action(graph_action);
                }
            });

        // ── Right panel: diff ──────────────────────────────────────────────────
        CentralPanel::default().show(ctx, |ui| {
            diff::show(ui, &self.state);
        });

        // ── Settings window ────────────────────────────────────────────────────
        settings::show(ctx, &mut self.state, self.system_ppp);

        // ── Add-repo: native folder picker ─────────────────────────────────────
        if self.state.ui.show_add_dialog {
            self.state.ui.show_add_dialog = false;
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                if self.state.repos.iter().any(|r| r.path == path) {
                    // already added — do nothing
                } else if git2::Repository::open(&path).is_ok() {
                    self.state.settings.repo_paths.push(path.clone());
                    self.state.settings.save();
                    if let Ok(r) = crate::git::load_repo(&path) {
                        self.state.repos.push(r);
                        self.state.repos.sort_by(|a, b| a.name.cmp(&b.name));
                    }
                } else {
                    self.state.ui.pending_scan_dir = Some(path);
                }
            }
        }

        // ── Add-repo: confirm scan subdirectories ──────────────────────────────
        if let Some(dir) = self.state.ui.pending_scan_dir.clone() {
            egui::Window::new("Add repositories")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(format!("'{}' is not a git repository.", dir.display()));
                    ui.label("Scan its subdirectories and add all repos found?");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Yes, scan subdirectories").clicked() {
                            let discovered = discover_repos_in_dir(&dir);
                            let new_paths: Vec<_> = discovered
                                .into_iter()
                                .filter(|p| !self.state.repos.iter().any(|r| &r.path == p))
                                .collect();
                            if !new_paths.is_empty() {
                                self.state.settings.scan_dirs.push(dir);
                                self.state.settings.save();
                                let mut new_repos = load_repos_parallel(&new_paths);
                                self.state.repos.append(&mut new_repos);
                                self.state.repos.sort_by(|a, b| a.name.cmp(&b.name));
                            }
                            self.state.ui.pending_scan_dir = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.state.ui.pending_scan_dir = None;
                        }
                    });
                });
        }
    }
}
