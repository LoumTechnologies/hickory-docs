<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lang/src/project/tree.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a485394d6759"></a>
## `crates/hick-lang/src/project/tree.rs`

Turn declared document constructions into immutable project trees, retaining byte origins so generated edits can return to their source.

Related contracts: [Check every source file, including assets and permissions](../../../../../guarantees/literate-projects/complete-inventory-is-byte-exact/contract.md), [Reject a patch that does not fit its exact input](../../../../../guarantees/literate-projects/replay-is-exact-and-atomic/contract.md).

<a id="f-a485394d6759-EDITOR_BUFFER_PREFIX"></a>
### `EDITOR_BUFFER_PREFIX`

Reserved evaluator addresses for editor state. Never materialize as filesystem paths.

<storycode:copy id="f-a485394d6759-EDITOR_BUFFER_PREFIX">
/// Reserved evaluator addresses for editor state. Never materialize as filesystem paths.
pub const EDITOR_BUFFER_PREFIX: &str = "__hickory_editor_buffers__/";
</storycode:copy>
<a id="f-a485394d6759-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-a485394d6759-use_declaration">
use std::{collections::BTreeMap, sync::Arc};

</storycode:copy>
<a id="f-a485394d6759-EntryKind"></a>
### `EntryKind`

Portable entry metadata. Symlink bytes are the exact UTF-8 target.

<storycode:copy id="f-a485394d6759-EntryKind">
/// Portable entry metadata. Symlink bytes are the exact UTF-8 target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EntryKind {
    File,
    Symlink,
}

</storycode:copy>
<a id="f-a485394d6759-ByteOrigin"></a>
### `ByteOrigin`

Each origin pairs a half-open output byte range with a source-document byte range. The lengths must agree for a literal reverse edit; these are byte positions, not editor character offsets.

<storycode:copy id="f-a485394d6759-ByteOrigin">
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ByteOrigin {
    pub output: (usize, usize),
    pub document: String,
    pub source: (usize, usize),
}

</storycode:copy>
<a id="f-a485394d6759-ProjectEntry"></a>
### `ProjectEntry`

Keep bytes, file kind, executable metadata and origins in the same value. Equal visible text alone does not establish that two entries are the same output or have the same edit destinations.

<storycode:copy id="f-a485394d6759-ProjectEntry">
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectEntry {
    pub bytes: Arc<[u8]>,
    pub kind: EntryKind,
    pub executable: bool,
    pub origins: Vec<ByteOrigin>,
}
</storycode:copy>
<a id="f-a485394d6759-impl-ProjectEntry"></a>
### `impl-ProjectEntry`

<storycode:copy id="f-a485394d6759-impl-ProjectEntry">
impl ProjectEntry {
    pub fn file(bytes: impl Into<Arc<[u8]>>) -> Self {
        Self {
            bytes: bytes.into(),
            kind: EntryKind::File,
            executable: false,
            origins: vec![],
        }
    }
    pub fn same_output(&self, other: &Self) -> bool {
        self.bytes == other.bytes && self.kind == other.kind && self.executable == other.executable
    }
}

</storycode:copy>
<a id="f-a485394d6759-ProjectTree"></a>
### `ProjectTree`

Store entries by portable path and share their immutable values through `Arc`. A candidate tree can reuse unchanged entries without mutating the tree supplied to an operation.

<storycode:copy id="f-a485394d6759-ProjectTree">
/// Immutable entry sharing makes before/after snapshots cheap. There is no IO.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectTree {
    entries: BTreeMap<String, Arc<ProjectEntry>>,
}
</storycode:copy>
<a id="f-a485394d6759-impl-ProjectTree"></a>
### `impl-ProjectTree`

<storycode:copy id="f-a485394d6759-impl-ProjectTree">
impl ProjectTree {
    pub fn entries(&self) -> &BTreeMap<String, Arc<ProjectEntry>> {
        &self.entries
    }
    pub fn get(&self, path: &str) -> Option<&Arc<ProjectEntry>> {
        self.entries.get(path)
    }
    pub fn insert(&mut self, path: &str, entry: ProjectEntry) -> Result<(), String> {
        validate_project_path(path)?;
        for origin in &entry.origins {
            if origin.output.0 > origin.output.1
                || origin.output.1 > entry.bytes.len()
                || origin.source.0 > origin.source.1
                || origin.output.1 - origin.output.0 != origin.source.1 - origin.source.0
            {
                return Err(format!("invalid byte provenance range for {path}"));
            }
        }
        for existing in self.entries.keys() {
            validate_path_collision(existing, path)?;
        }
        self.entries.insert(path.to_owned(), Arc::new(entry));
        Ok(())
    }
    pub fn remove(&mut self, path: &str) -> Result<Arc<ProjectEntry>, String> {
        validate_project_path(path)?;
        self.entries
            .remove(path)
            .ok_or_else(|| format!("missing project entry: {path}"))
    }
    pub fn compare(&self, expected: &Self) -> Vec<ProjectDifference> {
        let mut differences = vec![];
        for (path, entry) in &self.entries {
            match expected.get(path) {
                None => differences.push(ProjectDifference {
                    path: path.clone(),
                    kind: DifferenceKind::Extra,
                }),
                Some(other) if !entry.same_output(other) => differences.push(ProjectDifference {
                    path: path.clone(),
                    kind: DifferenceKind::Different,
                }),
                _ => (),
            }
        }
        for path in expected.entries.keys() {
            if !self.entries.contains_key(path) {
                differences.push(ProjectDifference {
                    path: path.clone(),
                    kind: DifferenceKind::Missing,
                });
            }
        }
        differences.sort_by(|a, b| a.path.cmp(&b.path));
        differences
    }
}
</storycode:copy>
<a id="f-a485394d6759-DifferenceKind"></a>
### `DifferenceKind`

<storycode:copy id="f-a485394d6759-DifferenceKind">
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DifferenceKind {
    Missing,
    Extra,
    Different,
}
</storycode:copy>
<a id="f-a485394d6759-ProjectDifference"></a>
### `ProjectDifference`

<storycode:copy id="f-a485394d6759-ProjectDifference">
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProjectDifference {
    pub path: String,
    pub kind: DifferenceKind,
}

</storycode:copy>
<a id="f-a485394d6759-validate_project_path"></a>
### `validate_project_path`

Require portable relative paths before placing entries in a tree. Reject ambiguous separators, traversal, reserved names and invalid segments here so generation and editing do not disagree about which output a path identifies.

<storycode:copy id="f-a485394d6759-validate_project_path">
/// One normalized, platform-independent spelling. Refuse ambiguous native paths.
pub fn validate_project_path(path: &str) -> Result<(), String> {
    if !path.is_ascii() {
        return Err(format!(
            "unsupported non-ASCII project path (Unicode normalization is not implemented): {path:?}"
        ));
    }
    if path.is_empty() || path.contains(['\\', ':', '\0']) || path.starts_with('/') {
        return Err(format!("invalid project path: {path:?}"));
    }
    for part in path.split('/') {
        let stem = part.split('.').next().unwrap().to_ascii_uppercase();
        if part.is_empty()
            || part == "."
            || part == ".."
            || part.ends_with(['.', ' '])
            || part
                .chars()
                .any(|c| c.is_control() || "<>\"|?*".contains(c))
            || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        {
            return Err(format!("ambiguous project path: {path:?}"));
        }
    }
    Ok(())
}

</storycode:copy>
<a id="f-a485394d6759-copy_origins"></a>
### `copy_origins`

<storycode:copy id="f-a485394d6759-copy_origins">
pub(crate) fn copy_origins(
    entry: &ProjectEntry,
    range: (usize, usize),
    offset: usize,
    into: &mut Vec<ByteOrigin>,
) {
    for origin in &entry.origins {
        let start = range.0.max(origin.output.0);
        let end = range.1.min(origin.output.1);
        if start < end {
            into.push(ByteOrigin {
                output: (offset + start - range.0, offset + end - range.0),
                document: origin.document.clone(),
                source: (
                    origin.source.0 + start - origin.output.0,
                    origin.source.0 + end - origin.output.0,
                ),
            });
        }
    }
}

</storycode:copy>
<a id="f-a485394d6759-validate_path_collision"></a>
### `validate_path_collision`

<storycode:copy id="f-a485394d6759-validate_path_collision">
pub(crate) fn validate_path_collision(existing: &str, path: &str) -> Result<(), String> {
    for (left, right) in existing.split('/').zip(path.split('/')) {
        if !left.eq_ignore_ascii_case(right) {
            break;
        }
        if left != right {
            return Err(format!(
                "ambiguous project path casing: {existing} and {path}"
            ));
        }
    }
    if existing != path
        && (existing.eq_ignore_ascii_case(path)
            || existing.starts_with(&format!("{path}/"))
            || path.starts_with(&format!("{existing}/")))
    {
        return Err(format!("project path collision: {existing} and {path}"));
    }
    Ok(())
}
</storycode:copy>

### Assemble `crates/hick-lang/src/project/tree.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang/src/project/tree.rs"><storycode:paste select="#f-a485394d6759-EDITOR_BUFFER_PREFIX"/><storycode:paste select="#f-a485394d6759-use_declaration"/><storycode:paste select="#f-a485394d6759-EntryKind"/><storycode:paste select="#f-a485394d6759-ByteOrigin"/><storycode:paste select="#f-a485394d6759-ProjectEntry"/><storycode:paste select="#f-a485394d6759-impl-ProjectEntry"/><storycode:paste select="#f-a485394d6759-ProjectTree"/><storycode:paste select="#f-a485394d6759-impl-ProjectTree"/><storycode:paste select="#f-a485394d6759-DifferenceKind"/><storycode:paste select="#f-a485394d6759-ProjectDifference"/><storycode:paste select="#f-a485394d6759-validate_project_path"/><storycode:paste select="#f-a485394d6759-copy_origins"/><storycode:paste select="#f-a485394d6759-validate_path_collision"/></storycode:file>

</storycode:doc>
