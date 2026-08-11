//! GET/PUT /api/docs/:id, GET /api/docs/:id/render.

use axum::Json;
use axum::extract::{Path, State};
use chrono::{DateTime, Utc};
use hick_literate::render::Block;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::AppState;
use crate::auth::{AuthUser, MaybeUser, User};
use crate::error::{ApiError, ApiResult};

/// A doc row joined with its project's access facts.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DocRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub path: String,
    pub source: String,
    pub updated_at: DateTime<Utc>,
    pub owner_id: Uuid,
    pub visibility: String,
}

pub async fn load_doc(state: &AppState, id: Uuid) -> Result<DocRow, ApiError> {
    sqlx::query_as::<_, DocRow>(
        "SELECT d.id, d.project_id, d.path, d.source, d.updated_at,
                p.owner_id, p.visibility
         FROM docs d JOIN projects p ON p.id = d.project_id
         WHERE d.id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("doc not found"))
}

/// Read access: owner, or anyone when the project is public.
pub fn check_read(doc: &DocRow, user: Option<&User>) -> Result<(), ApiError> {
    if doc.visibility == "public" || user.is_some_and(|u| u.id == doc.owner_id) {
        Ok(())
    } else {
        Err(ApiError::forbidden("no access to this document"))
    }
}

/// `GET /api/docs/:id`, `PUT /api/docs/:id`, and the doc-creation responses
/// in `routes::projects` all return this exact shape.
#[derive(Debug, Serialize, ToSchema)]
pub struct DocOut {
    pub id: Uuid,
    pub path: String,
    pub source: String,
    pub updated_at: String,
}

fn doc_out(doc: &DocRow) -> DocOut {
    DocOut {
        id: doc.id,
        path: doc.path.clone(),
        source: doc.source.clone(),
        updated_at: doc.updated_at.to_rfc3339(),
    }
}

#[utoipa::path(
    get,
    path = "/api/docs/{id}",
    params(("id" = Uuid, Path, description = "doc id")),
    responses((status = 200, description = "the doc", body = DocOut)),
    tag = "docs"
)]
pub async fn get_doc(
    State(state): State<AppState>,
    MaybeUser(user): MaybeUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<DocOut>> {
    let doc = load_doc(&state, id).await?;
    check_read(&doc, user.as_ref())?;
    Ok(Json(doc_out(&doc)))
}

#[derive(Deserialize, ToSchema)]
pub struct SaveDoc {
    pub source: String,
}

#[utoipa::path(
    put,
    path = "/api/docs/{id}",
    params(("id" = Uuid, Path, description = "doc id")),
    request_body = SaveDoc,
    responses((status = 200, description = "the saved doc", body = DocOut)),
    tag = "docs"
)]
pub async fn put_doc(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<SaveDoc>,
) -> ApiResult<Json<DocOut>> {
    let mut doc = load_doc(&state, id).await?;
    if doc.owner_id != user.id {
        return Err(ApiError::forbidden(
            "only the project owner can save this doc",
        ));
    }
    let row = sqlx::query_as::<_, (DateTime<Utc>,)>(
        "UPDATE docs SET source = $1, updated_at = now() WHERE id = $2 RETURNING updated_at",
    )
    .bind(&body.source)
    .bind(id)
    .fetch_one(&state.db)
    .await?;
    // Any editor with this document open holds its own CRDT copy, and that
    // copy wins the next time the room persists. Without this, saving through
    // the API while someone has the document open silently reverts a moment
    // later — the same lost update that `/outputs/edit` had.
    state
        .rooms
        .apply_external_source(&doc.id.to_string(), &body.source)
        .await;
    state
        .git
        .save_file(
            doc.project_id,
            &doc.path,
            &body.source,
            &format!("Save {}", doc.path),
        )
        .await?;
    doc.source = body.source;
    doc.updated_at = row.0;
    Ok(Json(doc_out(&doc)))
}

/// `blocks` is `hick_literate::render::Block`, overlaid in place with run
/// status/transcript as generic JSON below — the overlay reads/writes fields
/// by name (`kind`, `id`, `status`, `transcript`) on `serde_json::Value`
/// rather than through `Block`'s own enum variants, so the response is kept
/// as a generic array rather than asserting a schema the handler doesn't
/// actually construct through the typed enum.
#[derive(Debug, Serialize, ToSchema)]
pub struct RenderOut {
    #[schema(value_type = Vec<Object>)]
    pub blocks: Vec<Value>,
}

/// Render the block model for the doc's current source, overlaying the last
/// finished run's per-exec status + transcript. Blocks whose source changed
/// after that run (doc updated later) are marked `stale`.
///
/// The weave is served from `AppState::renders` when nothing it depends on
/// has changed (see `crate::render_cache`); the run overlay below is always
/// recomputed, so statuses and transcripts are never cached.
#[utoipa::path(
    get,
    path = "/api/docs/{id}/render",
    params(("id" = Uuid, Path, description = "doc id")),
    responses((status = 200, description = "rendered block model", body = RenderOut)),
    tag = "docs"
)]
pub async fn render_doc(
    State(state): State<AppState>,
    MaybeUser(user): MaybeUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<RenderOut>> {
    let doc = load_doc(&state, id).await?;
    check_read(&doc, user.as_ref())?;

    // Project-wide revisions: newest doc save (includes) and newest finished
    // run (committed outputs in the checkout the weave reads).
    let (docs_revision, outputs_revision): (Option<DateTime<Utc>>, Option<DateTime<Utc>>) =
        sqlx::query_as(
            "SELECT (SELECT max(updated_at) FROM docs WHERE project_id = $1),
                    (SELECT max(finished_at) FROM runs r
                       JOIN docs d ON d.id = r.doc_id
                      WHERE d.project_id = $1)",
        )
        .bind(doc.project_id)
        .fetch_one(&state.db)
        .await?;
    let cache_key =
        crate::render_cache::RenderKey::new(doc.id, &doc.source, docs_revision, outputs_revision);

    let woven: Value = match state.renders.get(&cache_key) {
        Some(hit) => hit,
        None => {
            let value = crate::runs::weave_blocks_json(&state, &doc)
                .await
                .map_err(|e| ApiError::unprocessable(format!("render failed: {e}")))?;
            state.renders.insert(cache_key, value.clone());
            value
        }
    };

    // Last finished run for this doc.
    let last_run: Option<(Value, Option<DateTime<Utc>>)> = sqlx::query_as(
        "SELECT blocks, finished_at FROM runs
         WHERE doc_id = $1 AND status IN ('ok', 'failed') AND kind IN ('run', 'check')
         ORDER BY started_at DESC LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let mut rendered: Vec<Value> = match woven {
        Value::Array(blocks) => blocks,
        other => vec![other],
    };

    if let Some((run_blocks, finished_at)) = last_run {
        let stale = finished_at.is_some_and(|t| doc.updated_at > t);
        let by_id: std::collections::HashMap<&str, &Value> = run_blocks
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|b| b.get("exec_id").and_then(Value::as_str).map(|k| (k, b)))
                    .collect()
            })
            .unwrap_or_default();
        for block in &mut rendered {
            if block.get("kind").and_then(Value::as_str) != Some("exec") {
                continue;
            }
            let Some(bid) = block.get("id").and_then(Value::as_str) else {
                continue;
            };
            if let Some(run_block) = by_id.get(bid) {
                let obj = block.as_object_mut().unwrap();
                if let Some(t) = run_block
                    .get("transcript")
                    .filter(|t| t.as_array().is_some_and(|a| !a.is_empty()))
                {
                    obj.insert("transcript".to_string(), t.clone());
                }
                let status = if stale {
                    Value::String("stale".to_string())
                } else {
                    run_block.get("status").cloned().unwrap_or(Value::Null)
                };
                if !status.is_null() {
                    obj.insert("status".to_string(), status);
                }
            }
        }
    }

    let _unused: Option<&Block> = None; // (type anchor: rendered mirrors render::Block)
    Ok(Json(RenderOut { blocks: rendered }))
}
