//! Generated outputs & lineage (api.md v0.2):
//! - `GET  /api/docs/:id/outputs`             — files from the last successful run
//! - `GET  /api/docs/:id/outputs/file?path=…` — content + `Provenance[]`
//! - `POST /api/docs/:id/outputs/edit`        — map output edits to source edits,
//!   apply them to the doc source (Postgres row + git commit), return them.

use axum::Json;
use axum::extract::{Path, Query, State};
use hickory_lineage::{LineageError, OutputEdit, Provenance};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::auth::{AuthUser, MaybeUser};
use crate::error::{ApiError, ApiResult};
use crate::routes::docs::{DocRow, check_read, load_doc};

/// The last successful `run` for a doc, if any.
async fn last_ok_run(state: &AppState, doc_id: Uuid) -> ApiResult<Option<Uuid>> {
    Ok(sqlx::query_scalar(
        "SELECT id FROM runs
         WHERE doc_id = $1 AND kind = 'run' AND status = 'ok'
         ORDER BY started_at DESC LIMIT 1",
    )
    .bind(doc_id)
    .fetch_optional(&state.db)
    .await?)
}

#[derive(sqlx::FromRow)]
struct OutputRow {
    path: String,
    language: String,
    content: String,
    provenance: Value,
}

async fn load_output(
    state: &AppState,
    doc_id: Uuid,
    path: &str,
) -> ApiResult<OutputRow> {
    let run_id = last_ok_run(state, doc_id)
        .await?
        .ok_or_else(|| ApiError::not_found("no successful run for this doc yet"))?;
    sqlx::query_as::<_, OutputRow>(
        "SELECT path, language, content, provenance FROM run_outputs
         WHERE run_id = $1 AND path = $2",
    )
    .bind(run_id)
    .bind(path)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("no such output file for the last successful run"))
}

/// GET /api/docs/:id/outputs → `{files: [{path, language}]}`
pub async fn list_outputs(
    State(state): State<AppState>,
    MaybeUser(user): MaybeUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Value>> {
    let doc = load_doc(&state, id).await?;
    check_read(&doc, user.as_ref())?;

    let files: Vec<(String, String)> = match last_ok_run(&state, id).await? {
        None => Vec::new(),
        Some(run_id) => {
            sqlx::query_as(
                "SELECT path, language FROM run_outputs WHERE run_id = $1 ORDER BY path",
            )
            .bind(run_id)
            .fetch_all(&state.db)
            .await?
        }
    };
    let files: Vec<Value> = files
        .into_iter()
        .map(|(path, language)| json!({ "path": path, "language": language }))
        .collect();
    Ok(Json(json!({ "files": files })))
}

#[derive(Deserialize)]
pub struct FileQuery {
    pub path: String,
}

/// GET /api/docs/:id/outputs/file?path=<rel> →
/// `{path, language, content, provenance: Provenance[]}`
pub async fn get_output_file(
    State(state): State<AppState>,
    MaybeUser(user): MaybeUser,
    Path(id): Path<Uuid>,
    Query(q): Query<FileQuery>,
) -> ApiResult<Json<Value>> {
    let doc = load_doc(&state, id).await?;
    check_read(&doc, user.as_ref())?;
    let row = load_output(&state, id, &q.path).await?;
    Ok(Json(json!({
        "path": row.path,
        "language": row.language,
        "content": row.content,
        "provenance": row.provenance,
    })))
}

#[derive(Deserialize)]
pub struct EditRequest {
    pub path: String,
    pub edits: Vec<OutputEdit>,
}

/// POST /api/docs/:id/outputs/edit → `{source_edits, applied: true}`
///
/// Maps output-range edits through the stored provenance to source-document
/// edits, applies them to the doc source (Postgres row + one git commit per
/// batch), and returns them. Edits overlapping `synthetic` ranges → 422 with
/// the offending range.
pub async fn edit_outputs(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<EditRequest>,
) -> ApiResult<Json<Value>> {
    let doc = load_doc(&state, id).await?;
    if doc.owner_id != user.id {
        return Err(ApiError::forbidden(
            "only the project owner can edit generated outputs",
        ));
    }
    if body.edits.is_empty() {
        return Err(ApiError::bad_request("edits must not be empty"));
    }

    let row = load_output(&state, id, &body.path).await?;
    let provenance: Vec<Provenance> = serde_json::from_value(row.provenance)
        .map_err(|e| ApiError::internal(format!("stored provenance unreadable: {e}")))?;

    let source_edits =
        hickory_lineage::map_edits(&row.content, &body.edits, &provenance).map_err(|e| match e {
            LineageError::SyntheticOverlap { start, end } => ApiError::unprocessable(format!(
                "edit overlaps a synthetic (non-editable) output range at bytes {start}..{end}"
            ))
            .with_detail(json!({ "range": { "start": start, "end": end } })),
            LineageError::InvalidEdit(m) => ApiError::bad_request(m),
            LineageError::Conflict(m) => ApiError::unprocessable(m),
        })?;

    // Resolve each referenced source doc within the same project and verify
    // the mapped spans still hold the bytes the run's output was built from —
    // if the doc changed since the last successful run, the provenance is
    // stale and blind application would corrupt the source.
    let mut touched: Vec<String> = source_edits.iter().map(|e| e.doc_path.clone()).collect();
    touched.sort();
    touched.dedup();

    let mut sources: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut doc_rows: std::collections::HashMap<String, DocRow> =
        std::collections::HashMap::new();
    for doc_path in &touched {
        let target = if *doc_path == doc.path {
            doc.clone()
        } else {
            sqlx::query_as::<_, DocRow>(
                "SELECT d.id, d.project_id, d.path, d.source, d.updated_at,
                        p.owner_id, p.visibility
                 FROM docs d JOIN projects p ON p.id = d.project_id
                 WHERE d.project_id = $1 AND d.path = $2",
            )
            .bind(doc.project_id)
            .bind(doc_path)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                ApiError::conflict(format!(
                    "source document {doc_path} referenced by provenance no longer exists"
                ))
            })?
        };
        sources.insert(doc_path.clone(), target.source.clone());
        doc_rows.insert(doc_path.clone(), target);
    }

    // Staleness check: each edit's replaced source bytes must still match the
    // output bytes they produced (byte-precise provenance invariant).
    for edit in &source_edits {
        let src = &sources[&edit.doc_path];
        let (s, e) = edit.span;
        if e > src.len() || !src.is_char_boundary(s) || !src.is_char_boundary(e) {
            return Err(ApiError::conflict(
                "doc changed since the last successful run — re-run before editing outputs",
            ));
        }
    }
    for provenance_entry in &provenance {
        if let Some((doc_path, s, e)) = provenance_entry.origin.source()
            && let Some(src) = sources.get(doc_path)
        {
            let out = &row.content[provenance_entry.start..provenance_entry.end];
            if src.get(s..e) != Some(out) {
                return Err(ApiError::conflict(
                    "doc changed since the last successful run — re-run before editing outputs",
                ));
            }
        }
    }

    let updated = hickory_lineage::apply_source_edits(&sources, &source_edits)
        .map_err(|e| ApiError::unprocessable(e.to_string()))?;

    // Reject edits that break the doc: the next run must reproduce them.
    for (doc_path, new_source) in &updated {
        hick_lang::parse(new_source).map_err(|e| {
            ApiError::unprocessable(format!(
                "edit would make {doc_path} unparseable: {e}"
            ))
        })?;
    }

    // Apply: Postgres row + git commit per doc ("lineage edit via <path>").
    for (doc_path, new_source) in &updated {
        let target = &doc_rows[doc_path];
        sqlx::query("UPDATE docs SET source = $1, updated_at = now() WHERE id = $2")
            .bind(new_source)
            .bind(target.id)
            .execute(&state.db)
            .await?;
        state
            .git
            .save_file(
                doc.project_id,
                doc_path,
                new_source,
                &format!("lineage edit via {}", body.path),
            )
            .await?;
    }

    state.analytics.capture(
        &user.id.to_string(),
        "output_lineage_edit",
        json!({ "doc_id": doc.id, "project_id": doc.project_id, "output_path": body.path }),
    );

    Ok(Json(json!({
        "source_edits": source_edits,
        "applied": true,
    })))
}
