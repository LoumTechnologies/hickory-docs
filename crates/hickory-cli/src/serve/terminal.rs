//! Terminal sessions over the local API.
//!
//! The routes are thin on purpose: everything that decides anything —
//! what state a session is in, who gets to interrupt you, what turbo may
//! answer — lives in `hick-term`, where it is pure and tested. This module
//! moves bytes and translates errors into sentences a person can act on.
//!
//! ## Why terminals have their own socket
//!
//! `/api/ws` is the document socket: it is keyed by `?doc=doc:<id>`, and it
//! refuses anything that is not a room this session serves. A terminal
//! belongs to a directory and a task, not to a document — several of them can
//! exist in a session with no documents open at all — so it gets
//! `/api/terminals/ws?session=<id>` rather than a widened gate on the other
//! one.
//!
//! There is no authorization here for the same reason there is none anywhere
//! else in this server: it binds loopback and answers one person, the one
//! whose machine it is. A terminal runs what that person could already run by
//! typing it. See `docs/specs/freeform/local-only.md`.

use std::path::PathBuf;
use std::sync::Arc;

use axum::Json;
use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use axum::extract::{Path as UrlPath, Query, State};
use axum::response::{IntoResponse, Response};
use futures::{SinkExt as _, StreamExt as _};
use hick_term::{Session, SessionSpec};
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

/// `POST /api/terminals` — start a session.
#[derive(Deserialize)]
pub struct OpenBody {
    /// What the tab says. A task's name, not a number.
    pub title: Option<String>,
    /// Where it runs, relative to the open folder. Defaults to the folder.
    pub cwd: Option<String>,
    /// The command. Empty or absent means the configured shell.
    #[serde(default)]
    pub argv: Vec<String>,
    /// A support process for the monitor dock.
    #[serde(default)]
    pub monitor: bool,
    /// Run in a fresh git worktree on this new branch, rather than in the
    /// open folder — so two agents cannot fight over one checkout.
    pub worktree_branch: Option<String>,
}

pub async fn open(
    State(state): State<LocalState>,
    Json(body): Json<OpenBody>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let spec = SessionSpec {
        title: body
            .title
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| "Terminal".to_string()),
        cwd: resolve_cwd(&root, body.cwd.as_deref()),
        argv: body.argv,
        monitor: body.monitor,
    };

    let session = match &body.worktree_branch {
        Some(branch) if !branch.trim().is_empty() => state
            .terminals
            .open_in_worktree(&root, branch.trim(), spec)
            .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?,
        _ => state
            .terminals
            .open(spec)
            .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?,
    };
    Ok(Json(json!(session.summary())))
}

/// `GET /api/terminals` — every session, and the one queue across them.
pub async fn list(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    // Auto-answering happens here, once, before the snapshot is taken — so
    // the queue a client reads already reflects what turbo handled.
    state.terminals.sweep_turbo();
    let sessions = state.terminals.summaries();
    let attention = state.terminals.attention(&sessions);
    Ok(Json(json!({
        "sessions": sessions,
        "attention": attention,
        "turbo": state.terminals.turbo(),
    })))
}

/// `DELETE /api/terminals/{id}` — stop a session and forget it.
pub async fn close(
    State(state): State<LocalState>,
    UrlPath(id): UrlPath<String>,
) -> ApiResult<Json<Value>> {
    state
        .terminals
        .close(&id)
        .map_err(|e| ApiError::not_found(format!("{e:#}")))?;
    Ok(Json(json!({ "closed": id })))
}

#[derive(Deserialize)]
pub struct InputBody {
    pub data: String,
}

/// `POST /api/terminals/{id}/input` — type into a session.
pub async fn input(
    State(state): State<LocalState>,
    UrlPath(id): UrlPath<String>,
    Json(body): Json<InputBody>,
) -> ApiResult<Json<Value>> {
    let session = session_of(&state, &id)?;
    session
        .write(body.data.as_bytes())
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct ResizeBody {
    pub rows: u16,
    pub cols: u16,
}

/// `POST /api/terminals/{id}/resize` — follow the pane.
pub async fn resize(
    State(state): State<LocalState>,
    UrlPath(id): UrlPath<String>,
    Json(body): Json<ResizeBody>,
) -> ApiResult<Json<Value>> {
    if body.rows == 0 || body.cols == 0 {
        return Err(ApiError::bad_request(
            "a terminal cannot be resized to zero rows or columns — send the pane's \
             measured size, or leave the size alone until it has one",
        ));
    }
    let session = session_of(&state, &id)?;
    session
        .resize(body.rows, body.cols)
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    Ok(Json(json!({ "ok": true })))
}

/// `POST /api/terminals/{id}/interrupt` — what ^C does.
pub async fn interrupt(
    State(state): State<LocalState>,
    UrlPath(id): UrlPath<String>,
) -> ApiResult<Json<Value>> {
    let session = session_of(&state, &id)?;
    session
        .interrupt()
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct AnswerBody {
    /// Exactly what to send — the `send` of the choice that was pressed.
    pub send: String,
}

/// `POST /api/terminals/{id}/answer` — answer the attention card.
///
/// Separate from `/input` because it means something different: it clears the
/// declared prompt as well as writing, so the session leaves the queue on the
/// same request that answers it rather than on the next poll.
pub async fn answer(
    State(state): State<LocalState>,
    UrlPath(id): UrlPath<String>,
    Json(body): Json<AnswerBody>,
) -> ApiResult<Json<Value>> {
    let session = session_of(&state, &id)?;
    session
        .write(body.send.as_bytes())
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    session.declare_prompt(None);
    Ok(Json(json!(session.summary())))
}

#[derive(Deserialize)]
pub struct TurboBody {
    pub enabled: bool,
}

/// `PUT /api/terminals/turbo` — let routine declared prompts answer
/// themselves. See `hick_term::turbo` for everything it refuses to do.
pub async fn set_turbo(
    State(state): State<LocalState>,
    Json(body): Json<TurboBody>,
) -> ApiResult<Json<Value>> {
    state.terminals.set_turbo(body.enabled);
    Ok(Json(json!({ "turbo": state.terminals.turbo() })))
}

#[derive(Deserialize)]
pub struct WsParams {
    session: String,
}

/// `WS /api/terminals/ws?session=<id>` — the bytes.
///
/// Out: binary frames of raw PTY output, starting with everything the session
/// has already said. In: whatever you type, binary or text. Sizing goes over
/// REST, so every frame on this socket means the same thing and there is no
/// control channel to get out of step with the data.
pub async fn ws_handler(
    State(state): State<LocalState>,
    Query(params): Query<WsParams>,
    upgrade: WebSocketUpgrade,
) -> Response {
    let Some(session) = state.terminals.get(&params.session) else {
        return ApiError::not_found(format!(
            "no terminal {} in this session — it may have been closed",
            params.session
        ))
        .into_response();
    };
    upgrade.on_upgrade(move |socket| pump(socket, session))
}

async fn pump(socket: WebSocket, session: Arc<Session>) {
    let (mut tx, mut rx) = socket.split();
    let (replay, mut live) = session.attach();

    if !replay.is_empty() && tx.send(WsMessage::Binary(replay.into())).await.is_err() {
        return;
    }

    let writer = session.clone();
    let mut from_client = tokio::spawn(async move {
        while let Some(Ok(message)) = rx.next().await {
            let bytes = match message {
                WsMessage::Binary(b) => b.to_vec(),
                WsMessage::Text(t) => t.as_bytes().to_vec(),
                WsMessage::Close(_) => break,
                _ => continue,
            };
            if writer.write(&bytes).is_err() {
                break;
            }
        }
    });

    loop {
        tokio::select! {
            _ = &mut from_client => break,
            received = live.recv() => match received {
                Ok(bytes) => {
                    if tx.send(WsMessage::Binary(bytes.to_vec().into())).await.is_err() {
                        break;
                    }
                }
                // Lagged: this client fell far enough behind that output was
                // dropped rather than buffered forever. Reconnecting replays
                // the scrollback, which is the whole state anyway.
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => break,
            },
        }
    }
    from_client.abort();
}

fn session_of(state: &LocalState, id: &str) -> Result<Arc<Session>, ApiError> {
    state.terminals.get(id).ok_or_else(|| {
        ApiError::not_found(format!(
            "no terminal {id} in this session — it may have been closed already"
        ))
    })
}

/// Where a session runs: the open folder, or a directory under it.
///
/// A relative path is resolved against the folder because that is what the
/// client knows about; an absolute one is taken as given, since a terminal
/// can go wherever the person whose machine this is could `cd`.
fn resolve_cwd(root: &std::path::Path, cwd: Option<&str>) -> PathBuf {
    match cwd.map(str::trim).filter(|c| !c.is_empty()) {
        None => root.to_path_buf(),
        Some(path) => {
            let candidate = PathBuf::from(path);
            if candidate.is_absolute() {
                candidate
            } else {
                root.join(candidate)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_relative_directory_lands_under_the_open_folder() {
        let root = std::path::Path::new("/home/someone/project");
        assert_eq!(
            resolve_cwd(root, Some("crates/hick-term")).as_path(),
            root.join("crates/hick-term")
        );
        assert_eq!(resolve_cwd(root, None).as_path(), root);
        assert_eq!(resolve_cwd(root, Some("  ")).as_path(), root);
    }

    #[test]
    fn an_absolute_directory_is_taken_as_given() {
        let root = std::path::Path::new("/home/someone/project");
        assert_eq!(
            resolve_cwd(root, Some("/tmp")).as_path(),
            std::path::Path::new("/tmp")
        );
    }
}
