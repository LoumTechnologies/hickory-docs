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
    /// Legacy: comma-separated names. V1: a JSON string array preserving
    /// literal names and empty membership. Absent means all worktrees.
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
    let wanted = params.targets.as_ref().map(|targets| {
        targets
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_owned)
            .collect()
    });
    merged_selected(state, params.path.trim().to_owned(), wanted).await
}

/// Versioned selection preserves empty membership and names containing commas.
pub async fn merged_v1(
    State(state): State<LocalState>,
    Query(params): Query<MergedParams>,
) -> ApiResult<Json<Value>> {
    let wanted = selected_v1(params.targets.as_deref())?;
    merged_selected(state, params.path, wanted).await
}
fn selected_v1(encoded: Option<&str>) -> ApiResult<Option<Vec<String>>> {
    let Some(encoded) = encoded else {
        return Ok(None);
    };
    if encoded.len() > 65536 {
        return Err(ApiError::bad_request("Worktree selection is too large"));
    }
    let names: Vec<String> = serde_json::from_str(encoded).map_err(|error| {
        ApiError::bad_request(format!(
            "Worktree targets must be a JSON string array: {error}"
        ))
    })?;
    let mut seen = std::collections::BTreeSet::new();
    if names.len() > 1024
        || names
            .iter()
            .any(|name| name.is_empty() || name.len() > 4096 || !seen.insert(name))
    {
        return Err(ApiError::bad_request(
            "Worktree targets must be bounded, nonempty and unique",
        ));
    }
    Ok(Some(names))
}
async fn merged_selected(
    state: LocalState,
    path: String,
    wanted: Option<Vec<String>>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let rel = path;
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
            "snapshot": sources.iter().map(|s|(s.name.clone(),s.text.clone())).collect::<std::collections::BTreeMap<_,_>>(),
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

pub use super::merged_write::{MergedUndo, MergedWrite, TargetStatus, undo, write};

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

#[cfg(test)]
mod selection_tests {
    use super::*;
    #[test]
    fn versioned_selection_keeps_none_empty_and_literal_names_distinct() {
        assert_eq!(selected_v1(None).unwrap(), None);
        assert_eq!(selected_v1(Some("[]")).unwrap(), Some(vec![]));
        assert_eq!(
            selected_v1(Some(r#"["branch,with,commas"," quote ","λ"]"#)).unwrap(),
            Some(vec![
                "branch,with,commas".into(),
                " quote ".into(),
                "λ".into()
            ])
        );
    }
    #[test]
    fn versioned_selection_refuses_ambiguous_or_unbounded_encodings() {
        for encoded in ["repo,side", "null", r#"[1]"#, r#"[""]"#, r#"["a","a"]"#] {
            assert!(selected_v1(Some(encoded)).is_err());
        }
        assert!(selected_v1(Some(&"x".repeat(65537))).is_err());
    }
}
