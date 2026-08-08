//! POST /api/docs/:id/run, POST /api/docs/:id/check, GET /api/runs/:id.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::AppState;
use crate::auth::{AuthUser, MaybeUser};
use crate::error::{ApiError, ApiResult};
use crate::routes::docs::{check_read, load_doc};
use crate::runs::{RunKind, start_run};

#[derive(Deserialize, Default, ToSchema)]
pub struct RunRequest {
    /// Requested cell subset. v0 executes the whole document regardless:
    /// exec blocks form a dependency DAG, so partial runs need a dependency
    /// closure the pipeline does not expose yet. Accepted for forward
    /// compatibility.
    #[serde(default)]
    pub cells: Option<Vec<String>>,
}

#[utoipa::path(
    post,
    path = "/api/docs/{id}/run",
    params(("id" = Uuid, Path, description = "doc id")),
    request_body(content = RunRequest, description = "optional cell subset (v0 ignores it)"),
    responses((status = 202, description = "run started", body = Value)),
    tag = "runs"
)]
pub async fn run_doc(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    body: Option<Json<RunRequest>>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let doc = load_doc(&state, id).await?;
    if doc.owner_id != user.id {
        return Err(ApiError::forbidden(
            "only the project owner can run this doc",
        ));
    }
    if let Some(Json(req)) = &body
        && req.cells.as_ref().is_some_and(|c| !c.is_empty())
    {
        log::debug!("run request with cell subset; executing the whole document (v0)");
    }
    let run_id = start_run(&state, doc, &user, RunKind::Run).await?;
    Ok((StatusCode::ACCEPTED, Json(json!({ "run_id": run_id }))))
}

#[utoipa::path(
    post,
    path = "/api/docs/{id}/check",
    params(("id" = Uuid, Path, description = "doc id")),
    responses((status = 202, description = "verification run started", body = Value)),
    tag = "runs"
)]
pub async fn check_doc(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let doc = load_doc(&state, id).await?;
    if doc.owner_id != user.id {
        return Err(ApiError::forbidden(
            "only the project owner can check this doc",
        ));
    }
    let run_id = start_run(&state, doc, &user, RunKind::Check).await?;
    Ok((StatusCode::ACCEPTED, Json(json!({ "run_id": run_id }))))
}

#[utoipa::path(
    get,
    path = "/api/runs/{id}",
    params(("id" = Uuid, Path, description = "run id")),
    responses((status = 200, description = "run status", body = Value)),
    tag = "runs"
)]
pub async fn get_run(
    State(state): State<AppState>,
    MaybeUser(user): MaybeUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Value>> {
    type RunRow = (Uuid, Uuid, String, DateTime<Utc>, Value, Option<String>);
    let row: Option<RunRow> = sqlx::query_as(
        "SELECT id, doc_id, status, started_at, blocks, error FROM runs WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;
    let Some((run_id, doc_id, status, started_at, blocks, error)) = row else {
        return Err(ApiError::not_found("run not found"));
    };
    let doc = load_doc(&state, doc_id).await?;
    check_read(&doc, user.as_ref())?;
    let mut body = json!({
        "id": run_id,
        "status": status,
        "started_at": started_at.to_rfc3339(),
        "blocks": blocks,
    });
    if let Some(error) = error {
        body["error"] = json!(error);
    }
    Ok(Json(body))
}
