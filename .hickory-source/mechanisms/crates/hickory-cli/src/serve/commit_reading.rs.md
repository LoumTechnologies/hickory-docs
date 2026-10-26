<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/commit_reading.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4798664e2dcf"></a>
## `crates/hickory-cli/src/serve/commit_reading.rs`

Related contracts: [A commit reads as a literate change](../../../../../guarantees/lenses/a-commit-reads-as-a-literate-change/contract.md).

<a id="f-4798664e2dcf-CommitQuery"></a>
### `CommitQuery`

Immutable commit readings: message and exact before/after blobs, never a checkout.

<storycode:copy id="f-4798664e2dcf-CommitQuery">
//! Immutable commit readings: message and exact before/after blobs, never a checkout.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
    revision,
};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{path::Path, process::Command};

#[derive(Deserialize)]
pub struct CommitQuery {
    pub sha: String,
}

</storycode:copy>
<a id="f-4798664e2dcf-blob"></a>
### `blob`

<storycode:copy id="f-4798664e2dcf-blob">
fn blob(root: &Path, revision: Option<&str>, path: &str) -> ApiResult<(String, bool)> {
    let Some(revision) = revision else {
        return Ok((String::new(), false));
    };
    let object = format!("{revision}:{path}");
    let kind = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "-t", &object])
        .output()
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if !kind.status.success() {
        return Ok((String::new(), false));
    }
    if kind.stdout != b"blob\n" {
        return Ok((String::new(), true));
    }
    let result = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "blob", &object])
        .output()
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if !result.status.success() {
        return Err(ApiError::unprocessable(
            "Could not read this commit's file.",
        ));
    }
    if result.stdout.contains(&0) {
        return Ok((String::new(), true));
    }
    match String::from_utf8(result.stdout) {
        Ok(text) => Ok((text, false)),
        Err(_) => Ok((String::new(), true)),
    }
}

</storycode:copy>
<a id="f-4798664e2dcf-read"></a>
### `read`

<storycode:copy id="f-4798664e2dcf-read">
pub async fn read(
    State(state): State<LocalState>,
    Query(query): Query<CommitQuery>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    tokio::task::spawn_blocking(move || {
        let sha = revision::resolve(&root, &query.sha)?;
        if sha == "INDEX" { return Err(ApiError::bad_request("Choose a commit to read.")); }
        let parents = revision::git(&root, &["rev-list", "--parents", "-n", "1", &sha])?;
        let parent = parents.split_whitespace().nth(1);
        let message = revision::git(&root, &["show", "-s", "--format=%B", &sha])?;
        let prefix = revision::git(&root, &["rev-parse", "--show-prefix"])?;
        let prefix = prefix.trim();
        let changed = if let Some(parent) = parent {
            revision::git(&root, &["diff", "--name-status", "-z", "-M", parent, &sha, "--", "."])?
        } else {
            revision::git(&root, &["diff-tree", "--root", "--no-commit-id", "-r", "--name-status", "-z", &sha, "--", "."])?
        };
        let mut tokens = changed.split('\0').filter(|part| !part.is_empty());
        let mut files = Vec::new();
        while let Some(status) = tokens.next() {
            let first = tokens.next().ok_or_else(|| ApiError::internal("Incomplete git change."))?;
            let (old_path, path) = if status.starts_with('R') || status.starts_with('C') {
                (first, tokens.next().ok_or_else(|| ApiError::internal("Incomplete git rename."))?)
            } else { (first, first) };
            let Some(local_path) = path.strip_prefix(prefix) else { continue; };
            let (before, old_binary) = if status.starts_with('A') { (String::new(), false) } else { blob(&root, parent, old_path)? };
            let (after, new_binary) = if status.starts_with('D') { (String::new(), false) } else { blob(&root, Some(&sha), path)? };
            files.push(json!({"path": local_path, "from": old_path.strip_prefix(prefix).unwrap_or(old_path),
                "status": status, "before": before, "after": after, "binary": old_binary || new_binary}));
        }
        Ok(Json(json!({"sha": sha, "parent": parent, "message": message, "files": files})))
    }).await.map_err(|e| ApiError::internal(e.to_string()))?
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/commit_reading.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/commit_reading.rs"><storycode:paste select="#f-4798664e2dcf-CommitQuery"/><storycode:paste select="#f-4798664e2dcf-blob"/><storycode:paste select="#f-4798664e2dcf-read"/></storycode:file>

</storycode:doc>
