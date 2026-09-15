//! The file operations a tree has in dired: rename, move, copy, delete, a
//! new file, a new folder. `POST /api/files/op`.
//!
//! Each verb is one filesystem call on a path inside the open folder, with
//! the refusals said plainly: nothing outside the folder, nothing under
//! `.git`, never the folder itself, never over something that exists. There
//! is no trash — a delete is a delete, which is why the pane asks twice —
//! and no `git mv`: the Git pane shows what changed and stages it as itself.
//!
//! See docs/guarantees/authoring/the-tree-is-a-dired.md.

use std::path::{Component, Path, PathBuf};

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

#[derive(Deserialize)]
pub struct FileOp {
    pub op: String,
    /// Root-relative path the verb acts on.
    pub path: String,
    /// `rename`: the new name (or root-relative path); `move`/`copy`: the
    /// destination directory.
    #[serde(default)]
    pub to: Option<String>,
}

/// The absolute path `rel` names inside `root`, or why this surface will
/// not touch it.
fn resolve(root: &Path, rel: &str) -> Result<PathBuf, ApiError> {
    let rel = rel.trim().trim_end_matches('/');
    let as_path = Path::new(rel);
    if rel.is_empty() {
        return Err(ApiError::bad_request(
            "path must name something inside this folder — the folder itself is not a target",
        ));
    }
    if as_path.is_absolute()
        || as_path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(ApiError::bad_request(format!(
            "path must be relative to the open folder, with no `..` — got {rel:?}"
        )));
    }
    if as_path.components().next() == Some(Component::Normal(".git".as_ref())) {
        return Err(ApiError::bad_request(format!(
            "{rel} is inside .git, which this app never edits by hand — the Git pane is the way in"
        )));
    }
    Ok(root.join(rel))
}

/// A destination that must not already exist.
fn vacant(target: &Path, rel: &str) -> Result<(), ApiError> {
    if target.exists() {
        return Err(ApiError::conflict(format!(
            "{rel} already exists — nothing here overwrites; choose another name or delete it first"
        )));
    }
    Ok(())
}

fn rel_of(root: &Path, abs: &Path) -> String {
    abs.strip_prefix(root)
        .unwrap_or(abs)
        .to_string_lossy()
        .replace('\\', "/")
}

fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    if from.is_dir() {
        std::fs::create_dir_all(to)?;
        for entry in std::fs::read_dir(from)? {
            let entry = entry?;
            copy_tree(&entry.path(), &to.join(entry.file_name()))?;
        }
        Ok(())
    } else {
        std::fs::copy(from, to).map(|_| ())
    }
}

/// `POST /api/files/op` — one dired verb.
pub async fn file_op(
    State(state): State<LocalState>,
    Json(body): Json<FileOp>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let source = resolve(&root, &body.path)?;
    let op = body.op.as_str();
    let needs_source = matches!(op, "rename" | "move" | "copy" | "delete");
    if needs_source && !source.exists() {
        return Err(ApiError::not_found(format!(
            "{} is not in this folder any more — the tree may be behind; it refreshes on its own",
            body.path
        )));
    }
    let to = body.to.as_deref().map(str::trim).filter(|t| !t.is_empty());
    let outcome = match op {
        "rename" => {
            let name =
                to.ok_or_else(|| ApiError::bad_request("rename needs `to`: the new name"))?;
            // A bare name stays in the same directory; a path moves as well.
            let target_rel = if name.contains('/') {
                name.to_string()
            } else {
                let dir = body
                    .path
                    .trim_end_matches('/')
                    .rsplit_once('/')
                    .map(|(d, _)| d);
                match dir {
                    Some(dir) => format!("{dir}/{name}"),
                    None => name.to_string(),
                }
            };
            let target = resolve(&root, &target_rel)?;
            vacant(&target, &target_rel)?;
            std::fs::rename(&source, &target)
                .map_err(|e| ApiError::unprocessable(format!("rename failed: {e}")))?;
            json!({ "op": "rename", "from": body.path, "to": rel_of(&root, &target) })
        }
        "move" | "copy" => {
            let dir_rel = to.ok_or_else(|| {
                ApiError::bad_request(format!("{op} needs `to`: the destination directory"))
            })?;
            let dir = if dir_rel == "." {
                root.clone()
            } else {
                resolve(&root, dir_rel)?
            };
            if !dir.is_dir() {
                return Err(ApiError::bad_request(format!(
                    "{dir_rel} is not a directory in this folder"
                )));
            }
            let name = source
                .file_name()
                .ok_or_else(|| ApiError::bad_request("no file name"))?;
            let target = dir.join(name);
            let target_rel = rel_of(&root, &target);
            if source.is_dir() && target.starts_with(&source) {
                return Err(ApiError::bad_request(format!(
                    "{} cannot be {}d into itself",
                    body.path, op
                )));
            }
            vacant(&target, &target_rel)?;
            if op == "move" {
                std::fs::rename(&source, &target)
            } else {
                copy_tree(&source, &target)
            }
            .map_err(|e| ApiError::unprocessable(format!("{op} failed: {e}")))?;
            json!({ "op": op, "from": body.path, "to": target_rel })
        }
        "delete" => {
            if source.is_dir() {
                std::fs::remove_dir_all(&source)
            } else {
                std::fs::remove_file(&source)
            }
            .map_err(|e| ApiError::unprocessable(format!("delete failed: {e}")))?;
            json!({ "op": "delete", "path": body.path })
        }
        "mkdir" => {
            vacant(&source, &body.path)?;
            std::fs::create_dir_all(&source).map_err(|e| {
                ApiError::unprocessable(format!("could not create the folder: {e}"))
            })?;
            json!({ "op": "mkdir", "path": body.path })
        }
        "create" => {
            if body.path.ends_with(".hick") {
                return Err(ApiError::bad_request(
                    "a `.hick` file is a document — File → New Document makes one, with a room to edit it in",
                ));
            }
            vacant(&source, &body.path)?;
            if let Some(parent) = source.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    ApiError::unprocessable(format!("could not create the folder: {e}"))
                })?;
            }
            std::fs::write(&source, b"")
                .map_err(|e| ApiError::unprocessable(format!("could not create the file: {e}")))?;
            json!({ "op": "create", "path": body.path })
        }
        other => {
            return Err(ApiError::bad_request(format!(
                "unknown file operation {other:?} — one of rename, move, copy, delete, mkdir, create"
            )));
        }
    };
    Ok(Json(outcome))
}
