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
use super::workspace_entry_authority::acquire_namespace_coordination;

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
    super::workspace_fs::view::confined(root,rel).map_err(|error|ApiError::bad_request(error.to_string()))
}

/// A destination that must not already exist.
fn vacant(target: &Path, rel: &str) -> Result<(), ApiError> {
    match std::fs::symlink_metadata(target) {
        Ok(_) => Err(ApiError::conflict(format!(
            "{rel} already exists — nothing here overwrites; choose another name or delete it first"
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ApiError::unprocessable(format!("could not inspect {rel}: {error}"))),
    }
}

fn rel_of(root: &Path, abs: &Path) -> String {
    abs.strip_prefix(root)
        .unwrap_or(abs)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn rename_exclusive(from: &Path, to: &Path) -> std::io::Result<()> {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    let from = CString::new(from.as_os_str().as_bytes())?;
    let to = CString::new(to.as_os_str().as_bytes())?;
    // These system calls reject a competing destination atomically.
    #[cfg(target_os = "macos")]
    let status = unsafe { libc::renamex_np(from.as_ptr(), to.as_ptr(), libc::RENAME_EXCL) as libc::c_long };
    #[cfg(target_os = "linux")]
    let status = unsafe { libc::syscall(libc::SYS_renameat2, libc::AT_FDCWD, from.as_ptr(), libc::AT_FDCWD, to.as_ptr(), libc::RENAME_NOREPLACE) };
    if status == 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn rename_exclusive(from: &Path, to: &Path) -> std::io::Result<()> {
    match std::fs::symlink_metadata(to) {
        Ok(_) => Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => std::fs::rename(from, to),
        Err(error) => Err(error),
    }
}

fn publish_copy(from: &Path, to: &Path) -> std::io::Result<()> {
    let parent = to.parent().ok_or_else(|| std::io::Error::other("Copy destination has no parent"))?;
    let temporary = tempfile::Builder::new().prefix(".hick-copy-").tempdir_in(parent)?;
    let staged = temporary.path().join("content");
    copy_tree(from, &staged)?;
    rename_exclusive(&staged, to)
}

fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    let metadata = std::fs::symlink_metadata(from)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() && !metadata.is_dir() {
        return Err(std::io::Error::other("copy requires ordinary files and directories"));
    }
    if metadata.is_dir() {
        std::fs::create_dir(to)?;
        for entry in std::fs::read_dir(from)? {
            let entry = entry?;
            copy_tree(&entry.path(), &to.join(entry.file_name()))?;
        }
        Ok(())
    } else {
        let mut source = std::fs::File::open(from)?;
        let mut target = std::fs::OpenOptions::new().write(true).create_new(true).open(to)?;
        std::io::copy(&mut source, &mut target)?;
        target.set_permissions(metadata.permissions())
    }
}

#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
mod namespace_publication_tests {
    use super::*;

    #[test]
    fn exclusive_rename_keeps_both_original_files_when_a_destination_exists() {
        let root = tempfile::tempdir().unwrap();
        let from = root.path().join("source");
        let to = root.path().join("target");
        std::fs::write(&from, "source bytes").unwrap();
        std::fs::write(&to, "independent destination bytes").unwrap();
        let error = rename_exclusive(&from, &to).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(std::fs::read_to_string(from).unwrap(), "source bytes");
        assert_eq!(std::fs::read_to_string(to).unwrap(), "independent destination bytes");
    }
}

fn publish_namespace(
    state: &LocalState,
    root: &Path,
    affected: &[&Path],
    perform: impl FnOnce() -> ApiResult<Value>,
) -> ApiResult<Value> {
    state.resource_root.check(root).map_err(ApiError::conflict)?;
    let names: Vec<String> = affected.iter().map(|path| rel_of(root, path)).collect();
    let paths: Vec<&str> = names.iter().map(String::as_str).collect();
    let lease = acquire_namespace_coordination(root, &paths).map_err(ApiError::conflict)?;
    lease.check().map_err(ApiError::conflict)?;
    for path in paths { resolve(root, path)?; }
    let result = perform();
    if let Err(reason) = lease.check().and_then(|()| state.resource_root.check(root).map(|_| ())) {
        return Err(ApiError::conflict(format!("File operation result is unconfirmed: {reason}. Refresh Files before trying again.")));
    }
    result
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
            publish_namespace(&state, &root, &[&source, &target], || {
                vacant(&target, &target_rel)?;
                rename_exclusive(&source, &target)
                    .map_err(|e| ApiError::unprocessable(format!("rename failed: {e}")))?;
                Ok(json!({ "op": "rename", "from": body.path, "to": rel_of(&root, &target) }))
            })?
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
            let affected: &[&Path] = if op == "move" { &[&source, &target] } else { &[&target] };
            publish_namespace(&state, &root, affected, || {
                resolve(&root, &rel_of(&root, &source))?;
                vacant(&target, &target_rel)?;
                if op == "move" {
                    rename_exclusive(&source, &target)
                } else {
                    publish_copy(&source, &target)
                }
                .map_err(|e| ApiError::unprocessable(format!("{op} failed: {e}")))?;
                Ok(json!({ "op": op, "from": body.path, "to": target_rel }))
            })?
        }
        "delete" => publish_namespace(&state, &root, &[&source], || {
            if source.is_dir() {
                std::fs::remove_dir_all(&source)
            } else {
                std::fs::remove_file(&source)
            }
            .map_err(|e| ApiError::unprocessable(format!("delete failed: {e}")))?;
            Ok(json!({ "op": "delete", "path": body.path }))
        })?,
        "mkdir" => publish_namespace(&state, &root, &[&source], || {
            vacant(&source, &body.path)?;
            std::fs::create_dir_all(&source).map_err(|e| {
                ApiError::unprocessable(format!("could not create the folder: {e}"))
            })?;
            Ok(json!({ "op": "mkdir", "path": body.path }))
        })?,
        "create" => {
            if body.path.ends_with(".md") {
                return Err(ApiError::bad_request(
                    "a `.md` file is a document — File → New Document makes one, with a room to edit it in",
                ));
            }
            publish_namespace(&state, &root, &[&source], || {
                vacant(&source, &body.path)?;
                if let Some(parent) = source.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        ApiError::unprocessable(format!("could not create the folder: {e}"))
                    })?;
                }
                std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&source)
                    .map_err(|e| ApiError::unprocessable(format!("could not create the file: {e}")))?;
                Ok(json!({ "op": "create", "path": body.path }))
            })?
        }
        other => {
            return Err(ApiError::bad_request(format!(
                "unknown file operation {other:?} — one of rename, move, copy, delete, mkdir, create"
            )));
        }
    };
    Ok(Json(outcome))
}
