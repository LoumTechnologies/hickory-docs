<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Rebuild the project graph from the actual open drafts

For maintainers connecting unsaved contributors to one coherent language-service graph.

The compiler already defines closure loading, inventory, evaluation, selected endpoints and byte origins. Supply it one immutable reader snapshot containing current open text, with confined disk fallback for unopened sources. Do not append per-block output or infer dependencies in a renderer. Literal source filenames use encoded path segments; `%` and `#` never become URI escapes or fragments. Portable compiler-invalid paths (`?`, backslash and non-ASCII spellings) still fail explicitly at the shared compiler boundary. Reading a draft grants no filesystem write permission.

## Capture the closure through the existing compiler reader

The entry buffer remains authoritative. Every recursive closure read first consults the captured open-document map at its exact source identity, preserving endpoint query scope. Newly added dirty inventory entries participate; removed entries do not borrow stale disk or editor fragments. Every output and origin is produced by the same complete compiler evaluation.

<ui:copy id="project-overlay-source-reader">
/// Treat compiler-validated source names as literal path segments, never URI syntax.
pub(crate) fn source_document_uri(primary:&tower_lsp::lsp_types::Url,document:&str)->Result<tower_lsp::lsp_types::Url,String>{
 hick_lang::validate_project_path(document)?;
 let mut uri=primary.clone();
 {let mut path=uri.path_segments_mut().map_err(|_|"Source identity has no hierarchical path")?;path.pop_if_empty();path.pop();for part in document.split('/'){path.push(part);}}
 uri.set_fragment(None);uri.set_query(primary.query());Ok(uri)
}
/// Evaluate one complete closure from captured open text, falling back to confined disk sources.
pub(crate) fn state_for_uri_with_overlays(uri:&tower_lsp::lsp_types::Url,source:&str,overlays:&BTreeMap<tower_lsp::lsp_types::Url,String>)->Result<crate::document::HickDocumentState,hick_lang::ParseError>{
 let doc=hick_lang::parse(source)?;
 if doc.find_tags("project").is_empty(){return crate::document::HickDocumentState::from_source(source);}
 let failure=|message|hick_lang::ParseError::Syntax{line:1,message};
 let path=uri.to_file_path().map_err(|_|failure("project closure requires a file identity".into()))?;
 let root=path.parent().ok_or_else(||failure("project has no directory".into()))?;
 let entry=path.file_name().and_then(|s|s.to_str()).ok_or_else(||failure("entry name is not UTF-8".into()))?;
 let sources=hick_lang::load_project_sources(entry,&|name|{
  if name==entry{return Ok(source.into());}
  let address=source_document_uri(uri,name)?;
  if let Some(text)=overlays.get(&address){return Ok(text.clone());}
  let source_path=root.join(name);
  if !std::fs::symlink_metadata(&source_path).map_err(|e|e.to_string())?.is_file(){return Err(format!("Markdown source is not a regular file: {name}"));}
  let path=source_path.canonicalize().map_err(|e|e.to_string())?;
  if !path.starts_with(root.canonicalize().map_err(|e|e.to_string())?){return Err(format!("Markdown closure escapes project directory: {name}"));}
  std::fs::read_to_string(path).map_err(|e|e.to_string())
 }).map_err(failure)?;
 let selection=uri.query_pairs().find(|(name,_)|name=="hick-project").map(|(_,value)|value.into_owned()).unwrap_or_else(||"new".into());
 let endpoint=ProjectEndpoint::parse(&selection).map_err(failure)?;
 crate::document::HickDocumentState::from_project_at(entry,&sources,&ProjectTree::default(),&endpoint)
}
</ui:copy>

## Prove closure replacement and literal source identity

The core witnesses supply two dirty drafts simultaneously, then replace the inventory with a new unsaved contributor. They inspect complete compiler output and provenance, not a hand-built merged file. Literal percent/hash names round-trip through both source proofs and edit projection; compiler-invalid portable paths remain explicit errors.

<ui:copy id="project-overlay-core-witness">
#[cfg(test)]
mod overlay_graph_tests {
 use super::*;
 use tower_lsp::lsp_types::Url;
 fn copy(id:&str,value:&str)->String{format!("\x3chick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\x3chick:copy id=\"{id}\">value = {value}\n\x3c/hick:copy>\x3c/hick:doc>")}
 fn entry(sources:&str,pastes:&str)->String{format!("\x3chick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\x3chick:project sources=\"{sources}\"/>\x3chick:file path=\"out.py\">{pastes}\x3c/hick:file>\x3c/hick:doc>")}
 #[test]
 fn complete_loader_uses_all_dirty_drafts_and_replaces_changed_inventory(){
  let dir=tempfile::tempdir().unwrap();let uri=Url::from_file_path(dir.path().join("project.md")).unwrap();
  std::fs::write(dir.path().join("a.md"),copy("a","1")).unwrap();std::fs::write(dir.path().join("b.md"),copy("b","2")).unwrap();
  let a=source_document_uri(&uri,"a.md").unwrap();let b=source_document_uri(&uri,"b.md").unwrap();let c=source_document_uri(&uri,"new.md").unwrap();
  let drafts=BTreeMap::from([(a,copy("a","11")),(b,copy("b","22")),(c,copy("c","33"))]);
  let source=entry("a.md b.md","\x3chick:paste select=\"#a\"/>\x3chick:paste select=\"#b\"/>");
  let state=state_for_uri_with_overlays(&uri,&source,&drafts).unwrap();assert_eq!(state.virtual_files[0].written_content(),"value = 11\nvalue = 22\n");assert!(!state.project_sources.contains_key("new.md"));
  let next=entry("b.md new.md","\x3chick:paste select=\"#b\"/>\x3chick:paste select=\"#c\"/>");let state=state_for_uri_with_overlays(&uri,&next,&drafts).unwrap();assert_eq!(state.virtual_files[0].written_content(),"value = 22\nvalue = 33\n");assert!(!state.project_sources.contains_key("a.md"));assert!(state.project_sources.contains_key("new.md"));
  assert_eq!(std::fs::read_to_string(dir.path().join("a.md")).unwrap(),copy("a","1"));assert!(!dir.path().join("new.md").exists());
 }
 #[test]
 fn literal_percent_hash_sources_project_to_exact_authored_uris_without_aliases(){
  let dir=tempfile::tempdir().unwrap();let uri=Url::from_file_path(dir.path().join("project.md")).unwrap();let name="literal%2F#.md";let source=entry(name,"\x3chick:paste select=\"#a\"/>");let address=source_document_uri(&uri,name).unwrap();
  assert!(address.as_str().contains("literal%252F%23.md"));assert_eq!(address.to_file_path().unwrap().file_name().unwrap(),name);
  let state=state_for_uri_with_overlays(&uri,&source,&BTreeMap::from([(address.clone(),copy("a","1"))])).unwrap();let map=state.project_maps["out.py"].clone();let child=Url::from_file_path(dir.path().join("out.py")).unwrap();let key=child.as_str();
  let index=std::collections::HashMap::from([(crate::backend::uri_key(key),(uri.clone(),map))]);
  let edit=serde_json::json!({"changes":{key:[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"newText":"fixed"}]}});
  let mapped=crate::backend::translate_provider_edits(edit,&index,None).unwrap();assert_eq!(mapped["documentChanges"][0]["textDocument"]["uri"],address.as_str());
  let selected=Url::parse("file:///workspace/project.md?hick-project=new").unwrap();assert_eq!(source_document_uri(&selected,name).unwrap().query(),selected.query());
 }
 #[test]
 fn compiler_invalid_portable_paths_refuse_instead_of_becoming_uri_syntax(){
  let uri=Url::parse("file:///workspace/project.md").unwrap();for name in ["question?.md","back\\slash.md","é.md","../escape.md"]{assert!(source_document_uri(&uri,name).is_err(),"{name}");}
 }
}
</ui:copy>

## Refresh actual affected roots, retain drafts and recover missing contributors

This bridge witness edits two copy-only documents while their root remains open, changes the root inventory to a newly opened unsaved source, and removes a dependency. Closing a dirty contributor restores its saved bytes; closing a never-saved required contributor retires the stale graph until that source is available again. Root text and source files are never rewritten by capability preparation.

<ui:copy id="project-overlay-backend-witness">
#[cfg(test)]
mod overlay_refresh_tests {
 use super::*;
 fn copy(id:&str,value:&str)->String{format!("\x3chick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\x3chick:copy id=\"{id}\">value = {value}\n\x3c/hick:copy>\x3c/hick:doc>")}
 fn root(sources:&str,ids:&[&str])->String{format!("\x3chick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\x3chick:project sources=\"{sources}\"/>\x3chick:file path=\"out.txt\">{}\x3c/hick:file>\x3c/hick:doc>",ids.iter().map(|id|format!("\x3chick:paste select=\"#{id}\"/>")).collect::<String>())}
 async fn output(backend:&HickBackend,uri:&Url)->String{backend.documents.read().await[uri].state.as_ref().unwrap().virtual_files[0].written_content()}
 #[tokio::test]
 async fn dirty_contributors_rebuild_complete_roots_and_inventory_without_replacing_root_text(){
  let dir=tempfile::tempdir().unwrap();let project=Url::from_file_path(dir.path().join("project.md")).unwrap();let a=Url::from_file_path(dir.path().join("a.md")).unwrap();let b=Url::from_file_path(dir.path().join("b.md")).unwrap();let c=crate::project::source_document_uri(&project,"literal%2F#.md").unwrap();
  let original=root("a.md b.md",&["a","b"]);std::fs::write(dir.path().join("project.md"),&original).unwrap();std::fs::write(dir.path().join("a.md"),copy("a","1")).unwrap();std::fs::write(dir.path().join("b.md"),copy("b","2")).unwrap();
  let (service,_)=tower_lsp::LspService::new(HickBackend::new);let backend=service.inner();backend.process_document(&project,&original).await;backend.process_document(&a,&copy("a","11")).await;backend.process_document(&b,&copy("b","22")).await;
  assert_eq!(output(backend,&project).await,"value = 11\nvalue = 22\n");assert_eq!(backend.documents.read().await[&project].source_text,original);
  backend.process_document(&c,&copy("c","33")).await;let changed=root("b.md literal%2F#.md",&["b","c"]);backend.process_document(&project,&changed).await;assert_eq!(output(backend,&project).await,"value = 22\nvalue = 33\n");
  let root_revision=backend.documents.read().await[&project].source_revision;backend.process_document(&a,&copy("a","111")).await;assert_eq!(backend.documents.read().await[&project].source_revision,root_revision,"removed inventory source must not refresh this root");
  backend.did_close(DidCloseTextDocumentParams{text_document:TextDocumentIdentifier{uri:b}}).await;assert_eq!(output(backend,&project).await,"value = 2\nvalue = 33\n");
  backend.did_close(DidCloseTextDocumentParams{text_document:TextDocumentIdentifier{uri:c.clone()}}).await;assert!(backend.documents.read().await[&project].state.is_none(),"missing required source must retire stale project capability");
  backend.process_document(&c,&copy("c","44")).await;assert_eq!(output(backend,&project).await,"value = 2\nvalue = 44\n");assert_eq!(backend.documents.read().await[&project].source_text,changed);
  assert_eq!(std::fs::read_to_string(dir.path().join("project.md")).unwrap(),original);assert_eq!(std::fs::read_to_string(dir.path().join("b.md")).unwrap(),copy("b","2"));assert!(!c.to_file_path().unwrap().exists());
 }
}
</ui:copy>

</ui:doc>
