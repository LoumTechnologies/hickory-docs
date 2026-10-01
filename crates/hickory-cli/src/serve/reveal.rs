//! Handing a file to the rest of the machine: the desktop's file manager, and
//! whatever program the user has already chosen to open that kind of file.
//!
//! The tree pane can show a file and this app can edit most of them, but a
//! notes folder is an ordinary directory on an ordinary computer — sometimes
//! the thing you want is Finder, or the editor you have used for ten years.
//! These two routes are that door, and they are the whole door: they hand a
//! path to the platform and stop. Nothing here reads the file, and nothing
//! here chooses the program — the OS's own association does.
//!
//! Why the server and not the desktop shell: `hick up` and the desktop app are
//! one engine, and the machine whose file manager this opens is the machine
//! this process runs on, which local-only makes the same machine as the user's.
//! Putting it in the Tauri shell instead would give the same feature two
//! implementations and one of them no tests.
//!
//! The refusals matter more than the commands. A path that escapes the served
//! folder is refused before anything is spawned, symlinks included: "open
//! whatever path you name in your default program" is a much larger surface
//! than "open something in this folder", and only the second one is a feature.

use std::path::{Path as FsPath, PathBuf};
use std::process::Command;

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

#[derive(Deserialize)]
pub struct RevealRequest {
    /// Root-relative, forward slashes, exactly as `GET /api/files` spells it.
    /// Empty (or absent) names the open folder itself.
    #[serde(default)]
    pub path: String,
}

/// `POST /api/reveal` — show the path in the platform's file manager.
///
/// A file is *selected* in its containing folder rather than opened; a
/// directory is opened. That difference is the whole point of a separate
/// route from [`open_external`]: "show me where this lives".
pub async fn reveal(
    State(state): State<LocalState>,
    Json(body): Json<RevealRequest>,
) -> ApiResult<Json<Value>> {
    let target = resolve_in_root(state.index.root(), &body.path)?;
    let is_dir = target.is_dir();
    tokio::task::spawn_blocking(move || spawn_reveal(&target, is_dir))
        .await
        .map_err(|e| ApiError::internal(format!("reveal task failed: {e}")))??;
    Ok(Json(json!({ "ok": true })))
}

/// `POST /api/open-external` — open the path in the user's default program
/// for that kind of file (a directory opens in the file manager).
pub async fn open_external(
    State(state): State<LocalState>,
    Json(body): Json<RevealRequest>,
) -> ApiResult<Json<Value>> {
    let target = resolve_in_root(state.index.root(), &body.path)?;
    tokio::task::spawn_blocking(move || spawn_open(&target))
        .await
        .map_err(|e| ApiError::internal(format!("open task failed: {e}")))??;
    Ok(Json(json!({ "ok": true })))
}

// ---------------------------------------------------------------------------
// Which path, and whether it is ours to hand over
// ---------------------------------------------------------------------------

/// The absolute path `rel` names inside `root`, canonicalized and verified to
/// still be inside it.
///
/// Unlike the plain-file surface this accepts directories and `.md` files:
/// handing a document to another editor is a legitimate thing to want, because
/// nothing is being written behind the room's back — the file is only being
/// shown to a program the user picked.
fn resolve_in_root(root: &FsPath, rel: &str) -> Result<PathBuf, ApiError> {
    let rel = rel.trim().trim_end_matches('/');
    let canonical_root = root
        .canonicalize()
        .map_err(|e| ApiError::internal(format!("cannot resolve the open folder: {e}")))?;
    if rel.is_empty() {
        return Ok(canonical_root);
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
    let joined = canonical_root.join(as_path);
    if !joined.exists() {
        return Err(ApiError::not_found(format!(
            "nothing named {rel} in the open folder. If it was renamed or \
             deleted outside the app, the tree refreshes when the window \
             regains focus."
        )));
    }
    let canonical = joined
        .canonicalize()
        .map_err(|e| ApiError::internal(format!("cannot resolve {rel}: {e}")))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(ApiError::forbidden(format!(
            "{rel} resolves outside the open folder (a symlink?), so this \
             session will not hand it to another program"
        )));
    }
    Ok(canonical)
}

// ---------------------------------------------------------------------------
// The platform commands
// ---------------------------------------------------------------------------

/// What this platform's file manager is called, for a menu that says "Reveal
/// in Finder" on the machine where that is its name.
pub fn file_manager_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "Finder"
    } else if cfg!(target_os = "windows") {
        "File Explorer"
    } else {
        "file manager"
    }
}

/// Start a command and do not wait for it: a file manager outlives the click
/// that opened it, and a `wait` here would pin a runtime thread to a window
/// the user may leave open all day. A failure to *start* is still reported —
/// that is the case worth a message ("no xdg-open on this machine"), and it is
/// the only one we can honestly detect.
fn spawn_detached(mut command: Command, what: &str) -> Result<(), ApiError> {
    command.spawn().map(|_| ()).map_err(|e| {
        ApiError::unavailable(format!(
            "could not start {what}: {e}. This is the program that opens files \
             on this machine; on Linux it comes from a desktop environment, so \
             a session with no desktop installed will not have it."
        ))
    })
}

fn spawn_reveal(target: &FsPath, is_dir: bool) -> Result<(), ApiError> {
    if cfg!(target_os = "macos") {
        let mut command = Command::new("open");
        command.arg("-R").arg(target);
        return spawn_detached(command, "Finder (`open -R`)");
    }
    if cfg!(target_os = "windows") {
        let mut command = Command::new("explorer.exe");
        if is_dir {
            command.arg(target);
        } else {
            // One argument, comma and all: `/select,` is explorer's spelling,
            // not a shell construct, so the path must not be split off it.
            let mut arg = std::ffi::OsString::from("/select,");
            arg.push(target);
            command.arg(arg);
        }
        return spawn_detached(command, "File Explorer");
    }
    // Linux and the other Unixes: the freedesktop file-manager interface is
    // the only portable way to *select* an entry, and it is not always there.
    // When it is missing, opening the containing folder is the honest
    // degradation — the folder is what the user asked to see.
    if !is_dir && dbus_show_item(target).is_ok() {
        return Ok(());
    }
    let folder = if is_dir {
        target.to_path_buf()
    } else {
        target.parent().unwrap_or(target).to_path_buf()
    };
    let mut command = Command::new("xdg-open");
    command.arg(folder);
    spawn_detached(command, "the file manager (`xdg-open`)")
}

/// Ask the desktop's file manager to show (and select) one item. Waits, unlike
/// everything else here: the answer is the only way to know whether to fall
/// back to `xdg-open`, and the call returns as soon as the manager has been
/// told.
fn dbus_show_item(target: &FsPath) -> Result<(), ()> {
    let uri = format!("file://{}", target.to_string_lossy());
    let status = Command::new("dbus-send")
        .arg("--session")
        .arg("--dest=org.freedesktop.FileManager1")
        .arg("--type=method_call")
        .arg("/org/freedesktop/FileManager1")
        .arg("org.freedesktop.FileManager1.ShowItems")
        .arg(format!("array:string:{uri}"))
        .arg("string:")
        .status();
    match status {
        Ok(status) if status.success() => Ok(()),
        _ => Err(()),
    }
}

fn spawn_open(target: &FsPath) -> Result<(), ApiError> {
    if cfg!(target_os = "macos") {
        let mut command = Command::new("open");
        command.arg(target);
        return spawn_detached(command, "the default program (`open`)");
    }
    if cfg!(target_os = "windows") {
        // Not `cmd /C start`: that hands the path to a shell that would
        // reinterpret `&` and friends in a filename. This entry point takes
        // the path as one argument and applies the file association itself.
        let mut command = Command::new("rundll32.exe");
        command.arg("url.dll,FileProtocolHandler").arg(target);
        return spawn_detached(command, "the default program (`rundll32 url.dll`)");
    }
    let mut command = Command::new("xdg-open");
    command.arg(target);
    spawn_detached(command, "the default program (`xdg-open`)")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Everything the resolver refuses, and the one thing it accepts. None of
    /// these cases spawns anything: the refusal happens first, which is the
    /// property worth a test — see
    /// docs/guarantees/authoring/a-tree-row-opens-in-the-platform.md.
    #[test]
    fn resolves_inside_the_root_and_refuses_everything_else() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::create_dir(root.join("sub")).unwrap();
        std::fs::write(root.join("sub/note.txt"), "hi").unwrap();
        std::fs::write(root.join("doc.hick"), "<doc/>").unwrap();

        // A file, a directory, a document, and the root itself.
        assert_eq!(
            resolve_in_root(&root, "sub/note.txt").unwrap(),
            root.join("sub/note.txt")
        );
        assert_eq!(resolve_in_root(&root, "sub").unwrap(), root.join("sub"));
        assert_eq!(resolve_in_root(&root, "sub/").unwrap(), root.join("sub"));
        assert_eq!(
            resolve_in_root(&root, "doc.hick").unwrap(),
            root.join("doc.hick")
        );
        assert_eq!(resolve_in_root(&root, "").unwrap(), root);

        // Traversal, absolute paths, and names that are simply not there.
        assert!(resolve_in_root(&root, "../secret").is_err());
        assert!(resolve_in_root(&root, "sub/../../secret").is_err());
        assert!(resolve_in_root(&root, "/etc/passwd").is_err());
        assert!(resolve_in_root(&root, "nope.txt").is_err());
    }

    /// A symlink is the interesting escape: the path has no `..` in it and the
    /// file exists, so only canonicalization catches it.
    #[cfg(unix)]
    #[test]
    fn refuses_a_symlink_that_leaves_the_root() {
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("secret.txt"), "s").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::os::unix::fs::symlink(outside.path().join("secret.txt"), root.join("link.txt"))
            .unwrap();

        assert!(resolve_in_root(&root, "link.txt").is_err());
    }
}
