<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/src/project/snapshot.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ffa383fded4ff8ccb71deffafc30cac1969d244f9a32629c0605fd5bb237f900`.

<book:entry path="crates/hick-lang/src/project/snapshot.rs">
use super::{ByteOrigin, EntryKind, ProjectEntry, ProjectTree};
use std::collections::BTreeMap;

/// A portable captured value. Restoring validates paths and provenance just as
/// construction does; serialized data cannot manufacture an unchecked tree.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProjectSnapshot {
    entries: BTreeMap<String, SnapshotEntry>,
}
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct SnapshotEntry {
    bytes: Vec<u8>,
    kind: EntryKind,
    executable: bool,
    origins: Vec<ByteOrigin>,
}
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
</book:entry>

</book:doc>
