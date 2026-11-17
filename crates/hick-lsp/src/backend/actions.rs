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
#[derive(Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedSourceCapture {source_hash:String,source_revision:String}
#[derive(serde::Deserialize)]
pub struct BoundRenameParams {#[serde(flatten)]params:RenameParams,expected:ExpectedSourceCapture}
#[derive(serde::Deserialize)]
pub struct BoundFormattingParams {#[serde(flatten)]params:DocumentFormattingParams,expected:ExpectedSourceCapture}
struct BoundSourceSnapshot {
 maps:HashMap<Url,VFileMapping>,
 documents:HashMap<Url,(crate::source_identity::SourceRevision,Option<String>,bool)>,
}
impl BoundSourceSnapshot {
 fn index(&self)->HashMap<String,(Url,PositionMap)>{self.maps.iter().map(|(uri,map)|(uri_key(uri.as_str()),(map.hick_uri.clone(),map.position_map.clone()))).collect()}
 fn expected(&self,uri:&Url,expected:&ExpectedSourceCapture)->Result<()> {
  let valid=self.documents.get(uri).is_some_and(|(revision,hash,preparing)|{
   !preparing&&hash.as_deref()==Some(expected.source_hash.as_str())&&crate::source_identity::source_graph_fingerprint(std::iter::once((uri.as_str(),*revision)))==expected.source_revision
  });
  if !valid{return Err(tower_lsp::jsonrpc::Error::invalid_params("Source capture changed or is still preparing. Synchronize the intended source and prepare again."));}Ok(())
 }
 fn sources(&self,maps:&[Url])->Result<Arc<Vec<ProviderSourceProof>>> {
  let mut rows=std::collections::BTreeMap::<String,ProviderSourceProof>::new();
  for uri in maps {
   let mapping=self.maps.get(uri).ok_or_else(||tower_lsp::jsonrpc::Error::invalid_params("Provider mapping changed. Prepare again."))?;
   for mut proof in action_source_proofs(mapping)?.iter().cloned(){
    if let Some((revision,hash,preparing))=self.documents.get(&proof.uri){
     if *preparing||hash.as_deref()!=Some(proof.hash.as_str()){return Err(tower_lsp::jsonrpc::Error::invalid_params("Captured provider source changed or is still preparing. Prepare again."));}
     proof.open_revision=Some(*revision);
    }
    if rows.get(proof.uri.as_str()).is_some_and(|old|old!=&proof){return Err(tower_lsp::jsonrpc::Error::invalid_params("Provider mappings captured conflicting source bytes. Prepare again."));}
    rows.insert(proof.uri.to_string(),proof);
   }
  }
  Ok(Arc::new(rows.into_values().collect()))
 }
 fn affected_maps(&self,original:&Value,targets:&[Url])->Result<Vec<Url>> {
  let mut selected=targets.to_vec();
  for raw in action_input_uris(original).ok_or_else(||tower_lsp::jsonrpc::Error::invalid_params("Malformed source-bound provider edits."))? {
   for (uri,map) in &self.maps {
    if uri_key(uri.as_str())==uri_key(&raw)||map.hick_uri.as_str()==raw {selected.push(uri.clone());}
   }
  }
  selected.sort_by(|a,b|a.as_str().cmp(b.as_str()));selected.dedup();Ok(selected)
 }
}
impl HickBackend {
 async fn bound_snapshot(&self)->BoundSourceSnapshot {
  let maps=self.vfile_index.read().await.clone();
  let documents=self.documents.read().await.iter().map(|(uri,doc)|(uri.clone(),(doc.source_revision,doc.source_hash.clone(),doc.preparing))).collect();
  BoundSourceSnapshot{maps,documents}
 }
 async fn require_bound_capture(&self,uri:&Url,expected:&ExpectedSourceCapture)->Result<()> {
  let documents=self.documents.read().await;
  let valid=documents.get(uri).is_some_and(|doc|!doc.preparing&&doc.source_hash.as_deref()==Some(expected.source_hash.as_str())&&crate::source_identity::source_graph_fingerprint(std::iter::once((uri.as_str(),doc.source_revision)))==expected.source_revision);
  if !valid{return Err(tower_lsp::jsonrpc::Error::invalid_params("Source capture changed or is still preparing. Synchronize the intended source and prepare again."));}Ok(())
 }
 async fn require_bound_maps(&self,snapshot:&BoundSourceSnapshot,targets:&[Url])->Result<()> {
  let current=self.vfile_index.read().await;
  if targets.iter().any(|uri|snapshot.maps.get(uri).map(|map|map.source_revision)!=current.get(uri).map(|map|map.source_revision)) {return Err(tower_lsp::jsonrpc::Error::invalid_params("Provider ownership changed. Prepare again."));}Ok(())
 }
 async fn bound_child(&self,mapping:&VFileMapping)->Result<ChildLspHandle>{
  let dispatcher=self.dispatcher.lock().await;
  dispatcher.get_child(&mapping.language_id,&mapping.root_uri).cloned().map_err(|_|tower_lsp::jsonrpc::Error::invalid_params("The required provider is unavailable. No partial source change was prepared."))
 }
 async fn finish_bound_edit(&self,title:&str,edit:Value,snapshot:&BoundSourceSnapshot,targets:&[Url],handle:ChildLspHandle,origin:&Url,expected:&ExpectedSourceCapture)->Result<Value>{
  let original=json!({"title":title,"edit":edit});let targets=snapshot.affected_maps(&original,targets)?;
  let proofs=snapshot.sources(&targets)?;self.require_action_sources(&proofs).await?;self.require_bound_maps(snapshot,&targets).await?;self.require_bound_capture(origin,expected).await?;
  let index=snapshot.index();let root=self.workspace_root.read().await.clone();
  let mut action=bound_project_edit(original,&index,root.as_ref())?;
  self.require_action_sources(&proofs).await?;self.require_bound_maps(snapshot,&targets).await?;self.require_bound_capture(origin,expected).await?;
  let revision=self.provider_source_revision().await;
  let data=action_discovery_data(action.get("data"));
  let token=self.prepared_actions.lock().await.values.insert(Instant::now(),revision,PreparedAction{original:action.clone(),handle,resolve:false,refactor_query:None,already_mapped:true,display_title:None,source_proofs:proofs.clone()});
  action["data"]=data;action["data"]["hickory_action"]=json!(token);action["data"]["hickory_sources"]=action_source_data(&proofs);action["source_files_modified"]=json!(false);
  self.require_action_sources(&proofs).await?;self.require_bound_maps(snapshot,&targets).await?;self.require_bound_capture(origin,expected).await?;Ok(action)
 }
 pub async fn source_bound_rename(&self,params:BoundRenameParams)->Result<Value>{
  let origin=params.params.text_document_position.text_document.uri.clone();let snapshot=self.bound_snapshot().await;snapshot.expected(&origin,&params.expected)?;
  let Some((target,_,position))=self.virtual_target(&origin,params.params.text_document_position.position).await else{return Err(tower_lsp::jsonrpc::Error::invalid_params("This source position has no unambiguous provider rename target."));};
  let mapping=snapshot.maps.get(&target).ok_or_else(||tower_lsp::jsonrpc::Error::invalid_params("Provider mapping changed. Prepare again."))?;
  let sources=snapshot.sources(std::slice::from_ref(&target))?;self.require_action_sources(&sources).await?;self.require_bound_maps(&snapshot,std::slice::from_ref(&target)).await?;self.require_bound_capture(&origin,&params.expected).await?;
  let handle=self.bound_child(mapping).await?;
  let edit=handle.request("textDocument/rename",json!({"textDocument":{"uri":target},"position":position,"newName":params.params.new_name})).await.map_err(child_request_error)?;
  let edit=if edit.is_null(){json!({"documentChanges":[]})}else{edit};
  self.finish_bound_edit("Rename",edit,&snapshot,&[target],handle,&origin,&params.expected).await
 }
 pub async fn source_bound_formatting(&self,params:BoundFormattingParams)->Result<Value>{
  let origin=params.params.text_document.uri;let snapshot=self.bound_snapshot().await;snapshot.expected(&origin,&params.expected)?;
  let associated=snapshot.maps.iter().filter(|(_,map)|provider_target_contributes(map,&origin)).collect::<Vec<_>>();let project=associated.iter().any(|(_,map)|map.position_map.is_project());
  let mut targets=associated.into_iter().filter(|(_,map)|!project||map.position_map.is_project()).map(|(uri,_)|uri.clone()).collect::<Vec<_>>();targets.sort_by(|a,b|a.as_str().cmp(b.as_str()));
  if targets.is_empty()||targets.len()>512{return Err(tower_lsp::jsonrpc::Error::invalid_params("Complete formatting requires between one and 512 unambiguous provider outputs."));}
  let sources=snapshot.sources(&targets)?;self.require_action_sources(&sources).await?;self.require_bound_maps(&snapshot,&targets).await?;self.require_bound_capture(&origin,&params.expected).await?;
  let mut changes=Vec::new();let mut first=None;
  for target in &targets {
   self.require_action_sources(&sources).await?;self.require_bound_maps(&snapshot,&targets).await?;self.require_bound_capture(&origin,&params.expected).await?;
   let handle=self.bound_child(&snapshot.maps[target]).await?;if first.is_none(){first=Some(handle.clone());}
   let edits=handle.request("textDocument/formatting",json!({"textDocument":{"uri":target},"options":params.params.options})).await.map_err(child_request_error)?;
   if edits.is_null(){continue;}if !edits.is_array(){return Err(tower_lsp::jsonrpc::Error::invalid_params("A formatter returned malformed edits. No partial formatting was prepared."));}
   changes.push(json!({"textDocument":{"uri":target,"version":null},"edits":edits}));
  }
  self.finish_bound_edit("Format document",json!({"documentChanges":changes}),&snapshot,&targets,first.expect("nonempty complete formatting"),&origin,&params.expected).await
 }
}
fn bound_project_edit(original:Value,index:&HashMap<String,(Url,PositionMap)>,root:Option<&Url>)->Result<Value>{
 if let Some(discovery)=discover_action_sources(&original,index,root){return Ok(discovery);}
 let value=translate_provider_edits(json!([original]),index,root).map_err(tower_lsp::jsonrpc::Error::invalid_params)?;
 let action=value.as_array().and_then(|rows|rows.first()).cloned().ok_or_else(||tower_lsp::jsonrpc::Error::invalid_params("Malformed complete source-bound edit."))?;
 if action.get("disabled").is_some_and(|value|!value.is_null()){return Err(tower_lsp::jsonrpc::Error::invalid_params("A complete provider edit cannot be projected safely. No partial change was prepared."));}Ok(action)
}
#[cfg(test)]
mod bound_edit_tests {
 use super::*;
 fn expected(uri:&Url,entry:&DocEntry)->ExpectedSourceCapture{ExpectedSourceCapture{source_hash:entry.source_hash.clone().unwrap(),source_revision:crate::source_identity::source_graph_fingerprint(std::iter::once((uri.as_str(),entry.source_revision)))}}
 fn mapping(uri:&Url,text:&str)->VFileMapping{VFileMapping{source_revision:crate::source_identity::SourceRevision::fresh(),hick_uri:uri.clone(),language_id:"python".into(),root_uri:"file:///workspace/".into(),position_map:PositionMap::identity(10),provider_diagnostics:vec![],provider_source:text.into(),provider_version:Some(1),primary_source_hash:diagnostic_source_digest(text)}}
 #[tokio::test]
 async fn exact_origin_capture_rejects_equal_byte_aba_and_preparing_without_dispatch(){
  let (service,_)=tower_lsp::LspService::new(HickBackend::new);let backend=service.inner();let uri=Url::parse("file:///workspace/a.py").unwrap();
  let mut entry=DocEntry::empty();entry.source_hash=Some(diagnostic_source_digest("same"));let receipt=expected(&uri,&entry);backend.documents.write().await.insert(uri.clone(),entry);
  backend.require_bound_capture(&uri,&receipt).await.unwrap();
  backend.documents.write().await.get_mut(&uri).unwrap().source_revision=crate::source_identity::SourceRevision::fresh();
  assert!(backend.require_bound_capture(&uri,&receipt).await.is_err());
  let params=BoundRenameParams{params:serde_json::from_value(json!({"textDocument":{"uri":uri},"position":{"line":0,"character":0},"newName":"later"})).unwrap(),expected:receipt};
  assert!(backend.source_bound_rename(params).await.unwrap_err().message.contains("capture changed"));
 }
 #[tokio::test]
 async fn touched_source_and_map_lifetimes_never_rebind_while_unrelated_preparation_remains_usable(){
  let (service,_)=tower_lsp::LspService::new(HickBackend::new);let backend=service.inner();let uri=Url::parse("file:///workspace/a.py").unwrap();let busy=Url::parse("file:///workspace/unrelated.md").unwrap();
  let mut entry=DocEntry::empty();entry.source_hash=Some(diagnostic_source_digest("same"));let receipt=expected(&uri,&entry);
  let mut preparing=DocEntry::empty();preparing.preparing=true;
  backend.documents.write().await.extend([(uri.clone(),entry),(busy,preparing)]);backend.vfile_index.write().await.insert(uri.clone(),mapping(&uri,"same"));
  let snapshot=backend.bound_snapshot().await;snapshot.expected(&uri,&receipt).unwrap();let proofs=snapshot.sources(std::slice::from_ref(&uri)).unwrap();backend.require_action_sources(&proofs).await.unwrap();backend.require_bound_maps(&snapshot,std::slice::from_ref(&uri)).await.unwrap();
  backend.documents.write().await.get_mut(&uri).unwrap().source_revision=crate::source_identity::SourceRevision::fresh();assert!(backend.require_action_sources(&proofs).await.is_err());
  backend.vfile_index.write().await.get_mut(&uri).unwrap().source_revision=crate::source_identity::SourceRevision::fresh();assert!(backend.require_bound_maps(&snapshot,std::slice::from_ref(&uri)).await.is_err());
 }
 #[tokio::test]
 async fn affected_preimages_come_from_the_original_snapshot_not_a_later_equal_byte_owner(){
  let (service,_)=tower_lsp::LspService::new(HickBackend::new);let backend=service.inner();let a=Url::parse("file:///workspace/a.py").unwrap();let b=Url::parse("file:///workspace/b.py").unwrap();
  for uri in [&a,&b]{let mut doc=DocEntry::empty();doc.source_hash=Some(diagnostic_source_digest("same"));backend.documents.write().await.insert(uri.clone(),doc);backend.vfile_index.write().await.insert(uri.clone(),mapping(uri,"same"));}
  let snapshot=backend.bound_snapshot().await;let key=b.as_str();let original=json!({"edit":{"changes":{key:[]}}});let targets=snapshot.affected_maps(&original,std::slice::from_ref(&a)).unwrap();assert_eq!(targets.len(),2);
  backend.documents.write().await.get_mut(&b).unwrap().source_revision=crate::source_identity::SourceRevision::fresh();let proofs=snapshot.sources(&targets).unwrap();assert!(backend.require_action_sources(&proofs).await.is_err());
 }
 #[test]
 fn complete_formatting_projection_refuses_a_valid_subset_when_any_output_is_unmappable(){
  let uri=Url::parse("hick:///a.py").unwrap();let map=PositionMap::build(&[crate::virtual_file::VirtualSegment{text:"x".into(),source_span:Some(hick_lang::SourceSpan::new(0,1,1,0)),source_line:1,source_column:0}]);let index=HashMap::from([(uri_key(uri.as_str()),(uri.clone(),map))]);let key=uri.as_str();
  let edit=json!({"title":"Format","edit":{"changes":{key:[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},"newText":"valid"},{"range":{"start":{"line":900,"character":0},"end":{"line":900,"character":1}},"newText":"invalid"}]}}});
  assert!(bound_project_edit(edit,&index,None).is_err());
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
