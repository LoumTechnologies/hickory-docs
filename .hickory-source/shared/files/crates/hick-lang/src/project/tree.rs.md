<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/src/project/tree.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Check every source file, including assets and permissions](../../../../../../guarantees/literate-projects/complete-inventory-is-byte-exact/contract.md)
- [Reject a patch that does not fit its exact input](../../../../../../guarantees/literate-projects/replay-is-exact-and-atomic/contract.md)

Captured SHA-256: `9bfaaaeda9f25823f653166f6d5b57c0345a1435614526d4fdb560569268d76d`.

<book:entry path="crates/hick-lang/src/project/tree.rs">
/// Reserved evaluator addresses for editor state. Never materialize as filesystem paths.
pub const EDITOR_BUFFER_PREFIX: &str = "__hickory_editor_buffers__/";
use std::{collections::BTreeMap, sync::Arc};

/// Portable entry metadata. Symlink bytes are the exact UTF-8 target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EntryKind {
    File,
    Symlink,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ByteOrigin {
    pub output: (usize, usize),
    pub document: String,
    pub source: (usize, usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectEntry {
    pub bytes: Arc<[u8]>,
    pub kind: EntryKind,
    pub executable: bool,
    pub origins: Vec<ByteOrigin>,
}
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

/// Immutable entry sharing makes before/after snapshots cheap. There is no IO.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectTree {
    entries: BTreeMap<String, Arc<ProjectEntry>>,
}
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
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DifferenceKind {
    Missing,
    Extra,
    Different,
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProjectDifference {
    pub path: String,
    pub kind: DifferenceKind,
}

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
</book:entry>

</book:doc>
