use super::*;
use crate::child_lsp::ChildLspHandle;
use crate::provider_artifacts::ProviderArtifacts;
use serde_json::{Value, json};
use std::time::{Duration, Instant};
#[derive(Clone)]
struct PreparedAction {
    original: Value,
    handle: ChildLspHandle,
    resolve: bool,
}
pub(super) struct PreparedActions {
    values: ProviderArtifacts<PreparedAction>,
}
impl Default for PreparedActions {
    fn default() -> Self {
        Self {
            values: ProviderArtifacts::new(1024, Duration::from_secs(120)),
        }
    }
}
fn provider_action_context(context: &CodeActionContext, diagnostics: Vec<Value>) -> Value {
    let mut value = json!({"diagnostics":diagnostics});
    if let Some(only) = &context.only {
        value["only"] = json!(only);
    }
    if let Some(trigger) = context.trigger_kind {
        value["triggerKind"] = json!(trigger);
    }
    value
}
#[cfg(test)]
mod request_context_tests {
    use super::*;
    #[test]
    fn bulk_action_kind_and_trigger_survive_without_trusting_client_diagnostics() {
        let context: CodeActionContext = serde_json::from_value(
            json!({"diagnostics":[],"only":["source.fixAll"],"triggerKind":1}),
        )
        .unwrap();
        let diagnostic = json!({"message":"captured provider diagnostic"});
        assert_eq!(
            provider_action_context(&context, vec![diagnostic.clone()]),
            json!({"diagnostics":[diagnostic],"only":["source.fixAll"],"triggerKind":1})
        );
        let context: CodeActionContext = serde_json::from_value(json!({"diagnostics":[]})).unwrap();
        assert_eq!(
            provider_action_context(&context, vec![]),
            json!({"diagnostics":[]})
        );
    }
}
impl HickBackend {
    pub(super) async fn provider_source_revision(&self) -> String {
        let index = self.vfile_index.read().await;
        crate::source_identity::source_graph_fingerprint(
            index
                .iter()
                .map(|(uri, mapping)| (uri.as_str(), mapping.source_revision)),
        )
    }
    pub(super) async fn require_provider_revision(&self, expected: &str) -> Result<()> {
        if self.provider_source_revision().await == expected {
            Ok(())
        } else {
            Err(tower_lsp::jsonrpc::Error::invalid_params(
                "Source or ownership changed. Request provider information again.",
            ))
        }
    }
    pub(super) async fn provider_code_actions(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        let revision = self.provider_source_revision().await;
        let uri = params.text_document.uri;
        let Some((vf_uri, language, start)) = self.virtual_target(&uri, params.range.start).await
        else {
            return Ok(None);
        };
        let (end, diagnostics) = {
            let index = self.vfile_index.read().await;
            let end = index
                .get(&vf_uri)
                .and_then(|mapping| {
                    mapping
                        .position_map
                        .to_virtual(params.range.end.line, params.range.end.character)
                })
                .unwrap_or((start.line, start.character));
            let requested = Range::new(start, Position::new(end.0, end.1));
            (
                end,
                index
                    .get(&vf_uri)
                    .map(|mapping| {
                        provider_action_diagnostics(&mapping.provider_diagnostics, requested)
                    })
                    .unwrap_or_default(),
            )
        };
        let Some(root) = self.root_of(&vf_uri).await else {
            return Ok(None);
        };
        let (handle, resolve) = {
            let dispatcher = self.dispatcher.lock().await;
            let Ok(handle) = dispatcher.get_child(&language, &root) else {
                return Ok(None);
            };
            let resolve = dispatcher
                .server_capabilities(&language, &root)
                .and_then(|value| value.pointer("/codeActionProvider/resolveProvider"))
                .and_then(Value::as_bool)
                == Some(true);
            (handle.clone(), resolve)
        };
        self.require_provider_revision(&revision).await?;
        let result=handle.request("textDocument/codeAction",json!({"textDocument":{"uri":vf_uri},"range":{"start":start,"end":{"line":end.0,"character":end.1}},"context":provider_action_context(&params.context,diagnostics)})).await.map_err(child_request_error)?;
        self.require_provider_revision(&revision).await?;
        if result.is_null() {
            return Ok(None);
        }
        let originals = result.as_array().ok_or_else(|| {
            tower_lsp::jsonrpc::Error::invalid_params("Malformed provider action list")
        })?;
        let index = self.index_snapshot().await;
        self.require_provider_revision(&revision).await?;
        let workspace_root = self.workspace_root.read().await.clone();
        let mut translated = translate_provider_edits(result.clone(), &index, workspace_root.as_ref())
            .map_err(tower_lsp::jsonrpc::Error::invalid_params)?;
        let entries = translated.as_array_mut().expect("translated action list");
        let mut held = self.prepared_actions.lock().await;
        for (original, action) in originals.iter().zip(entries.iter_mut()) {
            // Raw Command alternatives are retained for an explicit client refusal.
            if original.get("command").is_some_and(Value::is_string) {
                continue;
            }
            let discovery = discover_action_sources(original, &index, workspace_root.as_ref());
            let mut data =
                action_discovery_data(discovery.as_ref().and_then(|value| value.get("data")));
            if let Some(discovery) = discovery {
                *action = discovery;
            }
            let token = held.values.insert(
                Instant::now(),
                revision.clone(),
                PreparedAction {
                    original: original.clone(),
                    handle: handle.clone(),
                    resolve,
                },
            );
            let object = action.as_object_mut().ok_or_else(|| {
                tower_lsp::jsonrpc::Error::invalid_params("Malformed provider action")
            })?;
            data["hickory_action"] = json!(token);
            object.insert("data".into(), data);
        }
        serde_json::from_value(translated)
            .map(Some)
            .map_err(|error| {
                tower_lsp::jsonrpc::Error::invalid_params(format!(
                    "Malformed mapped action list: {error}"
                ))
            })
    }
    pub(super) async fn resolve_provider_action(&self, item: CodeAction) -> Result<CodeAction> {
        let token = item
            .data
            .as_ref()
            .and_then(|value| value.get("hickory_action"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                tower_lsp::jsonrpc::Error::invalid_params(
                    "Unknown provider action. Request actions again.",
                )
            })?;
        let revision = self.provider_source_revision().await;
        let prepared = self
            .prepared_actions
            .lock()
            .await
            .values
            .get(Instant::now(), token, &revision)
            .ok_or_else(|| {
                tower_lsp::jsonrpc::Error::invalid_params(
                    "Provider action expired or its source changed. Request actions again.",
                )
            })?;
        let mut original = prepared.original;
        if prepared.resolve {
            self.require_provider_revision(&revision).await?;
            let result = prepared
                .handle
                .request("codeAction/resolve", original.clone())
                .await
                .map_err(child_request_error)?;
            self.require_provider_revision(&revision).await?;
            let resolved = result.as_object().ok_or_else(|| {
                tower_lsp::jsonrpc::Error::invalid_params("Provider did not resolve this action")
            })?;
            let object = original.as_object_mut().expect("held literal action");
            for (key, value) in resolved {
                object.insert(key.clone(), value.clone());
            }
        }
        let index = self.index_snapshot().await;
        self.require_provider_revision(&revision).await?;
        let workspace_root = self.workspace_root.read().await.clone();
        let discovery = discover_action_sources(&original, &index, workspace_root.as_ref());
        let mapped = translate_provider_edits(json!([original]), &index, workspace_root.as_ref())
            .map_err(tower_lsp::jsonrpc::Error::invalid_params)?;
        let mut action = mapped
            .as_array()
            .and_then(|entries| entries.first())
            .cloned()
            .ok_or_else(|| {
                tower_lsp::jsonrpc::Error::invalid_params("Malformed resolved provider action")
            })?;
        let mut data =
            action_discovery_data(discovery.as_ref().and_then(|value| value.get("data")));
        if let Some(discovery) = discovery {
            action = discovery;
        }
        data["hickory_action"] = json!(token);
        action["data"] = data;
        self.require_provider_revision(&revision).await?;
        serde_json::from_value(action).map_err(|error| {
            tower_lsp::jsonrpc::Error::invalid_params(format!(
                "Malformed resolved provider action: {error}"
            ))
        })
    }
}
fn action_discovery_data(value: Option<&Value>) -> Value {
    value
        .and_then(|value| value.get("hickory_acquire"))
        .map(|paths| json!({"hickory_acquire":paths}))
        .unwrap_or_else(|| json!({}))
}
// Discovery identifies files to load, never source ranges or edit authority.
fn discover_action_sources(
    original: &Value,
    index: &HashMap<String, (Url, PositionMap)>,
    root: Option<&Url>,
) -> Option<Value> {
    if original.get("command").is_some() || original.get("disabled").is_some() {
        return None;
    }
    let edit = original.get("edit")?.as_object()?;
    let mut uris = Vec::new();
    if let Some(changes) = edit.get("changes") {
        for (uri, edits) in changes.as_object()? {
            edits.as_array()?;
            uris.push(uri.clone());
        }
    }
    if let Some(changes) = edit.get("documentChanges") {
        if !uris.is_empty() {
            return None;
        }
        let mut introduced = std::collections::HashSet::new();
        for change in changes.as_array()? {
            match change.get("kind").and_then(Value::as_str) {
                Some("create") => {
                    introduced.insert(uri_key(change.get("uri")?.as_str()?));
                }
                Some("rename") => {
                    let old = change.get("oldUri")?.as_str()?;
                    if !introduced.remove(&uri_key(old)) {
                        uris.push(old.to_owned());
                    }
                    introduced.insert(uri_key(change.get("newUri")?.as_str()?));
                }
                Some("delete") => {
                    let old = change.get("uri")?.as_str()?;
                    if !introduced.remove(&uri_key(old)) {
                        uris.push(old.to_owned());
                    }
                }
                Some(_) => return None,
                None => {
                    change.get("edits")?.as_array()?;
                    let uri = change.pointer("/textDocument/uri")?.as_str()?;
                    if !introduced.contains(&uri_key(uri)) {
                        uris.push(uri.to_owned());
                    }
                }
            }
        }
    }
    let root = root?.to_file_path().ok()?;
    let mut missing = std::collections::BTreeSet::new();
    for uri in uris {
        if index.contains_key(&uri_key(&uri)) {
            continue;
        }
        let url = Url::parse(&uri).ok()?;
        if url.query().is_some() || url.fragment().is_some() {
            return None;
        }
        let path = url.to_file_path().ok()?;
        // The host must still establish listed authored ownership. A path here
        // only requests acquisition; it is not an identity position map.
        if !path.starts_with(&root) || path == root {
            return None;
        }
        missing.insert(url.to_string());
        if missing.len() > 10000 {
            return None;
        }
    }
    if missing.is_empty() {
        return None;
    }
    let mut answer = original.clone();
    let object = answer.as_object_mut()?;
    object.remove("edit");
    object.insert("data".into(), json!({"hickory_acquire":missing.into_iter().map(|uri|json!({"uri":uri})).collect::<Vec<_>>()}));
    Some(answer)
}
#[cfg(test)]
mod discovery_tests {
    use super::*;
    #[test]
    fn unknown_targets_are_discovery_only_and_must_stay_in_workspace() {
        let root = Url::parse("file:///workspace/").unwrap();
        let index = HashMap::new();
        let action = json!({"title":"Fix","edit":{"changes":{"file:///workspace/other.py":[]}}});
        let discovered = discover_action_sources(&action, &index, Some(&root)).unwrap();
        assert!(discovered.get("edit").is_none());
        assert_eq!(
            discovered["data"]["hickory_acquire"][0]["uri"],
            "file:///workspace/other.py"
        );
        for edit in [
            json!({"changes":{"file:///outside.py":[]}}),
            json!({"documentChanges":[{"kind":"create","uri":"file:///workspace/new.py"}]}),
            json!({"changes":{"file:///workspace/other.py":"malformed"}}),
        ] {
            assert!(
                discover_action_sources(&json!({"title":"Fix","edit":edit}), &index, Some(&root))
                    .is_none()
            );
        }
        let mut command = action;
        command["command"] = json!({"command":"effect"});
        assert!(discover_action_sources(&command, &index, Some(&root)).is_none());
        let resources = json!({"title":"Move and generate","edit":{"documentChanges":[
            {"kind":"rename","oldUri":"file:///workspace/old.py","newUri":"file:///workspace/moved.py"},
            {"textDocument":{"uri":"file:///workspace/moved.py"},"edits":[]},
            {"kind":"create","uri":"file:///workspace/new.py"},
            {"textDocument":{"uri":"file:///workspace/new.py"},"edits":[]}
        ]}});
        let discovered = discover_action_sources(&resources, &index, Some(&root)).unwrap();
        assert!(discovered.get("edit").is_none());
        assert_eq!(
            discovered["data"]["hickory_acquire"],
            json!([{"uri":"file:///workspace/old.py"}])
        );
    }
}
