//! Generated outputs & lineage (api.md v0.2):
//! - `GET  /api/docs/:id/outputs`             — files from the last successful run
//! - `GET  /api/docs/:id/outputs/file?path=…` — content + `Provenance[]`
//! - `POST /api/docs/:id/outputs/edit`        — map output edits to source edits,
//!   apply them to the doc source (Postgres row + git commit), return them.
//! - `POST /api/docs/:id/outputs/nav`         — LSP bridge v0.3: map an output
//!   byte offset through provenance into source coordinates, ask a short-lived
//!   hick-lsp session for definition/references, map results back.

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

async fn load_output(state: &AppState, doc_id: Uuid, path: &str) -> ApiResult<OutputRow> {
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
            sqlx::query_as("SELECT path, language FROM run_outputs WHERE run_id = $1 ORDER BY path")
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

    let source_edits = hickory_lineage::map_edits(&row.content, &body.edits, &provenance).map_err(
        |e| match e {
            LineageError::SyntheticOverlap { start, end } => ApiError::unprocessable(format!(
                "edit overlaps a synthetic (non-editable) output range at bytes {start}..{end}"
            ))
            .with_detail(json!({ "range": { "start": start, "end": end } })),
            LineageError::InvalidEdit(m) => ApiError::bad_request(m),
            LineageError::Conflict(m) => ApiError::unprocessable(m),
        },
    )?;

    // Resolve each referenced source doc within the same project and verify
    // the mapped spans still hold the bytes the run's output was built from —
    // if the doc changed since the last successful run, the provenance is
    // stale and blind application would corrupt the source.
    let mut touched: Vec<String> = source_edits.iter().map(|e| e.doc_path.clone()).collect();
    touched.sort();
    touched.dedup();

    let mut sources: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut doc_rows: std::collections::HashMap<String, DocRow> = std::collections::HashMap::new();
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
            ApiError::unprocessable(format!("edit would make {doc_path} unparseable: {e}"))
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

// ---------------------------------------------------------------------------
// POST /api/docs/:id/outputs/nav — LSP bridge v0.3
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct NavRequest {
    pub path: String,
    pub offset: usize,
    pub kind: String,
}

/// `{path, offset, kind: "definition"|"references"}` →
/// `{targets: [{uri, range: {start, end}}]}` (byte offsets).
///
/// The output byte offset is mapped through the last run's provenance into
/// source-document coordinates, a short-lived hick-lsp session answers the
/// request over the seeded checkout, and results are mapped back: locations
/// in `.hick` docs become `hick:///<doc-path>` byte ranges; locations that
/// stay inside generated outputs are re-mapped through provenance where
/// possible, else returned as `hick-output:///<output-path>` byte ranges.
/// Read access follows doc visibility, like every other doc read.
pub async fn outputs_nav(
    State(state): State<AppState>,
    MaybeUser(user): MaybeUser,
    Path(id): Path<Uuid>,
    Json(body): Json<NavRequest>,
) -> ApiResult<Json<Value>> {
    use hick_lsp::structural::{byte_to_position, position_to_byte};

    let method = match body.kind.as_str() {
        "definition" => "textDocument/definition",
        "references" => "textDocument/references",
        other => {
            return Err(ApiError::bad_request(format!(
                "kind must be \"definition\" or \"references\", got {other:?}"
            )));
        }
    };

    let doc = load_doc(&state, id).await?;
    check_read(&doc, user.as_ref())?;
    let row = load_output(&state, id, &body.path).await?;
    if body.offset > row.content.len() {
        return Err(ApiError::bad_request("offset out of bounds for output"));
    }
    let provenance: Vec<Provenance> = serde_json::from_value(row.provenance)
        .map_err(|e| ApiError::internal(format!("stored provenance unreadable: {e}")))?;

    // Output byte → source-document byte through provenance. Synthetic bytes
    // have no source position: no targets.
    let Some((src_doc_path, src_offset)) = provenance
        .iter()
        .find(|p| p.start <= body.offset && body.offset < p.end)
        .and_then(|p| {
            let (doc_path, s, _) = p.origin.location()?;
            Some((doc_path.to_string(), s + (body.offset - p.start)))
        })
    else {
        return Ok(Json(json!({ "targets": [] })));
    };

    // All project doc sources: the request doc, the provenance target doc,
    // and any doc a result location may land in.
    let doc_sources: std::collections::HashMap<String, String> =
        sqlx::query_as::<_, (String, String)>(
            "SELECT path, source FROM docs WHERE project_id = $1",
        )
        .bind(doc.project_id)
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .collect();
    let src_source = doc_sources
        .get(&src_doc_path)
        .ok_or_else(|| ApiError::conflict("provenance references a doc that no longer exists"))?;
    let (line, character) = byte_to_position(src_source, src_offset);

    // Short-lived hick-lsp session over the seeded checkout.
    let (mut session, mut rx) = crate::lsp::LspSession::start(&state.git, doc.project_id)
        .await
        .map_err(|e| ApiError::internal(format!("could not start hick-lsp session: {e:#}")))?;
    let workdir_uri = session.workdir_uri().to_string();
    let doc_uri = format!("{workdir_uri}{src_doc_path}");

    session
        .send(&json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": doc_uri,
                "languageId": "hick",
                "version": 1,
                "text": src_source,
            }},
        }))
        .await
        .map_err(|e| ApiError::internal(format!("hick-lsp didOpen failed: {e:#}")))?;

    // Ask (with a few retries: didOpen is a notification, its processing can
    // land after the first request), collecting the matching response.
    let mut result = Value::Null;
    'attempts: for attempt in 0..20u32 {
        let req_id = format!("nav:{attempt}");
        let mut params = json!({
            "textDocument": { "uri": format!("{workdir_uri}{src_doc_path}") },
            "position": { "line": line, "character": character },
        });
        if method == "textDocument/references"
            && let Some(obj) = params.as_object_mut()
        {
            obj.insert("context".into(), json!({ "includeDeclaration": false }));
        }
        session
            .send(&json!({ "jsonrpc": "2.0", "id": req_id, "method": method, "params": params }))
            .await
            .map_err(|e| ApiError::internal(format!("hick-lsp request failed: {e:#}")))?;

        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let msg = match tokio::time::timeout_at(deadline, rx.recv()).await {
                Ok(Some(msg)) => msg,
                _ => break 'attempts,
            };
            if msg.get("id").and_then(|i| i.as_str()) == Some(req_id.as_str()) {
                result = msg.get("result").cloned().unwrap_or(Value::Null);
                break;
            }
        }
        let empty = result.is_null() || result.as_array().is_some_and(|a| a.is_empty());
        if !empty {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    session.shutdown().await;

    // Normalize Location | Location[] | LocationLink[] into targets.
    let locations: Vec<Value> = match result {
        Value::Array(items) => items,
        Value::Null => Vec::new(),
        one => vec![one],
    };
    let mut targets = Vec::new();
    for loc in &locations {
        let (uri, range) = match (loc.get("uri"), loc.get("targetUri")) {
            (Some(Value::String(u)), _) => (u.clone(), loc.get("range")),
            (_, Some(Value::String(u))) => (u.clone(), loc.get("targetRange")),
            _ => continue,
        };
        let Some(range) = range else { continue };
        let range_bytes = |content: &str| -> Option<(usize, usize)> {
            let sl = range.pointer("/start/line")?.as_u64()? as u32;
            let sc = range.pointer("/start/character")?.as_u64()? as u32;
            let el = range.pointer("/end/line")?.as_u64()? as u32;
            let ec = range.pointer("/end/character")?.as_u64()? as u32;
            Some((
                position_to_byte(content, sl, sc)?,
                position_to_byte(content, el, ec)?,
            ))
        };

        if let Some(doc_path) = uri.strip_prefix(&workdir_uri) {
            // A .hick source location.
            if let Some(source) = doc_sources.get(doc_path)
                && let Some((s, e)) = range_bytes(source)
            {
                targets.push(json!({
                    "uri": format!("hick:///{doc_path}"),
                    "range": { "start": s, "end": e },
                }));
            }
        } else if let Some(out_path) = crate::lsp::vfile_output_path(&uri) {
            // A generated-output location: provenance-map when possible.
            let out_row: Option<(String, Value)> = sqlx::query_as(
                "SELECT o.content, o.provenance FROM run_outputs o
                 JOIN runs r ON r.id = o.run_id
                 WHERE r.doc_id = $1 AND r.kind = 'run' AND r.status = 'ok' AND o.path = $2
                 ORDER BY r.started_at DESC LIMIT 1",
            )
            .bind(id)
            .bind(out_path)
            .fetch_optional(&state.db)
            .await?;
            let Some((content, prov)) = out_row else {
                continue;
            };
            let Some((s, e)) = range_bytes(&content) else {
                continue;
            };
            let prov: Vec<Provenance> = serde_json::from_value(prov).unwrap_or_default();
            let map_byte = |b: usize| -> Option<(String, usize)> {
                let p = prov
                    .iter()
                    .find(|p| p.start <= b && b < p.end)
                    .or_else(|| prov.iter().find(|p| p.end == b && p.start < p.end))?;
                let (d, ps, _) = p.origin.location()?;
                Some((d.to_string(), ps + (b - p.start)))
            };
            match (map_byte(s), map_byte(e)) {
                (Some((d1, ms)), Some((d2, me))) if d1 == d2 && ms <= me => {
                    targets.push(json!({
                        "uri": format!("hick:///{d1}"),
                        "range": { "start": ms, "end": me },
                    }));
                }
                _ => {
                    targets.push(json!({
                        "uri": format!("hick-output:///{out_path}"),
                        "range": { "start": s, "end": e },
                    }));
                }
            }
        }
    }

    Ok(Json(json!({ "targets": targets })))
}
