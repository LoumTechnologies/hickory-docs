//! The outputs a document writes: listing them, opening one with its
//! provenance, and carrying an edit in one back to its source.
//!
//! Split out of api.rs as one subject. The thing to know here is naming: a
//! document writes its files relative to ITSELF and the weave keys them that
//! way, while the tree, the tabs and every path a client sends are relative
//! to the open FOLDER. `doc_dir`, `output_key` and `output_path` are the one
//! translation, at this boundary — see
//! docs/guarantees/authoring/an-output-is-named-from-the-open-folder.md.

use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult, language_of};

/// The directory a document lives in, relative to the open folder, with a
/// trailing slash — or empty for a document at the root.
fn doc_dir(state: &LocalState, id: &str) -> String {
    let path = state.index.path_of(id).unwrap_or_default();
    match path.rfind('/') {
        Some(i) => path[..=i].to_string(),
        None => String::new(),
    }
}

/// The key an output is stored under, from the path a client names it by.
///
/// A document writes its files relative to ITSELF (`<hick:file path="app.py">`
/// in `tools/app.md` is `app.py`), and that is how the weave keys them. The
/// tree, the tabs and every path a client sends are relative to the open
/// FOLDER (`tools/app.py`). The two agree only for a document at the root,
/// which is why the generated-file pane worked there and answered "this
/// document produces no output named tools/app.py — it produces: app.py"
/// anywhere else. Both spellings are accepted; the folder-relative one is
/// what every answer publishes.
fn output_key(dir: &str, path: &str) -> String {
    match path.strip_prefix(dir) {
        Some(rest) if !dir.is_empty() => rest.to_string(),
        _ => path.to_string(),
    }
}

/// The folder-relative path of an output stored under `key`.
fn output_path(dir: &str, key: &str) -> String {
    format!("{dir}{key}")
}

/// `GET /api/docs/:id/outputs`
pub async fn list_outputs(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let run = state.weave(&id).await?;
    let dir = doc_dir(&state, &id);
    let mut files: Vec<Value> = run
        .result
        .files
        .iter()
        .filter(|(_, content)| content.as_text().is_some())
        .map(|(key, _)| {
            let path = output_path(&dir, key);
            json!({ "language": language_of(&path), "path": path })
        })
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
    let dir = doc_dir(&state, &id);
    let key = output_key(&dir, &q.path);
    let content = run
        .result
        .files
        .get(&key)
        .and_then(|c| c.as_text())
        .ok_or_else(|| {
            let mut available: Vec<String> = run
                .result
                .files
                .keys()
                .map(|k| output_path(&dir, k))
                .collect();
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
    let provenance = crate::output_lineage(&run, &key)?;
    let path = output_path(&dir, &key);
    Ok(Json(json!({
        "language": language_of(&path),
        "path": path,
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
    let key = output_key(&doc_dir(&state, &id), &body.path);
    let content = run
        .result
        .files
        .get(&key)
        .and_then(|c| c.as_text())
        .ok_or_else(|| ApiError::not_found(format!("no output named {:?}", body.path)))?
        .to_string();
    let provenance = crate::output_lineage(&run, &key)?;

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
