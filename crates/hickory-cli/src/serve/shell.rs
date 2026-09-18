//! The things only the program *around* this server can do.
//!
//! The server is an axum router. It can commit a scaffold, run a terminal and
//! read a repository; it cannot open a window, and it cannot put a native
//! modal in front of anyone. Those need an event loop, a window handle and a
//! platform toolkit, all of which belong to whatever is hosting this router —
//! the desktop app for one person, and for another a browser tab that `hick
//! up` has no authority over at all.
//!
//! So the shell hands its own powers down here, after [`super::prepare`],
//! rather than through `ServeOptions`: the powers are the *host's*, and saying
//! so in a type keeps them from being quietly assumed anywhere else. A host
//! that has none installs nothing, and every route behind this answers with a
//! sentence saying which program would have had to do it.
//!
//! Compare `reveal.rs`, which is deliberately the opposite: handing a path
//! *to* the machine needs nothing but a subprocess, so it lives in the server
//! where it can be tested. Asking the machine *for* a path cannot.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

/// Where a folder should be opened, when the app is asked to open one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OpenWhere {
    /// Leave every window as it is.
    None,
    /// A second process on that folder — this one is untouched.
    NewWindow,
    /// This process, on that folder instead. A session is a process here
    /// (the directory lock and the watcher are per-process), so this is a
    /// restart: everything in this window goes, terminals included.
    ThisWindow,
}

/// The host's powers, as closures.
pub struct Shell {
    /// Open `folder` in a window. `OpenWhere::None` never reaches this.
    #[allow(clippy::type_complexity)]
    pub open_folder: Arc<dyn Fn(&Path, OpenWhere) -> Result<()> + Send + Sync>,
    /// Put a native folder picker in front of the person, starting at the
    /// given folder. `Ok(None)` is a cancel, which is an ordinary answer and
    /// never an error.
    #[allow(clippy::type_complexity)]
    pub pick_folder: Arc<dyn Fn(&Path) -> Result<Option<PathBuf>> + Send + Sync>,
    /// Put a native Save dialog in front of the person. The name is only a
    /// suggestion; the chooser remains the person's decision about both name
    /// and location.
    #[allow(clippy::type_complexity)]
    pub save_file: Arc<dyn Fn(&Path, &str) -> Result<Option<PathBuf>> + Send + Sync>,
    /// Close this host window after the page has resolved unsaved buffers.
    pub close_window: Arc<dyn Fn() -> Result<()> + Send + Sync>,
}

impl std::fmt::Debug for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Shell")
    }
}

impl LocalState {
    /// Hand the server the shell's own powers. The desktop app calls this on
    /// the [`super::Prepared`] state before serving.
    pub fn set_shell(&self, shell: Shell) {
        if let Ok(mut slot) = self.shell.lock() {
            *slot = Some(shell);
        }
    }

    /// Whether this program is one that has windows and dialogs at all.
    ///
    /// Published so a client can leave out an affordance it cannot honour,
    /// rather than draw a button that always fails. A browser tab served by
    /// `hick up` gets no ellipsis beside the location field, because there is
    /// no native picker on the other end of it.
    pub fn has_shell(&self) -> bool {
        self.shell.lock().map(|s| s.is_some()).unwrap_or(false)
    }

    /// Open a folder in a window, or say why this program cannot.
    pub fn open_folder(&self, folder: &Path, where_: OpenWhere) -> Result<()> {
        if where_ == OpenWhere::None {
            return Ok(());
        }
        match self.hook(|s| s.open_folder.clone()) {
            Some(open) => open(folder, where_),
            None => anyhow::bail!(
                "this engine has no window to open: it is being served by `hick up`, and the \
                 page you are looking at is a tab in your own browser.\n  \
                 Next step: open {} with `hick open`, or point another `hick up` at it.",
                folder.display()
            ),
        }
    }

    pub fn close_window(&self) -> Result<()> {
        match self.hook(|s| s.close_window.clone()) {
            Some(close) => close(),
            None => {
                anyhow::bail!("this engine is running in a browser tab; close it with the browser")
            }
        }
    }

    fn hook<T>(&self, take: impl Fn(&Shell) -> T) -> Option<T> {
        self.shell
            .lock()
            .ok()
            .and_then(|slot| slot.as_ref().map(take))
    }
}

#[derive(Deserialize)]
pub struct PickRequest {
    /// Where the picker opens. Absolute, `~`-prefixed, or relative to the
    /// open folder; anything unusable falls back to the open folder rather
    /// than refusing, because a picker is a place to *browse from* and being
    /// told your starting point was invalid is not help.
    #[serde(default)]
    pub start: String,
}

#[derive(Deserialize)]
pub struct SaveFileRequest {
    pub name: String,
}

/// `POST /api/save-file-dialog` — the platform Save dialog, initially in the
/// open folder. A browser-hosted engine has no such power and says so plainly.
pub async fn save_file_dialog(
    State(state): State<LocalState>,
    Json(body): Json<SaveFileRequest>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let hook = state.hook(|s| s.save_file.clone()).ok_or_else(|| {
        ApiError::unavailable(
            "this engine has no native Save dialog: it is being served by `hick up`, and the \
             page you are looking at is a tab in your own browser.",
        )
    })?;
    let picked = tokio::task::spawn_blocking(move || hook(&root, &body.name))
        .await
        .map_err(|e| ApiError::internal(format!("the Save dialog did not finish: {e}")))?
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    Ok(Json(
        json!({ "path": picked.map(|p| p.to_string_lossy().into_owned()) }),
    ))
}

/// `POST /api/pick-folder` — the platform's own folder chooser.
///
/// A blocking native modal, so it runs on a blocking thread: the desktop
/// shell's own menu handler has the same note, and for the same reason —
/// putting one of these on the main thread deadlocks the app.
///
/// Cancelling answers `{"path": null}` and not an error. A person closing a
/// dialog has said something perfectly clear, and a client that has to catch
/// an exception to hear it will eventually show it as one.
pub async fn pick_folder(
    State(state): State<LocalState>,
    Json(body): Json<PickRequest>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let start = crate::scaffold_commit::absolute_folder(&root, &body.start);
    let start = if start.is_dir() { start } else { root };
    let hook = state.hook(|s| s.pick_folder.clone()).ok_or_else(|| {
        ApiError::unavailable(
            "this engine has no native folder picker: it is being served by `hick up`, and the \
             page you are looking at is a tab in your own browser, which cannot be shown one.\n  \
             Next step: type or paste the path instead.",
        )
    })?;
    let picked = tokio::task::spawn_blocking(move || hook(&start))
        .await
        .map_err(|e| ApiError::internal(format!("the folder picker did not finish: {e}")))?
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    Ok(Json(
        json!({ "path": picked.map(|p| p.to_string_lossy().into_owned()) }),
    ))
}

/// `POST /api/window/close` — called only after the page's save prompt.
pub async fn close_window(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    state
        .close_window()
        .map_err(|e| ApiError::unavailable(format!("{e:#}")))?;
    Ok(Json(json!({ "ok": true })))
}
