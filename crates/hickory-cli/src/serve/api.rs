//! The REST subset the document view needs, answered from files.
//!
//! Every handler here has a hosted counterpart in `apps/server/src/routes/`.
//! They agree on the wire shape — the same React client talks to both, and
//! `docs/specs/freeform/api.md` is the contract — but not on where the answer
//! comes from: there is no database, no run history, and no account. A weave
//! is computed from the file in front of us, which makes the local answers
//! *fresher* than the hosted ones (which serve the last successful run).
//!
//! What is deliberately absent: billing, analytics, and the agent. A local
//! session answers those with a static "not here" rather than 404, because the
//! client asks for plans on load and a 404 would look like a broken deploy.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{LocalState, RunRecord};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// The error body shape the web client parses (`{"error": "…"}`).
pub struct ApiError(StatusCode, String, Option<Value>);

impl ApiError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self(StatusCode::NOT_FOUND, msg.into(), None)
    }
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self(StatusCode::BAD_REQUEST, msg.into(), None)
    }
    pub fn unprocessable(msg: impl Into<String>) -> Self {
        Self(StatusCode::UNPROCESSABLE_ENTITY, msg.into(), None)
    }
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self(StatusCode::FORBIDDEN, msg.into(), None)
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, msg.into(), None)
    }
    pub fn with_detail(mut self, detail: Value) -> Self {
        self.2 = Some(detail);
        self
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut body = json!({ "error": self.1 });
        if let Some(Value::Object(extra)) = self.2
            && let Some(obj) = body.as_object_mut()
        {
            obj.extend(extra);
        }
        (self.0, Json(body)).into_response()
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        ApiError::internal(format!("{e:#}"))
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

// ---------------------------------------------------------------------------
// Identity and project shape
// ---------------------------------------------------------------------------

/// `GET /api/projects` — the served directory, as one project.
pub async fn projects(State(state): State<LocalState>) -> Json<Value> {
    Json(json!([project_of(&state)]))
}

fn project_of(state: &LocalState) -> Value {
    let name = state
        .index
        .root()
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "hick".to_string());
    json!({
        "id": "local",
        "name": name,
        "visibility": "private",
        "created_at": "1970-01-01T00:00:00Z",
    })
}

/// `GET /api/projects/:id/docs` — every `.hick` file under the root.
pub async fn project_docs(State(state): State<LocalState>) -> Json<Value> {
    let docs: Vec<Value> = state
        .index
        .entries()
        .into_iter()
        .map(|(id, path)| {
            json!({
                "id": id,
                "path": path,
                "updated_at": modified_at(&state, &id),
            })
        })
        .collect();
    Json(json!(docs))
}

fn modified_at(state: &LocalState, id: &str) -> String {
    let stamp = state
        .index
        .absolute(id)
        .and_then(|p| std::fs::metadata(p).ok())
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // RFC 3339 without pulling in a date library: the client only displays it.
    let days = stamp / 86_400;
    let secs = stamp % 86_400;
    let (y, m, d) = civil_from_days(days as i64);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

/// Days since the Unix epoch → (year, month, day). Howard Hinnant's
/// `civil_from_days`, which is exact and needs no table.
pub fn civil_from_days_public(z: i64) -> (i64, u32, u32) {
    civil_from_days(z)
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

// ---------------------------------------------------------------------------
// Documents
// ---------------------------------------------------------------------------

fn doc_json(state: &LocalState, id: &str) -> ApiResult<Value> {
    let path = state
        .index
        .path_of(id)
        .ok_or_else(|| ApiError::not_found(format!("no document {id} under the served directory")))?
        .to_string();
    let source = state.read_source(id)?;
    Ok(json!({
        "id": id,
        "path": path,
        "source": source,
        "updated_at": modified_at(state, id),
    }))
}

/// `GET /api/docs/:id`
pub async fn get_doc(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    Ok(Json(doc_json(&state, &id)?))
}

#[derive(Deserialize)]
pub struct SaveDoc {
    pub source: String,
}

/// `PUT /api/docs/:id` — write the file, and tell the live room.
pub async fn put_doc(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<SaveDoc>,
) -> ApiResult<Json<Value>> {
    state.write_source(&id, &body.source)?;
    // A room holding the pre-save text would write it straight back over this
    // on its next debounce.
    state.rooms.apply_external_source(&id, &body.source).await;
    Ok(Json(doc_json(&state, &id)?))
}

/// `GET /api/docs/:id/render` — the block model for the current source.
///
/// Weave, never execute: the client asks for this on every load and after
/// every save, and a render that ran the document would turn scrolling into
/// arbitrary code execution.
pub async fn render_doc(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let run = state.weave(&id).await?;
    Ok(Json(crate::block_model_json(&run).map_err(|e| {
        ApiError::unprocessable(format!("render failed: {e}"))
    })?))
}

// ---------------------------------------------------------------------------
// Outputs — the lineage ribbons
// ---------------------------------------------------------------------------

/// `GET /api/docs/:id/outputs`
pub async fn list_outputs(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let run = state.weave(&id).await?;
    let mut files: Vec<Value> = run
        .result
        .files
        .iter()
        .filter(|(_, content)| content.as_text().is_some())
        .map(|(path, _)| json!({ "path": path, "language": language_of(path) }))
        .collect();
    files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    Ok(Json(json!({ "files": files })))
}

#[derive(Deserialize)]
pub struct FileQuery {
    pub path: String,
}

/// `GET /api/docs/:id/outputs/file?path=…` — content plus byte-precise
/// provenance. This is what the ribbon layer draws.
pub async fn get_output_file(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Query(q): Query<FileQuery>,
) -> ApiResult<Json<Value>> {
    let run = state.weave(&id).await?;
    let content = run
        .result
        .files
        .get(&q.path)
        .and_then(|c| c.as_text())
        .ok_or_else(|| {
            let mut available: Vec<&str> = run.result.files.keys().map(String::as_str).collect();
            available.sort();
            ApiError::not_found(format!(
                "this document produces no output named {:?} — it produces: {}",
                q.path,
                if available.is_empty() {
                    "(nothing)".to_string()
                } else {
                    available.join(", ")
                }
            ))
        })?
        .to_string();
    let provenance = crate::output_lineage(&run, &q.path)?;
    Ok(Json(json!({
        "path": q.path,
        "language": language_of(&q.path),
        "content": content,
        "provenance": provenance,
    })))
}

#[derive(Deserialize)]
pub struct EditRequest {
    pub path: String,
    pub edits: Vec<hickory_lineage::OutputEdit>,
}

/// `POST /api/docs/:id/outputs/edit` — edit the generated file; the change is
/// mapped back into the documents that produced it.
///
/// Simpler than the hosted counterpart in one way that matters: the weave was
/// computed from the files in this same request, so the "doc changed since the
/// run this output came from" conflict cannot arise. There is no stale
/// provenance to guard against, because there is no stored run.
pub async fn edit_outputs(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<EditRequest>,
) -> ApiResult<Json<Value>> {
    let run = state.weave(&id).await?;
    let content = run
        .result
        .files
        .get(&body.path)
        .and_then(|c| c.as_text())
        .ok_or_else(|| ApiError::not_found(format!("no output named {:?}", body.path)))?
        .to_string();
    let provenance = crate::output_lineage(&run, &body.path)?;

    let source_edits =
        hickory_lineage::map_edits(&content, &body.edits, &provenance).map_err(|e| match e {
            hickory_lineage::LineageError::SyntheticOverlap { start, end } => {
                ApiError::unprocessable(format!(
                    "edit overlaps a synthetic (non-editable) output range at bytes {start}..{end}"
                ))
                .with_detail(json!({ "range": { "start": start, "end": end } }))
            }
            hickory_lineage::LineageError::InvalidEdit(m) => ApiError::bad_request(m),
            hickory_lineage::LineageError::Conflict(m) => ApiError::unprocessable(m),
        })?;

    // Provenance names documents by the path the weave knew them as; read each
    // one from disk.
    let mut sources: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for edit in &source_edits {
        if sources.contains_key(&edit.doc_path) {
            continue;
        }
        let text = state.read_source_by_doc_path(&edit.doc_path)?;
        sources.insert(edit.doc_path.clone(), text);
    }

    let updated = hickory_lineage::apply_source_edits(&sources, &source_edits)
        .map_err(|e| ApiError::unprocessable(e.to_string()))?;

    // Reject an edit that breaks the document: the next run must reproduce it.
    for (doc_path, new_source) in &updated {
        hick_lang::parse(new_source).map_err(|e| {
            ApiError::unprocessable(format!("edit would make {doc_path} unparseable: {e}"))
        })?;
    }

    for (doc_path, new_source) in &updated {
        let target_id = state.id_for_doc_path(doc_path);
        state.write_source_by_doc_path(doc_path, new_source)?;
        state
            .rooms
            .apply_external_source(&target_id, new_source)
            .await;
    }

    Ok(Json(
        json!({ "source_edits": source_edits, "applied": true }),
    ))
}

/// Language tag for an output path, matching the hosted server's mapping.
fn language_of(path: &str) -> String {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "rs" => "rust",
        "py" => "python",
        "ts" => "typescript",
        "tsx" => "typescript",
        "js" | "mjs" | "cjs" => "javascript",
        "jsx" => "javascript",
        "go" => "go",
        "sh" | "bash" => "shell",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" | "markdown" => "markdown",
        "html" => "html",
        "css" => "css",
        "sql" => "sql",
        other => other,
    }
    .to_string()
}

// ---------------------------------------------------------------------------
// Running
// ---------------------------------------------------------------------------

#[derive(Deserialize, Default)]
pub struct RunRequest {
    #[serde(default)]
    pub cells: Option<Vec<String>>,
}

/// `POST /api/docs/:id/run` — execute for real, streaming events on the run
/// channel.
pub async fn run_doc(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let run_id = state.start_run(&id, false).await?;
    Ok((StatusCode::ACCEPTED, Json(json!({ "run_id": run_id }))))
}

/// `POST /api/docs/:id/check` — verify without writing outputs.
pub async fn check_doc(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let run_id = state.start_run(&id, true).await?;
    Ok((StatusCode::ACCEPTED, Json(json!({ "run_id": run_id }))))
}

/// `GET /api/runs/:id`
pub async fn get_run(
    State(state): State<LocalState>,
    Path(run_id): Path<String>,
) -> ApiResult<Json<Value>> {
    let record: RunRecord = state
        .runs
        .lock()
        .unwrap()
        .get(&run_id)
        .cloned()
        .ok_or_else(|| ApiError::not_found("no such run in this session"))?;
    Ok(Json(json!({
        "id": run_id,
        "status": record.status,
        "started_at": record.started_at,
        "blocks": record.blocks,
    })))
}

// ---------------------------------------------------------------------------
// Static answers
// ---------------------------------------------------------------------------

/// `GET /api/executor` — which backend this session runs on. The web app shows
/// it, and in a shared session it is the difference between "sandboxed" and
/// "as the host".
pub async fn executor(State(state): State<LocalState>) -> Json<Value> {
    Json(json!({ "kind": state.executor_kind(), "images": Value::Null }))
}

/// `GET /api/health`
pub async fn health(State(state): State<LocalState>) -> Json<Value> {
    Json(json!({ "ok": true, "executor": state.executor_kind(), "db": false }))
}

#[derive(Serialize)]
pub struct Unavailable {
    pub error: String,
}

/// The agent endpoints, absent locally.
pub async fn agent_unavailable() -> (StatusCode, Json<Unavailable>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(Unavailable {
            error: "the hosted agent is not part of a local session — run `hick agent \"…\"` \
                    in this directory instead, or open the document on hickorydocs.com"
                .to_string(),
        }),
    )
}

/// `GET /api/docs/:id/agent/turns` — no conversation exists locally, and an
/// empty list is the truthful answer (the panel renders empty rather than
/// erroring).
pub async fn agent_turns() -> Json<Value> {
    Json(json!({ "turns": [] }))
}

/// Arc-friendly alias used by the router module.
pub type Shared = Arc<LocalState>;
