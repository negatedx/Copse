use std::sync::mpsc;
use tracing::{debug, warn};

const RELEASES_API: &str =
    "https://api.github.com/repos/negatedx/gitrove/releases/latest";

/// Spawns a background thread that fetches the latest GitHub release tag.
/// Returns a channel that will receive `Some(tag)` if a newer version exists,
/// or `None` if already up to date or the check fails.
pub fn spawn_update_check() -> mpsc::Receiver<Option<String>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(fetch_latest_version());
    });
    rx
}

fn fetch_latest_version() -> Option<String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("gitrove/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| warn!("update check: failed to build client: {e}"))
        .ok()?;

    let body = client
        .get(RELEASES_API)
        .send()
        .and_then(|r| r.text())
        .map_err(|e| warn!("update check: request failed: {e}"))
        .ok()?;

    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| warn!("update check: invalid JSON: {e}"))
        .ok()?;

    let tag = json["tag_name"].as_str()?;
    if is_newer(tag) {
        debug!("update available: {tag}");
        Some(tag.to_owned())
    } else {
        debug!("already up to date (latest: {tag})");
        None
    }
}

fn is_newer(tag: &str) -> bool {
    let remote = parse_semver(tag.trim_start_matches('v'));
    let current = parse_semver(env!("CARGO_PKG_VERSION"));
    remote > current
}

fn parse_semver(s: &str) -> (u32, u32, u32) {
    let mut parts = s.splitn(3, '.');
    let major = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}
