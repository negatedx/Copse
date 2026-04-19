use anyhow::Result;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Sender},
    time::Duration,
};
use tracing::{info, warn};

#[allow(dead_code)]
pub fn spawn_watcher(paths: Vec<PathBuf>) -> Result<mpsc::Receiver<PathBuf>> {
    let (tx, rx) = mpsc::channel::<PathBuf>();

    std::thread::spawn(move || {
        if let Err(e) = run_watcher(paths, tx) {
            warn!("file watcher error: {e}");
        }
    });

    Ok(rx)
}

fn run_watcher(paths: Vec<PathBuf>, tx: Sender<PathBuf>) -> Result<()> {
    let (debounce_tx, debounce_rx) = mpsc::channel::<DebounceEventResult>();

    let mut debouncer = new_debouncer(Duration::from_millis(300), debounce_tx)?;

    for path in &paths {
        if path.exists() {
            debouncer.watcher().watch(path, RecursiveMode::Recursive)?;
            info!("watching {}", path.display());
        }
    }

    for result in debounce_rx {
        match result {
            Ok(events) => {
                // Deduplicate — only emit one signal per distinct repo root
                let mut seen = std::collections::HashSet::new();
                for event in events {
                    // Walk up to find the root being watched
                    let root = paths.iter().find(|p| event.path.starts_with(p));
                    if let Some(root) = root {
                        if seen.insert(root.clone()) {
                            let _ = tx.send(root.clone());
                        }
                    }
                }
            }
            Err(e) => warn!("debounce error: {e:?}"),
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn all_watch_paths(repos: &[crate::git::RepoInfo]) -> Vec<PathBuf> {
    repos
        .iter()
        .flat_map(|r| r.worktrees.iter().map(|wt| wt.path.clone()))
        .collect()
}
