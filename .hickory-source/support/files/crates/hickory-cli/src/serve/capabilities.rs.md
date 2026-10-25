<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/capabilities.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `fbfe78e64a47ca0827e9db9d6563f2be14ded790d9095f07a263ef91974e67b7`.

<book:entry path="crates/hickory-cli/src/serve/capabilities.rs">
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
</book:entry>

</book:doc>
