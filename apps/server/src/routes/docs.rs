//! GET/PUT /api/docs/:id, GET /api/docs/:id/render.

use axum::Json;
use axum::extract::{Path, State};
use chrono::{DateTime, Utc};
use hick_literate::render::Block;
use serde::Deserialize;
use serde_json::{Value, json};
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

fn doc_json(doc: &DocRow) -> Value {
    json!({
        "id": doc.id,
        "path": doc.path,
        "source": doc.source,
        "updated_at": doc.updated_at.to_rfc3339(),
    })
}

pub async fn get_doc(
    State(state): State<AppState>,
    MaybeUser(user): MaybeUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Value>> {
    let doc = load_doc(&state, id).await?;
    check_read(&doc, user.as_ref())?;
    Ok(Json(doc_json(&doc)))
}

#[derive(Deserialize)]
pub struct SaveDoc {
    pub source: String,
}

pub async fn put_doc(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<SaveDoc>,
) -> ApiResult<Json<Value>> {
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
    Ok(Json(doc_json(&doc)))
}

/// Render the block model for the doc's current source, overlaying the last
/// finished run's per-exec status + transcript. Blocks whose source changed
/// after that run (doc updated later) are marked `stale`.
pub async fn render_doc(
    State(state): State<AppState>,
    MaybeUser(user): MaybeUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Value>> {
    let doc = load_doc(&state, id).await?;
    check_read(&doc, user.as_ref())?;

    let blocks = crate::runs::weave_blocks(&state, &doc)
        .await
        .map_err(|e| ApiError::unprocessable(format!("render failed: {e}")))?;

    // Last finished run for this doc.
    let last_run: Option<(Value, Option<DateTime<Utc>>)> = sqlx::query_as(
        "SELECT blocks, finished_at FROM runs
         WHERE doc_id = $1 AND status IN ('ok', 'failed') AND kind IN ('run', 'check')
         ORDER BY started_at DESC LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let mut rendered: Vec<Value> = blocks
        .iter()
        .map(|b| serde_json::to_value(b).expect("serializable block"))
        .collect();

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
    Ok(Json(json!({ "blocks": rendered })))
}
