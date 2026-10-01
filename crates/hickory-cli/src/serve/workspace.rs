//! What you had open, and what you had not saved.
//!
//! Five routes over [`hickory_workspace::WorkspaceStore`], which writes under
//! the user's own data directory rather than inside the project. The reason is
//! in that crate's header and worth repeating here, because this is the file
//! somebody would reach for when they wanted to "just put it in `.hickory/`":
//! a draft is unfinished work its author has not decided to keep, and a
//! `.gitignore` entry is a promise this tool cannot keep on somebody else's
//! machine.
//!
//! Nothing here is required for the app to work. A store that cannot be
//! opened — a read-only home directory, a platform with no data directory —
//! degrades to "the window forgets its layout", never to a startup failure.
//! See `.instructions/config-and-environments.md`.

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::{Value, json};

use hickory_workspace::{Draft, WorkspaceStore};

use super::LocalState;
use super::api::{ApiError, ApiResult};

/// The store for the served folder.
///
/// Opened per request rather than held on [`LocalState`]: it is a path and a
/// `create_dir_all`, these routes fire a handful of times a session, and a
/// handle cached at boot would keep pointing at the old folder after the
/// desktop shell switched projects.
fn store(state: &LocalState) -> ApiResult<WorkspaceStore> {
    WorkspaceStore::for_project(state.index.root())
        .and_then(|store| match &state.window_slot {
            Some(slot) => store.window(slot),
            None => Ok(store),
        })
        .map_err(|e| {
            ApiError::unavailable(format!(
                "{e:#}\n  The app works without this — it will just forget which tabs \
             were open the next time it starts."
            ))
        })
}

/// `GET /api/workspace/ui` — the stored layout, or null.
pub async fn get_ui(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let stored = store(&state)?.load_ui();
    Ok(Json(json!({ "state": stored })))
}

#[derive(Deserialize)]
pub struct PutUi {
    /// Opaque to the server: which tabs sit in which panes is a shape the UI
    /// owns, and a second definition here would be one more thing to keep in
    /// step for no benefit.
    pub state: Value,
}

/// `PUT /api/workspace/ui` — replace the stored layout.
pub async fn put_ui(
    State(state): State<LocalState>,
    Json(body): Json<PutUi>,
) -> ApiResult<Json<Value>> {
    store(&state)?
        .save_ui(&body.state)
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    Ok(Json(json!({ "ok": true })))
}

/// `GET /api/workspace/drafts` — every buffer with unsaved changes.
///
/// Each carries the bytes it was taken from, so the page can tell "the file
/// is untouched, just restore it" from "the file moved on, this needs a
/// merge" without a second round trip.
pub async fn list_drafts(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    Ok(Json(json!({ "drafts": store(&state)?.list_drafts() })))
}

#[derive(Deserialize)]
pub struct PutDraft {
    pub path: String,
    pub contents: String,
    /// The file's contents when this editing session began.
    #[serde(default)]
    pub base: String,
    #[serde(default)]
    pub saved_at: u64,
}

/// `PUT /api/workspace/drafts` — write one buffer down.
pub async fn put_draft(
    State(state): State<LocalState>,
    Json(body): Json<PutDraft>,
) -> ApiResult<Json<Value>> {
    store(&state)?
        .save_draft(&Draft {
            path: body.path,
            contents: body.contents,
            base: body.base,
            saved_at: body.saved_at,
        })
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct DraftQuery {
    pub path: String,
}

/// `DELETE /api/workspace/drafts?path=…` — the buffer was saved, or the
/// reader threw the draft away.
pub async fn discard_draft(
    State(state): State<LocalState>,
    Query(q): Query<DraftQuery>,
) -> ApiResult<Json<Value>> {
    store(&state)?.discard_draft(&q.path)?;
    Ok(Json(json!({ "ok": true })))
}
