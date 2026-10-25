<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/story.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Recipe Commit Can Be Replayed, Into A New Commit, Never Over The Old One](../../../../../../guarantees/lenses/a-recipe-commit-can-be-replayed/contract.md)
- [The Past Is Edited By Rebase, Above The Floor Only](../../../../../../guarantees/lenses/the-past-is-edited-by-rebase-above-the-floor/contract.md)
- [The Tail Of The Story Is The Next Commit](../../../../../../guarantees/lenses/the-tail-of-the-story-is-the-next-commit/contract.md)

Captured SHA-256: `5f6fcd0472d2d28237a87752c9e9b42fb769937f1197bde15bd673443fa573f7`.

<book:entry path="crates/hickory-cli/src/serve/story.rs">
//! The history lens's verbs: replay a recipe, run one at the tail, and edit
//! the drafts above the floor. `docs/specs/freeform/lenses.md`, steps 4–6.
//!
//! Each is one git operation run as itself (`crate::recipe`, `crate::story`),
//! and each refuses in words: a dirty tree, a record below the floor, a merge
//! among the drafts. A rebase or merge that stops on a conflict is answered
//! with `409` and git's own words, and the repository is left where git left
//! it — never hidden behind a spinner, never undone silently.

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

fn words(error: anyhow::Error) -> ApiError {
    if let Some(stopped) = error.downcast_ref::<crate::recipe::JoinStopped>() {
        return ApiError::conflict(format!("{stopped}")).with_detail(json!({
            "replayed": stopped.replayed,
            "moved": stopped.moved,
        }));
    }
    let text = format!("{error:#}");
    if text.contains("rebase stopped") {
        return ApiError::conflict(text);
    }
    ApiError::unprocessable(text)
}

#[derive(Deserialize)]
pub struct ShaRequest {
    pub sha: String,
}

/// `POST /api/git/replay {sha}` — run a recipe commit's recipe again and
/// join the result: rebase above the floor, merge below.
pub async fn replay(
    State(state): State<LocalState>,
    Json(body): Json<ShaRequest>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let replay = tokio::task::spawn_blocking(move || crate::recipe::replay(&root, &body.sha))
        .await
        .map_err(|e| ApiError::internal(format!("the replay did not finish: {e}")))?
        .map_err(words)?;
    Ok(Json(
        serde_json::to_value(replay).unwrap_or_else(|_| json!({})),
    ))
}

#[derive(Deserialize)]
pub struct EmitRequest {
    pub command: String,
    pub output: String,
}

/// `POST /api/git/recipe {command, output}` — the tail: run a command in a
/// clean worktree at HEAD and commit what it wrote under `output/` as
/// HEAD's child, with the recipe in the trailers.
pub async fn emit(
    State(state): State<LocalState>,
    Json(body): Json<EmitRequest>,
) -> ApiResult<Json<Value>> {
    if body.command.trim().is_empty() {
        return Err(ApiError::bad_request("a recipe needs a command to run."));
    }
    let root = state.index.root().to_path_buf();
    let run = tokio::task::spawn_blocking(move || {
        crate::recipe::emit(&root, &body.command, &body.output)
    })
    .await
    .map_err(|e| ApiError::internal(format!("the run did not finish: {e}")))?
    .map_err(words)?;
    Ok(Json(json!({
        "sha": run.sha,
        "short": run.short,
        "output_tree": run.output_tree,
        "said": run.said,
    })))
}

#[derive(Deserialize)]
pub struct RewordRequest {
    pub sha: String,
    pub message: String,
}

/// `POST /api/git/reword {sha, message}` — change a draft's message.
pub async fn reword(
    State(state): State<LocalState>,
    Json(body): Json<RewordRequest>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let head =
        tokio::task::spawn_blocking(move || crate::story::reword(&root, &body.sha, &body.message))
            .await
            .map_err(|e| ApiError::internal(format!("the rebase did not finish: {e}")))?
            .map_err(words)?;
    Ok(Json(json!({ "head": head })))
}

/// `POST /api/git/drop {sha}` — remove a draft from the history.
pub async fn drop(
    State(state): State<LocalState>,
    Json(body): Json<ShaRequest>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let head = tokio::task::spawn_blocking(move || crate::story::drop(&root, &body.sha))
        .await
        .map_err(|e| ApiError::internal(format!("the rebase did not finish: {e}")))?
        .map_err(words)?;
    Ok(Json(json!({ "head": head })))
}

#[derive(Deserialize)]
pub struct MoveRequest {
    pub sha: String,
    pub direction: crate::story::Direction,
}

/// `POST /api/git/move {sha, direction}` — swap a draft with its neighbour.
pub async fn move_commit(
    State(state): State<LocalState>,
    Json(body): Json<MoveRequest>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let head = tokio::task::spawn_blocking(move || {
        crate::story::move_commit(&root, &body.sha, body.direction)
    })
    .await
    .map_err(|e| ApiError::internal(format!("the rebase did not finish: {e}")))?
    .map_err(words)?;
    Ok(Json(json!({ "head": head })))
}
</book:entry>

</book:doc>
