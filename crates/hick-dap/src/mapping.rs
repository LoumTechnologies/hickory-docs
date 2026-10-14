use anyhow::Result;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// Document <-> generated-file coordinates for the files a cell can stop in.
pub struct Mapping {
    /// Generated file path (as the adapter sees it) -> the document's mapping.
    files: HashMap<PathBuf, hick_lsp::position_map::PositionMap>,
    /// The document's own path, for naming.
    document: PathBuf,
    /// True when `document` is a plain file being debugged as itself: every
    /// line maps to the same line, and the "generated file" is the file.
    plain: bool,
    captured: Option<(String, PathBuf)>,
}

impl Mapping {
    pub fn captured(
        tree: &hick_lang::ProjectTree,
        workdir: &Path,
        id: &str,
        program: &str,
    ) -> Result<Self> {
        let root = workdir.canonicalize()?;
        let files = tree
            .entries()
            .iter()
            .filter_map(|(path, entry)| {
                std::str::from_utf8(&entry.bytes).ok().map(|text| {
                    (
                        root.join(path),
                        hick_lsp::position_map::PositionMap::identity(
                            text.split('\n').count() as u32
                        ),
                    )
                })
            })
            .collect();
        Ok(Self {
            files,
            document: PathBuf::from(format!("hick-project:///{id}/{program}")),
            plain: false,
            captured: Some((id.into(), root)),
        })
    }
    /// The mapping for a file that is not a document — `src/main.rs`,
    /// `app.py` — debugged as itself, at its own path.
    ///
    /// The same move `hick-lsp` makes for a plain file's language server:
    /// skip the weave and hand the file over as it is, so every caller that
    /// translates through a `Mapping` works unchanged. A line is its own
    /// line in both directions, and no line is outside the file.
    pub fn identity(file: &Path) -> Self {
        Self {
            files: HashMap::new(),
            document: file.to_path_buf(),
            plain: true,
            captured: None,
        }
    }

    /// Whether this maps a plain file to itself.
    pub fn is_identity(&self) -> bool {
        self.plain
    }

    /// Build the mapping for a document by weaving it the way the language
    /// server does.
    pub fn for_document(document: &Path, source: &str, workdir: &Path) -> Result<Self> {
        let state = hick_lsp::document::HickDocumentState::from_source(source)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Self::for_state(document, &state, workdir)
    }

    /// Map only surviving bytes from the selected complete project state.
    pub fn for_state(
        document: &Path,
        state: &hick_lsp::document::HickDocumentState,
        workdir: &Path,
    ) -> Result<Self> {
        let files = crate::project::document_maps(state, workdir);
        Ok(Self {
            files,
            document: document.to_path_buf(),
            plain: false,
            captured: None,
        })
    }

    /// Document line -> (generated file, line) for setting a breakpoint.
    pub fn to_generated(&self, line: u32) -> Option<(PathBuf, u32)> {
        if let Some((_, root)) = &self.captured {
            let name = self.document.to_str()?.splitn(5, '/').nth(4)?;
            return self
                .files
                .contains_key(&root.join(name))
                .then_some((root.join(name), line));
        }
        if self.plain {
            return Some((self.document.clone(), line));
        }
        crate::project::generated_line(&self.files, line)
    }

    /// (generated file, line) -> document line for showing a frame.
    pub fn to_document(&self, path: &Path, line: u32) -> Option<u32> {
        if self.plain {
            return same_file(path, &self.document).then_some(line);
        }
        if self.captured.is_some() {
            return self
                .source_location(path, line)
                .filter(|(p, _)| p == &self.document)
                .map(|(_, line)| line);
        }
        let map = self.file_map(path)?;
        map.to_source(line, 0).map(|(l, _)| l)
    }

    /// Frames in another contributor name that document and its own line.
    pub fn source_location(&self, path: &Path, line: u32) -> Option<(PathBuf, u32)> {
        if self.plain {
            return same_file(path, &self.document).then_some((self.document.clone(), line));
        }
        if let Some((id, root)) = &self.captured {
            let full = path.canonicalize().unwrap_or_else(|_| path.into());
            let name = full.strip_prefix(root).ok()?.to_str()?.replace('\\', "/");
            self.files.get(&full)?.to_source(line, 0)?;
            return Some((PathBuf::from(format!("hick-project:///{id}/{name}")), line));
        }
        let map = self.file_map(path)?;
        let (document, line, _) = map.source_document(line, 0)?;
        Some((
            document
                .map(|name| self.document.parent().unwrap_or(Path::new(".")).join(name))
                .unwrap_or_else(|| self.document.clone()),
            line,
        ))
    }
    pub fn to_generated_in(&self, document: &Path, line: u32) -> Option<(PathBuf, u32)> {
        if same_file(document, &self.document) {
            return self.to_generated(line);
        }
        let name = document
            .strip_prefix(self.document.parent()?)
            .ok()?
            .to_str()?
            .replace('\\', "/");
        let mut found = None;
        for (path, map) in &self.files {
            if let Some(line) = map.generated_line_in(&name, line) {
                if found.is_some() {
                    return None;
                }
                found = Some((path.clone(), line));
            }
        }
        found
    }

    pub fn document(&self) -> &Path {
        &self.document
    }

    fn file_map(&self, path: &Path) -> Option<&hick_lsp::position_map::PositionMap> {
        self.files.get(path).or_else(|| {
            path.canonicalize()
                .ok()
                .and_then(|canonical| self.files.get(&canonical))
        })
    }

    /// Every generated file this document produces, for the launch config.
    pub fn generated_files(&self) -> Vec<PathBuf> {
        if self.plain {
            return vec![self.document.clone()];
        }
        self.files.keys().cloned().collect()
    }
}

/// Whether two paths name one file.
///
/// Adapters report a frame's source in their own spelling — codelldb gives
/// the path cargo compiled with, debugpy the path it was launched with — so
/// the comparison is made on the canonical path when both resolve, and on
/// the bytes when either does not.
pub(crate) fn same_file(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}
