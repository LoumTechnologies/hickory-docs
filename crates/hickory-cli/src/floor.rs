//! The publication floor: which commits on this branch are still drafts.
//!
//! See `docs/specs/freeform/expression-and-log.md`. A document is an
//! expression and a repository is the log of its values, so a document that
//! emits history needs git *more*, not less — and the one boundary that
//! decides what emission may touch is publication:
//!
//! > **The floor is publication**, computed and not felt: `merge-base(HEAD,
//! > origin/master)`, or whatever the repository's published ref is. Below
//! > it, commits are records. Above it is the **frontier**, and the frontier
//! > is derived — edit the document, re-emit, and it is replaced.
//!
//! Nothing here emits anything. This computes the fact and surfaces it, which
//! is useful on its own: the mutable/immutable boundary lives in the commit
//! graph and never in the document text, and a person cannot respect a
//! boundary they cannot see.
//!
//! **Merging moves the floor**, and yesterday's rewritable commits become
//! permanent. That is why this is computed on every read rather than recorded
//! anywhere: a recorded floor would be a claim about the past that the next
//! fetch falsifies.

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use serde::Serialize;

/// Where the floor was found, so the UI can say *why* these commits are
/// drafts rather than only that they are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FloorSource {
    /// This branch's own upstream (`@{upstream}`) — the ref it pushes to.
    Upstream,
    /// The remote's default branch (`origin/HEAD`), when the branch has no
    /// upstream of its own: a feature branch nobody has pushed yet.
    RemoteDefault,
    /// A remote branch named like the default, found by name.
    RemoteNamed,
    /// No published ref at all. Every commit is a draft, and saying that is
    /// honest — a repository with no remote has published nothing.
    None,
}

/// The computed floor.
#[derive(Debug, Clone, Serialize)]
pub struct Floor {
    /// The published ref the floor was computed against, when there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_ref: Option<String>,
    /// `merge-base(HEAD, published_ref)`. Absent when there is no published
    /// ref, or when HEAD and it share no history.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    pub source: FloorSource,
    /// Commits reachable from HEAD but not from the floor: the frontier.
    /// These are drafts — re-emission may replace them.
    pub drafts: Vec<String>,
    /// One sentence a person can read. Never "unpushed", which is about a
    /// transport; publication is about whether someone else could be holding
    /// the commit.
    pub summary: String,
}

impl Floor {
    /// Nothing published: every commit above the root is a draft, and the
    /// honest reading is that this repository has published nothing.
    fn unpublished(drafts: Vec<String>) -> Self {
        let summary = if drafts.is_empty() {
            "No published ref, and no commits — nothing is a record yet.".to_string()
        } else {
            format!(
                "No published ref (no remote-tracking branch), so all {} commit(s) \
                 here are drafts: nobody else can be holding them.",
                drafts.len()
            )
        };
        Self {
            published_ref: None,
            sha: None,
            source: FloorSource::None,
            drafts,
            summary,
        }
    }

    /// Whether `sha` is above the floor.
    pub fn is_draft(&self, sha: &str) -> bool {
        self.drafts.iter().any(|d| d == sha)
    }
}

fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// The ref this repository publishes to, and how it was found.
fn published_ref(dir: &Path) -> (Option<String>, FloorSource) {
    // The branch's own upstream first: it is the ref THIS branch pushes to,
    // and on a feature branch that has been pushed it is the only correct
    // answer — `origin/master` would call already-published commits drafts.
    if let Some(up) = git(
        dir,
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
    ) && !up.is_empty()
    {
        return (Some(up), FloorSource::Upstream);
    }
    if let Some(head) = git(
        dir,
        &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
    ) && !head.is_empty()
    {
        return (Some(head), FloorSource::RemoteDefault);
    }
    for candidate in ["origin/master", "origin/main"] {
        if git(dir, &["rev-parse", "--verify", "--quiet", candidate]).is_some() {
            return (Some(candidate.to_string()), FloorSource::RemoteNamed);
        }
    }
    (None, FloorSource::None)
}

/// Compute the floor for the repository at `dir`.
///
/// Returns `None` when `dir` is not a git work tree — which is an entirely
/// normal thing for a folder of notes to be, and not an error.
pub fn compute(dir: &Path) -> Option<Floor> {
    git(dir, &["rev-parse", "--is-inside-work-tree"])?;

    let all_reachable = |exclude: Option<&str>| -> Vec<String> {
        let mut args = vec!["rev-list", "HEAD"];
        let excluded;
        if let Some(floor) = exclude {
            excluded = format!("^{floor}");
            args.push(&excluded);
        }
        git(dir, &args)
            .map(|s| s.lines().map(str::to_string).collect())
            .unwrap_or_default()
    };

    let (published, source) = published_ref(dir);
    let Some(published) = published else {
        return Some(Floor::unpublished(all_reachable(None)));
    };

    let Some(base) = git(dir, &["merge-base", "HEAD", &published]).filter(|s| !s.is_empty()) else {
        // Shares no history with the published ref — an orphan branch. Every
        // commit on it is a draft, and the reason is worth naming rather than
        // reporting as "no floor".
        let drafts = all_reachable(None);
        let mut floor = Floor::unpublished(drafts);
        floor.published_ref = Some(published.clone());
        floor.source = source;
        floor.summary = format!(
            "This branch shares no history with {published}, so nothing here \
             has been published: every commit is a draft."
        );
        return Some(floor);
    };

    let drafts = all_reachable(Some(&base));
    let summary = if drafts.is_empty() {
        format!(
            "Everything on this branch is at or below {published} — every commit \
             here is a record, and re-emission may not touch any of them."
        )
    } else {
        format!(
            "{} commit(s) above the floor at {published}: still drafts, because \
             nobody else can be holding them yet. Everything below is a record.",
            drafts.len()
        )
    };

    Some(Floor {
        published_ref: Some(published),
        sha: Some(base),
        source,
        drafts,
        summary,
    })
}

/// The drafts as a set, for callers marking a long log.
pub fn draft_set(floor: &Floor) -> HashSet<&str> {
    floor.drafts.iter().map(String::as_str).collect()
}
