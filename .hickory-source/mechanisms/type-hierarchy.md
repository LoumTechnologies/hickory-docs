<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Query type relationships through retained provider artifacts

For maintainers extending navigation without coupling provider state to a renderer.

## Hold provider context independently of views

A type item is an opaque child value plus its actual provider connection. The shared bounded, expiring artifact model binds its token to installed source and ownership revisions; displaying an item grants no write authority.

<ui:copy id="type-hierarchy-retained-context">
use super::*;
use crate::child_lsp::ChildLspHandle;
use crate::provider_artifacts::ProviderArtifacts;
use serde_json::{Value, json};
use std::time::{Duration, Instant};
#[derive(Clone)]
struct HeldTypeItem {
    original: Value,
    handle: ChildLspHandle,
}
pub(super) struct TypeHierarchyArtifacts {
    items: ProviderArtifacts<HeldTypeItem>,
}
impl Default for TypeHierarchyArtifacts {
    fn default() -> Self {
        Self {
            items: ProviderArtifacts::new(1024, Duration::from_secs(120)),
        }
    }
}
</ui:copy>

## Project the type name through its own source

Validate required navigation fields while retaining opaque child fields, including standard tag arrays. Use the item's own file mapping, not the requesting file. If a declaration crosses authored owners, display only the exact type-name range; the child's complete original scope remains retained for follow-up queries.

<ui:copy id="type-hierarchy-source-projection">
fn map_type_item(original: &Value, index: &HashMap<String, (Url, PositionMap)>) -> Result<Value> {
    let invalid = || tower_lsp::jsonrpc::Error::invalid_params("Malformed provider type item");
    if original.get("name").and_then(Value::as_str).is_none()
        || !original
            .get("kind")
            .and_then(Value::as_u64)
            .is_some_and(|kind| (1..=26).contains(&kind))
    {
        return Err(invalid());
    }
    let uri = original
        .get("uri")
        .and_then(Value::as_str)
        .and_then(|uri| Url::parse(uri).ok())
        .ok_or_else(invalid)?;
    let range: Range = serde_json::from_value(original.get("range").cloned().ok_or_else(invalid)?)
        .map_err(|_| invalid())?;
    let selection: Range = serde_json::from_value(
        original
            .get("selectionRange")
            .cloned()
            .ok_or_else(invalid)?,
    )
    .map_err(|_| invalid())?;
    if range.start > selection.start || selection.start > selection.end || selection.end > range.end
    {
        return Err(invalid());
    }
    let Some((primary, map)) = index.get(&uri_key(uri.as_str())) else {
        return Ok(original.clone());
    };
    let (owner, selection) = map.source_edit_range(selection).ok_or_else(|| {
        tower_lsp::jsonrpc::Error::invalid_params("Type name has no exact authored source position")
    })?;
    let mut uri = match &owner {
        Some(path) => primary.join(path).map_err(|_| {
            tower_lsp::jsonrpc::Error::invalid_params("Invalid type source identity")
        })?,
        None => primary.clone(),
    };
    uri.set_query(primary.query());
    // A generated declaration can span owners. Only the exact name occurrence
    // is then displayed as a range; the untouched child scope remains held.
    let range = map
        .source_edit_range(range)
        .filter(|(path, _)| path == &owner)
        .map(|(_, range)| range)
        .unwrap_or(selection);
    let mut mapped = original.clone();
    mapped["uri"] = json!(uri);
    mapped["selectionRange"] = json!(selection);
    mapped["range"] = json!(range);
    Ok(mapped)
}
</ui:copy>

## Retain only a source-current answer

Project the whole response before publishing it. Return no more than 1,000 items per provider response and retain up to 1,024 items for 120 seconds. Source/ownership changes before or after asynchronous work refuse the answer. Expiry asks for a new preparation instead of guessing a provider.

<ui:copy id="type-hierarchy-retain-results">
impl HickBackend {
    async fn remember_type_items(
        &self,
        result: Value,
        handle: ChildLspHandle,
        revision: &str,
    ) -> Result<Value> {
        if result.is_null() {
            return Ok(Value::Null);
        }
        let originals = result.as_array().ok_or_else(|| {
            tower_lsp::jsonrpc::Error::invalid_params("Malformed provider type hierarchy")
        })?;
        if originals.len() > 1000 {
            return Err(tower_lsp::jsonrpc::Error::invalid_params(
                "More than 1,000 type items. Narrow the query.",
            ));
        }
        let index = self.index_snapshot().await;
        self.require_provider_revision(revision).await?;
        let mapped = originals
            .iter()
            .map(|item| map_type_item(item, &index))
            .collect::<Result<Vec<_>>>()?;
        let mut held = self.prepared_types.lock().await;
        let mut answer = Vec::with_capacity(mapped.len());
        for (original, mut mapped) in originals.iter().zip(mapped) {
            let token = held.items.insert(
                Instant::now(),
                revision.into(),
                HeldTypeItem {
                    original: original.clone(),
                    handle: handle.clone(),
                },
            );
            mapped["data"] = json!({"hickory_type":token});
            answer.push(mapped);
        }
        drop(held);
        self.require_provider_revision(revision).await?;
        Ok(json!(answer))
    }
</ui:copy>

## Prepare and follow the provider protocol

Prepare at a mapped source position, then return only a backend token as client data. Follow-ups use the held original item and child connection; caller modifications to names, URIs, ranges or opaque fields cannot redirect them. Both fresh results and follow-up items pass the same projection and revision checks.

<ui:copy id="type-hierarchy-provider-queries">
    pub(super) async fn provider_type_hierarchy(&self, params: Value) -> Result<Value> {
        let revision = self.provider_source_revision().await;
        let origin: TextDocumentPositionParams = serde_json::from_value(params).map_err(|_| {
            tower_lsp::jsonrpc::Error::invalid_params("Malformed type hierarchy source position")
        })?;
        let Some((uri, language, position)) = self
            .virtual_target(&origin.text_document.uri, origin.position)
            .await
        else {
            return Ok(Value::Null);
        };
        let Some(root) = self.root_of(&uri).await else {
            return Ok(Value::Null);
        };
        let handle = {
            let dispatcher = self.dispatcher.lock().await;
            let Ok(handle) = dispatcher.get_child(&language, &root) else {
                return Ok(Value::Null);
            };
            handle.clone()
        };
        self.require_provider_revision(&revision).await?;
        let result = handle
            .request(
                "textDocument/prepareTypeHierarchy",
                json!({"textDocument":{"uri":uri},"position":position}),
            )
            .await
            .map_err(child_request_error)?;
        self.require_provider_revision(&revision).await?;
        self.remember_type_items(result, handle, &revision).await
    }
    async fn type_hierarchy_followup(&self, method: &str, item: Value) -> Result<Value> {
        let token = item
            .get("data")
            .and_then(|data| data.get("hickory_type"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                tower_lsp::jsonrpc::Error::invalid_params(
                    "Unknown type item. Prepare the hierarchy again.",
                )
            })?;
        let revision = self.provider_source_revision().await;
        let held = self
            .prepared_types
            .lock()
            .await
            .items
            .get(Instant::now(), token, &revision)
            .ok_or_else(|| {
                tower_lsp::jsonrpc::Error::invalid_params(
                    "Type hierarchy expired or its source changed. Prepare again.",
                )
            })?;
        let result = held
            .handle
            .request(method, json!({"item":held.original}))
            .await
            .map_err(child_request_error)?;
        self.require_provider_revision(&revision).await?;
        self.remember_type_items(result, held.handle, &revision)
            .await
    }
    pub async fn type_supertypes(&self, params: Value) -> Result<Value> {
        self.type_hierarchy_followup(
            "typeHierarchy/supertypes",
            params.get("item").cloned().unwrap_or(Value::Null),
        )
        .await
    }
    pub async fn type_subtypes(&self, params: Value) -> Result<Value> {
        self.type_hierarchy_followup(
            "typeHierarchy/subtypes",
            params.get("item").cloned().unwrap_or(Value::Null),
        )
        .await
    }
}
</ui:copy>

## Check authored names and untouched child fields

This focused case uses a type from a different virtual file, with dedented source columns and a nonzero authored line. It requires exact name projection, preserves opaque data and tag arrays, and refuses a reversed selection range.

<ui:copy id="type-hierarchy-source-witness">
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn type_names_map_against_their_own_source_and_preserve_opaque_child_fields() {
        let child = "file:///virtual/second.ts";
        let map = PositionMap::build(&[crate::virtual_file::VirtualSegment {
            text: "  Type\n".into(),
            source_span: None,
            source_line: 11,
            source_column: 3,
        }]);
        let index = HashMap::from([(
            uri_key(child),
            (Url::parse("file:///project/read.md").unwrap(), map),
        )]);
        let original = json!({"name":"Type","kind":5,"uri":child,"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":6}},"selectionRange":{"start":{"line":0,"character":2},"end":{"line":0,"character":6}},"tags":[1],"data":{"opaque":[1,{"uri":"file:///opaque"}]}});
        let mapped = map_type_item(&original, &index).unwrap();
        assert_eq!(mapped["uri"], "file:///project/read.md");
        assert_eq!(
            mapped["selectionRange"]["start"],
            json!({"line":10,"character":5})
        );
        assert_eq!(mapped["tags"], json!([1]));
        assert_eq!(mapped["data"], original["data"]);
        assert_eq!(original["selectionRange"]["start"]["line"], 0);
        let mut invalid = original;
        invalid["selectionRange"]["end"]["character"] = json!(1);
        assert!(map_type_item(&invalid, &index).is_err());
    }
}
</ui:copy>

## Register both transports through one factory

Stdio and shared workspace sockets use the same factory for custom capture. Standard type-hierarchy requests enter the explicit LanguageServer trait handlers; registering a custom method with an existing standard name cannot override the library's default handler. The pinned library lacks the standard capability field, so initialization advertises this extension in experimental capabilities and simple clients invoke its standard methods explicitly.

<ui:copy id="type-hierarchy-service-registration">
/// Keep stdio and shared workspace sockets on the same protocol registration.
pub fn service<F>(
    init: F,
) -> (
    tower_lsp::LspService<crate::HickBackend>,
    tower_lsp::ClientSocket,
)
where
    F: FnOnce(tower_lsp::Client) -> crate::HickBackend,
{
    tower_lsp::LspService::build(init)
        .custom_method("hick/captureDocument", crate::HickBackend::capture_document)
        .custom_method("hick/completion", crate::HickBackend::source_bound_completion)
        .custom_method("hick/rename", crate::HickBackend::source_bound_rename)
        .custom_method("hick/formatting", crate::HickBackend::source_bound_formatting)
        .finish()
}
</ui:copy>

## Cross the pinned typed response boundary

The standard trait handlers delegate to the raw retained-item model. The pinned type library incorrectly represents the optional tag array as a single tag; omit that display-only field at the typed response boundary while keeping every original tag in the held child payload. Name, kind, detail, source ranges and backend token remain available. This avoids returning an invalid scalar tag or corrupting later provider requests; deprecated-tag styling in the type list is not claimed.

<ui:copy id="type-hierarchy-typed-boundary">
pub(super) fn typed_type_items(value: Value) -> Result<Option<Vec<TypeHierarchyItem>>> {
    if value.is_null() {
        return Ok(None);
    }
    let mut values = value.as_array().cloned().ok_or_else(|| {
        tower_lsp::jsonrpc::Error::invalid_params("Malformed type hierarchy response")
    })?;
    // lsp-types 0.94 spells optional tags as one SymbolTag, rather than an
    // array. Omit that display-only field; the held child originals keep it.
    for item in &mut values {
        if let Some(object) = item.as_object_mut() {
            object.remove("tags");
        }
    }
    serde_json::from_value(json!(values))
        .map(Some)
        .map_err(|_| tower_lsp::jsonrpc::Error::invalid_params("Malformed type hierarchy response"))
}
</ui:copy>

<ui:file path="crates/hick-lsp/src/backend/types.rs"><ui:paste select="#type-hierarchy-retained-context"/><ui:paste select="#type-hierarchy-source-projection"/><ui:paste select="#type-hierarchy-retain-results"/><ui:paste select="#type-hierarchy-provider-queries"/><ui:paste select="#type-hierarchy-source-witness"/><ui:paste select="#type-hierarchy-typed-boundary"/></ui:file>
<ui:file path="crates/hick-lsp/src/service.rs"><ui:paste select="#type-hierarchy-service-registration"/></ui:file>
</ui:doc>
