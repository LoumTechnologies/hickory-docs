<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lang/src/project/snapshot.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0c552f23bf83"></a>
## `crates/hick-lang/src/project/snapshot.rs`

Turn declared document constructions into immutable project trees, retaining byte origins so generated edits can return to their source.

<a id="f-0c552f23bf83-ProjectSnapshot"></a>
### `ProjectSnapshot`

<storycode:copy id="f-0c552f23bf83-ProjectSnapshot">
use super::{ByteOrigin, EntryKind, ProjectEntry, ProjectTree};
use std::collections::BTreeMap;

/// A portable captured value. Restoring validates paths and provenance just as
/// construction does; serialized data cannot manufacture an unchecked tree.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProjectSnapshot {
    entries: BTreeMap<String, SnapshotEntry>,
}
</storycode:copy>
<a id="f-0c552f23bf83-SnapshotEntry"></a>
### `SnapshotEntry`

<storycode:copy id="f-0c552f23bf83-SnapshotEntry">
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct SnapshotEntry {
    bytes: Vec<u8>,
    kind: EntryKind,
    executable: bool,
    origins: Vec<ByteOrigin>,
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProjectSnapshotDelta {
    updated: BTreeMap<String, SnapshotEntry>,
    removed: Vec<String>,
}
</storycode:copy>
<a id="f-0c552f23bf83-impl-ProjectSnapshot"></a>
### `impl-ProjectSnapshot`

Restore through the same complete-inventory check as a disk capture. Byte
ownership, portable paths and metadata must all be valid before the restored
tree can support a recorded changeset.
To store a before and after pair, record only the added, removed or changed
entries after the starting snapshot. Comparing complete entries preserves mode,
kind and provenance differences even when the output bytes match.

<storycode:copy id="f-0c552f23bf83-impl-ProjectSnapshot">
impl ProjectSnapshot {
    pub fn capture(tree: &ProjectTree) -> Self {
        Self {
            entries: tree
                .entries()
                .iter()
                .map(|(path, e)| {
                    (
                        path.clone(),
                        SnapshotEntry {
                            bytes: e.bytes.to_vec(),
                            kind: e.kind,
                            executable: e.executable,
                            origins: e.origins.clone(),
                        },
                    )
                })
                .collect(),
        }
    }
    pub fn changes_since(&self, starting: &Self) -> ProjectSnapshotDelta {
        let updated = self.entries.iter()
            .filter(|(path, entry)| starting.entries.get(*path) != Some(*entry))
            .map(|(path, entry)| (path.clone(), entry.clone()))
            .collect();
        let removed = starting.entries.keys()
            .filter(|path| !self.entries.contains_key(*path))
            .cloned()
            .collect();
        ProjectSnapshotDelta { updated, removed }
    }

    pub fn applying(mut self, changes: ProjectSnapshotDelta) -> Self {
        for path in changes.removed {
            self.entries.remove(&path);
        }
        self.entries.extend(changes.updated);
        self
    }

    pub fn restore(&self) -> Result<ProjectTree, String> {
        ProjectTree::from_inventory(
            self.entries
                .iter()
                .map(|(path, entry)| {
                    (
                        path.clone(),
                        ProjectEntry {
                            bytes: entry.bytes.clone().into(),
                            kind: entry.kind,
                            executable: entry.executable,
                            origins: entry.origins.clone(),
                        },
                    )
                })
                .collect(),
        )
    }
}
</storycode:copy>

### Assemble `crates/hick-lang/src/project/snapshot.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang/src/project/snapshot.rs"><storycode:paste select="#f-0c552f23bf83-ProjectSnapshot"/><storycode:paste select="#f-0c552f23bf83-SnapshotEntry"/><storycode:paste select="#f-0c552f23bf83-impl-ProjectSnapshot"/></storycode:file>

</storycode:doc>
