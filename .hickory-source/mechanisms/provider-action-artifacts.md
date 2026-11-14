<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Resolve held provider actions against their source graph

For maintainers adding provider operations independently of the editor presentation.

An action holds the original child payload and child process handle, not whichever provider answered last. Every installed source/ownership mapping has a fresh revision; a content-free graph fingerprint retires actions after replacement, addition, removal or changed ownership. Diagnostic-cache updates alone do not replace source identity. The engine checks this graph before and after provider reads and before projection. Tokens select held data and never authorize writes or provider-command execution.

## provider-source-graph-identity

A mapping revision names one installed construction. Rebuilding an identical construction conservatively retires old provider state. Sort and frame identities before hashing so iteration order cannot change the graph identity; source content is not copied into the token store. The focused witness checks stable ordering and changed source/owner membership.

The model owns only identity and retained values; editor layout and application remain separate boundaries.

<ui:copy id="provider-source-graph-identity-model">
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
/// One installed source/ownership mapping, independent of provider diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceRevision(u64);
impl SourceRevision {
    pub(crate) fn fresh() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}
/// Content-free identity of the installed mapping graph. Replacement is conservative.
pub(crate) fn source_graph_fingerprint<'a>(
    rows: impl Iterator<Item = (&'a str, SourceRevision)>,
) -> String {
    let mut rows: Vec<_> = rows.collect();
    rows.sort_by(|a, b| a.0.cmp(b.0));
    let mut digest = Sha256::new();
    digest.update(b"hickory-provider-source-graph-v1\0");
    for (uri, revision) in rows {
        digest.update((uri.len() as u64).to_be_bytes());
        digest.update(uri.as_bytes());
        digest.update(revision.0.to_be_bytes());
    }
    format!("{:x}", digest.finalize())
}
</ui:copy>

Exercise actual model transitions with controlled values and time: stable graph ordering, source/owner replacement, bounded retention, expiration and independent opaque payloads must not fabricate a valid lookup.

<ui:copy id="provider-source-graph-identity-witness">
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installed_source_identity_rejects_replacement_removal_and_reordering_only_is_stable() {
        let a = SourceRevision::fresh();
        let b = SourceRevision::fresh();
        let fingerprint =
            |rows: Vec<(&str, SourceRevision)>| source_graph_fingerprint(rows.into_iter());
        let first = fingerprint(vec![("a", a), ("b", b)]);
        assert_eq!(first, fingerprint(vec![("b", b), ("a", a)]));
        assert_ne!(
            first,
            fingerprint(vec![("a", SourceRevision::fresh()), ("b", b)])
        );
        assert_ne!(first, fingerprint(vec![("a", a)]));
        assert_ne!(first, fingerprint(vec![("other-owner", a), ("b", b)]));
    }
}
</ui:copy>

<ui:file path="crates/hick-lsp/src/source_identity.rs"><ui:paste select="#provider-source-graph-identity-model"/><ui:paste select="#provider-source-graph-identity-witness"/></ui:file>

## provider-artifact-retention

Retain at most 1,024 action values for two minutes in production. Each store has its own process-instance epoch; expiration, eviction and changed source refuse a lookup. Held opaque values are cloned rather than replaced by client-supplied action fields. The focused witnesses check bounds, expiry, source identity, separate stores and unchanged nested provider data.

The model owns only identity and retained values; editor layout and application remain separate boundaries.

<ui:copy id="provider-artifact-retention-model">
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
struct Entry<T> {
    token: String,
    source: String,
    expires: Instant,
    value: T,
}
/// Bounded, expiring provider values. A token selects held data; it never authorizes a write.
pub(crate) struct ProviderArtifacts<T> {
    epoch: String,
    next: u64,
    capacity: usize,
    lifetime: Duration,
    entries: VecDeque<Entry<T>>,
}
impl<T: Clone> ProviderArtifacts<T> {
    pub(crate) fn new(capacity: usize, lifetime: Duration) -> Self {
        assert!(capacity > 0);
        static EPOCH: AtomicU64 = AtomicU64::new(1);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        Self {
            epoch: format!(
                "{:x}-{:x}-{:x}",
                std::process::id(),
                time,
                EPOCH.fetch_add(1, Ordering::Relaxed)
            ),
            next: 0,
            capacity,
            lifetime,
            entries: VecDeque::new(),
        }
    }
    pub(crate) fn insert(&mut self, now: Instant, source: String, value: T) -> String {
        self.entries.retain(|entry| entry.expires > now);
        while self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.next += 1;
        let token = format!("{}:{:x}", self.epoch, self.next);
        self.entries.push_back(Entry {
            token: token.clone(),
            source,
            expires: now + self.lifetime,
            value,
        });
        token
    }
    pub(crate) fn get(&mut self, now: Instant, token: &str, source: &str) -> Option<T> {
        self.entries.retain(|entry| entry.expires > now);
        let index = self.entries.iter().position(|entry| entry.token == token)?;
        if self.entries[index].source != source {
            self.entries.remove(index);
            return None;
        }
        Some(self.entries[index].value.clone())
    }
}
</ui:copy>

Exercise actual model transitions with controlled values and time: stable graph ordering, source/owner replacement, bounded retention, expiration and independent opaque payloads must not fabricate a valid lookup.

<ui:copy id="provider-artifact-retention-witness">
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn provider_values_are_bounded_expiring_and_source_bound() {
        let now = Instant::now();
        let mut values = ProviderArtifacts::new(2, Duration::from_secs(10));
        let first = values.insert(now, "source".into(), "first");
        let second = values.insert(now, "source".into(), "second");
        let third = values.insert(now, "source".into(), "third");
        assert!(values.get(now, &first, "source").is_none());
        assert_eq!(values.get(now, &second, "source"), Some("second"));
        assert!(values.get(now, &third, "changed").is_none());
        assert!(
            values
                .get(now + Duration::from_secs(10), &second, "source")
                .is_none()
        );
    }
    #[test]
    fn tokens_cannot_alias_another_store_and_opaque_data_is_held_verbatim() {
        let now = Instant::now();
        let mut first = ProviderArtifacts::new(2, Duration::from_secs(10));
        let mut second = ProviderArtifacts::new(2, Duration::from_secs(10));
        let original = serde_json::json!({"data":{"uri":"file:///provider-owned","nested":[1,2]},"title":"Fix"});
        let token = first.insert(now, "source".into(), original.clone());
        let other = second.insert(now, "source".into(), original.clone());
        assert_ne!(token, other);
        assert!(second.get(now, &token, "source").is_none());
        let mut returned = first.get(now, &token, "source").unwrap();
        returned["data"] = serde_json::Value::Null;
        assert_eq!(first.get(now, &token, "source"), Some(original));
    }
}
</ui:copy>

<ui:file path="crates/hick-lsp/src/provider_artifacts.rs"><ui:paste select="#provider-artifact-retention-model"/><ui:paste select="#provider-artifact-retention-witness"/></ui:file>

## provider-action-resolution

Use exact child diagnostics in provider coordinates, retain raw child action values before translating editable source, and call lazy resolution only when that child advertised it. Eager children return their held complete value through the same boundary. Ignore caller mutations except the lookup token, translate the complete resolved edit through current compiler mappings and retain disabled/refused outcomes. Neither query nor resolution executes a command or applies an edit.

Hold one child connection and its original action value. Source and ownership revisions are independent of diagnostic-cache messages; a reply must still belong to the same installed graph.

<ui:copy id="provider-action-source-context">
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
impl HickBackend {
    async fn action_source_revision(&self) -> String {
        let index = self.vfile_index.read().await;
        crate::source_identity::source_graph_fingerprint(
            index
                .iter()
                .map(|(uri, mapping)| (uri.as_str(), mapping.source_revision)),
        )
    }
    async fn require_action_revision(&self, expected: &str) -> Result<()> {
        if self.action_source_revision().await == expected {
            Ok(())
        } else {
            Err(tower_lsp::jsonrpc::Error::invalid_params(
                "Source or ownership changed. Request provider actions again.",
            ))
        }
    }
</ui:copy>

Build provider context from the child’s own diagnostics. Read its actual capability declaration, retain the original unprojected payload, and return an editor-facing value with a lookup token. A raw Command alternative is preserved for explicit refusal rather than partially treated as a text edit.

<ui:copy id="provider-action-query">
    pub(super) async fn provider_code_actions(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        let revision = self.action_source_revision().await;
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
        self.require_action_revision(&revision).await?;
        let result=handle.request("textDocument/codeAction",json!({"textDocument":{"uri":vf_uri},"range":{"start":start,"end":{"line":end.0,"character":end.1}},"context":{"diagnostics":diagnostics}})).await.map_err(child_request_error)?;
        self.require_action_revision(&revision).await?;
        if result.is_null() {
            return Ok(None);
        }
        let originals = result.as_array().ok_or_else(|| {
            tower_lsp::jsonrpc::Error::invalid_params("Malformed provider action list")
        })?;
        let index = self.index_snapshot().await;
        self.require_action_revision(&revision).await?;
        let mut translated = translate_edit_uris(result.clone(), &index)
            .map_err(tower_lsp::jsonrpc::Error::invalid_params)?;
        let entries = translated.as_array_mut().expect("translated action list");
        let mut held = self.prepared_actions.lock().await;
        for (original, action) in originals.iter().zip(entries.iter_mut()) {
            // Raw Command alternatives are retained for an explicit client refusal.
            if original.get("command").is_some_and(Value::is_string) {
                continue;
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
            object.insert("data".into(), json!({"hickory_action":token}));
        }
        serde_json::from_value(translated)
            .map(Some)
            .map_err(|error| {
                tower_lsp::jsonrpc::Error::invalid_params(format!(
                    "Malformed mapped action list: {error}"
                ))
            })
    }
</ui:copy>

The lookup token selects held data; every other caller-supplied field is ignored. An advertised lazy resolver receives the original child value through its original handle. Check source identity again after the await, then project the complete edit through current compiler ownership. Disabled or unmappable actions remain explicit outcomes.

<ui:copy id="provider-action-resolve">
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
        let revision = self.action_source_revision().await;
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
            self.require_action_revision(&revision).await?;
            let result = prepared
                .handle
                .request("codeAction/resolve", original.clone())
                .await
                .map_err(child_request_error)?;
            self.require_action_revision(&revision).await?;
            let resolved = result.as_object().ok_or_else(|| {
                tower_lsp::jsonrpc::Error::invalid_params("Provider did not resolve this action")
            })?;
            let object = original.as_object_mut().expect("held literal action");
            for (key, value) in resolved {
                object.insert(key.clone(), value.clone());
            }
        }
        let index = self.index_snapshot().await;
        self.require_action_revision(&revision).await?;
        let mapped = translate_edit_uris(json!([original]), &index)
            .map_err(tower_lsp::jsonrpc::Error::invalid_params)?;
        let mut action = mapped
            .as_array()
            .and_then(|entries| entries.first())
            .cloned()
            .ok_or_else(|| {
                tower_lsp::jsonrpc::Error::invalid_params("Malformed resolved provider action")
            })?;
        action["data"] = json!({"hickory_action":token});
        serde_json::from_value(action).map_err(|error| {
            tower_lsp::jsonrpc::Error::invalid_params(format!(
                "Malformed resolved provider action: {error}"
            ))
        })
    }
}
</ui:copy>

<ui:file path="crates/hick-lsp/src/backend/actions.rs"><ui:paste select="#provider-action-source-context"/><ui:paste select="#provider-action-query"/><ui:paste select="#provider-action-resolve"/></ui:file>

</ui:doc>
