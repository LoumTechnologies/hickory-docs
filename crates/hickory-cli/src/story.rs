//! Editing the past, above the publication floor: reword, reorder, drop.
//!
//! `docs/specs/freeform/lenses.md`, step 6. In the history lens a draft
//! commit is a card, and editing its prose is a reword, moving it is a
//! reorder, deleting it is a drop. Each is **git's own rebase, run as
//! itself**, with git's own words shown when it refuses — never a rewrite
//! this module performs by hand on objects.
//!
//! Three refusals hold the line:
//!
//! * **Below the floor, never.** A commit somebody else may hold is a record
//!   (`expression-and-log.md`); only drafts are rebased. The same line the
//!   Git pane's amend and `hick emit` draw.
//! * **A dirty working tree, never.** A rebase over uncommitted work is a
//!   decision with conflicts in it that nobody asked for.
//! * **A merge among the drafts, never.** Rebasing through a merge
//!   linearises it, which is a story that did not happen. That case goes to
//!   a terminal.
//!
//! ## How a non-interactive interactive rebase works
//!
//! `git rebase -i` asks an editor for the todo list and, for a reword, for
//! the message. The editor is `GIT_SEQUENCE_EDITOR` / `GIT_EDITOR`, and git
//! runs it as `<editor> <file>`. So the editor here is `cp <ours>`, which
//! copies a todo list (or a message) this module wrote over the file git
//! hands it. Nothing is typed; the list is the whole instruction.

use std::path::Path;

use anyhow::{Result, bail};

use crate::recipe::{git_ok, is_clean};

/// Which way to move a card in story order (oldest first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Earlier in time: swap with the commit before it.
    Earlier,
    /// Later in time: swap with the commit after it.
    Later,
}

/// The drafts, oldest first, and the commit the rebase starts from.
struct Drafts {
    /// `None` means the whole history is drafts (`rebase --root`).
    base: Option<String>,
    /// Oldest first.
    shas: Vec<String>,
}

fn drafts(root: &Path) -> Result<Drafts> {
    let floor = crate::floor::compute(root);
    let base = floor.as_ref().and_then(|f| f.sha.clone());
    let range = match &base {
        Some(base) => format!("{base}..HEAD"),
        None => "HEAD".to_string(),
    };
    let shas: Vec<String> = git_ok(root, &["rev-list", "--reverse", &range], None)?
        .lines()
        .map(str::to_string)
        .filter(|l| !l.is_empty())
        .collect();
    Ok(Drafts { base, shas })
}

/// The checks every edit of the past makes first.
fn editable(root: &Path, sha: &str) -> Result<(Drafts, usize)> {
    if !is_clean(root)? {
        bail!(
            "the working tree has uncommitted changes, and editing history over them is a \
             decision with conflicts in it that nobody asked for.\n  Next step: commit or \
             stash what is uncommitted first."
        );
    }
    let sha = git_ok(
        root,
        &["rev-parse", "--verify", &format!("{sha}^{{commit}}")],
        None,
    )?;
    let drafts = drafts(root)?;
    let Some(index) = drafts.shas.iter().position(|s| *s == sha) else {
        bail!(
            "commit {} is below the publication floor — somebody else may already hold it, \
             so it is a record and cannot be reworded, moved or dropped. Make a new commit \
             instead.",
            &sha[..sha.len().min(12)]
        );
    };
    for draft in &drafts.shas {
        let parents = git_ok(root, &["log", "-1", "--format=%P", draft], None)?;
        if parents.split_whitespace().count() > 1 {
            bail!(
                "the drafts include a merge ({}), and rebasing through a merge would \
                 linearise it — a story that did not happen. Edit this history from a \
                 terminal.",
                &draft[..draft.len().min(12)]
            );
        }
    }
    Ok((drafts, index))
}

/// Run `git rebase -i` over the drafts with `todo` as the whole list, and
/// `message` as the editor's answer when a `reword` asks for one.
fn rebase_with(
    root: &Path,
    base: Option<&str>,
    todo: &str,
    message: Option<&str>,
) -> Result<String> {
    let scratch = tempfile::tempdir()?;
    let todo_file = scratch.path().join("todo");
    std::fs::write(&todo_file, todo)?;
    let sequence_editor = format!("cp {}", shell_path(&todo_file));
    let mut command = std::process::Command::new("git");
    command
        .arg("-C")
        .arg(root)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_SEQUENCE_EDITOR", &sequence_editor);
    if let Some(message) = message {
        let message_file = scratch.path().join("message");
        std::fs::write(&message_file, message)?;
        command.env("GIT_EDITOR", format!("cp {}", shell_path(&message_file)));
    }
    command.args(["rebase", "-i"]);
    match base {
        Some(base) => {
            command.arg(base);
        }
        None => {
            command.arg("--root");
        }
    }
    let out = command.output()?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        bail!(
            "git rebase stopped: {}\n  The repository is where git left it (a rebase in \
             progress); finish or abort it from a terminal, the way git says.",
            if stderr.is_empty() { stdout } else { stderr }
        );
    }
    git_ok(root, &["rev-parse", "HEAD"], None)
}

/// A path as the editor command's argument.
fn shell_path(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    if text.contains(' ') {
        format!("'{text}'")
    } else {
        text
    }
}

fn todo_of(shas: &[String], verb_for: impl Fn(&str) -> Option<&'static str>) -> String {
    let mut todo = String::new();
    for sha in shas {
        if let Some(verb) = verb_for(sha) {
            todo.push_str(&format!("{verb} {sha}\n"));
        }
    }
    todo
}

/// Change a draft's message. Returns the new HEAD.
pub fn reword(root: &Path, sha: &str, message: &str) -> Result<String> {
    if message.trim().is_empty() {
        bail!("a commit needs a message; an empty one is not a reword but a mistake.");
    }
    let (drafts, index) = editable(root, sha)?;
    let target = drafts.shas[index].clone();
    let todo = todo_of(&drafts.shas, |s| {
        Some(if s == target { "reword" } else { "pick" })
    });
    rebase_with(root, drafts.base.as_deref(), &todo, Some(message))
}

/// Remove a draft from the history. Returns the new HEAD.
pub fn drop(root: &Path, sha: &str) -> Result<String> {
    let (drafts, index) = editable(root, sha)?;
    let target = drafts.shas[index].clone();
    if drafts.shas.len() == 1 && drafts.base.is_none() {
        bail!("this is the only commit in the repository; dropping it would leave nothing.");
    }
    let todo = todo_of(
        &drafts.shas,
        |s| if s == target { None } else { Some("pick") },
    );
    rebase_with(root, drafts.base.as_deref(), &todo, None)
}

/// Swap a draft with its neighbour. Returns the new HEAD.
pub fn move_commit(root: &Path, sha: &str, direction: Direction) -> Result<String> {
    let (drafts, index) = editable(root, sha)?;
    let mut order = drafts.shas.clone();
    let other = match direction {
        Direction::Earlier if index > 0 => index - 1,
        Direction::Later if index + 1 < order.len() => index + 1,
        Direction::Earlier => bail!("that commit is already the earliest draft."),
        Direction::Later => bail!("that commit is already the latest draft."),
    };
    order.swap(index, other);
    let todo = todo_of(&order, |_| Some("pick"));
    rebase_with(root, drafts.base.as_deref(), &todo, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn sh(root: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn commit_file(root: &Path, name: &str, subject: &str) -> String {
        std::fs::write(root.join(name), format!("{name}\n")).unwrap();
        sh(root, &["add", name]);
        sh(root, &["commit", "-qm", subject]);
        sh(root, &["rev-parse", "HEAD"])
    }

    /// A repository with a published `start` and three drafts A, B, C.
    fn repo() -> (tempfile::TempDir, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        sh(root, &["init", "-q", "-b", "master"]);
        sh(root, &["config", "user.email", "t@example.com"]);
        sh(root, &["config", "user.name", "T"]);
        commit_file(root, "start.txt", "start");
        let remote = tempfile::tempdir().unwrap();
        sh(remote.path(), &["init", "-q", "--bare"]);
        sh(
            root,
            &["remote", "add", "origin", &remote.path().to_string_lossy()],
        );
        sh(root, &["push", "-q", "-u", "origin", "master"]);
        commit_file(root, "a.txt", "A");
        commit_file(root, "b.txt", "B");
        commit_file(root, "c.txt", "C");
        (dir, remote)
    }

    fn subjects(root: &Path) -> String {
        sh(root, &["log", "--reverse", "--format=%s"])
    }

    #[test]
    fn a_draft_can_be_reworded_moved_and_dropped_and_a_record_cannot() {
        // Protects docs/guarantees/lenses/the-past-is-edited-by-rebase-above-the-floor.md
        let (dir, _remote) = repo();
        let root = dir.path();
        let b = sh(root, &["rev-parse", "HEAD~1"]);
        let start = sh(root, &["rev-parse", "HEAD~3"]);

        reword(root, &b, "B, reworded\n\nWith a body.").unwrap();
        assert_eq!(subjects(root), "start\nA\nB, reworded\nC");
        assert_eq!(
            sh(root, &["log", "-1", "--format=%b", "HEAD~1"]),
            "With a body."
        );

        let b = sh(root, &["rev-parse", "HEAD~1"]);
        move_commit(root, &b, Direction::Earlier).unwrap();
        assert_eq!(subjects(root), "start\nB, reworded\nA\nC");
        let b = sh(root, &["rev-parse", "HEAD~2"]);
        move_commit(root, &b, Direction::Later).unwrap();
        assert_eq!(subjects(root), "start\nA\nB, reworded\nC");
        let b = sh(root, &["rev-parse", "HEAD~1"]);
        assert!(move_commit(root, &sh(root, &["rev-parse", "HEAD"]), Direction::Later).is_err());

        drop(root, &b).unwrap();
        assert_eq!(subjects(root), "start\nA\nC");
        assert!(!root.join("b.txt").exists());
        assert!(root.join("c.txt").exists());

        // The record below the floor is refused by name.
        let refused = reword(root, &start, "nope").unwrap_err();
        assert!(
            format!("{refused:#}").contains("below the publication floor"),
            "{refused:#}"
        );
        assert_eq!(subjects(root), "start\nA\nC");
    }

    #[test]
    fn a_dirty_tree_is_refused_before_anything_moves() {
        let (dir, _remote) = repo();
        let root = dir.path();
        std::fs::write(root.join("a.txt"), "changed\n").unwrap();
        let head = sh(root, &["rev-parse", "HEAD"]);
        let refused = drop(root, &head).unwrap_err();
        assert!(
            format!("{refused:#}").contains("uncommitted"),
            "{refused:#}"
        );
        assert_eq!(subjects(root), "start\nA\nB\nC");
    }
}
