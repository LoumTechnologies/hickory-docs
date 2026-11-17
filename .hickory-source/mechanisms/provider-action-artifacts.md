<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Resolve held provider actions against their source graph

For maintainers adding provider operations independently of the editor presentation.

An action holds the original child payload and child process handle, not whichever provider answered last. Every installed source/ownership mapping has a fresh revision; a content-free graph fingerprint retires actions after replacement, addition, removal or changed ownership. Diagnostic-cache updates alone do not replace source identity. The engine checks this graph before and after provider reads and before projection. Tokens select held data and never authorize writes or provider-command execution.

Focused resource integration (2026-11-15): the ordered projection and acquisition
witnesses passed, including generated/external refusal, annotations, virtual-version
retirement and literal URI escaping. The actual-engine opaque-action protocol
witness passed original-payload routing, caller substitution refusal and stale/foreign
token refusal. The controlled-provider browser resource flow passed unopened-source
acquisition, complete ordered resource review, explicit publication and guarded
undo/redo. Its real plain-file path exposed and corrected an incorrect assumption
that an identity map's primary URI always used the client `hick` scheme: the
workspace wire supplies confined physical `file` identities inside the language
engine. These are protocol and connected browser receipts, not language-analysis
completeness or native physical-input acceptance.

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
    refactor_query: Option<Value>,
    already_mapped: bool,
    display_title: Option<String>,
    source_proofs: Arc<Vec<ProviderSourceProof>>,
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
#[derive(Clone, Debug, PartialEq, Eq)]
struct ProviderSourceProof {
    uri: Url,
    hash: String,
    open_revision: Option<crate::source_identity::SourceRevision>,
}
#[derive(Clone)]
struct ProviderActionTarget {
    uri: Url,
    context: Url,
    language: String,
    range: Range,
    label: Option<String>,
    source_proofs: Arc<Vec<ProviderSourceProof>>,
}
fn document_wide_source_request(context: &CodeActionContext) -> bool {
    context.only.as_ref().is_some_and(|only| {
        !only.is_empty()
            && only.iter().all(|kind| {
                kind.as_str() == "source" || kind.as_str().starts_with("source.")
            })
    })
}
fn provider_target_contributes(mapping: &VFileMapping, requested: &Url) -> bool {
    &mapping.hick_uri == requested
        || mapping
            .position_map
            .contributing_documents()
            .into_iter()
            .any(|document| {
                crate::project::source_document_uri(&mapping.hick_uri,document).ok().is_some_and(|mut uri| {
                    uri.set_query(mapping.hick_uri.query());
                    &uri == requested
                })
            })
}
fn merge_action_source_proofs(
    targets: &[ProviderActionTarget],
) -> Result<Vec<ProviderSourceProof>> {
    let mut proofs = std::collections::BTreeMap::<String, ProviderSourceProof>::new();
    for target in targets {
        for proof in target.source_proofs.iter() {
            if proofs
                .get(proof.uri.as_str())
                .is_some_and(|before| before != proof)
            {
                return Err(tower_lsp::jsonrpc::Error::invalid_params(format!(
                    "Provider outputs captured different revisions of {}. Refresh their project entries from the intended sources and request complete fixes again.",
                    proof.uri
                )));
            }
            proofs.insert(proof.uri.to_string(), proof.clone());
        }
    }
    Ok(proofs.into_values().collect())
}
fn action_source_proofs(mapping: &VFileMapping) -> Result<Arc<Vec<ProviderSourceProof>>> {
    let mut proofs = std::collections::BTreeMap::from([(
        mapping.hick_uri.to_string(),
        ProviderSourceProof {
            uri: mapping.hick_uri.clone(),
            hash: mapping.primary_source_hash.clone(),
            open_revision: None,
        },
    )]);
    for document in mapping.position_map.contributing_documents() {
        let mut uri=crate::project::source_document_uri(&mapping.hick_uri,document).map_err(|message|tower_lsp::jsonrpc::Error::invalid_params(format!("A captured contributor has no safe source address: {message}")))?;
        uri.set_query(mapping.hick_uri.query());
        let source=mapping.position_map.project_source(document).ok_or_else(||tower_lsp::jsonrpc::Error::invalid_params("A captured contributor is unavailable. Refresh the complete project before requesting fixes."))?;
        proofs.insert(
            uri.to_string(),
            ProviderSourceProof {
                uri,
                hash: diagnostic_source_digest(source),
                open_revision: None,
            },
        );
    }
    Ok(Arc::new(proofs.into_values().collect()))
}
fn provider_output_label(uri: &Url, root: &str) -> String {
    uri.as_str()
        .strip_prefix(&format!("{}/", root.trim_end_matches('/')))
        .unwrap_or(uri.as_str())
        .to_string()
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
</ui:copy>

A copy-only contributor has no virtual-file identity to replace. Exercise source-equal Undo and close/reopen explicitly, along with exact capture acknowledgement. A separate preparation witness keeps a coherent target usable while an unrelated source is busy. Loaded project sources are not contributor evidence, repeated source copies are still contributors, and conflicting captured revisions must refuse as one complete read.

<ui:copy id="provider-action-source-proof-witness">
#[cfg(test)]
mod request_context_tests {
    use super::*;
    #[tokio::test]
    async fn copy_only_document_generations_retire_equal_byte_undo_and_reopen() {
        let directory = tempfile::tempdir().unwrap();
        let uri = Url::from_file_path(directory.path().join("copy.md")).unwrap();
        let (service, _socket) = tower_lsp::LspService::new(HickBackend::new);
        let backend = service.inner();
        let source = "\x3chick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\x3chick:copy id=\"code\">value = 1\n\x3c/hick:copy>\x3c/hick:doc>";
        backend.process_document(&uri, source).await;
        assert!(backend.vfile_index.read().await.is_empty());
        let acknowledgement = backend
            .capture_document(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "markdown".into(),
                    version: 1,
                    text: source.into(),
                },
            })
            .await
            .unwrap();
        assert_eq!(acknowledgement["current"], true);
        assert_eq!(acknowledgement["processed"], true);
        assert_eq!(
            acknowledgement["source_hash"],
            diagnostic_source_digest(source)
        );
        assert_eq!(acknowledgement["source_files_modified"], false);
        assert_eq!(
            acknowledgement["source_revision"].as_str().unwrap().len(),
            64
        );
        let original = backend.provider_source_revision().await;
        backend
            .process_document(&uri, &source.replace("value", "other"))
            .await;
        backend.process_document(&uri, source).await;
        assert_ne!(backend.provider_source_revision().await, original);
        let before_close = backend.provider_source_revision().await;
        backend
            .did_close(DidCloseTextDocumentParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
            })
            .await;
        backend.process_document(&uri, source).await;
        assert_ne!(backend.provider_source_revision().await, before_close);
        assert_ne!(backend.provider_source_revision().await, original);
        assert!(backend.vfile_index.read().await.is_empty());
    }
    #[tokio::test]
    async fn preparation_refuses_only_the_requested_sources_not_unrelated_ready_work() {
        let (service, _socket) = tower_lsp::LspService::new(HickBackend::new);
        let backend = service.inner();
        let ready = Url::parse("file:///workspace/ready.py").unwrap();
        let busy = Url::parse("file:///workspace/busy.md").unwrap();
        let mut first = DocEntry::empty();
        first.source_hash = Some("ready-hash".into());
        let revision = first.source_revision;
        let mut second = DocEntry::empty();
        second.source_hash = Some("busy-hash".into());
        second.preparing = true;
        let busy_revision = second.source_revision;
        {
            let mut documents = backend.documents.write().await;
            documents.insert(ready.clone(), first);
            documents.insert(busy.clone(), second);
        }
        let graph = backend.provider_source_revision().await;
        backend.require_provider_revision(&graph).await.unwrap();
        backend
            .require_action_sources(&[ProviderSourceProof {
                uri: ready,
                hash: "ready-hash".into(),
                open_revision: Some(revision),
            }])
            .await
            .unwrap();
        assert!(
            backend
                .require_action_sources(&[ProviderSourceProof {
                    uri: busy,
                    hash: "busy-hash".into(),
                    open_revision: Some(busy_revision)
                }])
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn action_coordinate_proofs_cover_known_affected_sources_and_never_rebind_old_owners() {
        let (service, _socket) = tower_lsp::LspService::new(HickBackend::new);
        let backend = service.inner();
        let a = Url::parse("file:///workspace/a.py").unwrap();
        let b = Url::parse("file:///workspace/b.py").unwrap();
        let ah = diagnostic_source_digest("aaa");
        let bh = diagnostic_source_digest("bbb");
        let mut first = DocEntry::empty();
        first.source_hash = Some(ah.clone());
        let original = Arc::new(vec![ProviderSourceProof {
            uri: a.clone(),
            hash: ah,
            open_revision: Some(first.source_revision),
        }]);
        let mut second = DocEntry::empty();
        second.source_hash = Some(bh.clone());
        {
            let mut documents = backend.documents.write().await;
            documents.insert(a.clone(), first);
            documents.insert(b.clone(), second);
        }
        backend.vfile_index.write().await.insert(
            b.clone(),
            VFileMapping {
                source_revision: crate::source_identity::SourceRevision::fresh(),
                hick_uri: b.clone(),
                language_id: "python".into(),
                root_uri: "file:///workspace/".into(),
                position_map: PositionMap::identity(1),
                provider_diagnostics: vec![],
                provider_source: "bbb".into(),
                provider_version: Some(1),
                primary_source_hash: bh,
            },
        );
        let b_uri = b.as_str();
        let edit = json!({"command":null,"disabled":null,"edit":{"changes":{b_uri:[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},"newText":"x"}]}},"data":{"hickory_sources":[{"uri":"file:///foreign","source_hash":"forged"}]}});
        let proofs = backend
            .extend_action_sources(&edit, &original)
            .await
            .unwrap();
        assert_eq!(proofs.len(), 2);
        let wire = action_source_data(&proofs);
        assert_eq!(wire[0]["uri"], a.as_str());
        assert_eq!(wire[1]["uri"], b.as_str());
        assert_eq!(wire[1]["source_hash"], diagnostic_source_digest("bbb"));
        backend
            .documents
            .write()
            .await
            .get_mut(&b)
            .unwrap()
            .preparing = true;
        assert!(
            backend
                .extend_action_sources(&edit, &original)
                .await
                .is_err()
        );
        {
            let mut documents = backend.documents.write().await;
            let source = documents.get_mut(&b).unwrap();
            source.preparing = false;
            source.source_revision = crate::source_identity::SourceRevision::fresh();
        }
        assert!(
            backend.extend_action_sources(&edit, &proofs).await.is_err(),
            "an equal-byte replacement lifetime cannot renew a held affected-source proof"
        );
    }
    #[test]
    fn complete_file_reads_refuse_conflicting_contributor_snapshots() {
        let source = Url::parse("file:///workspace/a.md").unwrap();
        let source_revision = crate::source_identity::SourceRevision::fresh();
        let target = |hash: &str| ProviderActionTarget {
            uri: Url::parse("file:///stage/a.py").unwrap(),
            context: Url::parse("file:///workspace/project.md").unwrap(),
            language: "python".into(),
            range: Range::default(),
            label: None,
            source_proofs: Arc::new(vec![ProviderSourceProof {
                uri: source.clone(),
                hash: hash.into(),
                open_revision: Some(source_revision),
            }]),
        };
        assert_eq!(
            merge_action_source_proofs(&[target("one"), target("one")])
                .unwrap()
                .len(),
            1
        );
        assert!(merge_action_source_proofs(&[target("one"), target("two")]).is_err());
    }
    #[test]
    fn complete_file_routing_requires_only_the_explicit_source_family() {
        for (only, expected) in [
            (json!(["source.fixAll"]), true),
            (json!(["source.fixAll.python"]), true),
            (json!([]), false),
            (json!(["source.fixAllWrong"]), true),
            (json!(["source"]), true),
            (json!(["source.organizeImports"]), true),
            (json!(["sourceWrong"]), false),
            (json!(["quickfix", "source.fixAll"]), false),
        ] {
            let context: CodeActionContext =
                serde_json::from_value(json!({"diagnostics":[],"only":only})).unwrap();
            assert_eq!(document_wide_source_request(&context), expected);
        }
        assert_eq!(
            provider_output_label(
                &Url::parse("file:///stage/out/a.py").unwrap(),
                "file:///stage/"
            ),
            "out/a.py"
        );
    }
    #[test]
    fn complete_file_routing_follows_repeated_contributors_and_endpoint_identity() {
        let mut entry =
            hick_lang::ProjectEntry::file(std::sync::Arc::<[u8]>::from(&b"one\none\n"[..]));
        entry.origins = vec![
            hick_lang::ByteOrigin {
                output: (0, 4),
                document: "nested/a.md".into(),
                source: (0, 4),
            },
            hick_lang::ByteOrigin {
                output: (4, 8),
                document: "nested/a.md".into(),
                source: (0, 4),
            },
        ];
        let sources = std::sync::Arc::new(std::collections::BTreeMap::from([
            ("project.md".into(), "root".into()),
            ("nested/a.md".into(), "one\n".into()),
            ("unrelated.md".into(), "unused".into()),
        ]));
        let mut mapping = VFileMapping {
            source_revision: crate::source_identity::SourceRevision::fresh(),
            hick_uri: Url::parse("file:///workspace/project.md").unwrap(),
            language_id: "python".into(),
            root_uri: "file:///stage/".into(),
            position_map: PositionMap::for_project(&entry, sources, "project.md").unwrap(),
            provider_diagnostics: vec![],
            provider_source: "one\none\n".into(),
            provider_version: Some(1),
            primary_source_hash: "captured".into(),
        };
        assert!(provider_target_contributes(
            &mapping,
            &Url::parse("file:///workspace/project.md").unwrap()
        ));
        assert!(provider_target_contributes(
            &mapping,
            &Url::parse("file:///workspace/nested/a.md").unwrap()
        ));
        assert!(!provider_target_contributes(
            &mapping,
            &Url::parse("file:///workspace/unrelated.md").unwrap()
        ));
        mapping.hick_uri.set_query(Some("hick-project=old"));
        assert!(!provider_target_contributes(
            &mapping,
            &Url::parse("file:///workspace/nested/a.md").unwrap()
        ));
        assert!(provider_target_contributes(
            &mapping,
            &Url::parse("file:///workspace/nested/a.md?hick-project=old").unwrap()
        ));
    }
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
</ui:copy>

Frame both installed document lifetimes and virtual mappings in the shared provider revision. Snapshot each collection independently rather than holding document and virtual-file locks across awaits. Fresh server identities prevent ABA; the target and contributor proofs, not an unrelated document's preparing state, decide whether an action can be queried or resolved. A scope changing between these reads fails the later original-revision check.

<ui:copy id="provider-action-graph-revision">
impl HickBackend {
    pub(super) async fn provider_source_revision(&self) -> String {
        let mut rows = {
            let documents = self.documents.read().await;
            documents
                .iter()
                .map(|(uri, entry)| (format!("document:{uri}"), entry.source_revision))
                .collect::<Vec<_>>()
        };
        {
            let index = self.vfile_index.read().await;
            rows.extend(
                index
                    .iter()
                    .map(|(uri, mapping)| (format!("virtual:{uri}"), mapping.source_revision)),
            );
        }
        crate::source_identity::source_graph_fingerprint(
            rows.iter().map(|(uri, revision)| (uri.as_str(), *revision)),
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
</ui:copy>

A complete-file `source.fixAll` request is document-wide, so prose at the start of Markdown cannot choose a positional child. Enumerate every installed output associated with the requested document, including byte-origin contributors and repeated copies; request each complete child source independently. Stable output-qualified titles keep multiple actions explicit through requery and opaque resolution. Never merge their edits or return a partial over-budget list. One source-graph revision binds the entire fan-out.

Build provider context from the child’s own diagnostics. Read its actual capability declaration, retain the original unprojected payload, and return an editor-facing value with a lookup token. A raw Command alternative is preserved for explicit refusal rather than partially treated as a text edit.

<ui:copy id="provider-action-query">
    async fn bind_action_source_rows(
        &self,
        mut proofs: Vec<ProviderSourceProof>,
    ) -> Result<Arc<Vec<ProviderSourceProof>>> {
        let documents = self.documents.read().await;
        for proof in &mut proofs {
            if let Some(document) = documents.get(&proof.uri) {
                if document.preparing || document.source_hash.as_ref() != Some(&proof.hash) {
                    return Err(tower_lsp::jsonrpc::Error::invalid_params(format!(
                        "A captured provider source changed or is still being prepared: {}. Preserve unsaved drafts, refresh the source or project entry from the intended sources, and request the action again.",
                        proof.uri
                    )));
                }
                proof.open_revision = Some(document.source_revision);
            }
        }
        Ok(Arc::new(proofs))
    }
    async fn bind_action_sources(&self, targets: &mut [ProviderActionTarget]) -> Result<()> {
        for target in targets {
            target.source_proofs = self
                .bind_action_source_rows(target.source_proofs.as_ref().clone())
                .await?;
        }
        Ok(())
    }
    async fn extend_action_sources(
        &self,
        original: &Value,
        before: &Arc<Vec<ProviderSourceProof>>,
    ) -> Result<Arc<Vec<ProviderSourceProof>>> {
        self.require_action_sources(before).await?;
        let extra = {
            let index = self.vfile_index.read().await;
            let mut by_uri = HashMap::new();
            for (uri, mapping) in index.iter() {
                if by_uri.insert(uri_key(uri.as_str()), mapping).is_some() {
                    return Err(tower_lsp::jsonrpc::Error::invalid_params(
                        "Provider source identities are ambiguous. Refresh the source scope and request the complete action again.",
                    ));
                }
            }
            let mut extra = Vec::new();
            for uri in action_input_uris(original).unwrap_or_default() {
                if let Some(mapping) = by_uri.get(&uri_key(&uri)) {
                    extra.extend(action_source_proofs(mapping)?.iter().cloned());
                }
            }
            extra
        };
        let extra = self.bind_action_source_rows(extra).await?;
        let mut merged = std::collections::BTreeMap::<String, ProviderSourceProof>::new();
        for proof in before.iter().chain(extra.iter()) {
            if merged
                .get(proof.uri.as_str())
                .is_some_and(|old| old != proof)
            {
                return Err(tower_lsp::jsonrpc::Error::invalid_params(
                    "Provider source revisions changed while reading the complete action. Preserve drafts and prepare again.",
                ));
            }
            merged.insert(proof.uri.to_string(), proof.clone());
        }
        let proofs = Arc::new(merged.into_values().collect::<Vec<_>>());
        self.require_action_sources(&proofs).await?;
        Ok(proofs)
    }
    async fn require_action_sources(&self, sources: &[ProviderSourceProof]) -> Result<()> {
        let documents = self.documents.read().await;
        for proof in sources {
            let document = documents.get(&proof.uri);
            if document.map(|document| document.source_revision) != proof.open_revision
                || document.is_some_and(|document| {
                    document.preparing || document.source_hash.as_ref() != Some(&proof.hash)
                })
            {
                return Err(tower_lsp::jsonrpc::Error::invalid_params(format!(
                    "A captured provider source changed or is still being prepared: {}. Preserve unsaved drafts, refresh the source or project entry from the intended sources, and request the action again.",
                    proof.uri
                )));
            }
        }
        Ok(())
    }
    pub(super) async fn provider_code_actions(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        let revision = self.provider_source_revision().await;
        if document_wide_source_request(&params.context) {
            let mut targets = {
                let index = self.vfile_index.read().await;
                let associated = index
                    .iter()
                    .filter(|(_, mapping)| {
                        provider_target_contributes(mapping, &params.text_document.uri)
                    })
                    .collect::<Vec<_>>();
                let project = associated
                    .iter()
                    .any(|(_, mapping)| mapping.position_map.is_project());
                associated
                    .into_iter()
                    .filter(|(_, mapping)| !project || mapping.position_map.is_project())
                    .map(|(uri, mapping)| {
                        let end = crate::project_positions::position(
                            &mapping.provider_source,
                            mapping.provider_source.len(),
                        )
                        .expect("complete UTF-8 child source");
                        Ok(ProviderActionTarget {
                            uri: uri.clone(),
                            context: mapping.hick_uri.clone(),
                            language: mapping.language_id.clone(),
                            range: Range::new(Position::new(0, 0), Position::new(end.0, end.1)),
                            label: Some(provider_output_label(uri, &mapping.root_uri)),
                            source_proofs: action_source_proofs(mapping)?,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?
            };
            self.bind_action_sources(&mut targets).await?;
            let proofs = merge_action_source_proofs(&targets)?;
            targets.sort_by(|a, b| a.uri.as_str().cmp(b.uri.as_str()));
            if targets.len() > 512 {
                return Err(tower_lsp::jsonrpc::Error::invalid_params(
                    "This document has too many complete provider outputs for one fix review. Open a narrower source or output and request fixes again.",
                ));
            }
            let mut labels = HashMap::<String, usize>::new();
            for target in &targets {
                if let Some(label) = &target.label {
                    *labels.entry(label.clone()).or_default() += 1;
                }
            }
            for target in &mut targets {
                if target.label.as_ref().is_some_and(|label| labels[label] > 1) {
                    let source = target.context.as_str();
                    target.label = target
                        .label
                        .as_ref()
                        .map(|label| format!("{source} → {label}"));
                }
            }
            let qualify = targets.len() > 1;
            let mut actions = Vec::new();
            for mut target in targets {
                if !qualify {
                    target.label = None;
                }
                if let Some(next) = self
                    .provider_code_actions_target(&params, &revision, target)
                    .await?
                {
                    if actions.len() + next.len() > 512 {
                        return Err(tower_lsp::jsonrpc::Error::invalid_params(
                            "The complete provider fix list exceeds the review budget. Choose a narrower source or provider and request fixes again; no partial list was returned.",
                        ));
                    }
                    actions.extend(next);
                }
            }
            self.require_provider_revision(&revision).await?;
            self.require_action_sources(&proofs).await?;
            return Ok((!actions.is_empty()).then_some(actions));
        }
        let Some((uri, language, start)) = self
            .virtual_target(&params.text_document.uri, params.range.start)
            .await
        else {
            return Ok(None);
        };
        let (end, source_proofs) = {
            let index = self.vfile_index.read().await;
            let mapping = index.get(&uri);
            let end = mapping
                .and_then(|mapping| {
                    mapping
                        .position_map
                        .to_virtual(params.range.end.line, params.range.end.character)
                })
                .unwrap_or((start.line, start.character));
            let sources = mapping
                .map(action_source_proofs)
                .transpose()?
                .unwrap_or_else(|| Arc::new(vec![]));
            (end, sources)
        };
        let mut target = ProviderActionTarget {
            context: params.text_document.uri.clone(),
            uri,
            language,
            range: Range::new(start, Position::new(end.0, end.1)),
            label: None,
            source_proofs,
        };
        self.bind_action_sources(std::slice::from_mut(&mut target))
            .await?;
        self.provider_code_actions_target(&params, &revision, target)
            .await
    }
    async fn provider_code_actions_target(
        &self,
        params: &CodeActionParams,
        revision: &str,
        target: ProviderActionTarget,
    ) -> Result<Option<CodeActionResponse>> {
        let ProviderActionTarget {
            uri: vf_uri,
            context: _,
            language,
            range,
            label,
            source_proofs,
        } = target;
        let start = range.start;
        let end = (range.end.line, range.end.character);
        let diagnostics = {
            let index = self.vfile_index.read().await;
            index
                .get(&vf_uri)
                .map(|mapping| provider_action_diagnostics(&mapping.provider_diagnostics, range))
                .unwrap_or_default()
        };
        let Some(root) = self.root_of(&vf_uri).await else {
            return Ok(None);
        };
        let (handle, resolve, refactor_reads) = {
            let dispatcher = self.dispatcher.lock().await;
            let Ok(handle) = dispatcher.get_child(&language, &root) else {
                return Ok(None);
            };
            let resolve = dispatcher
                .server_capabilities(&language, &root)
                .and_then(|value| value.pointer("/codeActionProvider/resolveProvider"))
                .and_then(Value::as_bool)
                == Some(true);
            let refactor_reads = dispatcher.server_capabilities(&language, &root)
                .and_then(|value| value.pointer("/executeCommandProvider/commands"))
                .and_then(Value::as_array)
                .is_some_and(|commands| commands.iter().any(|value| value.as_str() == Some("typescript.tsserverRequest")));
            (handle.clone(), resolve, refactor_reads)
        };
        self.require_provider_revision(revision).await?;
        self.require_action_sources(&source_proofs).await?;
        let result=handle.request("textDocument/codeAction",json!({"textDocument":{"uri":vf_uri},"range":{"start":start,"end":{"line":end.0,"character":end.1}},"context":provider_action_context(&params.context,diagnostics)})).await.map_err(child_request_error)?;
        self.require_provider_revision(revision).await?;
        self.require_action_sources(&source_proofs).await?;
        if result.is_null() {
            return Ok(None);
        }
        let originals = result.as_array().ok_or_else(|| {
            tower_lsp::jsonrpc::Error::invalid_params("Malformed provider action list")
        })?;
        let index = self.index_snapshot().await;
        self.require_provider_revision(revision).await?;
        let workspace_root = self.workspace_root.read().await.clone();
        let mut translated =
            translate_provider_edits(result.clone(), &index, workspace_root.as_ref())
                .map_err(tower_lsp::jsonrpc::Error::invalid_params)?;
        let entries = translated.as_array_mut().expect("translated action list");
        let mut action_proofs = Vec::new();
        for original in originals {
            action_proofs.push(self.extend_action_sources(original, &source_proofs).await?);
        }
        self.require_provider_revision(revision).await?;
        let mut held = self.prepared_actions.lock().await;
        for ((original, action), proofs) in
            originals.iter().zip(entries.iter_mut()).zip(action_proofs)
        {
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
            let display_title = label.as_ref().and_then(|label| {
                original
                    .get("title")
                    .and_then(Value::as_str)
                    .map(|title| format!("{title} · {label}"))
            });
            let token = held.values.insert(
                Instant::now(),
                revision.to_string(),
                PreparedAction {
                    original: original.clone(),
                    handle: handle.clone(),
                    resolve,
                    refactor_query: crate::provider_refactors::query(original, &vf_uri, refactor_reads),
                    already_mapped: false,
                    display_title: display_title.clone(),
                    source_proofs: proofs.clone(),
                },
            );
            let object = action.as_object_mut().ok_or_else(|| {
                tower_lsp::jsonrpc::Error::invalid_params("Malformed provider action")
            })?;
            data["hickory_action"] = json!(token);
            data["hickory_sources"] = action_source_data(&proofs);
            object.insert("data".into(), data);
            if let Some(title) = display_title {
                object.insert("title".into(), json!(title));
            }
        }
        drop(held);
        self.require_provider_revision(revision).await?;
        self.require_action_sources(&source_proofs).await?;
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

A held TypeScript refactoring may select the advertised edits-only query adapter.
It bypasses command execution and lazy command resolution, then rechecks the
original graph and source proofs around the read. The resulting complete edit
rejoins the same acquisition/projection path as ordinary resolved actions; a new
file remains only an ordered resource proposal. Unknown commands keep their
existing explicit refusal.

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
        self.require_action_sources(&prepared.source_proofs).await?;
        if prepared.already_mapped {
            let mut action=prepared.original;
            let mut data=action_discovery_data(action.get("data"));
            data["hickory_action"]=json!(token);data["hickory_sources"]=action_source_data(&prepared.source_proofs);action["data"]=data;
            self.require_provider_revision(&revision).await?;self.require_action_sources(&prepared.source_proofs).await?;
            return serde_json::from_value(action).map_err(|_|tower_lsp::jsonrpc::Error::invalid_params("Malformed held source-bound action"));
        }
        let source_proofs = prepared.source_proofs;
        let display_title = prepared.display_title;
        let mut original = prepared.original;
        if let Some(query) = prepared.refactor_query {
            self.require_provider_revision(&revision).await?;
            self.require_action_sources(&source_proofs).await?;
            let reading = prepared.handle.request("workspace/executeCommand", query).await.map_err(child_request_error)?;
            self.require_provider_revision(&revision).await?;
            self.require_action_sources(&source_proofs).await?;
            let edit = crate::provider_refactors::edits(&reading).map_err(tower_lsp::jsonrpc::Error::invalid_params)?;
            let object = original.as_object_mut().expect("held literal refactor action");
            object.remove("command");
            object.insert("edit".into(), edit);
        } else if prepared.resolve {
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
        let source_proofs = self
            .extend_action_sources(&original, &source_proofs)
            .await?;
        let index = self.index_snapshot().await;
        self.require_provider_revision(&revision).await?;
        self.require_action_sources(&source_proofs).await?;
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
        data["hickory_sources"] = action_source_data(&source_proofs);
        action["data"] = data;
        if let Some(title) = display_title {
            action["title"] = json!(title);
        }
        self.require_provider_revision(&revision).await?;
        self.require_action_sources(&source_proofs).await?;
        serde_json::from_value(action).map_err(|error| {
            tower_lsp::jsonrpc::Error::invalid_params(format!(
                "Malformed resolved provider action: {error}"
            ))
        })
    }
}
</ui:copy>

## Discover unavailable source without inventing a position map

An unknown child target has no authored range authority. For an edit-only action, retain only workspace-local file identities as an explicit acquisition request, with no edit or command. The host must establish listed writable ownership, synchronize text, and query a new action; generated, external and resource-operation paths never gain an identity map by this mechanism. The held provider original remains unchanged. Only backend-owned acquisition and coordinate-source evidence survive alongside the opaque token; arbitrary child data is not exposed as host metadata. `hickory_sources` contains sorted unique `{uri, source_hash}` records for the exact authored bytes behind the origin, contributors and known affected preimages. It is coordinate evidence, not write permission. Hosts compare every record with their frozen source snapshots and refuse unproved text coordinates; another window cannot rebind the action to different draft bytes. Unknown targets still require acquisition and a fresh complete query.

<ui:copy id="provider-action-unmapped-source-discovery">
fn action_input_uris(original: &Value) -> Option<Vec<String>> {
    if original.get("command").is_some_and(|value|!value.is_null()) || original.get("disabled").is_some_and(|value|!value.is_null()) {
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
    Some(uris)
}
fn action_source_data(proofs: &[ProviderSourceProof]) -> Value {
    json!(
        proofs
            .iter()
            .map(|proof| json!({"uri":proof.uri,"source_hash":proof.hash}))
            .collect::<Vec<_>>()
    )
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
    let uris = action_input_uris(original)?;
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
</ui:copy>

## Refuse unavailable identities outside discovery scope

These focused cases require an acquisition-only response without an edit, and refuse external identities, resource operations, malformed lists and command-bearing actions. The separate child-protocol witness supplies hostile metadata to verify it cannot become backend acquisition authority.

<ui:copy id="provider-action-unmapped-source-discovery-witness">
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
</ui:copy>

<ui:file path="crates/hick-lsp/src/backend/actions.rs"><ui:paste select="#provider-action-source-context"/><ui:paste select="#provider-action-source-proof-witness"/><ui:paste select="#bound-source-edit-queries"/><ui:paste select="#bound-source-edit-witness"/><ui:paste select="#provider-action-graph-revision"/><ui:paste select="#provider-action-query"/><ui:paste select="#provider-action-resolve"/><ui:paste select="#provider-action-unmapped-source-discovery"/><ui:paste select="#provider-action-unmapped-source-discovery-witness"/></ui:file>

</ui:doc>
