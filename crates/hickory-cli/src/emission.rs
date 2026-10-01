//! Stage-shaped emission, read-only: the commits a re-emission would produce.
//!
//! See `docs/specs/freeform/expression-and-log.md`. **A document is an
//! expression; a repository is the log of its values.** Evaluate a document
//! and you get what is true *now*; git stores the sequence of those values,
//! immutably, with parentage and attribution. So a document that emits history
//! needs git **more**, not less — and this module shows what it would emit
//! without emitting anything.
//!
//! Two rules shape the answer:
//!
//! - **Commit boundaries come from stages.** `stages-write-forward.md` already
//!   establishes the stage — one document plus the files it generates — as a
//!   real boundary that never writes upstream, which is the same shape as a
//!   stack of dependent changes. So **one stage is one commit** by default:
//!   the shape is structural rather than an artifact of the order you typed,
//!   and re-emission is deterministic given the document.
//! - **Nothing may re-produce a *published* commit.** Publication is what
//!   makes emission one-way. Below the floor a commit is a record; above it,
//!   it is a draft that a re-emission may replace. Two guards follow, because
//!   cheap emission plus cheap re-running makes accidental rewriting cheap:
//!   emission appends and never amends below the floor, and a refusal names
//!   the published commit and where the floor is.
//!
//! **Each emitted commit carries the document version that emitted it** — a
//! property to build in from the first day, because retrofitting it means a
//! generation of commits that cannot explain themselves. The loop this seems
//! to open terminates: commit N holds document N, no commit's content depends
//! on a future document, so the circle is a spiral.
//!
//! Nothing here writes. This is step 3 of that design — *show the commits a
//! re-emission would produce* — and the steps that actually emit are not
//! built.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use serde::Serialize;

/// One commit a re-emission would produce: a stage, and the files it owns.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PlannedCommit {
    /// The document that owns the stage — and whose version this commit would
    /// carry, so it can explain itself later.
    pub document: String,
    /// A commit message, from the document's own first heading where it has
    /// one. Derived, and a person's to change.
    pub subject: String,
    /// The files this stage generates, in path order.
    pub files: Vec<String>,
    /// Which frontier commit this would replace, when it maps onto one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaces: Option<String>,
}

/// What a re-emission would do, and what it may not touch.
#[derive(Debug, Clone, Serialize)]
pub struct EmissionPlan {
    pub commits: Vec<PlannedCommit>,
    /// The floor, as `crate::floor` computed it.
    pub floor: Option<crate::floor::Floor>,
    /// Commits below the floor that a re-emission would have had to rewrite.
    /// Always empty in a plan that is allowed to run; non-empty is the
    /// refusal, and it names them.
    pub published: Vec<String>,
    /// One sentence a person reads before deciding anything.
    pub summary: String,
}

impl EmissionPlan {
    /// Whether this plan may run at all.
    pub fn allowed(&self) -> bool {
        self.published.is_empty()
    }
}

/// The first markdown heading in a document, as its commit subject.
fn subject_of(source: &str, fallback: &str) -> String {
    source
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("Emit {fallback}"))
}

/// Plan the commits a re-emission of `docs` would produce.
///
/// `docs` are the stage documents, in pipeline order — one stage, one commit.
/// `outputs` gives each document's generated files.
pub fn plan(root: &Path, docs: &[(String, String, Vec<String>)]) -> EmissionPlan {
    let floor = crate::floor::compute(root);

    let mut commits: Vec<PlannedCommit> = docs
        .iter()
        .map(|(path, source, files)| {
            let mut files = files.clone();
            files.sort();
            files.dedup();
            PlannedCommit {
                subject: subject_of(source, path),
                document: path.clone(),
                files,
                replaces: None,
            }
        })
        .collect();

    // Map each planned commit onto a frontier commit that already touches the
    // same document, oldest first. That is what makes re-emission a
    // REPLACEMENT of drafts rather than an unbounded pile of new commits.
    let mut published = Vec::new();
    if let Some(floor) = &floor {
        let drafts: BTreeSet<&str> = floor.drafts.iter().map(String::as_str).collect();
        for commit in commits.iter_mut() {
            let touching = commits_touching(root, &commit.document);
            if let Some(sha) = touching.iter().find(|sha| drafts.contains(sha.as_str())) {
                commit.replaces = Some(sha.clone());
                continue;
            }
            // A document whose only commits are BELOW the floor is one a
            // re-emission would have to rewrite history to replace.
            if let Some(sha) = touching.first() {
                published.push(sha.clone());
            }
        }
    }
    published.sort();
    published.dedup();

    let summary = if commits.is_empty() {
        "No stages, so there is nothing to emit.".to_string()
    } else if !published.is_empty() {
        format!(
            "This would have to rewrite {} published commit(s), which emission \
             may never do: below the floor a commit is a record, and someone \
             else may be holding it. Nothing was emitted.",
            published.len()
        )
    } else {
        format!(
            "{} stage(s), {} commit(s) — one stage, one commit, so the shape \
             comes from the document rather than from the order you typed. \
             Nothing is emitted: this is what it WOULD do.",
            commits.len(),
            commits.len()
        )
    };

    EmissionPlan {
        commits,
        floor,
        published,
        summary,
    }
}

/// Every commit touching `path`, newest first.
fn commits_touching(root: &Path, path: &str) -> Vec<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "--format=%H", "--", path])
        .output();
    let Ok(out) = out else { return Vec::new() };
    if !out.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

/// Build a plan for every `.md` document under `dir`, weaving each to learn
/// what it generates.
pub async fn plan_for(dir: &Path) -> Result<EmissionPlan> {
    let root = crate::replay::git_root(dir).unwrap_or_else(|| dir.to_path_buf());
    let root = std::fs::canonicalize(&root).unwrap_or(root);

    let mut docs: Vec<(String, String, Vec<String>)> = Vec::new();
    for doc_path in crate::expand_docs(dir)? {
        let source = std::fs::read_to_string(&doc_path)
            .with_context(|| format!("could not read {}", doc_path.display()))?;
        let rel = relative(&root, &doc_path);
        // Weave-only: planning must not run anything. A plan that executed
        // would be a plan with side effects, which is not a plan.
        let files = match crate::run_doc(
            &doc_path,
            &[],
            crate::RunMode::Weave,
            crate::ExecutorChoice::Local,
        )
        .await
        {
            Ok(run) => run.result.files.keys().cloned().collect(),
            // A document that will not weave contributes a commit with no
            // files rather than being dropped: it is still a stage, and
            // saying so is more useful than pretending it is not there.
            Err(_) => Vec::new(),
        };
        docs.push((rel, source, files));
    }
    Ok(plan(&root, &docs))
}

fn relative(root: &Path, path: &Path) -> String {
    let abs = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    abs.strip_prefix(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

pub fn journal_path(root: &Path) -> PathBuf {
    root.join(".hick-journal")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stage_is_a_commit_and_its_subject_comes_from_the_document() {
        let dir = tempfile::tempdir().unwrap();
        let plan = plan(
            dir.path(),
            &[
                (
                    "requirements.hick".into(),
                    "# What the thing must do\n\nprose".into(),
                    vec!["spec.md".into()],
                ),
                (
                    "implementation.hick".into(),
                    "no heading here".into(),
                    vec!["src/main.rs".into(), "src/lib.rs".into()],
                ),
            ],
        );
        assert_eq!(plan.commits.len(), 2);
        assert_eq!(plan.commits[0].subject, "What the thing must do");
        // A document with no heading still gets an honest subject rather than
        // an empty one.
        assert_eq!(plan.commits[1].subject, "Emit implementation.hick");
        // Files in path order, so two runs plan the same commit.
        assert_eq!(plan.commits[1].files, vec!["src/lib.rs", "src/main.rs"]);
    }

    #[test]
    fn no_stages_is_nothing_to_emit_rather_than_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let plan = plan(dir.path(), &[]);
        assert!(plan.commits.is_empty());
        assert!(plan.allowed());
        assert!(plan.summary.contains("nothing to emit"));
    }

    #[test]
    fn a_plan_says_it_emits_nothing() {
        // Step 3 is read-only, and the difference between "what it would do"
        // and "what it did" is the only thing standing between a person and a
        // rewritten history.
        let dir = tempfile::tempdir().unwrap();
        let plan = plan(
            dir.path(),
            &[("a.hick".into(), "# A".into(), vec!["a.txt".into()])],
        );
        assert!(plan.summary.contains("Nothing is emitted"));
    }
}
