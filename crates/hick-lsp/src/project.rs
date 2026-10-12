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
}
impl ProjectEndpoint {
    pub fn parse(value: &str) -> Result<Self, String> {
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
