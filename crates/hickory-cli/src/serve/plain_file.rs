//! Plain files: open and save any text file in the served folder.
//!
//! Documents have rooms and woven outputs have lineage, but most of a
//! repository is neither — a README, a workflow, a justfile. These two
//! routes are their editing surface: whole-file read, whole-file write,
//! guarded by a content hash so a save can never silently overwrite an edit
//! made outside the app.
//!
//! What this deliberately is NOT:
//!
//! - **Not a document surface.** A `.hick` file is refused in both
//!   directions: it has a CRDT room, and a whole-file write behind the
//!   room's back is exactly the divergence the room exists to prevent.
//! - **Not a lineage bypass.** A woven output that is writable on disk may
//!   be saved here — the in-app up-loop treats the write like any external
//!   edit and carries it back into its document, the same path a vim edit
//!   takes. A fully generated output is read-only on disk while the loop
//!   runs (see docs/guarantees/authoring/a-generated-file-refuses-an-edit.md),
//!   and the save reports that instead of forcing it.

use std::path::{Path as FsPath, PathBuf};

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

/// Whole-file editing stops here. CodeMirror holds a few megabytes happily;
/// past this, the file is more likely a dataset than something to type in.
const MAX_PLAIN_FILE_BYTES: u64 = 10 * 1024 * 1024;

#[derive(Deserialize)]
pub struct PlainFileQuery {
    pub path: String,
}

/// `GET /api/file?path=…` — the file's text, its language tag, and the
/// content hash a later save passes back as `base_hash`.
pub async fn get_file(
    State(state): State<LocalState>,
    Query(q): Query<PlainFileQuery>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let (content, hash) = read_plain(&root, &q.path)?;
    Ok(Json(json!({
        "path": q.path,
        "language": super::api::language_of(&q.path),
        "content": content,
        "hash": hash,
    })))
}

#[derive(Deserialize)]
pub struct SavePlainFile {
    pub path: String,
    pub content: String,
    /// The hash of the content this edit was based on (from GET, or the
    /// previous save's answer). A mismatch means the file changed on disk
    /// underneath the buffer, and the save is refused with 409.
    #[serde(default)]
    pub base_hash: Option<String>,
    /// Overwrite even on a hash mismatch — the deliberate second step after
    /// a 409, never the default.
    #[serde(default)]
    pub force: bool,
}

/// `PUT /api/file` — write the file whole, unless it moved underneath us.
pub async fn put_file(
    State(state): State<LocalState>,
    Json(body): Json<SavePlainFile>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let hash = write_plain(
        &root,
        &body.path,
        &body.content,
        body.base_hash.as_deref(),
        body.force,
    )?;
    Ok(Json(json!({ "path": body.path, "hash": hash })))
}

/// FNV-1a over the content bytes, as hex — the same cheap fingerprint the
/// up-loop uses for change marks. Not cryptographic; it only has to answer
/// "is this the text the buffer was loaded from".
fn content_hash(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash = (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// The absolute path `rel` names inside `root`, or the refusal that says why
/// it names nothing this surface will touch.
fn resolve(root: &FsPath, rel: &str) -> Result<PathBuf, ApiError> {
    let rel = rel.trim();
    if rel.is_empty() {
        return Err(ApiError::bad_request(
            "path must name a file inside this folder — it cannot be empty",
        ));
    }
    let as_path = FsPath::new(rel);
    let escapes = as_path
        .components()
        .any(|c| !matches!(c, std::path::Component::Normal(_)));
    if escapes || as_path.is_absolute() {
        return Err(ApiError::bad_request(format!(
            "path must be relative to the open folder, with no `..` — got {rel:?}"
        )));
    }
    if rel.ends_with(".hick") {
        return Err(ApiError::bad_request(format!(
            "{rel} is a document — open it as one. Documents have live editing \
             rooms, and a whole-file write behind a room's back would lose edits."
        )));
    }
    Ok(root.join(rel))
}

/// Resolve, then verify nothing along the way (a symlink, a case-folding
/// filesystem) lands outside the served root. Only for paths that exist —
/// `canonicalize` needs a real file to walk.
fn resolve_existing(root: &FsPath, rel: &str) -> Result<PathBuf, ApiError> {
    let joined = resolve(root, rel)?;
    if !joined.is_file() {
        return Err(ApiError::not_found(format!(
            "no file named {rel} in the open folder. If it was just created \
             outside the app, the tree refreshes when the window regains focus."
        )));
    }
    let canonical_root = root
        .canonicalize()
        .map_err(|e| ApiError::internal(format!("cannot resolve the served folder: {e}")))?;
    let canonical = joined
        .canonicalize()
        .map_err(|e| ApiError::internal(format!("cannot resolve {rel}: {e}")))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(ApiError::forbidden(format!(
            "{rel} resolves outside the open folder (a symlink?), so this \
             session will not touch it"
        )));
    }
    Ok(canonical)
}

/// Read `rel` as text: `(content, hash)`.
fn read_plain(root: &FsPath, rel: &str) -> Result<(String, String), ApiError> {
    let path = resolve_existing(root, rel)?;
    let meta = std::fs::metadata(&path)
        .map_err(|e| ApiError::internal(format!("cannot stat {rel}: {e}")))?;
    if meta.len() > MAX_PLAIN_FILE_BYTES {
        return Err(ApiError::unprocessable(format!(
            "{rel} is {} bytes — larger than the {} MB this editor opens. \
             Open it in a tool meant for large files.",
            meta.len(),
            MAX_PLAIN_FILE_BYTES / (1024 * 1024),
        )));
    }
    let bytes = std::fs::read(&path)
        .map_err(|e| ApiError::internal(format!("could not read {rel}: {e}")))?;
    let hash = content_hash(&bytes);
    let content = String::from_utf8(bytes).map_err(|_| {
        ApiError::unprocessable(format!(
            "{rel} is not UTF-8 text — probably a binary file, which this \
             editor cannot show. If it is text in another encoding, convert \
             it to UTF-8 first."
        ))
    })?;
    Ok((content, hash))
}

/// Write `rel` whole; answers the new content hash.
fn write_plain(
    root: &FsPath,
    rel: &str,
    content: &str,
    base_hash: Option<&str>,
    force: bool,
) -> Result<String, ApiError> {
    let path = resolve_existing(root, rel)?;
    let meta = std::fs::metadata(&path)
        .map_err(|e| ApiError::internal(format!("cannot stat {rel}: {e}")))?;
    if meta.permissions().readonly() {
        // The up-loop's mark on a fully generated file — see the module doc.
        return Err(ApiError::conflict(format!(
            "{rel} is read-only. Every byte of it is generated, so there is \
             nothing an edit could map back to — open the document that \
             produces it and edit there."
        )));
    }
    let on_disk = std::fs::read(&path)
        .map_err(|e| ApiError::internal(format!("could not read {rel}: {e}")))?;
    // Refuse to replace binary bytes with text: an honest 422 beats
    // clobbering a file the GET route would never have served.
    if String::from_utf8(on_disk.clone()).is_err() {
        return Err(ApiError::unprocessable(format!(
            "{rel} is not UTF-8 text on disk — refusing to overwrite what \
             this editor cannot have shown you."
        )));
    }
    let current = content_hash(&on_disk);
    if let Some(base) = base_hash
        && base != current
        && !force
    {
        return Err(ApiError::conflict(format!(
            "{rel} changed on disk while you were editing — another program \
             (git, a formatter, an agent) wrote it. Reload to pick up the \
             disk version, or save again with force to overwrite it."
        ))
        .with_detail(json!({ "hash": current })));
    }
    std::fs::write(&path, content).map_err(|e| {
        ApiError::internal(format!(
            "could not write {rel}: {e}. Check that the file and its \
             directory are writable and the disk is not full."
        ))
    })?;
    Ok(content_hash(content.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root_with(files: &[(&str, &[u8])]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        for (rel, bytes) in files {
            let path = dir.path().join(rel);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(path, bytes).unwrap();
        }
        dir
    }

    #[test]
    fn reads_a_text_file_and_round_trips_its_hash_through_a_save() {
        let dir = root_with(&[("docs/readme.md", b"hello\n")]);
        let (content, hash) = read_plain(dir.path(), "docs/readme.md").unwrap();
        assert_eq!(content, "hello\n");
        let new_hash = write_plain(
            dir.path(),
            "docs/readme.md",
            "hello world\n",
            Some(&hash),
            false,
        )
        .expect("save with the loaded hash succeeds");
        assert_eq!(
            std::fs::read_to_string(dir.path().join("docs/readme.md")).unwrap(),
            "hello world\n"
        );
        let (reread, hash_after) = read_plain(dir.path(), "docs/readme.md").unwrap();
        assert_eq!(reread, "hello world\n");
        assert_eq!(hash_after, new_hash);
    }

    #[test]
    fn a_stale_base_hash_is_refused_and_force_overrides() {
        let dir = root_with(&[("notes.txt", b"original\n")]);
        let (_, hash) = read_plain(dir.path(), "notes.txt").unwrap();
        // Another program writes underneath the buffer.
        std::fs::write(dir.path().join("notes.txt"), "external edit\n").unwrap();
        let refused = write_plain(dir.path(), "notes.txt", "mine\n", Some(&hash), false);
        assert!(refused.is_err(), "stale hash must 409");
        assert_eq!(
            std::fs::read_to_string(dir.path().join("notes.txt")).unwrap(),
            "external edit\n",
            "a refused save changes nothing"
        );
        write_plain(dir.path(), "notes.txt", "mine\n", Some(&hash), true)
            .expect("force is the deliberate overwrite");
        assert_eq!(
            std::fs::read_to_string(dir.path().join("notes.txt")).unwrap(),
            "mine\n"
        );
    }

    #[test]
    fn refuses_documents_escapes_binaries_and_missing_files() {
        let dir = root_with(&[
            ("paper.hick", b"<h:doc/>"),
            ("logo.png", &[0x89, 0x50, 0xff, 0x00]),
        ]);
        for rel in ["paper.hick", "../outside.txt", "/etc/passwd", ""] {
            assert!(
                read_plain(dir.path(), rel).is_err(),
                "{rel:?} must be refused"
            );
            assert!(
                write_plain(dir.path(), rel, "x", None, false).is_err(),
                "{rel:?} must be refused on write too"
            );
        }
        assert!(
            read_plain(dir.path(), "logo.png").is_err(),
            "binary is not text"
        );
        assert!(
            write_plain(dir.path(), "logo.png", "text", None, true).is_err(),
            "binary bytes are never overwritten with text"
        );
        assert!(read_plain(dir.path(), "absent.txt").is_err());
        assert!(
            write_plain(dir.path(), "absent.txt", "x", None, false).is_err(),
            "this surface edits files that exist; creation is the tree's job, later"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_read_only_file_refuses_a_save_and_names_the_way_in() {
        let dir = root_with(&[("gen.py", b"print()\n")]);
        let path = dir.path().join("gen.py");
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&path, perms).unwrap();
        let refused = write_plain(dir.path(), "gen.py", "x\n", None, false);
        assert!(
            refused.is_err(),
            "read-only (a woven output's mark) refuses the save"
        );
        // Restore so the tempdir can be cleaned up on every platform.
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(false);
        std::fs::set_permissions(&path, perms).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_out_of_the_root_is_refused() {
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("secret.txt"), "s").unwrap();
        let dir = root_with(&[]);
        std::os::unix::fs::symlink(
            outside.path().join("secret.txt"),
            dir.path().join("link.txt"),
        )
        .unwrap();
        assert!(read_plain(dir.path(), "link.txt").is_err());
        assert!(write_plain(dir.path(), "link.txt", "x", None, true).is_err());
    }
}
