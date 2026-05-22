use std::path::PathBuf;
use std::process::Command;

/// Find the git repository root from the current working directory.
pub(crate) fn git_toplevel() -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if output.status.success() {
        let s = String::from_utf8(output.stdout).ok()?;
        Some(PathBuf::from(s.trim()))
    } else {
        None
    }
}

/// Return the user's home directory from environment variables.
pub(crate) fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .or_else(|| std::env::var("USERPROFILE").ok())
        .map(PathBuf::from)
}

/// Resolve the directory that holds per-machine `.ndjson` eval files.
///
/// Returns `<git-root>/.hick/eval/` when inside a repo,
/// `~/.local/share/hick/eval/` otherwise.
pub fn resolve_store_dir() -> PathBuf {
    if let Some(root) = git_toplevel() {
        return root.join(".hick").join("eval");
    }
    home_dir()
        .map(|h| h.join(".local/share/hick").join("eval"))
        .unwrap_or_else(|| PathBuf::from(".local/share/hick/eval"))
}
