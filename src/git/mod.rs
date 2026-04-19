use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use git2::{Delta, DiffOptions, Repository, Status, StatusOptions};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
    /// OID of HEAD commit, used to detect new commits without loading history.
    #[serde(default)]
    pub head_oid: Option<String>,
}

impl WorktreeInfo {
    pub fn change_count(&self) -> usize {
        self.pending_changes.len()
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
    #[serde(default)]
    pub head_branch: Option<String>,
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

    let main_wt = match load_worktree(&repo, path, true) {
        Ok(wt) => wt,
        Err(e) => {
            tracing::warn!("failed to load main worktree for {}: {e:#}", path.display());
            return Err(e);
        }
    };
    let mut worktrees = vec![main_wt];

    // Enumerate linked worktrees — non-fatal if the list can't be read.
    match repo.worktrees() {
        Err(e) => tracing::warn!("could not list worktrees for {}: {e:#}", path.display()),
        Ok(wt_names) => {
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
        }
    }

    worktrees.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(RepoInfo {
        name,
        path: path.to_owned(),
        worktrees,
    })
}

/// Load multiple repos in parallel using rayon.
/// Returns `(loaded, owner_error_paths)` — paths where libgit2 rejected the repo
/// due to a safe.directory ownership mismatch are returned separately so the caller
/// can prompt the user to fix them.
pub fn load_repos_parallel(paths: &[PathBuf]) -> (Vec<RepoInfo>, Vec<PathBuf>) {
    let results: Vec<Result<RepoInfo>> = paths.par_iter().map(|p| load_repo(p)).collect();

    let mut loaded = Vec::new();
    let mut owner_errors = Vec::new();

    for (path, result) in paths.iter().zip(results) {
        match result {
            Ok(r) => loaded.push(r),
            Err(e) => {
                if matches!(
                    git2::Repository::open(path),
                    Err(ref ge) if ge.code() == git2::ErrorCode::Owner
                ) {
                    owner_errors.push(path.clone());
                } else {
                    tracing::warn!("skipping {}: {e:#}", path.display());
                }
            }
        }
    }

    (loaded, owner_errors)
}

/// Add `path` to the user's global git `safe.directory` config so libgit2 will
/// open it without an ownership error. Writes to `~/.gitconfig`.
pub fn add_safe_directory(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy().replace('\\', "/");
    let config_path = git2::Config::find_global().unwrap_or_else(|_| {
        dirs::home_dir().unwrap_or_default().join(".gitconfig")
    });
    let mut config = git2::Config::open(&config_path)?;
    // set_multivar with a regexp that won't match any path value adds a new entry.
    config.set_multivar("safe.directory", "^$", &path_str)?;
    Ok(())
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
    let head_oid = repo.head().ok()
        .and_then(|h| h.peel_to_commit().ok())
        .map(|c| c.id().to_string());

    Ok(WorktreeInfo {
        name,
        path: path.to_owned(),
        branch,
        is_main,
        pending_changes,
        ahead_behind,
        head_oid,
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

    // Build OID → branch names map for all local and remote branches.
    let mut oid_branches: HashMap<git2::Oid, Vec<String>> = HashMap::new();
    if let Ok(branches) = repo.branches(None) {
        for branch_result in branches {
            let Ok((branch, _)) = branch_result else { continue };
            let Some(name) = branch.name().ok().flatten() else { continue };
            let name = name.to_string();
            let Some(oid) = branch.get().target() else { continue };
            oid_branches.entry(oid).or_default().push(name);
        }
    }
    for names in oid_branches.values_mut() {
        names.sort();
    }

    let head_ref = repo.head().ok();
    let head_id = head_ref.as_ref().and_then(|h| h.target());
    let head_branch: Option<String> = head_ref
        .as_ref()
        .filter(|h| h.is_branch())
        .and_then(|h| h.shorthand().map(String::from));

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

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

        let is_head = Some(oid) == head_id;
        let branches = oid_branches.remove(&oid).unwrap_or_default();
        let hb = if is_head { head_branch.clone() } else { None };

        commits.push(CommitInfo {
            short_id: oid.to_string()[..7].to_string(),
            id: oid.to_string(),
            message: commit.summary().unwrap_or("").trim().to_string(),
            author: commit.author().name().unwrap_or("").to_string(),
            time,
            is_head,
            branches,
            head_branch: hb,
        });
    }

    Ok(commits)
}

// ── Commit files + diff ───────────────────────────────────────────────────────

pub fn get_commit_files(repo_path: &Path, commit_id: &str) -> Result<Vec<FileChange>> {
    let repo = Repository::open(repo_path)?;
    let oid = git2::Oid::from_str(commit_id)?;
    let commit = repo.find_commit(oid)?;
    let tree = commit.tree()?;
    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

    let mut changes = Vec::new();
    for delta in diff.deltas() {
        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .map(PathBuf::from)
            .unwrap_or_default();
        let status = match delta.status() {
            Delta::Added => ChangeStatus::Added,
            Delta::Deleted => ChangeStatus::Deleted,
            Delta::Renamed => ChangeStatus::Renamed,
            Delta::Conflicted => ChangeStatus::Conflicted,
            _ => ChangeStatus::Modified,
        };
        changes.push(FileChange { path, status });
    }
    changes.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(changes)
}

pub fn get_commit_file_diff(
    repo_path: &Path,
    commit_id: &str,
    file_path: &Path,
) -> Result<Vec<DiffHunk>> {
    let repo = Repository::open(repo_path)?;
    let oid = git2::Oid::from_str(commit_id)?;
    let commit = repo.find_commit(oid)?;
    let tree = commit.tree()?;
    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

    let mut opts = DiffOptions::new();
    opts.pathspec(file_path);
    opts.context_lines(3);

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut opts))?;
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

// ── File diff (working tree) ───────────────────────────────────────────────────

/// Read an untracked/added file and return its full content as synthetic Added hunks.
pub fn get_file_content_as_diff(repo_path: &Path, file_path: &Path) -> Result<Vec<DiffHunk>> {
    let full_path = repo_path.join(file_path);
    let bytes = std::fs::read(&full_path)
        .with_context(|| format!("failed to read {}", full_path.display()))?;

    if bytes.contains(&0u8) {
        return Ok(vec![DiffHunk {
            header: "Binary file (new file)".to_string(),
            lines: vec![],
        }]);
    }

    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<DiffLine> = text
        .lines()
        .enumerate()
        .map(|(i, line)| DiffLine {
            kind: DiffLineKind::Added,
            old_lineno: None,
            new_lineno: Some((i + 1) as u32),
            content: line.to_string(),
        })
        .collect();

    if lines.is_empty() {
        return Ok(vec![]);
    }

    Ok(vec![DiffHunk {
        header: format!("@@ -0,0 +1,{} @@", lines.len()),
        lines,
    }])
}

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
