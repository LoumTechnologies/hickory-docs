//! Replay: exact lineage at any commit.
//!
//! See `docs/specs/freeform/provenance-across-versions.md`. `hick lineage`
//! answers *which span produced these bytes* exactly, and only for the
//! document as it stands. Blame back to an older commit lands in that
//! commit's document, and nothing relates that document's spans to today's.
//!
//! Replay needs no new data model at all: each commit contains its document,
//! a weave is deterministic, and weave-only checks are already affordable
//! (`/render` and the refactor badge never execute). So exact lineage is
//! **recomputable at any commit** — read the old document out of git, weave
//! it, map. What it cannot do is relate two versions to each other; it gives
//! the state *at* a commit, never the thread between commits. That thread is
//! a recorded correspondence, and is deliberately later.
//!
//! **The one limit is stated rather than discovered.** Replay weaves an old
//! document with today's binary, which asks for a grammar-compatibility
//! promise the product has not made — `pre-launch.md` reserves the right to
//! break the grammar, and says document formats deserve more care than code
//! precisely because a `.hick` file outlives any version of the tool. So the
//! honest claim is **replay works back to the last grammar change**, and a
//! commit past that boundary says so in those words instead of failing with
//! a parse error nobody can act on.
//!
//! Nothing here checks anything out. The tree at a commit is materialized
//! into a scratch directory with `git archive`, so the working tree is
//! untouched and the up-loop never sees a file appear — and the whole tree
//! rather than the document alone, because a document's pastes, includes and
//! upstream edges read their siblings, and replaying without them would
//! report a lineage the old commit never had.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context as _, Result, bail};
use serde::Serialize;

use crate::{DocRun, ExecutorChoice, RunMode, run_doc};

/// One commit that touched a document.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReplayCommit {
    pub sha: String,
    pub short: String,
    /// Unix seconds. Formatted by the reader, in their own locale.
    pub time: i64,
    pub author: String,
    pub subject: String,
    /// The path the document had AT this commit. `--follow` means the slider
    /// walks across renames, and a replay has to read the old name.
    pub path: String,
}

/// What a replay at one commit produced.
pub enum Replay {
    /// The old document wove. Its lineage is exact, recomputed — not
    /// recalled from any store.
    Woven(Box<DocRun>),
    /// The old document does not parse with today's binary. Stated as the
    /// boundary it is rather than as a failure.
    GrammarBoundary { commit: String, detail: String },
}

/// The message the boundary reads as, everywhere it is reported.
pub fn grammar_boundary_message(commit: &str, path: &str, detail: &str) -> String {
    format!(
        "replay stops here: {path} at {commit} does not parse with this \
         version of hick, which means the grammar changed between that \
         commit and now.\n  \
         Replay works back to the last grammar change — the document is \
         still exactly as it was, and `git show {commit}:{path}` reads it.\n  \
         Nothing is wrong with the commit; this version of the tool cannot \
         weave it.\n  \
         (parser said: {detail})"
    )
}

// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

const FIELD: char = '\u{1f}';
const RECORD: char = '\u{1e}';

/// Every commit that touched `rel` (repository-relative), newest first.
///
/// A directory that is not a repository, a machine with no `git`, and a file
/// with no history are all the same answer — an empty list — because opening
/// a folder of notes that is not under version control is entirely normal and
/// the slider simply has nowhere to go.
pub fn history(root: &Path, rel: &str, limit: usize) -> Vec<ReplayCommit> {
    let format = format!("{RECORD}%H{FIELD}%h{FIELD}%at{FIELD}%an{FIELD}%s");
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "log",
            &format!("-n{limit}"),
            // Renames as renames: a document that moved is one history, and
            // a slider that stopped at the rename would say the document was
            // born there.
            "--follow",
            "--name-only",
            &format!("--pretty=format:{format}"),
            "--",
            rel,
        ])
        .output();
    let Ok(out) = out else { return Vec::new() };
    if !out.status.success() {
        return Vec::new();
    }
    parse_history(&String::from_utf8_lossy(&out.stdout), rel)
}

/// Split out and pure so the awkward parts are arguable in a test.
fn parse_history(text: &str, fallback_path: &str) -> Vec<ReplayCommit> {
    let mut commits = Vec::new();
    for record in text.split(RECORD) {
        if record.trim().is_empty() {
            continue;
        }
        let mut parts = record.split(FIELD);
        let sha = parts.next().unwrap_or_default().trim().to_string();
        if sha.is_empty() {
            continue;
        }
        let short = parts.next().unwrap_or_default().trim().to_string();
        let time: i64 = parts.next().unwrap_or_default().trim().parse().unwrap_or(0);
        let author = parts.next().unwrap_or_default().to_string();
        let rest = parts.collect::<Vec<_>>().join(&FIELD.to_string());
        // `--name-only` appends the paths after the subject, separated by a
        // blank line. The FIRST of them is the name the document had at this
        // commit, which is what a replay must read.
        let mut lines = rest.lines();
        let subject = lines.next().unwrap_or_default().trim().to_string();
        let path = lines
            .map(str::trim)
            .find(|l| !l.is_empty())
            .unwrap_or(fallback_path)
            .to_string();
        commits.push(ReplayCommit {
            sha,
            short,
            time,
            author,
            subject,
            path,
        });
    }
    commits
}

// ---------------------------------------------------------------------------
// Replay
// ---------------------------------------------------------------------------

/// Materialize the tree at `commit` into `dest`, without touching the working
/// tree. `git archive` reads objects; nothing is checked out and no worktree
/// is added.
fn materialize(root: &Path, commit: &str, dest: &Path) -> Result<()> {
    let archive = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["archive", "--format=tar", commit])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("failed to run `git archive` (is git installed?)")?;
    if !archive.status.success() {
        bail!(
            "could not read the tree at {commit}: {}",
            String::from_utf8_lossy(&archive.stderr).trim()
        );
    }
    tar::Archive::new(archive.stdout.as_slice())
        .unpack(dest)
        .with_context(|| format!("could not unpack the tree at {commit}"))?;
    Ok(())
}

/// Weave the document `rel` as it stood at `commit`, and return its run.
///
/// **Weave-only, never execute.** A replay of last March must not run last
/// March's commands: they would run against today's machine, today's network
/// and today's containers, and whatever came out would be neither what
/// happened then nor what happens now. Cached transcripts where present,
/// never-run everywhere else — the same mode `/render` and the refactor badge
/// use.
pub async fn replay_at(root: &Path, rel: &str, commit: &str) -> Result<Replay> {
    let scratch = tempfile::tempdir().context("could not create a scratch directory")?;
    materialize(root, commit, scratch.path())?;

    let doc_path = scratch.path().join(rel);
    if !doc_path.is_file() {
        bail!(
            "{rel} does not exist at {commit}.\n  \
             The slider walks the commits that touched this document, so this \
             usually means the path was different then — `git log --follow -- \
             {rel}` names the commits and the names it had."
        );
    }

    // Parse first, and separately, so the grammar boundary is REPORTED as
    // itself. Weaving would report the same failure wrapped in pipeline
    // language, which is exactly the obscure parse failure this must not be.
    let source = std::fs::read_to_string(&doc_path)
        .with_context(|| format!("could not read {rel} at {commit}"))?;
    if let Err(e) = hick_lang::parse(&source) {
        return Ok(Replay::GrammarBoundary {
            commit: commit.to_string(),
            detail: e.to_string(),
        });
    }

    match run_doc(&doc_path, &[], RunMode::Weave, ExecutorChoice::Local).await {
        Ok(run) => Ok(Replay::Woven(Box::new(run))),
        // A document that parses but will not weave is a different fact from
        // a grammar change, and saying so is the whole point of splitting the
        // two: this one is about the commit, not about the tool.
        Err(e) => bail!(
            "{rel} at {commit} parses but does not weave with this version of \
             hick ({e:#}).\n  \
             This is not the grammar boundary — the document is readable. It \
             usually means the commit referred to something outside its own \
             tree, or to a feature this build does not have."
        ),
    }
}

/// The repository root for `dir`, or `None` when it is not a work tree.
pub fn git_root(dir: &Path) -> Option<PathBuf> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(PathBuf::from(
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(fields: &[&str]) -> String {
        format!("{RECORD}{}", fields.join(&FIELD.to_string()))
    }

    #[test]
    fn a_commit_carries_the_name_the_document_had_then() {
        // The whole reason for `--follow`: a slider that stopped at a rename
        // would say the document was born there.
        let text = record(&[
            "abc123",
            "abc123d",
            "1700000000",
            "Ada Lovelace",
            "Rename the document\n\ndocs/old-name.hick\n",
        ]);
        let commits = parse_history(&text, "docs/new-name.hick");
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].sha, "abc123");
        assert_eq!(commits[0].subject, "Rename the document");
        assert_eq!(commits[0].path, "docs/old-name.hick");
        assert_eq!(commits[0].time, 1_700_000_000);
    }

    #[test]
    fn a_commit_with_no_name_block_keeps_the_path_asked_about() {
        let text = record(&["a", "a", "1", "N", "Subject"]);
        let commits = parse_history(&text, "app.hick");
        assert_eq!(commits[0].path, "app.hick");
    }

    #[test]
    fn an_empty_history_is_no_commits_rather_than_a_broken_one() {
        assert!(parse_history("", "app.hick").is_empty());
        assert!(parse_history("\n\n", "app.hick").is_empty());
    }

    #[test]
    fn the_boundary_says_replay_works_back_to_the_last_grammar_change() {
        // The exact words matter: this is the sentence that tells a reader
        // the tool is at its limit rather than that their document is broken.
        let msg = grammar_boundary_message("abc123", "app.hick", "unexpected tag");
        assert!(msg.contains("Replay works back to the last grammar change"));
        assert!(msg.contains("git show abc123:app.hick"));
        assert!(msg.contains("Nothing is wrong with the commit"));
    }
}
