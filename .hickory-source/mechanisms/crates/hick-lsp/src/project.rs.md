<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lsp/src/project.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-144d2ea0027d"></a>
## `crates/hick-lsp/src/project.rs`

Related contracts: [Debug the latest atomic project that still contains the displayed bytes](../../../../guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md), [Keep language tools and debug launches tied to their captured project](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md).

<a id="f-144d2ea0027d-ProjectEndpoint"></a>
### `ProjectEndpoint`

Native services select one complete project. Materialization is explicit and
creates a fresh private identity per selection; portable replay never calls it.

<storycode:copy id="f-144d2ea0027d-ProjectEndpoint">
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
</storycode:copy>
<a id="f-144d2ea0027d-impl-ProjectEndpoint"></a>
### `impl-ProjectEndpoint`

<storycode:copy id="f-144d2ea0027d-impl-ProjectEndpoint">
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
</storycode:copy>
<a id="f-144d2ea0027d-endpoint"></a>
### `endpoint`

<storycode:copy id="f-144d2ea0027d-endpoint">
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
</storycode:copy>
<a id="f-144d2ea0027d-latest_file"></a>
### `latest_file`

Resolve the complete tree and revision-local filename for a clicked file.

<storycode:copy id="f-144d2ea0027d-latest_file">
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

</storycode:copy>
<a id="f-144d2ea0027d-debug_selection"></a>
### `debug_selection`

<storycode:copy id="f-144d2ea0027d-debug_selection">
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

</storycode:copy>
<a id="f-144d2ea0027d-MaterializedProject"></a>
### `MaterializedProject`

<storycode:copy id="f-144d2ea0027d-MaterializedProject">
#[derive(Debug)]
pub struct MaterializedProject {
    directory: tempfile::TempDir,
    pub endpoint: ProjectEndpoint,
    files: Vec<PathBuf>,
}
</storycode:copy>
<a id="f-144d2ea0027d-impl-MaterializedProject"></a>
### `impl-MaterializedProject`

<storycode:copy id="f-144d2ea0027d-impl-MaterializedProject">
impl MaterializedProject {
    pub fn root(&self) -> &Path {
        self.directory.path()
    }
    pub fn files(&self) -> Vec<PathBuf> {
        self.files.clone()
    }
}
</storycode:copy>
<a id="f-144d2ea0027d-materialize"></a>
### `materialize`

<storycode:copy id="f-144d2ea0027d-materialize">
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
</storycode:copy>
<a id="f-144d2ea0027d-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-144d2ea0027d-use_declaration">
pub use hick_project_env::write_tree;

</storycode:copy>
<a id="f-144d2ea0027d-to_endpoint"></a>
### `to_endpoint`

Exact byte mapping. A deleted local line has no endpoint mapping, and
repeated contributors are ambiguous rather than guessed at.

<storycode:copy id="f-144d2ea0027d-to_endpoint">
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
</storycode:copy>
<a id="f-144d2ea0027d-to_source"></a>
### `to_source`

<storycode:copy id="f-144d2ea0027d-to_source">
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
</storycode:copy>
<a id="f-144d2ea0027d-utf16_position"></a>
### `utf16_position`

Convert bytes to LSP/editor UTF-16 only at the coordinate boundary.

<storycode:copy id="f-144d2ea0027d-utf16_position">
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

</storycode:copy>
<a id="f-144d2ea0027d-virtual_files"></a>
### `virtual_files`

<storycode:copy id="f-144d2ea0027d-virtual_files">
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

</storycode:copy>
<a id="f-144d2ea0027d-state_for_uri"></a>
### `state_for_uri`

Host reader for the open entry's live source closure. The entry buffer is
authoritative; contributing sources are captured before evaluation.

<storycode:copy id="f-144d2ea0027d-state_for_uri">
/// Host reader for the open entry's live source closure. The entry buffer is
/// authoritative; contributing sources are captured before evaluation.
pub(crate) fn state_for_uri(
    uri: &tower_lsp::lsp_types::Url,
    source: &str,
) -> Result<crate::document::HickDocumentState, hick_lang::ParseError> {
    let doc = hick_lang::parse(source)?;
    if doc.find_tags("project").is_empty() {
        return crate::document::HickDocumentState::from_source(source);
    }
    let failure = |message| hick_lang::ParseError::Syntax { line: 1, message };
    let path = uri
        .to_file_path()
        .map_err(|_| failure("project closure requires a file identity".into()))?;
    let root = path
        .parent()
        .ok_or_else(|| failure("project has no directory".into()))?;
    let entry = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| failure("entry name is not UTF-8".into()))?;
    let sources = hick_lang::load_project_sources(entry, &|name| {
        if name == entry {
            return Ok(source.into());
        }
        let source_path = root.join(name);
        if !std::fs::symlink_metadata(&source_path)
            .map_err(|e| e.to_string())?
            .is_file()
        {
            return Err(format!("Markdown source is not a regular file: {name}"));
        }
        let path = source_path.canonicalize().map_err(|e| e.to_string())?;
        if !path.starts_with(root.canonicalize().map_err(|e| e.to_string())?) {
            return Err(format!(
                "Markdown closure escapes project directory: {name}"
            ));
        }
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    })
    .map_err(failure)?;
    let selection = uri
        .query_pairs()
        .find(|(name, _)| name == "hick-project")
        .map(|(_, value)| value.into_owned())
        .unwrap_or_else(|| "new".into());
    let endpoint = ProjectEndpoint::parse(&selection).map_err(failure)?;
    crate::document::HickDocumentState::from_project_at(
        entry,
        &sources,
        &ProjectTree::default(),
        &endpoint,
    )
}
</storycode:copy>

### Assemble `crates/hick-lsp/src/project.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lsp/src/project.rs"><storycode:paste select="#f-144d2ea0027d-ProjectEndpoint"/><storycode:paste select="#f-144d2ea0027d-impl-ProjectEndpoint"/><storycode:paste select="#f-144d2ea0027d-endpoint"/><storycode:paste select="#f-144d2ea0027d-latest_file"/><storycode:paste select="#f-144d2ea0027d-debug_selection"/><storycode:paste select="#f-144d2ea0027d-MaterializedProject"/><storycode:paste select="#f-144d2ea0027d-impl-MaterializedProject"/><storycode:paste select="#f-144d2ea0027d-materialize"/><storycode:paste select="#f-144d2ea0027d-use_declaration"/><storycode:paste select="#f-144d2ea0027d-to_endpoint"/><storycode:paste select="#f-144d2ea0027d-to_source"/><storycode:paste select="#f-144d2ea0027d-utf16_position"/><storycode:paste select="#f-144d2ea0027d-virtual_files"/><storycode:paste select="#f-144d2ea0027d-state_for_uri"/></storycode:file>

</storycode:doc>
