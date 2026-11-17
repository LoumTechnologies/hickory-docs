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
fn document_wide_fix_request(context: &CodeActionContext) -> bool {
    context.only.as_ref().is_some_and(|only| {
        !only.is_empty()
            && only.iter().all(|kind| {
                kind.as_str() == "source.fixAll" || kind.as_str().starts_with("source.fixAll.")
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
                mapping.hick_uri.join(document).ok().is_some_and(|mut uri| {
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
        if document.contains(['#', '?', '%', '\\']) {
            return Err(tower_lsp::jsonrpc::Error::invalid_params(format!(
                "The provider cannot safely project this contributor filename yet: {document}. Preserve its draft and choose a source without URI-reserved filename characters before requesting complete fixes."
            )));
        }
        let mut uri=mapping.hick_uri.join(document).map_err(|_|tower_lsp::jsonrpc::Error::invalid_params("A captured contributor has no safe source address. Refresh the project entry and request fixes again."))?;
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
    fn complete_file_routing_requires_only_the_explicit_fix_all_family() {
        for (only, expected) in [
            (json!(["source.fixAll"]), true),
            (json!(["source.fixAll.python"]), true),
            (json!([]), false),
            (json!(["source.fixAllWrong"]), false),
            (json!(["source"]), false),
            (json!(["quickfix", "source.fixAll"]), false),
        ] {
            let context: CodeActionContext =
                serde_json::from_value(json!({"diagnostics":[],"only":only})).unwrap();
            assert_eq!(document_wide_fix_request(&context), expected);
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
        if document_wide_fix_request(&params.context) {
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
        let source_proofs = prepared.source_proofs;
        let display_title = prepared.display_title;
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
