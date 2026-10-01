//! The merged view: one tab, several worktrees, the merge already done.
//!
//! See `docs/specs/freeform/the-merged-view.md`. A file as it exists in
//! several places at once, unified into a single surface: regions every source
//! agrees on appear once, regions that differ appear as variants.
//!
//! **Read-only, in this step.** The alignment is where the risk lives — a bad
//! one routes an edit silently into the wrong file — so it is proved before
//! anything can be written through it. The alignment itself is
//! `hick_merge::nway`, which is pure and unit-tested; this module is the door
//! to it: which worktrees exist, and what each one has at a path.
//!
//! **"Across branches" means across worktrees.** A branch that is not checked
//! out cannot be written to without going behind the working tree into the
//! object database — which bypasses hooks, surprises people, and produces
//! commits nobody watched. So each source here is a real worktree on disk,
//! which is also what grounds this in something that already exists
//! (`hick_term::git::add_worktree`).
//!
//! **It is a lens, not a document.** No save path, no `.md` extension, no
//! place in the folder tree. The same category as a diff view — and getting
//! that wrong reverses the product's central claim, because a synthetic
//! document assembled *from* files points the opposite way to a document that
//! is the source of its files.

use std::path::{Path, PathBuf};
use std::process::Command;

use axum::Json;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

/// One worktree of the repository the session is open on.
#[derive(Debug, Clone, Serialize)]
pub struct Worktree {
    /// Absolute path on disk.
    pub path: String,
    /// How a person names it: the last path component, which is what they
    /// called the directory.
    pub name: String,
    /// The branch checked out there, or `None` for a detached HEAD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// True for the worktree this session is open on.
    pub current: bool,
}

/// Every worktree of the repository containing `dir`, main one included.
///
/// `git worktree list --porcelain` is the only source: inventing a list from
/// sibling directories would guess, and a view over a directory that is not a
/// worktree of this repository is a view over an unrelated file.
pub fn worktrees(dir: &Path) -> Vec<Worktree> {
    let Ok(out) = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["worktree", "list", "--porcelain"])
        .output()
    else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    let current = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    parse_worktrees(&String::from_utf8_lossy(&out.stdout), &current)
}

/// Split out and pure so a detached HEAD, a bare main worktree and a prunable
/// entry are arguable in a test rather than against a real repository.
fn parse_worktrees(text: &str, current: &Path) -> Vec<Worktree> {
    let mut out: Vec<Worktree> = Vec::new();
    let mut path: Option<String> = None;
    let mut branch: Option<String> = None;
    let mut bare = false;

    let mut flush = |path: &mut Option<String>, branch: &mut Option<String>, bare: &mut bool| {
        if let Some(p) = path.take() {
            // A bare worktree has no working tree to read a file out of, so
            // it is not a source.
            if !*bare {
                let buf = PathBuf::from(&p);
                let name = buf
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&p)
                    .to_string();
                let canonical = std::fs::canonicalize(&buf).unwrap_or(buf);
                out.push(Worktree {
                    current: canonical == current || current.starts_with(&canonical),
                    path: p,
                    name,
                    branch: branch.take(),
                });
            }
        }
        *branch = None;
        *bare = false;
    };

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("worktree ") {
            flush(&mut path, &mut branch, &mut bare);
            path = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("branch ") {
            branch = Some(
                rest.trim()
                    .strip_prefix("refs/heads/")
                    .unwrap_or(rest.trim())
                    .to_string(),
            );
        } else if line.trim() == "bare" {
            bare = true;
        } else if line.trim() == "detached" {
            branch = None;
        }
    }
    flush(&mut path, &mut branch, &mut bare);
    out
}

#[derive(Deserialize)]
pub struct MergedParams {
    /// Repository-relative path of the file to view.
    pub path: String,
    /// Comma-separated worktree names. Absent means every worktree that has
    /// the file.
    #[serde(default)]
    pub targets: Option<String>,
}

/// `GET /api/merged?path=…&targets=…` — the synthesized view.
///
/// A worktree that does not have the file at all is left out and NAMED, which
/// is different from one that has it empty: "this branch does not have this
/// file yet" is the answer somebody opened the view to get.
pub async fn merged(
    State(state): State<LocalState>,
    Query(params): Query<MergedParams>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let rel = params.path.trim().to_string();
    if rel.is_empty()
        || Path::new(&rel).is_absolute()
        || Path::new(&rel)
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err(ApiError::bad_request(format!(
            "path must be relative to the repository, with no `..` — got {rel:?}"
        )));
    }

    let wanted: Option<Vec<String>> = params.targets.as_ref().map(|t| {
        t.split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    });

    let answer = tokio::task::spawn_blocking(move || {
        let trees = worktrees(&root);
        if trees.is_empty() {
            return json!({
                "repository": false,
                "sources": [],
                "regions": [],
                "missing": [],
            });
        }
        let chosen: Vec<Worktree> = match &wanted {
            Some(names) => trees
                .into_iter()
                .filter(|w| names.iter().any(|n| n == &w.name))
                .collect(),
            None => trees,
        };

        let mut sources = Vec::new();
        let mut missing = Vec::new();
        for tree in &chosen {
            let file = Path::new(&tree.path).join(&rel);
            match std::fs::read_to_string(&file) {
                Ok(text) => sources.push(hick_merge::Source {
                    name: tree.name.clone(),
                    text,
                }),
                // Named, not silently dropped: "this branch does not have
                // this file yet" is an answer somebody came here for.
                Err(_) => missing.push(tree.name.clone()),
            }
        }

        let view = hick_merge::merged_view(&sources);
        let regions: Vec<Value> = view
            .regions
            .iter()
            .map(|region| match region {
                hick_merge::Region::Shared { lines } => json!({
                    "kind": "shared",
                    "text": lines.concat(),
                }),
                hick_merge::Region::Variant { by_source } => json!({
                    "kind": "variant",
                    "by_source": by_source
                        .iter()
                        .map(|(name, lines)| (name.clone(), Value::String(lines.concat())))
                        .collect::<serde_json::Map<_, _>>(),
                }),
            })
            .collect();

        json!({
            "repository": true,
            "path": rel,
            "sources": chosen,
            "regions": regions,
            "missing": missing,
            "shared_lines": view.shared_line_count(),
            "variants": view.variant_count(),
            // Said at the top of the tab rather than left to be inferred: a
            // view that is mostly variants is a comparison, not a merge.
            "read_only": true,
        })
    })
    .await
    .map_err(|e| ApiError::internal(format!("the git task failed: {e}")))?;

    Ok(Json(answer))
}

/// `GET /api/worktrees` — what a merged view can be opened over.
pub async fn list(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let trees = tokio::task::spawn_blocking(move || worktrees(&root))
        .await
        .map_err(|e| ApiError::internal(format!("the git task failed: {e}")))?;
    Ok(Json(json!({
        "repository": !trees.is_empty(),
        "worktrees": trees,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_main_worktree_and_a_linked_one_are_both_sources() {
        let text = "worktree /repo\nHEAD abc\nbranch refs/heads/master\n\n\
                    worktree /repo-side\nHEAD def\nbranch refs/heads/side\n";
        let trees = parse_worktrees(text, Path::new("/repo"));
        assert_eq!(trees.len(), 2);
        assert_eq!(trees[0].name, "repo");
        assert_eq!(trees[0].branch.as_deref(), Some("master"));
        assert_eq!(trees[1].name, "repo-side");
        assert_eq!(trees[1].branch.as_deref(), Some("side"));
    }

    #[test]
    fn a_bare_worktree_is_not_a_source() {
        // There is no working tree to read a file out of.
        let text = "worktree /repo.git\nbare\n\n\
                    worktree /repo\nHEAD abc\nbranch refs/heads/master\n";
        let trees = parse_worktrees(text, Path::new("/repo"));
        assert_eq!(trees.len(), 1);
        assert_eq!(trees[0].name, "repo");
    }

    #[test]
    fn a_detached_head_has_no_branch_rather_than_a_wrong_one() {
        let text = "worktree /repo\nHEAD abc\ndetached\n";
        let trees = parse_worktrees(text, Path::new("/repo"));
        assert_eq!(trees.len(), 1);
        assert!(trees[0].branch.is_none());
    }

    #[test]
    fn no_repository_is_no_worktrees_rather_than_an_error() {
        assert!(parse_worktrees("", Path::new("/repo")).is_empty());
    }
}

// ---------------------------------------------------------------------------
// Writing through the view
// ---------------------------------------------------------------------------

/// What one target did with a write.
///
/// **A multi-target edit is not atomic.** You type in a shared region; three
/// targets accept the write and the fourth is read-only, gone, or unwritable.
/// Refusing the edit is intolerable and pretending it landed is worse — so
/// every target reports for itself, and the tab shows per-target status rather
/// than implying success. Designing this up front is the difference between a
/// feature that feels finished and one that does not.
#[derive(Debug, Clone, Serialize)]
pub struct TargetStatus {
    pub source: String,
    pub ok: bool,
    /// Why not, when it did not land. Never absent on a failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct MergedWrite {
    /// Repository-relative path of the file.
    pub path: String,
    /// Which regions the view was built from — the client sends back the
    /// sources it was showing, so a worktree that appeared or vanished since
    /// it opened cannot be written to by accident.
    pub targets: Vec<String>,
    /// Index of the region being edited, in the view the client is showing.
    pub region: usize,
    /// The region's new text.
    pub text: String,
    /// `"shared"` writes every target; `"just-here"` writes one.
    #[serde(default)]
    pub route: Option<String>,
    /// The one target, for `just-here`.
    #[serde(default)]
    pub source: Option<String>,
}

/// `POST /api/merged/write` — route an edit back to the sources.
///
/// **Read-only is the default for anything not explicitly opened for
/// writing.** A long-lived release branch in the view would otherwise
/// silently receive shared edits, so a target has to be named in `targets` to
/// be written at all — which is what the client sends when the person opened
/// it for writing.
pub async fn write(
    State(state): State<LocalState>,
    Json(body): Json<MergedWrite>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let rel = check_relative(&body.path)?;

    let route = match body.route.as_deref() {
        Some("just-here") => {
            let source = body.source.clone().ok_or_else(|| {
                ApiError::bad_request(
                    "a `just-here` write needs the one target it is for".to_string(),
                )
            })?;
            hick_merge::Route::JustHere { source }
        }
        Some("shared") | None => hick_merge::Route::Shared,
        Some(other) => {
            return Err(ApiError::bad_request(format!(
                "unknown route {other:?} — a write is `shared` or `just-here`"
            )));
        }
    };

    let answer = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let trees = worktrees(&root);
        let chosen: Vec<Worktree> = trees
            .into_iter()
            .filter(|w| body.targets.iter().any(|t| t == &w.name))
            .collect();
        if chosen.is_empty() {
            return Err("none of those targets is a worktree of this repository".to_string());
        }

        // Rebuild the view from disk RIGHT NOW rather than trusting the
        // client's copy: between opening the tab and typing, a source may have
        // moved, and writing a stale rebuild would silently revert somebody.
        let mut sources = Vec::new();
        for tree in &chosen {
            let file = Path::new(&tree.path).join(&rel);
            match std::fs::read_to_string(&file) {
                Ok(text) => sources.push(hick_merge::Source {
                    name: tree.name.clone(),
                    text,
                }),
                Err(e) => return Err(format!("{} has no readable {rel}: {e}", tree.name)),
            }
        }
        let view = hick_merge::merged_view(&sources);
        let writes = hick_merge::writes_for_edit(&view, body.region, &body.text, &route)
            .map_err(|e| e.to_string())?;

        // The bytes each target had BEFORE, so an undo across targets can put
        // them all back — including the ones a partial write did not reach.
        let undo: Vec<Value> = sources
            .iter()
            .map(|s| json!({ "source": s.name, "text": s.text }))
            .collect();

        let mut statuses = Vec::new();
        for w in &writes {
            let Some(tree) = chosen.iter().find(|t| t.name == w.source) else {
                statuses.push(TargetStatus {
                    source: w.source.clone(),
                    ok: false,
                    error: Some("no longer a worktree of this repository".to_string()),
                });
                continue;
            };
            let file = Path::new(&tree.path).join(&rel);
            match crate::serve::store::write_atomic(&file, w.text.as_bytes()) {
                Ok(()) => statuses.push(TargetStatus {
                    source: w.source.clone(),
                    ok: true,
                    error: None,
                }),
                Err(e) => statuses.push(TargetStatus {
                    source: w.source.clone(),
                    ok: false,
                    error: Some(format!("{e:#}")),
                }),
            }
        }

        let landed = statuses.iter().filter(|s| s.ok).count();
        Ok(json!({
            "path": rel,
            "targets": statuses,
            "landed": landed,
            // Stated rather than inferred from counting: a partial write is a
            // real state, and the tab must not round it up to success.
            "partial": landed > 0 && landed < writes.len(),
            "undo": undo,
        }))
    })
    .await
    .map_err(|e| ApiError::internal(format!("the write task failed: {e}")))?
    .map_err(ApiError::unprocessable)?;

    Ok(Json(answer))
}

#[derive(Deserialize)]
pub struct MergedUndo {
    pub path: String,
    /// One entry per target: the bytes it had before.
    pub undo: Vec<UndoEntry>,
}

#[derive(Deserialize)]
pub struct UndoEntry {
    pub source: String,
    pub text: String,
}

/// `POST /api/merged/undo` — put every target back.
///
/// **Undo across N worktrees has to be designed rather than assumed.** A
/// half-applied edit that is then undone in three of four places is a state
/// somebody will reach on the first day, so undo restores from the recorded
/// before-bytes of every target the write touched — including the ones the
/// write did not reach, which is exactly the case that would otherwise be
/// missed.
pub async fn undo(
    State(state): State<LocalState>,
    Json(body): Json<MergedUndo>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let rel = check_relative(&body.path)?;

    let answer = tokio::task::spawn_blocking(move || {
        let trees = worktrees(&root);
        let mut statuses = Vec::new();
        for entry in &body.undo {
            let Some(tree) = trees.iter().find(|t| t.name == entry.source) else {
                statuses.push(TargetStatus {
                    source: entry.source.clone(),
                    ok: false,
                    error: Some("no longer a worktree of this repository".to_string()),
                });
                continue;
            };
            let file = Path::new(&tree.path).join(&rel);
            match crate::serve::store::write_atomic(&file, entry.text.as_bytes()) {
                Ok(()) => statuses.push(TargetStatus {
                    source: entry.source.clone(),
                    ok: true,
                    error: None,
                }),
                Err(e) => statuses.push(TargetStatus {
                    source: entry.source.clone(),
                    ok: false,
                    error: Some(format!("{e:#}")),
                }),
            }
        }
        let restored = statuses.iter().filter(|s| s.ok).count();
        json!({
            "path": rel,
            "targets": statuses,
            "restored": restored,
            "partial": restored > 0 && restored < body.undo.len(),
        })
    })
    .await
    .map_err(|e| ApiError::internal(format!("the undo task failed: {e}")))?;

    Ok(Json(answer))
}

/// A repository-relative path, or a refusal.
fn check_relative(path: &str) -> Result<String, ApiError> {
    let rel = path.trim().to_string();
    if rel.is_empty()
        || Path::new(&rel).is_absolute()
        || Path::new(&rel)
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err(ApiError::bad_request(format!(
            "path must be relative to the repository, with no `..` — got {rel:?}"
        )));
    }
    Ok(rel)
}
