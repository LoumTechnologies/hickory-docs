<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-dap/src/mapping.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-32f353d4cad8"></a>
## `crates/hick-dap/src/mapping.rs`

Related contracts: [A frame stays in document coordinates](../../../../guarantees/debugging/a-frame-stays-in-document-coordinates/contract.md), [Observations read beside their captured source](../../../../guarantees/debugging/source-anchored-observation-reading/contract.md).

<a id="f-32f353d4cad8-Mapping"></a>
### `Mapping`

Keep source digests with the captured mapping, rather than reading a file after
the debugger stops and calling that its execution revision. Plain launch captures
its primary source and requested breakpoint sources before adapter startup; a
profile also retains its already observed project sources. Other frames stay unknown. Compiler-backed mappings
retain their pinned contributor digests. These identify launch/construction bytes,
not immutable runtime inputs or proof of which sources executed. Hosts can use a
matching digest to admit a source position while keeping that evidence limit clear.

<storycode:copy id="f-32f353d4cad8-Mapping">
use anyhow::Result;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// Document <-> generated-file coordinates for the files a cell can stop in.
#[derive(Clone)]
pub struct Mapping {
    observation_source: Option<Vec<u8>>,
    source_hashes: HashMap<PathBuf, String>,
    /// Generated file path (as the adapter sees it) -> the document's mapping.
    files: HashMap<PathBuf, hick_lsp::position_map::PositionMap>,
    /// The document's own path, for naming.
    document: PathBuf,
    /// True when `document` is a plain file being debugged as itself: every
    /// line maps to the same line, and the "generated file" is the file.
    plain: bool,
    captured: Option<(String, PathBuf)>,
}
</storycode:copy>
<a id="debug-plain-launch-snapshots"></a>
### Retain already observed project sources

A profile already observed its source tree to select a launch target. Reuse
those bytes rather than rereading them after a stop. A requested extra source
can join a plain launch before startup; later unknown sources stay unbound.
The startup guard separately checks requested physical bytes still match.

<storycode:copy id="debug-plain-launch-snapshots">
impl Mapping {
    /// Extend an identity mapping with the already captured launch project.
    /// These are snapshot identities, not evidence of runtime execution.
    pub fn with_plain_sources<'a>(
        mut self,
        root: &Path,
        sources: impl IntoIterator<Item = (&'a str, &'a [u8])>,
    ) -> Self {
        if self.plain {
            for (name, bytes) in sources {
                if std::str::from_utf8(bytes).is_ok() {
                    let path = root.join(name);
                    self.source_hashes
                        .insert(path.clone(), Self::source_digest(bytes));
                    if same_file(&path, &self.document) {
                        self.observation_source = Some(bytes.to_vec());
                    }
                }
            }
        }
        self
    }
    pub(crate) fn capture_initial_source(&mut self, path: &Path, bytes: &[u8]) {
        if self.plain {
            self.source_hashes
                .entry(path.to_path_buf())
                .or_insert_with(|| Self::source_digest(bytes));
        }
    }
}
</storycode:copy>

<a id="f-32f353d4cad8-impl-Mapping"></a>
### `impl-Mapping`

<storycode:copy id="f-32f353d4cad8-impl-Mapping">
impl Mapping {
    pub(crate) fn source_digest(source: &[u8]) -> String {
        hick_observe::Source::capture(String::new(), String::new(), source.to_vec()).revision
    }

    /// Captured launch/construction bytes, never a claim that mutable runtime
    /// inputs were frozen or that the adapter executed every captured source.
    pub fn source_hash(&self, document: &Path) -> Option<String> {
        self.source_hashes
            .get(document)
            .or_else(|| {
                self.source_hashes
                    .iter()
                    .find(|(path, _)| same_file(path, document))
                    .map(|(_, hash)| hash)
            })
            .cloned()
    }
    pub fn captured(
        tree: &hick_lang::ProjectTree,
        workdir: &Path,
        id: &str,
        program: &str,
    ) -> Result<Self> {
        let root = workdir.canonicalize()?;
        let document = PathBuf::from(
            hick_lsp::ProjectContexts::source_uri(id, program).map_err(anyhow::Error::msg)?,
        );
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
            source_hashes: tree
                .entries()
                .iter()
                .map(|(path, entry)| {
                    (
                        PathBuf::from(
                            hick_lsp::ProjectContexts::source_uri(id, path)
                                .expect("captured tree paths are validated"),
                        ),
                        Self::source_digest(&entry.bytes),
                    )
                })
                .collect(),
            observation_source: tree
                .entries()
                .get(program)
                .map(|entry| entry.bytes.to_vec()),
            files,
            document,
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
        let source = std::fs::read(file).ok();
        Self {
            source_hashes: source
                .as_ref()
                .map(|source| HashMap::from([(file.to_path_buf(), Self::source_digest(source))]))
                .unwrap_or_default(),
            observation_source: source,
            files: HashMap::new(),
            document: file.to_path_buf(),
            plain: true,
            captured: None,
        }
    }

    pub fn with_observation_source(mut self, source: &str) -> Self {
        self.observation_source = Some(source.as_bytes().to_vec());
        self.source_hashes.insert(
            self.document.clone(),
            Self::source_digest(source.as_bytes()),
        );
        self
    }

    pub fn observation_source(&self) -> Option<hick_observe::Source> {
        self.observation_source.clone().map(|bytes| {
            hick_observe::Source::capture(
                "document".into(),
                self.document.to_string_lossy().into_owned(),
                bytes,
            )
        })
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
        Ok(Self::for_state(document, &state, workdir)?.with_observation_source(source))
    }

    /// Map only surviving bytes from the selected complete project state.
    pub fn for_state(
        document: &Path,
        state: &hick_lsp::document::HickDocumentState,
        workdir: &Path,
    ) -> Result<Self> {
        let files = crate::project::document_maps(state, workdir);
        Ok(Self {
            source_hashes: state
                .project_sources
                .iter()
                .map(|(path, source)| {
                    (
                        document.parent().unwrap_or(Path::new(".")).join(path),
                        Self::source_digest(source.as_bytes()),
                    )
                })
                .chain(std::iter::once((
                    document.to_path_buf(),
                    Self::source_digest(state.doc.source.as_bytes()),
                )))
                .collect(),
            observation_source: Some(state.doc.source.as_bytes().to_vec()),
            files,
            document: document.to_path_buf(),
            plain: false,
            captured: None,
        })
    }

    /// Document line -> (generated file, line) for setting a breakpoint.
    pub fn to_generated(&self, line: u32) -> Option<(PathBuf, u32)> {
        if self.captured.is_some() {
            return self.to_generated_in(&self.document, line);
        }
        if self.plain {
            // Compilers write canonical source paths into debug symbols. Keep
            // the editor's identity for display, but bind the runtime spelling.
            return Some((
                self.document
                    .canonicalize()
                    .unwrap_or_else(|_| self.document.clone()),
                line,
            ));
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
            return Some((
                PathBuf::from(hick_lsp::ProjectContexts::source_uri(id, &name).ok()?),
                line,
            ));
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
        if let Some((id, root)) = &self.captured {
            let (context, name) =
                hick_lsp::ProjectContexts::decode_source_uri(document.to_str()?).ok()?;
            if &context != id {
                return None;
            }
            let path = root.join(name).canonicalize().ok()?;
            return (line < self.files.get(&path)?.virtual_lines()).then_some((path, line));
        }
        if self.plain {
            return document
                .canonicalize()
                .ok()
                .filter(|path| path.is_file())
                .map(|path| (path, line));
        }
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
            canonical_with_missing_tail(path).and_then(|canonical| self.files.get(&canonical))
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
</storycode:copy>
<a id="f-32f353d4cad8-canonical_with_missing_tail"></a>
### `canonical_with_missing_tail`

Resolve aliases in an existing parent before a generated output exists.
Only a normal missing suffix is accepted; lookup still requires an exact
generated-file identity from the selected document's mapping.

<storycode:copy id="f-32f353d4cad8-canonical_with_missing_tail">
/// Resolve aliases in an existing parent before a generated output exists.
/// Only a normal missing suffix is accepted; lookup still requires an exact
/// generated-file identity from the selected document's mapping.
fn canonical_with_missing_tail(path: &Path) -> Option<PathBuf> {
    let mut ancestor = path;
    let mut tail = Vec::new();
    loop {
        if let Ok(mut canonical) = ancestor.canonicalize() {
            for component in tail.iter().rev() {
                canonical.push(component);
            }
            return Some(canonical);
        }
        let component = match ancestor.components().next_back()? {
            std::path::Component::Normal(name) => name.to_os_string(),
            _ => return None,
        };
        tail.push(component);
        ancestor = ancestor.parent()?;
    }
}
</storycode:copy>
<a id="f-32f353d4cad8-same_file"></a>
### `same_file`

Whether two paths name one file.
Adapters report a frame's source in their own spelling — codelldb gives
the path cargo compiled with, debugpy the path it was launched with — so
the comparison is made on the canonical path when both resolve, and on
the bytes when either does not.

<storycode:copy id="f-32f353d4cad8-same_file">
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
</storycode:copy>
<a id="f-32f353d4cad8-observation_tests"></a>
### `observation_tests`

<storycode:copy id="f-32f353d4cad8-observation_tests">
#[cfg(test)]
mod observation_tests {
    use super::*;
    // Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
    #[test]
    fn literate_fragments_keep_original_bytes_after_disk_edits() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("demo.md");
        let source = "\u{feff}# 🦀\n<hick:file path=\"app.py\">\nvalue = 1\n</hick:file>\n\n<hick:file path=\"app.py\">\nvalue += 2\n</hick:file>\n";
        std::fs::write(&path, source).unwrap();
        let mapping = Mapping::for_document(&path, source, dir.path()).unwrap();
        let captured = mapping.observation_source().unwrap();
        std::fs::write(&path, "edited").unwrap();
        assert_eq!(captured.bytes, source.as_bytes());
        let (generated, line) = mapping.to_generated(6).unwrap();
        assert_eq!(mapping.to_document(&generated, line), Some(6));
        assert_eq!(mapping.observation_source().unwrap(), captured);
        assert_eq!(mapping.source_hash(&path), Some(captured.revision));
        assert_eq!(mapping.source_hash(&dir.path().join("external.py")), None);
    }
    #[test]
    fn plain_source_hashes_keep_exact_launch_bytes_and_never_borrow_external_authority() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("main.py");
        std::fs::write(&path, "é = 1\n").unwrap();
        let captured = Mapping::identity(&path);
        let expected = Mapping::source_digest("é = 1\n".as_bytes());
        std::fs::write(&path, "e\u{301} = 1\n").unwrap();
        assert_eq!(captured.source_hash(&path), Some(expected.clone()));
        assert_ne!(Mapping::identity(&path).source_hash(&path), Some(expected));
        std::fs::write(root.path().join("external.py"), "é = 1\n").unwrap();
        assert_eq!(captured.source_hash(&root.path().join("external.py")), None);
    }
    #[test]
    fn contributor_hashes_come_from_pinned_compiler_sources_instead_of_later_disk() {
        let root = tempfile::tempdir().unwrap();
        let entry = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"part.md\"/></h:doc>";
        let part = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"main.py\">value = 7</h:file></h:doc>";
        let sources = std::collections::BTreeMap::from([
            ("project.md".into(), entry.into()),
            ("part.md".into(), part.into()),
        ]);
        let state =
            hick_lsp::document::HickDocumentState::from_project("project.md", &sources).unwrap();
        let mapping =
            Mapping::for_state(&root.path().join("project.md"), &state, root.path()).unwrap();
        std::fs::write(root.path().join("part.md"), "independent new text").unwrap();
        let (source, _) = mapping
            .source_location(&root.path().join("main.py"), 0)
            .unwrap();
        assert_eq!(source, root.path().join("part.md"));
        assert_eq!(
            mapping.source_hash(&source),
            Some(Mapping::source_digest(part.as_bytes()))
        );
    }
    #[test]
    fn captured_literal_names_keep_one_hash_and_bidirectional_address() {
        let directory = tempfile::tempdir().unwrap();
        let name = "literal %2F #.py";
        let text = b"value=1\nprint(value)\n";
        std::fs::write(directory.path().join(name), text).unwrap();
        let mut tree = hick_lang::ProjectTree::default();
        tree.insert(name, hick_lang::ProjectEntry::file(text.to_vec()))
            .unwrap();
        let id = "0123456789abcdef";
        let mapping = Mapping::captured(&tree, directory.path(), id, name).unwrap();
        let physical = directory.path().join(name).canonicalize().unwrap();
        let (uri, line) = mapping.source_location(&physical, 1).unwrap();
        assert_eq!(
            uri.to_str().unwrap(),
            "hick-project:///0123456789abcdef/literal%20%252F%20%23.py"
        );
        assert_eq!(mapping.to_generated(1), Some((physical.clone(), 1)));
        assert_eq!(mapping.to_generated_in(&uri, line), Some((physical, 1)));
        assert_eq!(
            mapping.source_hash(&uri),
            Some(Mapping::source_digest(text))
        );
        let foreign = PathBuf::from("hick-project:///fedcba9876543210/literal%20%252F%20%23.py");
        assert!(mapping.to_generated_in(&foreign, 1).is_none());
    }
}
</storycode:copy>

### Assemble `crates/hick-dap/src/mapping.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/src/mapping.rs"><storycode:paste select="#f-32f353d4cad8-Mapping"/><storycode:paste select="#debug-plain-launch-snapshots"/><storycode:paste select="#f-32f353d4cad8-impl-Mapping"/><storycode:paste select="#f-32f353d4cad8-canonical_with_missing_tail"/><storycode:paste select="#f-32f353d4cad8-same_file"/><storycode:paste select="#f-32f353d4cad8-observation_tests"/></storycode:file>

</storycode:doc>
