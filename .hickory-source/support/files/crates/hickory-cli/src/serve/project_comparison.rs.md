<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/project_comparison.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `f549e0371f2e472b8e74fdce4c0cea7a45fe0006b9bb7f56738bcb1bb19a7d27`.

<book:entry path="crates/hickory-cli/src/serve/project_comparison.rs">
//! Captured Markdown inputs for the portable comparison editor. No execution.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct CaptureRequest {
    pub source: Option<String>,
}

pub async fn capture(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<CaptureRequest>,
) -> ApiResult<Json<Value>> {
    let document = state
        .index
        .absolute(&id)
        .ok_or_else(|| ApiError::not_found("No document for this comparison."))?;
    let root = document
        .parent()
        .ok_or_else(|| ApiError::bad_request("Project needs a directory."))?
        .canonicalize()
        .map_err(|e| ApiError::unprocessable(e.to_string()))?;
    let entry = document
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| ApiError::unprocessable("Project needs a UTF-8 filename."))?;
    let source = body.source.unwrap_or(state.read_source(&id)?);
    let mut live = BTreeMap::new();
    for (key, _) in state.index.entries() {
        if let Some(room) = state.rooms.get(&key).await
            && let Some(full) = state.index.absolute(&key)
            && let Ok(relative) = full.strip_prefix(&root)
        {
            live.insert(
                relative.to_string_lossy().replace('\\', "/"),
                room.text().await,
            );
        }
    }
    let sources = hick_lang::load_project_sources(entry, &|name| {
        if name == entry {
            return Ok(source.clone());
        }
        if let Some(source) = live.get(name) {
            return Ok(source.clone());
        }
        let full = root.join(name);
        if !std::fs::symlink_metadata(&full)
            .map_err(|e| e.to_string())?
            .is_file()
        {
            return Err(format!("Markdown source is not a regular file: {name}"));
        }
        let full = full.canonicalize().map_err(|e| e.to_string())?;
        if !full.starts_with(&root) {
            return Err(format!("Markdown input escapes root: {name}"));
        }
        let relative = full
            .strip_prefix(
                state
                    .index
                    .root()
                    .canonicalize()
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?
            .to_str()
            .ok_or("Non-UTF-8 Markdown path")?;
        state
            .read_source(&state.index.id_for_path(relative))
            .map_err(|e| e.detail().to_owned())
    })
    .map_err(ApiError::unprocessable)?;
    // Refuse malformed or incomplete contexts before presenting them as a project.
    hick_lang::evaluate_project(entry, &sources, &hick_lang::ProjectTree::default())
        .map_err(ApiError::unprocessable)?;
    Ok(Json(json!({"entry":entry,"sources":sources})))
}

#[derive(Deserialize)]
pub struct ApplyRequest {
    pub baseline: BTreeMap<String, String>,
    pub documents: BTreeMap<String, String>,
    pub expected: hick_lang::ProjectSnapshot,
}
/// A concrete source edit is reviewed in the comparison UI, then guarded here.
pub async fn apply(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<ApplyRequest>,
) -> ApiResult<Json<Value>> {
    let document = state
        .index
        .absolute(&id)
        .ok_or_else(|| ApiError::not_found("No project destination."))?;
    let root = document
        .parent()
        .ok_or_else(|| ApiError::bad_request("Project needs a root."))?
        .canonicalize()
        .map_err(|e| ApiError::unprocessable(e.to_string()))?;
    let entry = document
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| ApiError::bad_request("Non-UTF-8 entry"))?;
    let expected = body.expected.restore().map_err(ApiError::unprocessable)?;
    hick_lang::IncorporationCandidate::verify(
        entry,
        body.baseline.clone(),
        body.documents.clone(),
        expected,
    )
    .map_err(|e| ApiError::unprocessable(format!("Invalid source comparison: {e:?}")))?;
    crate::engine::writes::during(&state.writes, async {
        let applied = super::project_publish::publish(&state, &root, &body.baseline, &body.documents)
            .await
            .map_err(ApiError::from)?;
        let undo: Vec<_> = applied.iter().map(|path| json!({"path":path,"before":body.baseline.get(path),"after":body.documents.get(path)})).collect();
        Ok(Json(json!({"destinations":applied,"undo":undo})))
    })
    .await
}
#[derive(Deserialize)]
pub struct UndoRequest {
    pub undo: Vec<UndoSource>,
}
#[derive(Deserialize)]
pub struct UndoSource {
    pub path: String,
    pub before: Option<String>,
    pub after: Option<String>,
}
pub async fn undo(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<UndoRequest>,
) -> ApiResult<Json<Value>> {
    let document = state
        .index
        .absolute(&id)
        .ok_or_else(|| ApiError::not_found("No project destination."))?;
    let root = document
        .parent()
        .ok_or_else(|| ApiError::bad_request("Project needs a root."))?
        .canonicalize()
        .map_err(|e| ApiError::unprocessable(e.to_string()))?;
    crate::engine::writes::during(&state.writes, async {
        let mut statuses = vec![];
        for item in body.undo {
            let baseline = item
                .after
                .map(|text| BTreeMap::from([(item.path.clone(), text)]))
                .unwrap_or_default();
            let candidate = item
                .before
                .map(|text| BTreeMap::from([(item.path.clone(), text)]))
                .unwrap_or_default();
            let result =
                super::project_publish::publish(&state, &root, &baseline, &candidate).await;
            statuses.push(match result {
                Ok(_) => json!({"path":item.path,"ok":true}),
                Err(e) => json!({"path":item.path,"ok":false,"error":format!("{e:#}")}),
            });
        }
        Ok(Json(json!({"targets":statuses})))
    })
    .await
}
</book:entry>

</book:doc>
