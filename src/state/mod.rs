use crate::git::{CommitInfo, DiffHunk, FileChange, RepoInfo, WorktreeInfo};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
    System,
}

/// Persisted settings saved to disk between sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Repos explicitly added by the user.
    pub repo_paths: Vec<PathBuf>,
    /// Directories from which all repos were auto-discovered.
    pub scan_dirs: Vec<PathBuf>,
    /// How many commits to load in the history panel.
    pub history_limit: usize,
    /// UI colour theme.
    #[serde(default)]
    pub theme: Theme,
    /// UI scale multiplier applied on top of the system DPI (1.0 = native).
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
}

fn default_ui_scale() -> f32 {
    1.0
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            repo_paths: Vec::new(),
            scan_dirs: Vec::new(),
            history_limit: 100,
            theme: Theme::Dark,
            ui_scale: 1.0,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = settings_path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }
}

fn settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("gitwatcher")
        .join("settings.json")
}

// ── Selection state ────────────────────────────────────────────────────────────

/// Which repo + worktree the user has selected in the sidebar.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Selection {
    pub repo_idx: Option<usize>,
    pub worktree_idx: Option<usize>,
    pub file_idx: Option<usize>,
    pub commit_idx: Option<usize>,
}

impl Selection {
    pub fn repo(idx: usize) -> Self {
        Self {
            repo_idx: Some(idx),
            ..Default::default()
        }
    }

    pub fn worktree(repo: usize, wt: usize) -> Self {
        Self {
            repo_idx: Some(repo),
            worktree_idx: Some(wt),
            ..Default::default()
        }
    }
}

// ── UI state that does not need persisting ─────────────────────────────────────

#[derive(Debug, Default)]
pub struct UiState {
    pub worktree_filter: String,
    pub collapsed_repos: std::collections::HashSet<usize>,
    pub commits: Vec<CommitInfo>,
    pub diff_hunks: Vec<DiffHunk>,
    pub show_add_dialog: bool,
    pub show_settings: bool,
    pub pending_scan_dir: Option<PathBuf>,
    /// Files shown in the CHANGES panel (pending or from a selected commit).
    pub files_view: Vec<FileChange>,
    /// True when showing working-tree pending changes; false when showing a commit.
    pub viewing_pending: bool,
    /// Commit id currently being viewed (None when viewing pending changes).
    pub selected_commit_id: Option<String>,
}

// ── Top-level app state ────────────────────────────────────────────────────────

pub struct AppState {
    pub settings: Settings,
    pub repos: Vec<RepoInfo>,
    pub selection: Selection,
    pub ui: UiState,
    /// Channel for receiving reload signals from the file watcher.
    pub reload_rx: Option<std::sync::mpsc::Receiver<PathBuf>>,
}

impl AppState {
    pub fn new(reload_rx: std::sync::mpsc::Receiver<PathBuf>) -> Self {
        Self {
            settings: Settings::load(),
            repos: Vec::new(),
            selection: Selection::default(),
            ui: UiState::default(),
            reload_rx: Some(reload_rx),
        }
    }

    // ── Convenience accessors ──────────────────────────────────────────────────

    pub fn selected_repo(&self) -> Option<&RepoInfo> {
        self.selection.repo_idx.and_then(|i| self.repos.get(i))
    }

    pub fn selected_worktree(&self) -> Option<&WorktreeInfo> {
        let repo = self.selected_repo()?;
        self.selection
            .worktree_idx
            .and_then(|i| repo.worktrees.get(i))
    }

    pub fn selected_file(&self) -> Option<&FileChange> {
        self.selection.file_idx.and_then(|i| self.ui.files_view.get(i))
    }

    // ── Filtering ──────────────────────────────────────────────────────────────

    /// Returns indices of worktrees matching the current filter, per repo.
    pub fn filtered_worktrees(&self, repo_idx: usize) -> Vec<usize> {
        let Some(repo) = self.repos.get(repo_idx) else {
            return vec![];
        };
        let filter = self.ui.worktree_filter.to_lowercase();
        if filter.is_empty() {
            return (0..repo.worktrees.len()).collect();
        }
        repo.worktrees
            .iter()
            .enumerate()
            .filter(|(_, wt)| {
                wt.name.to_lowercase().contains(&filter)
                    || wt
                        .branch
                        .as_deref()
                        .map(|b| b.to_lowercase().contains(&filter))
                        .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// True if a repo has any matching worktrees under the current filter.
    pub fn repo_matches_filter(&self, repo_idx: usize) -> bool {
        self.ui.worktree_filter.is_empty() || !self.filtered_worktrees(repo_idx).is_empty()
    }
}
