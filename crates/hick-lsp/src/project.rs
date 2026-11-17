//! Native services select one complete project. Materialization is explicit and
//! creates a fresh private identity per selection; portable replay never calls it.
use hick_lang::{EntryKind, EvaluatedProject, ProjectTree};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectEndpoint {
    Old,
    New,
    ScopeOld(String),
    ScopeNew(String),
    LocalBefore(String),
    LocalAfter(String),
    /// File construction and the exact editor source that requested a launch.
    LatestFile {
        id: String,
        source: String,
    },
    LatestCode {
        path: String,
        code: String,
        source: String,
    },
}
impl ProjectEndpoint {
    pub fn parse(value: &str) -> Result<Self, String> {
        if let Some(json) = value.strip_prefix("latest-code:") {
            let (path, code, source): (String, String, String) =
                serde_json::from_str(json).map_err(|e| e.to_string())?;
            return Ok(Self::LatestCode { path, code, source });
        }
        if let Some(json) = value.strip_prefix("latest-file:") {
            let (id, source): (String, String) =
                serde_json::from_str(json).map_err(|e| e.to_string())?;
            return Ok(Self::LatestFile { id, source });
        }
        match value {
            "new" => Ok(Self::New),
            "old" => Ok(Self::Old),
            _ => {
                for (prefix, kind) in [
                    ("scope-old:", 0),
                    ("scope-new:", 1),
                    ("local-before:", 2),
                    ("local-after:", 3),
                ] {
                    if let Some(id) = value.strip_prefix(prefix) {
                        if id.is_empty() {
                            return Err("Project scope/state needs an identity".into());
                        }
                        return Ok(match kind {
                            0 => Self::ScopeOld(id.into()),
                            1 => Self::ScopeNew(id.into()),
                            2 => Self::LocalBefore(id.into()),
                            _ => Self::LocalAfter(id.into()),
                        });
                    }
                }
                Err(format!("Unknown project endpoint: {value}"))
            }
        }
    }
}
pub fn endpoint<'a>(
    project: &'a EvaluatedProject,
    side: &ProjectEndpoint,
) -> Result<&'a ProjectTree, String> {
    match side {
        ProjectEndpoint::LatestFile { .. } | ProjectEndpoint::LatestCode { .. } => {
            let (index, _) = debug_selection(project, side)?;
            Ok(&project.versions[index])
        }
        ProjectEndpoint::Old => Ok(&project.old),
        ProjectEndpoint::New => Ok(&project.new),
        ProjectEndpoint::ScopeOld(id) => project
            .scopes
            .get(id)
            .map(|s| &s.0)
            .ok_or_else(|| format!("Unknown atomic scope: {id}")),
        ProjectEndpoint::ScopeNew(id) => project
            .scopes
            .get(id)
            .map(|s| &s.1)
            .ok_or_else(|| format!("Unknown atomic scope: {id}")),
        ProjectEndpoint::LocalBefore(id) => project
            .local_states
            .get(id)
            .map(|s| &s.0)
            .ok_or_else(|| format!("unknown local state: {id}")),
        ProjectEndpoint::LocalAfter(id) => project
            .local_states
            .get(id)
            .map(|s| &s.1)
            .ok_or_else(|| format!("unknown local state: {id}")),
    }
}
/// Resolve the complete tree and revision-local filename for a clicked file.
pub fn latest_file(project: &EvaluatedProject, id: &str) -> Result<(usize, String), String> {
    let path = project
        .file_paths
        .get(id)
        .ok_or_else(|| format!("File construction {id} changed. Reload it before debugging."))?;
    let local = &project
        .local_states
        .get(id)
        .ok_or("Missing file construction")?
        .1;
    hick_lang::latest_debug_project(&project.versions, local, path)
}

pub fn debug_selection(
    project: &EvaluatedProject,
    side: &ProjectEndpoint,
) -> Result<(usize, String), String> {
    match side {
        ProjectEndpoint::LatestFile { id, .. } => latest_file(project, id),
        ProjectEndpoint::LatestCode { path, code, .. } => {
            let mut displayed = ProjectTree::default();
            let mut entry = hick_lang::ProjectEntry::file(code.as_bytes());
            // Recover lineage so unchanged renames can be followed.
            if let Some(file) = project.versions.iter().rev().find_map(|v| {
                v.get(path)
                    .filter(|e| e.kind == EntryKind::File && e.bytes.as_ref() == code.as_bytes())
            }) {
                entry.origins = file.origins.clone();
            }
            displayed.insert(path, entry)?;
            hick_lang::latest_debug_project(&project.versions, &displayed, path)
        }
        _ => Err("This endpoint is not an exact-source debug selection".into()),
    }
}

#[derive(Debug)]
pub struct MaterializedProject {
    directory: tempfile::TempDir,
    pub endpoint: ProjectEndpoint,
    files: Vec<PathBuf>,
}
impl MaterializedProject {
    pub fn root(&self) -> &Path {
        self.directory.path()
    }
    pub fn files(&self) -> Vec<PathBuf> {
        self.files.clone()
    }
}
pub fn materialize(
    project: &EvaluatedProject,
    side: ProjectEndpoint,
) -> Result<MaterializedProject, String> {
    let tree = endpoint(project, &side)?;
    if tree.entries().is_empty() {
        return Err("selected project is empty; no executable Old project exists".into());
    }
    let directory = tempfile::Builder::new()
        .prefix("hick-project-")
        .tempdir()
        .map_err(|e| e.to_string())?;
    let files = write_tree(directory.path(), tree)?;
    Ok(MaterializedProject {
        directory,
        endpoint: side,
        files,
    })
}
pub use hick_project_env::write_tree;

/// Exact byte mapping. A deleted local line has no endpoint mapping, and
/// repeated contributors are ambiguous rather than guessed at.
pub fn to_endpoint(
    tree: &ProjectTree,
    document: &str,
    source_byte: usize,
) -> Result<Option<(String, usize)>, String> {
    let mut found = None;
    for (path, entry) in tree.entries() {
        for origin in &entry.origins {
            if origin.document == document
                && origin.source.0 <= source_byte
                && source_byte < origin.source.1
            {
                if found.is_some() {
                    return Err(format!(
                        "ambiguous endpoint mapping for {document}:{source_byte}; edit the contributing source"
                    ));
                }
                found = Some((
                    path.clone(),
                    origin.output.0 + source_byte - origin.source.0,
                ));
            }
        }
    }
    Ok(found)
}
pub fn to_source(
    tree: &ProjectTree,
    path: &str,
    output_byte: usize,
) -> Result<Option<(String, usize)>, String> {
    let entry = tree
        .get(path)
        .ok_or_else(|| format!("file absent from selected project: {path}"))?;
    let matches: Vec<_> = entry
        .origins
        .iter()
        .filter(|o| o.output.0 <= output_byte && output_byte < o.output.1)
        .collect();
    match matches.as_slice() {
        [] => Ok(None),
        [origin] => Ok(Some((
            origin.document.clone(),
            origin.source.0 + output_byte - origin.output.0,
        ))),
        _ => Err(format!(
            "ambiguous origin for {path}; edit its contributing document"
        )),
    }
}
/// Convert bytes to LSP/editor UTF-16 only at the coordinate boundary.
pub fn utf16_position(
    sources: &BTreeMap<String, String>,
    document: &str,
    byte: usize,
) -> Result<(u32, u32), String> {
    let text = sources
        .get(document)
        .ok_or_else(|| format!("missing source {document}"))?;
    let prefix = text
        .get(..byte)
        .ok_or("coordinate is not a UTF-8 boundary")?;
    Ok((
        prefix.bytes().filter(|b| *b == b'\n').count() as u32,
        prefix
            .rsplit('\n')
            .next()
            .unwrap_or("")
            .encode_utf16()
            .count() as u32,
    ))
}

pub fn virtual_files(
    tree: &ProjectTree,
    sources: &BTreeMap<String, String>,
    document: &str,
) -> Result<Vec<crate::virtual_file::VirtualFile>, String> {
    let mut files = vec![];
    for (path, entry) in tree.entries() {
        if entry.kind != EntryKind::File {
            continue;
        }
        let Ok(text) = std::str::from_utf8(&entry.bytes) else {
            continue;
        };
        let mut segments = vec![];
        // Existing native mapping/writing callers remove the synthetic first
        // line. Preserve that contract while obtaining all bytes from the tree.
        segments.push(crate::virtual_file::VirtualSegment {
            text: "\n".into(),
            source_span: None,
            source_line: 0,
            source_column: 0,
        });
        let mut cursor = 0;
        for origin in &entry.origins {
            if cursor < origin.output.0 {
                segments.push(crate::virtual_file::VirtualSegment {
                    text: text[cursor..origin.output.0].into(),
                    source_span: None,
                    source_line: 0,
                    source_column: 0,
                });
            }
            let span = if origin.document == document {
                let source = sources.get(document).ok_or("missing project source")?;
                let prefix = source
                    .get(..origin.source.0)
                    .ok_or("invalid origin coordinate")?;
                Some(hick_lang::SourceSpan::new(
                    origin.source.0,
                    origin.source.1,
                    prefix.bytes().filter(|b| *b == b'\n').count() + 1,
                    prefix
                        .rsplit('\n')
                        .next()
                        .unwrap_or("")
                        .encode_utf16()
                        .count(),
                ))
            } else {
                None
            };
            segments.push(crate::virtual_file::VirtualSegment {
                text: text[origin.output.0..origin.output.1].into(),
                source_span: span,
                source_line: 0,
                source_column: span.map_or(0, |s| s.start_col),
            });
            cursor = origin.output.1;
        }
        if cursor < text.len() {
            segments.push(crate::virtual_file::VirtualSegment {
                text: text[cursor..].into(),
                source_span: None,
                source_line: 0,
                source_column: 0,
            });
        }
        files.push(crate::virtual_file::VirtualFile {
            path: path.clone(),
            language_id: crate::lang_detect::language_id(path),
            segments,
        });
    }
    Ok(files)
}

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
/// Ordinary callers preserve disk fallback; editor language services pass a coherent overlay snapshot.
pub(crate) fn state_for_uri(uri:&tower_lsp::lsp_types::Url,source:&str)->Result<crate::document::HickDocumentState,hick_lang::ParseError>{
 state_for_uri_with_overlays(uri,source,&BTreeMap::new())
}
