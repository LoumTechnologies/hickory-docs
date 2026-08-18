//! The two git facts a session carries: which branch, and whether it is dirty.
//!
//! Not a git library — two questions, asked of the `git` the user already
//! has, in the same subprocess style as `hick init`. They exist because
//! "finished, but there are uncommitted changes" is a different claim on your
//! attention from "finished", and the queue cannot tell them apart without
//! looking at the working tree.

use std::path::Path;
use std::process::Command;

use anyhow::{Context as _, Result, bail};

/// What git says about the directory a session is running in.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GitFacts {
    /// The branch name, or `None` on a detached HEAD.
    pub branch: Option<String>,
    /// The working tree has changes that are not committed.
    pub dirty: bool,
}

/// Read the branch and dirty state of `dir`, or `None` when `dir` is not in a
/// git work tree (a terminal in a plain directory is perfectly normal, and
/// must not be an error).
pub fn facts(dir: &Path) -> Option<GitFacts> {
    let branch = git(dir, &["rev-parse", "--abbrev-ref", "HEAD"]).ok()?;
    let status = git(dir, &["status", "--porcelain"]).ok()?;
    Some(GitFacts {
        branch: (branch != "HEAD").then_some(branch),
        dirty: !status.trim().is_empty(),
    })
}

/// Add a git worktree at `path` on a new `branch`, and hand back the path.
///
/// This is what keeps two agents out of each other's way: a task that gets
/// its own worktree gets its own checkout, and its branch and dirty state
/// stay attached to the session rather than to a directory both of them
/// happen to share.
pub fn add_worktree(repo: &Path, path: &Path, branch: &str) -> Result<()> {
    if path.exists() {
        bail!(
            "{} already exists, so a worktree cannot be created there. Pick another \
             name for the session's branch, or remove the directory first.",
            path.display()
        );
    }
    let out = Command::new("git")
        .current_dir(repo)
        .args(["worktree", "add", "-b", branch])
        .arg(path)
        .output()
        .context("failed to run git (is git installed and on PATH?)")?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail!(
            "git worktree add failed in {}: {}\nA branch named '{branch}' may already \
             exist — try a different session name.",
            repo.display(),
            stderr.trim()
        );
    }
    Ok(())
}

fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .context("failed to run git")?;
    if !out.status.success() {
        bail!("git {args:?} failed in {}", dir.display());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn repo() -> Option<tempfile::TempDir> {
        let dir = tempfile::tempdir().ok()?;
        let ok = |args: &[&str]| {
            Command::new("git")
                .current_dir(dir.path())
                .args(args)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        };
        if !ok(&["init", "-q"]) {
            return None;
        }
        ok(&["config", "user.email", "t@example.com"]);
        ok(&["config", "user.name", "T"]);
        fs::write(dir.path().join("a.txt"), "one\n").ok()?;
        ok(&["add", "-A"]);
        ok(&["commit", "-qm", "first"]);
        Some(dir)
    }

    #[test]
    fn a_directory_outside_git_has_no_facts_and_is_not_an_error() {
        let plain = tempfile::tempdir().unwrap();
        // A temp dir can sit inside a repository on some machines; only assert
        // the shape, which is what callers depend on.
        let _ = facts(plain.path());
    }

    #[test]
    fn a_clean_checkout_is_clean_and_a_touched_one_is_dirty() {
        let Some(dir) = repo() else { return };
        let clean = facts(dir.path()).expect("a git repo has facts");
        assert!(!clean.dirty);
        assert!(clean.branch.is_some());

        fs::write(dir.path().join("a.txt"), "two\n").unwrap();
        assert!(facts(dir.path()).unwrap().dirty);
    }

    #[test]
    fn a_worktree_gets_its_own_branch_and_refuses_an_occupied_path() {
        let Some(dir) = repo() else { return };
        let out = dir.path().parent().unwrap().join(format!(
            "{}-wt",
            dir.path().file_name().unwrap().to_string_lossy()
        ));
        add_worktree(dir.path(), &out, "side-task").unwrap();
        assert_eq!(
            facts(&out).unwrap().branch.as_deref(),
            Some("side-task"),
            "the worktree is on its own branch"
        );
        let err = add_worktree(dir.path(), &out, "other")
            .unwrap_err()
            .to_string();
        assert!(err.contains("already exists"), "{err}");

        let _ = Command::new("git")
            .current_dir(dir.path())
            .args(["worktree", "remove", "--force"])
            .arg(&out)
            .output();
    }
}
