<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/native_projects.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-846b2d315341"></a>
## `crates/hickory-cli/src/serve/native_projects.rs`

<a id="f-846b2d315341-Register"></a>
### `Register`

Tool contexts are immutable memory values. Creating one starts no tool.

<storycode:copy id="f-846b2d315341-Register">
//! Tool contexts are immutable memory values. Creating one starts no tool.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use hick_lang::ProjectSnapshot;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct Register {
    pub snapshot: ProjectSnapshot,
}
</storycode:copy>
<a id="f-846b2d315341-register"></a>
### `register`

<storycode:copy id="f-846b2d315341-register">
pub async fn register(
    State(state): State<LocalState>,
    Json(body): Json<Register>,
) -> ApiResult<Json<Value>> {
    let tree = body.snapshot.restore().map_err(ApiError::unprocessable)?;
    let id = format!("{:016x}", super::rand_id());
    let uri = state
        .lsp
        .projects()
        .register(&id, tree)
        .map_err(ApiError::unprocessable)?;
    Ok(Json(
        json!({"id":id,"uri":uri,"materialized":false,"workspace":state.index.root()}),
    ))
}
</storycode:copy>
<a id="f-846b2d315341-File"></a>
### `File`

<storycode:copy id="f-846b2d315341-File">
#[derive(Deserialize)]
pub struct File {
    pub uri: String,
}
</storycode:copy>
<a id="f-846b2d315341-file"></a>
### `file`

<storycode:copy id="f-846b2d315341-file">
pub async fn file(
    State(state): State<LocalState>,
    Query(body): Query<File>,
) -> ApiResult<Json<Value>> {
    let uri = tower_lsp::lsp_types::Url::parse(&body.uri)
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    let (_, path, context) = state
        .lsp
        .projects()
        .get(&uri)
        .map_err(ApiError::not_found)?;
    let entry = context
        .tree
        .get(&path)
        .ok_or_else(|| ApiError::not_found("Code is absent from this captured project."))?;
    let files: Vec<_> = context.tree.entries().iter().map(|(path,entry)| json!({"path":path,"bytes":entry.bytes.as_ref(),"kind":if entry.kind == hick_lang::EntryKind::File {"file"} else {"symlink"},"executable":entry.executable,"origins":entry.origins})).collect();
    Ok(Json(
        json!({"path":path,"bytes":entry.bytes.as_ref(),"files":files}),
    ))
}
</storycode:copy>
<a id="f-846b2d315341-remove"></a>
### `remove`

<storycode:copy id="f-846b2d315341-remove">
pub async fn remove(State(state): State<LocalState>, Path(id): Path<String>) -> Json<Value> {
    state.lsp.projects().remove(&id);
    Json(json!({"removed":true}))
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/native_projects.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/native_projects.rs"><storycode:paste select="#f-846b2d315341-Register"/><storycode:paste select="#f-846b2d315341-register"/><storycode:paste select="#f-846b2d315341-File"/><storycode:paste select="#f-846b2d315341-file"/><storycode:paste select="#f-846b2d315341-remove"/></storycode:file>

</storycode:doc>
