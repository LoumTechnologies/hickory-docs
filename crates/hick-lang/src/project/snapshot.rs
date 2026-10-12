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
