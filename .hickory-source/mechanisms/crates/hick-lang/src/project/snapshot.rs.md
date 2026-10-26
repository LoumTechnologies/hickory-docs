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
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct SnapshotEntry {
    bytes: Vec<u8>,
    kind: EntryKind,
    executable: bool,
    origins: Vec<ByteOrigin>,
}
</storycode:copy>
<a id="f-0c552f23bf83-impl-ProjectSnapshot"></a>
### `impl-ProjectSnapshot`

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
    pub fn restore(&self) -> Result<ProjectTree, String> {
        let mut tree = ProjectTree::default();
        for (path, e) in &self.entries {
            tree.insert(
                path,
                ProjectEntry {
                    bytes: e.bytes.clone().into(),
                    kind: e.kind,
                    executable: e.executable,
                    origins: e.origins.clone(),
                },
            )?;
        }
        Ok(tree)
    }
}
</storycode:copy>

### Assemble `crates/hick-lang/src/project/snapshot.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang/src/project/snapshot.rs"><storycode:paste select="#f-0c552f23bf83-ProjectSnapshot"/><storycode:paste select="#f-0c552f23bf83-SnapshotEntry"/><storycode:paste select="#f-0c552f23bf83-impl-ProjectSnapshot"/></storycode:file>

</storycode:doc>
