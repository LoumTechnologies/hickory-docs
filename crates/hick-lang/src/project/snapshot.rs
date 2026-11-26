use super::{ByteOrigin, EntryKind, ProjectEntry, ProjectTree};
use std::collections::BTreeMap;

/// A portable captured value. Restoring validates paths and provenance just as
/// construction does; serialized data cannot manufacture an unchecked tree.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProjectSnapshot {
    entries: BTreeMap<String, SnapshotEntry>,
}
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
