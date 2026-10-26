<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Tree Row Hands Its File To The Rest Of The Machine

For: Authors editing executable documents and the files those documents produce.

Given a folder open in the app, when a row in the file tree is right-clicked —
a file, a directory, a binary the app cannot display, or the folder itself —
then a menu offers the path as text (absolute, relative, and the bare name),
the path shown in this desktop's own file manager under that manager's real
name, and, for a file, the path opened in whatever program this machine
already opens that kind of file with; and when the named path is not inside
the open folder, then nothing is started and the refusal says why.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-tree-row-opens-in-the-platform.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-779a633f282e"></a>
## `apps/web/src/shell/TreeContextMenu.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Table Is A Dataset And A Paragraph](../a-table-is-a-dataset-and-a-paragraph/contract.md), [A Tree Row Hands Its File To The Rest Of The Machine](contract.md).

<a id="f-779a633f282e-TreeContextMenuProps"></a>
### `TreeContextMenuProps`

The tree's right-click menu: the items treeMenu.ts computed, at the pointer.
The behaviour — Escape, click-away, staying on screen, the first item taking
focus — is components/ContextMenu.tsx, shared with the table's menu. What is
left here is the tree's own vocabulary: it deals in `TreeMenuItem`s and
hands one back, so callers never have to look an id back up.

<storycode:copy id="f-779a633f282e-TreeContextMenuProps">
// The tree's right-click menu: the items treeMenu.ts computed, at the pointer.
//
// The behaviour — Escape, click-away, staying on screen, the first item taking
// focus — is components/ContextMenu.tsx, shared with the table's menu. What is
// left here is the tree's own vocabulary: it deals in `TreeMenuItem`s and
// hands one back, so callers never have to look an id back up.

import { ContextMenu } from "../components/ContextMenu";

import type { TreeMenuItem } from "./treeMenu";

export interface TreeContextMenuProps {
  /** Viewport coordinates of the click that opened it. */
  x: number;
  y: number;
  /** What this row offers. */
  items: readonly TreeMenuItem[];
  /** The row it belongs to, for the heading that says what will be acted on. */
  subject: string;
  onPick: (item: TreeMenuItem) => void;
  onClose: () => void;
}

</storycode:copy>
<a id="f-779a633f282e-TreeContextMenu"></a>
### `TreeContextMenu`

<storycode:copy id="f-779a633f282e-TreeContextMenu">
export function TreeContextMenu({ x, y, items, subject, onPick, onClose }: TreeContextMenuProps) {
  return (
    <ContextMenu
      x={x}
      y={y}
      subject={subject}
      items={items}
      onClose={onClose}
      onPick={(id) => {
        const item = items.find((candidate) => candidate.id === id);
        if (item) onPick(item);
      }}
    />
  );
}
</storycode:copy>

### Assemble `apps/web/src/shell/TreeContextMenu.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/TreeContextMenu.tsx"><storycode:paste select="#f-779a633f282e-TreeContextMenuProps"/><storycode:paste select="#f-779a633f282e-TreeContextMenu"/></storycode:file>

<a id="output-5a5f451acc1b"></a>
## `crates/hickory-cli/src/serve/reveal.rs`

Related contracts: [A Tree Row Hands Its File To The Rest Of The Machine](contract.md).

<a id="f-5a5f451acc1b-RevealRequest"></a>
### `RevealRequest`

Handing a file to the rest of the machine: the desktop's file manager, and
whatever program the user has already chosen to open that kind of file.
The tree pane can show a file and this app can edit most of them, but a
notes folder is an ordinary directory on an ordinary computer — sometimes
the thing you want is Finder, or the editor you have used for ten years.
These two routes are that door, and they are the whole door: they hand a
path to the platform and stop. Nothing here reads the file, and nothing
here chooses the program — the OS's own association does.
Why the server and not the desktop shell: `hick up` and the desktop app are
one engine, and the machine whose file manager this opens is the machine
this process runs on, which local-only makes the same machine as the user's.
Putting it in the Tauri shell instead would give the same feature two
implementations and one of them no tests.
The refusals matter more than the commands. A path that escapes the served
folder is refused before anything is spawned, symlinks included: "open
whatever path you name in your default program" is a much larger surface
than "open something in this folder", and only the second one is a feature.

<storycode:copy id="f-5a5f451acc1b-RevealRequest">
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

</storycode:copy>
<a id="f-5a5f451acc1b-reveal"></a>
### `reveal`

`POST /api/reveal` — show the path in the platform's file manager.
A file is *selected* in its containing folder rather than opened; a
directory is opened. That difference is the whole point of a separate
route from [`open_external`]: "show me where this lives".

<storycode:copy id="f-5a5f451acc1b-reveal">
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

</storycode:copy>
<a id="f-5a5f451acc1b-open_external"></a>
### `open_external`

`POST /api/open-external` — open the path in the user's default program
for that kind of file (a directory opens in the file manager).

<storycode:copy id="f-5a5f451acc1b-open_external">
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

</storycode:copy>
<a id="f-5a5f451acc1b-resolve_in_root"></a>
### `resolve_in_root`

Which path, and whether it is ours to hand over
The absolute path `rel` names inside `root`, canonicalized and verified to
still be inside it.
Unlike the plain-file surface this accepts directories and `.md` files:
handing a document to another editor is a legitimate thing to want, because
nothing is being written behind the room's back — the file is only being
shown to a program the user picked.

<storycode:copy id="f-5a5f451acc1b-resolve_in_root">
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

</storycode:copy>
<a id="f-5a5f451acc1b-file_manager_name"></a>
### `file_manager_name`

The platform commands
What this platform's file manager is called, for a menu that says "Reveal
in Finder" on the machine where that is its name.

<storycode:copy id="f-5a5f451acc1b-file_manager_name">
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

</storycode:copy>
<a id="f-5a5f451acc1b-spawn_detached"></a>
### `spawn_detached`

Start a command and do not wait for it: a file manager outlives the click
that opened it, and a `wait` here would pin a runtime thread to a window
the user may leave open all day. A failure to *start* is still reported —
that is the case worth a message ("no xdg-open on this machine"), and it is
the only one we can honestly detect.

<storycode:copy id="f-5a5f451acc1b-spawn_detached">
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

</storycode:copy>
<a id="f-5a5f451acc1b-spawn_reveal"></a>
### `spawn_reveal`

<storycode:copy id="f-5a5f451acc1b-spawn_reveal">
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

</storycode:copy>
<a id="f-5a5f451acc1b-dbus_show_item"></a>
### `dbus_show_item`

Ask the desktop's file manager to show (and select) one item. Waits, unlike
everything else here: the answer is the only way to know whether to fall
back to `xdg-open`, and the call returns as soon as the manager has been
told.

<storycode:copy id="f-5a5f451acc1b-dbus_show_item">
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

</storycode:copy>
<a id="f-5a5f451acc1b-spawn_open"></a>
### `spawn_open`

<storycode:copy id="f-5a5f451acc1b-spawn_open">
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

</storycode:copy>
<a id="f-5a5f451acc1b-tests"></a>
### `tests`

<storycode:copy id="f-5a5f451acc1b-tests">
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
        std::fs::write(root.join("doc.md"), "<doc/>").unwrap();

        // A file, a directory, a document, and the root itself.
        assert_eq!(
            resolve_in_root(&root, "sub/note.txt").unwrap(),
            root.join("sub/note.txt")
        );
        assert_eq!(resolve_in_root(&root, "sub").unwrap(), root.join("sub"));
        assert_eq!(resolve_in_root(&root, "sub/").unwrap(), root.join("sub"));
        assert_eq!(
            resolve_in_root(&root, "doc.md").unwrap(),
            root.join("doc.md")
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
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/reveal.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/reveal.rs"><storycode:paste select="#f-5a5f451acc1b-RevealRequest"/><storycode:paste select="#f-5a5f451acc1b-reveal"/><storycode:paste select="#f-5a5f451acc1b-open_external"/><storycode:paste select="#f-5a5f451acc1b-resolve_in_root"/><storycode:paste select="#f-5a5f451acc1b-file_manager_name"/><storycode:paste select="#f-5a5f451acc1b-spawn_detached"/><storycode:paste select="#f-5a5f451acc1b-spawn_reveal"/><storycode:paste select="#f-5a5f451acc1b-dbus_show_item"/><storycode:paste select="#f-5a5f451acc1b-spawn_open"/><storycode:paste select="#f-5a5f451acc1b-tests"/></storycode:file>

<a id="output-b3afb22aad03"></a>
## `crates/hickory-cli/tests/serve_reveal.rs`

Related contracts: [A Tree Row Hands Its File To The Rest Of The Machine](contract.md).

<a id="f-b3afb22aad03-Session"></a>
### `Session`

`POST /api/reveal` and `POST /api/open-external` — the tree's door to the
rest of the machine, and the refusals that keep it a door rather than a
hole.
Only the refusals are driven over the wire. A successful call ends in
`Finder`/`explorer`/`xdg-open` starting on whatever machine runs the suite,
which is not something a test may do — the accepting half is covered by the
resolver's unit tests in `src/serve/reveal.rs`, and by the fact that both
handlers do nothing else.
Guards docs/guarantees/authoring/a-tree-row-opens-in-the-platform.md.

<storycode:copy id="f-b3afb22aad03-Session">
//! `POST /api/reveal` and `POST /api/open-external` — the tree's door to the
//! rest of the machine, and the refusals that keep it a door rather than a
//! hole.
//!
//! Only the refusals are driven over the wire. A successful call ends in
//! `Finder`/`explorer`/`xdg-open` starting on whatever machine runs the suite,
//! which is not something a test may do — the accepting half is covered by the
//! resolver's unit tests in `src/serve/reveal.rs`, and by the fact that both
//! handlers do nothing else.
//!
//! Guards docs/guarantees/authoring/a-tree-row-opens-in-the-platform.md.

use std::path::PathBuf;

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};

struct Session {
    base: String,
    _dir: tempfile::TempDir,
}

</storycode:copy>
<a id="f-b3afb22aad03-start"></a>
### `start`

<storycode:copy id="f-b3afb22aad03-start">
async fn start() -> Session {
    let dir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("readme.md"), "hello").unwrap();

    let prepared = prepare(ServeOptions {
        target: root,
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .expect("session prepares");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    Session {
        base: format!("http://127.0.0.1:{port}"),
        _dir: dir,
    }
}

</storycode:copy>
<a id="f-b3afb22aad03-post"></a>
### `post`

<storycode:copy id="f-b3afb22aad03-post">
async fn post(session: &Session, path: &str, body: Value) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .post(format!("{}{path}", session.base))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap())
}

</storycode:copy>
<a id="f-b3afb22aad03-a_path_that_leaves_the_open_folder_is_refused_by_both_routes"></a>
### `a_path_that_leaves_the_open_folder_is_refused_by_both_routes`

<storycode:copy id="f-b3afb22aad03-a_path_that_leaves_the_open_folder_is_refused_by_both_routes">
#[tokio::test]
async fn a_path_that_leaves_the_open_folder_is_refused_by_both_routes() {
    let session = start().await;
    for route in ["/api/reveal", "/api/open-external"] {
        for escape in ["../secret", "/etc/passwd", "sub/../../secret"] {
            let (status, body) = post(&session, route, json!({ "path": escape })).await;
            assert_eq!(status, 400, "{route} accepted {escape}: {body}");
            // The message names what was rejected, per user-facing-errors.
            let error = body["error"].as_str().unwrap_or_default();
            assert!(
                error.contains("relative to the open folder"),
                "{route} refused {escape} without saying why: {body}"
            );
        }
    }
}

</storycode:copy>
<a id="f-b3afb22aad03-a_path_that_is_not_there_says_so_rather_than_starting_anything"></a>
### `a_path_that_is_not_there_says_so_rather_than_starting_anything`

<storycode:copy id="f-b3afb22aad03-a_path_that_is_not_there_says_so_rather_than_starting_anything">
#[tokio::test]
async fn a_path_that_is_not_there_says_so_rather_than_starting_anything() {
    let session = start().await;
    for route in ["/api/reveal", "/api/open-external"] {
        let (status, body) = post(&session, route, json!({ "path": "gone.md" })).await;
        assert_eq!(status, 404, "{route}: {body}");
        let error = body["error"].as_str().unwrap_or_default();
        assert!(error.contains("gone.md"), "{body}");
        // The most likely cause, named: the tree is a listing, and listings age.
        assert!(error.contains("refreshes"), "{body}");
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/serve_reveal.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/serve_reveal.rs"><storycode:paste select="#f-b3afb22aad03-Session"/><storycode:paste select="#f-b3afb22aad03-start"/><storycode:paste select="#f-b3afb22aad03-post"/><storycode:paste select="#f-b3afb22aad03-a_path_that_leaves_the_open_folder_is_refused_by_both_routes"/><storycode:paste select="#f-b3afb22aad03-a_path_that_is_not_there_says_so_rather_than_starting_anything"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/shell/FolderTreePane.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-9c0af3061e25)
- [apps/web/src/shell/TreeContextMenu.tsx](contract.md#output-779a633f282e)
- [apps/web/src/shell/treeMenu.ts](../../../mechanisms/apps/web/src/shell/treeMenu.ts.md#output-19605cc8cff0)
- [crates/hickory-cli/src/serve/api.rs](../../../mechanisms/crates/hickory-cli/src/serve/api.rs.md#output-e71b04ae5298)
- [crates/hickory-cli/src/serve/mod.rs](../../../mechanisms/crates/hickory-cli/src/serve/mod.rs.md#output-09ea558208cd)
- [crates/hickory-cli/src/serve/reveal.rs](contract.md#output-5a5f451acc1b)

### Tests and fixtures

- [apps/web/src/shell/FolderTreePane.test.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-508e32e299fd)
- [apps/web/src/shell/treeMenu.test.ts](../../../mechanisms/apps/web/src/shell/treeMenu.ts.md#output-f48c486ac3c1)
- [crates/hickory-cli/tests/serve_files.rs](../../../mechanisms/crates/hickory-cli/tests/serve_files.rs.md#output-44963d50453a)
- [crates/hickory-cli/tests/serve_reveal.rs](contract.md#output-b3afb22aad03)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
