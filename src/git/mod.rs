use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use git2::{Delta, DiffOptions, Repository, Status, StatusOptions};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// ── Data types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub name: String,
    pub path: PathBuf,
    pub worktrees: Vec<WorktreeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub name: String,
    pub path: PathBuf,
    pub branch: Option<String>,
    pub is_main: bool,
    pub pending_changes: Vec<FileChange>,
    pub ahead_behind: Option<(usize, usize)>,
}

impl WorktreeInfo {
    pub fn change_count(&self) -> usize {
        self.pending_changes.len()
    }

    pub fn is_clean(&self) -> bool {
        self.pending_changes.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: PathBuf,
    pub status: ChangeStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChangeStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
    Renamed,
    Conflicted,
}

impl ChangeStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Modified => "M",
            Self::Added => "A",
            Self::Deleted => "D",
            Self::Untracked => "U",
            Self::Renamed => "R",
            Self::Conflicted => "C",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub id: String,
    pub short_id: String,
    pub message: String,
    pub author: String,
    pub time: DateTime<Local>,
    pub is_head: bool,
    pub branches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiffLineKind {
    Context,
    Added,
    Deleted,
}

// ── Repo discovery ─────────────────────────────────────────────────────────────

/// Find all git repos directly inside a parent directory (non-recursive, one level).
pub fn discover_repos_in_dir(parent: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(parent) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join(".git").exists() {
            found.push(path);
        }
    }
    found.sort();
    found
}

/// Load a single repo and all its registered worktrees.
pub fn load_repo(path: &Path) -> Result<RepoInfo> {
    let repo = Repository::open(path)
        .with_context(|| format!("failed to open repo at {}", path.display()))?;

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    let mut worktrees = vec![load_worktree(&repo, path, true)?];

    // Enumerate linked worktrees
    let wt_names = repo.worktrees()?;
    for wt_name in wt_names.iter().flatten() {
        if let Ok(wt) = repo.find_worktree(wt_name) {
            let wt_path = PathBuf::from(wt.path());
            if let Ok(wt_repo) = Repository::open(&wt_path) {
                if let Ok(info) = load_worktree(&wt_repo, &wt_path, false) {
                    worktrees.push(info);
                }
            }
        }
    }

    Ok(RepoInfo {
        name,
        path: path.to_owned(),
        worktrees,
    })
}

/// Load multiple repos in parallel using rayon.
pub fn load_repos_parallel(paths: &[PathBuf]) -> Vec<RepoInfo> {
    paths
        .par_iter()
        .filter_map(|p| load_repo(p).ok())
        .collect()
}

// ── Worktree loading ───────────────────────────────────────────────────────────

fn load_worktree(repo: &Repository, path: &Path, is_main: bool) -> Result<WorktreeInfo> {
    let branch = current_branch(repo);
    let name = if is_main {
        "main".to_string()
    } else {
        branch.clone().unwrap_or_else(|| {
            path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "worktree".to_string())
        })
    };

    let pending_changes = get_pending_changes(repo)?;
    let ahead_behind = get_ahead_behind(repo).ok();

    Ok(WorktreeInfo {
        name,
        path: path.to_owned(),
        branch,
        is_main,
        pending_changes,
        ahead_behind,
    })
}

fn current_branch(repo: &Repository) -> Option<String> {
    repo.head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from))
}

fn get_pending_changes(repo: &Repository) -> Result<Vec<FileChange>> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_ignored(false);

    let statuses = repo.statuses(Some(&mut opts))?;
    let mut changes = Vec::new();

    for entry in statuses.iter() {
        let status = entry.status();
        let path = PathBuf::from(entry.path().unwrap_or(""));

        let kind = if status.contains(Status::WT_NEW) || status.contains(Status::INDEX_NEW) {
            ChangeStatus::Added
        } else if status.contains(Status::WT_DELETED) || status.contains(Status::INDEX_DELETED) {
            ChangeStatus::Deleted
        } else if status.contains(Status::WT_RENAMED) || status.contains(Status::INDEX_RENAMED) {
            ChangeStatus::Renamed
        } else if status.contains(Status::CONFLICTED) {
            ChangeStatus::Conflicted
        } else if status.intersects(
            Status::WT_MODIFIED
                | Status::INDEX_MODIFIED
                | Status::WT_TYPECHANGE
                | Status::INDEX_TYPECHANGE,
        ) {
            ChangeStatus::Modified
        } else if status.contains(Status::WT_NEW) {
            ChangeStatus::Untracked
        } else {
            continue;
        };

        changes.push(FileChange { path, status: kind });
    }

    changes.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(changes)
}

fn get_ahead_behind(repo: &Repository) -> Result<(usize, usize)> {
    let head = repo.head()?;
    let local = head.target().context("no HEAD target")?;
    let upstream_name = format!(
        "refs/remotes/origin/{}",
        head.shorthand().unwrap_or("main")
    );
    let upstream_ref = repo.find_reference(&upstream_name)?;
    let upstream = upstream_ref.target().context("no upstream target")?;
    Ok(repo.graph_ahead_behind(local, upstream)?)
}

// ── Commit history ─────────────────────────────────────────────────────────────

pub fn get_commits(repo_path: &Path, limit: usize) -> Result<Vec<CommitInfo>> {
    let repo = Repository::open(repo_path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let head_id = repo.head().ok().and_then(|h| h.target());

    let mut commits = Vec::new();
    for (i, oid) in revwalk.enumerate() {
        if i >= limit {
            break;
        }
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let time = Local
            .timestamp_opt(commit.time().seconds(), 0)
            .single()
            .unwrap_or_else(Local::now);

        commits.push(CommitInfo {
            short_id: oid.to_string()[..7].to_string(),
            id: oid.to_string(),
            message: commit
                .summary()
                .unwrap_or("")
                .trim()
                .to_string(),
            author: commit.author().name().unwrap_or("").to_string(),
            time,
            is_head: Some(oid) == head_id,
            branches: vec![],
        });
    }

    Ok(commits)
}

// ── File diff ─────────────────────────────────────────────────────────────────

pub fn get_file_diff(repo_path: &Path, file_path: &Path) -> Result<Vec<DiffHunk>> {
    let repo = Repository::open(repo_path)?;
    let mut opts = DiffOptions::new();
    opts.pathspec(file_path);
    opts.context_lines(3);

    let diff = repo.diff_index_to_workdir(None, Some(&mut opts))?;
    let hunks: RefCell<Vec<DiffHunk>> = RefCell::new(Vec::new());

    diff.foreach(
        &mut |_, _| true,
        None,
        Some(&mut |_, hunk| {
            let header = std::str::from_utf8(hunk.header()).unwrap_or("").trim().to_string();
            hunks.borrow_mut().push(DiffHunk { header, lines: vec![] });
            true
        }),
        Some(&mut |_, hunk, line| {
            if let Some(hunk) = hunk {
                let header = std::str::from_utf8(hunk.header()).unwrap_or("").trim().to_string();
                let kind = match line.origin() {
                    '+' => DiffLineKind::Added,
                    '-' => DiffLineKind::Deleted,
                    _ => DiffLineKind::Context,
                };
                let content = std::str::from_utf8(line.content())
                    .unwrap_or("")
                    .trim_end_matches('\n')
                    .to_string();
                if let Some(h) = hunks.borrow_mut().iter_mut().rev().find(|h| h.header == header) {
                    h.lines.push(DiffLine {
                        kind,
                        old_lineno: line.old_lineno(),
                        new_lineno: line.new_lineno(),
                        content,
                    });
                }
            }
            true
        }),
    )?;

    Ok(hunks.into_inner())
}
