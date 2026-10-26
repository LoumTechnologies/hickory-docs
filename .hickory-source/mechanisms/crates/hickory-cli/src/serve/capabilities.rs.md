<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/capabilities.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-faed2e5819db"></a>
## `crates/hickory-cli/src/serve/capabilities.rs`

<a id="f-faed2e5819db-CapabilityQuery"></a>
### `CapabilityQuery`

Read-only capability discovery from the same plugins used by execution.

<storycode:copy id="f-faed2e5819db-CapabilityQuery">
//! Read-only capability discovery from the same plugins used by execution.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Query, State},
};
use hick_plugins::capabilities::{Activation, CapabilityManager};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Default, Deserialize)]
pub struct CapabilityQuery {
    pub path: Option<String>,
}

</storycode:copy>
<a id="f-faed2e5819db-inspect"></a>
### `inspect`

<storycode:copy id="f-faed2e5819db-inspect">
pub async fn inspect(
    State(state): State<LocalState>,
    Query(query): Query<CapabilityQuery>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let path = query.path.unwrap_or_else(|| ".".into());
    // Detection can walk manifest directories; keep filesystem work off the reactor.
    let capabilities = tokio::task::spawn_blocking(move || {
        CapabilityManager::new(hick_plugins::registry()).inspect(&root, std::path::Path::new(&path))
    })
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?
    .map_err(ApiError::bad_request)?;
    Ok(Json(json!({
        "capabilities": capabilities.iter().map(|c| json!({
            "plugin": c.plugin,
            "language": c.language,
            "kind": format!("{:?}", c.kind),
            "workspace": c.workspace,
            "activation": match c.activation { Activation::Background => "background", Activation::OnRequest => "on-request" },
            "availability": "unchecked",
            "installer_language": c.interface.installer().map(|i| i.language),
        })).collect::<Vec<_>>(),
        "scope": "Declared providers for this file and its enclosing projects; tool readiness is not probed"
    })))
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/capabilities.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/capabilities.rs"><storycode:paste select="#f-faed2e5819db-CapabilityQuery"/><storycode:paste select="#f-faed2e5819db-inspect"/></storycode:file>

</storycode:doc>
