//! Revision reads never check out a commit or publish a representation.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
    representation::Backing,
};
use axum::{
    Json,
    extract::{Path as Id, Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{path::Path, process::Command};

pub fn git(root: &Path, args: &[&str]) -> ApiResult<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .output()
        .map_err(|e| {
            ApiError::unavailable(format!(
                "Could not run git: {e}. Check that git is installed."
            ))
        })?;
    if !output.status.success() {
        return Err(ApiError::unprocessable(format!(
            "Git refused: {}. Review the revisions and try again.",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    String::from_utf8(output.stdout).map_err(|_| {
        ApiError::unprocessable(
            "This revision contains non-UTF-8 content; open its binary diff with git.",
        )
    })
}

pub fn resolve(root: &Path, rev: &str) -> ApiResult<String> {
    if rev == "INDEX" {
        return Ok("INDEX".into());
    }
    if rev.is_empty() || rev.len() > 200 {
        return Err(ApiError::bad_request(
            "Choose a commit, branch, or tag to compare.",
        ));
    }
    Ok(git(
        root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{rev}^{{commit}}"),
        ],
    )?
    .trim()
    .into())
}

pub fn file(root: &Path, rev: &str, path: &str) -> ApiResult<String> {
    super::workspace_fs::view::confined(root, path)
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    let prefix = git(root, &["rev-parse", "--show-prefix"])?;
    let object = format!(
        "{}:{}{path}",
        if rev == "INDEX" { "" } else { rev },
        prefix.trim()
    );
    // A path absent at this revision is an addition or deletion, not a broken view.
    let exists = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "-e", &object])
        .output()
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if !exists.status.success() {
        return Ok(String::new());
    }
    git(root, &["show", &object])
}

#[derive(Deserialize)]
pub struct Compare {
    pub base: String,
    pub target: Option<String>,
}

pub async fn compare(
    State(state): State<LocalState>,
    Id(id): Id<String>,
    Query(query): Query<Compare>,
) -> ApiResult<Json<Value>> {
    let view = state
        .representations
        .lock()
        .await
        .get(&id)
        .cloned()
        .ok_or_else(|| ApiError::not_found("Open the literate view again before comparing."))?;
    let base = resolve(state.index.root(), &query.base)?;
    let target = query
        .target
        .as_deref()
        .map(|r| resolve(state.index.root(), r))
        .transpose()?;
    let (source, target_source) = match &view.backing {
        Backing::Document { doc_id } => {
            let path = state
                .index
                .path_of(doc_id)
                .ok_or_else(|| ApiError::not_found("The document is no longer open."))?;
            (
                file(state.index.root(), &base, &path)?,
                target
                    .as_deref()
                    .map(|r| file(state.index.root(), r, &path))
                    .transpose()?
                    .unwrap_or_else(|| view.source.clone()),
            )
        }
        Backing::Files { .. } => {
            let name = state
                .index
                .root()
                .join(format!("__lens-{}.md", view.id))
                .display()
                .to_string();
            let at = |revision: &str| -> ApiResult<String> {
                let mut edits = Vec::new();
                for f in &view.files {
                    edits.extend(crate::up::reverse::source_edits_for_save(&f.content, &file(state.index.root(), revision, &f.path)?, &f.provenance).map_err(|e| ApiError::unprocessable(format!("Cannot align this revision with the literate arrangement: {e}. Use a simpler arrangement or compare the source file.")))?);
                }
                if edits.is_empty() {
                    return Ok(view.source.clone());
                }
                hickory_lineage::apply_source_edits(
                    &std::collections::HashMap::from([(name.clone(), view.source.clone())]),
                    &edits,
                )
                .map_err(|e| ApiError::unprocessable(e.to_string()))?
                .remove(&name)
                .ok_or_else(|| {
                    ApiError::unprocessable(
                        "This revision could not be aligned; reopen the source view.",
                    )
                })
            };
            (
                at(&base)?,
                target
                    .as_deref()
                    .map(at)
                    .transpose()?
                    .unwrap_or_else(|| view.source.clone()),
            )
        }
    };
    Ok(Json(
        json!({"base":base,"target":target,"source":source,"target_source":target_source,"editable":target.is_none(),"revision":view.revision}),
    ))
}
