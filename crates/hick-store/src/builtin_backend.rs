//! Built-in VersionStore implementation backed by an ObjectStore.
//!
//! Layout:
//! ```text
//! blobs/{hash[0..2]}/{hash}
//! snapshots/{id}.json
//! branches/{name}.json
//! ```

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;

use crate::error::StoreError;
use crate::object_store::ObjectStore;
use crate::types::{BlobHash, Snapshot, SnapshotId};
use crate::version_store::VersionStore;

/// VersionStore backed by any ObjectStore implementation.
pub struct BuiltinVersionStore {
    store: Arc<dyn ObjectStore>,
}

impl BuiltinVersionStore {
    pub fn new(store: Arc<dyn ObjectStore>) -> Self {
        Self { store }
    }

    fn blob_key(hash: &BlobHash) -> String {
        let prefix = &hash.0[..2.min(hash.0.len())];
        format!("blobs/{prefix}/{}", hash.0)
    }

    fn snapshot_key(id: &SnapshotId) -> String {
        format!("snapshots/{}.json", id.0)
    }

    fn branch_key(name: &str) -> String {
        format!("branches/{name}.json")
    }
}

#[async_trait]
impl VersionStore for BuiltinVersionStore {
    async fn put_blob(&self, content: &[u8]) -> Result<BlobHash, StoreError> {
        let hash = BlobHash::of(content);
        let key = Self::blob_key(&hash);
        // Idempotent: content-addressed
        if !self.store.exists(&key).await? {
            self.store.put(&key, content).await?;
        }
        Ok(hash)
    }

    async fn get_blob(&self, hash: &BlobHash) -> Result<Vec<u8>, StoreError> {
        let key = Self::blob_key(hash);
        self.store
            .get(&key)
            .await
            .map_err(|_| StoreError::BlobNotFound(hash.clone()))
    }

    async fn put_snapshot(&self, snapshot: &Snapshot) -> Result<SnapshotId, StoreError> {
        let id = snapshot.compute_id();
        let mut stored = snapshot.clone();
        stored.id = id.clone();

        let json = serde_json::to_string_pretty(&stored)
            .map_err(|e| StoreError::Serialization(e.to_string()))?;

        let key = Self::snapshot_key(&id);
        self.store.put(&key, json.as_bytes()).await?;
        Ok(id)
    }

    async fn get_snapshot(&self, id: &SnapshotId) -> Result<Snapshot, StoreError> {
        let key = Self::snapshot_key(id);
        let data = self
            .store
            .get(&key)
            .await
            .map_err(|_| StoreError::SnapshotNotFound(id.clone()))?;

        let snapshot: Snapshot =
            serde_json::from_slice(&data).map_err(|e| StoreError::Serialization(e.to_string()))?;

        Ok(snapshot)
    }

    async fn common_ancestor(
        &self,
        a: &SnapshotId,
        b: &SnapshotId,
    ) -> Result<Option<SnapshotId>, StoreError> {
        if a == b {
            return Ok(Some(a.clone()));
        }

        // BFS from both snapshots to find first intersection
        let mut visited_a: HashSet<String> = HashSet::new();
        let mut visited_b: HashSet<String> = HashSet::new();
        let mut queue_a: VecDeque<SnapshotId> = VecDeque::new();
        let mut queue_b: VecDeque<SnapshotId> = VecDeque::new();

        visited_a.insert(a.0.clone());
        visited_b.insert(b.0.clone());
        queue_a.push_back(a.clone());
        queue_b.push_back(b.clone());

        loop {
            let progress_a = if let Some(current) = queue_a.pop_front() {
                if visited_b.contains(&current.0) {
                    return Ok(Some(current));
                }
                if let Ok(snap) = self.get_snapshot(&current).await {
                    for parent in &snap.parents {
                        if visited_a.insert(parent.0.clone()) {
                            queue_a.push_back(parent.clone());
                        }
                    }
                }
                true
            } else {
                false
            };

            let progress_b = if let Some(current) = queue_b.pop_front() {
                if visited_a.contains(&current.0) {
                    return Ok(Some(current));
                }
                if let Ok(snap) = self.get_snapshot(&current).await {
                    for parent in &snap.parents {
                        if visited_b.insert(parent.0.clone()) {
                            queue_b.push_back(parent.clone());
                        }
                    }
                }
                true
            } else {
                false
            };

            if !progress_a && !progress_b {
                break;
            }
        }

        Ok(None)
    }

    async fn get_branch(&self, name: &str) -> Result<Option<SnapshotId>, StoreError> {
        let key = Self::branch_key(name);
        match self.store.get(&key).await {
            Ok(data) => {
                let id_str = String::from_utf8_lossy(&data).trim().to_string();
                Ok(Some(SnapshotId(id_str)))
            }
            Err(_) => Ok(None),
        }
    }

    async fn set_branch(&self, name: &str, id: &SnapshotId) -> Result<(), StoreError> {
        let key = Self::branch_key(name);
        self.store.put(&key, id.0.as_bytes()).await
    }

    async fn list_branches(&self) -> Result<Vec<String>, StoreError> {
        let keys = self.store.list("branches/").await?;
        Ok(keys
            .into_iter()
            .filter_map(|k| {
                k.strip_prefix("branches/")
                    .and_then(|s| s.strip_suffix(".json"))
                    .map(|s| s.to_string())
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::memory_store::InMemoryObjectStore;

    fn make_store() -> BuiltinVersionStore {
        BuiltinVersionStore::new(Arc::new(InMemoryObjectStore::new()))
    }

    #[tokio::test]
    async fn blob_put_get_roundtrip() {
        let store = make_store();
        let hash = store.put_blob(b"hello world").await.unwrap();
        let data = store.get_blob(&hash).await.unwrap();
        assert_eq!(data, b"hello world");
    }

    #[tokio::test]
    async fn blob_idempotent() {
        let store = make_store();
        let h1 = store.put_blob(b"content").await.unwrap();
        let h2 = store.put_blob(b"content").await.unwrap();
        assert_eq!(h1, h2);
    }

    #[tokio::test]
    async fn blob_not_found() {
        let store = make_store();
        let result = store.get_blob(&BlobHash("nonexistent".to_string())).await;
        assert!(matches!(result, Err(StoreError::BlobNotFound(_))));
    }

    #[tokio::test]
    async fn snapshot_put_get_roundtrip() {
        let store = make_store();
        let snapshot = Snapshot {
            id: SnapshotId(String::new()),
            parents: vec![],
            files: BTreeMap::from([("a.txt".to_string(), BlobHash::of(b"aaa"))]),
            provenance: BTreeMap::new(),
            timestamp: 1000,
            message: Some("first".to_string()),
        };

        let id = store.put_snapshot(&snapshot).await.unwrap();
        let retrieved = store.get_snapshot(&id).await.unwrap();
        assert_eq!(retrieved.id, id);
        assert_eq!(retrieved.files.len(), 1);
        assert_eq!(retrieved.message, Some("first".to_string()));
    }

    #[tokio::test]
    async fn snapshot_not_found() {
        let store = make_store();
        let result = store
            .get_snapshot(&SnapshotId("nonexistent".to_string()))
            .await;
        assert!(matches!(result, Err(StoreError::SnapshotNotFound(_))));
    }

    #[tokio::test]
    async fn branch_operations() {
        let store = make_store();

        // No branch initially
        assert!(store.get_branch("main").await.unwrap().is_none());

        // Set and get
        let id = SnapshotId("snap123".to_string());
        store.set_branch("main", &id).await.unwrap();
        let got = store.get_branch("main").await.unwrap();
        assert_eq!(got, Some(id));

        // List
        store
            .set_branch("develop", &SnapshotId("snap456".to_string()))
            .await
            .unwrap();
        let branches = store.list_branches().await.unwrap();
        assert!(branches.contains(&"main".to_string()));
        assert!(branches.contains(&"develop".to_string()));
    }

    #[tokio::test]
    async fn common_ancestor_same() {
        let store = make_store();
        let id = SnapshotId("same".to_string());
        let result = store.common_ancestor(&id, &id).await.unwrap();
        assert_eq!(result, Some(id));
    }

    #[tokio::test]
    async fn common_ancestor_linear() {
        let store = make_store();

        // Create chain: s1 -> s2 -> s3
        let s1 = Snapshot {
            id: SnapshotId(String::new()),
            parents: vec![],
            files: BTreeMap::new(),
            provenance: BTreeMap::new(),
            timestamp: 1,
            message: Some("s1".to_string()),
        };
        let id1 = store.put_snapshot(&s1).await.unwrap();

        let s2 = Snapshot {
            id: SnapshotId(String::new()),
            parents: vec![id1.clone()],
            files: BTreeMap::new(),
            provenance: BTreeMap::new(),
            timestamp: 2,
            message: Some("s2".to_string()),
        };
        let id2 = store.put_snapshot(&s2).await.unwrap();

        let s3 = Snapshot {
            id: SnapshotId(String::new()),
            parents: vec![id2.clone()],
            files: BTreeMap::new(),
            provenance: BTreeMap::new(),
            timestamp: 3,
            message: Some("s3".to_string()),
        };
        let id3 = store.put_snapshot(&s3).await.unwrap();

        // Common ancestor of s2 and s3 should be s2
        let result = store.common_ancestor(&id2, &id3).await.unwrap();
        assert_eq!(result, Some(id2.clone()));

        // Common ancestor of s1 and s3 should be s1
        let result = store.common_ancestor(&id1, &id3).await.unwrap();
        assert_eq!(result, Some(id1));
    }

    #[tokio::test]
    async fn common_ancestor_forked() {
        let store = make_store();

        // Create fork:
        //   s1 -> s2a
        //   s1 -> s2b
        let s1 = Snapshot {
            id: SnapshotId(String::new()),
            parents: vec![],
            files: BTreeMap::new(),
            provenance: BTreeMap::new(),
            timestamp: 1,
            message: Some("root".to_string()),
        };
        let id1 = store.put_snapshot(&s1).await.unwrap();

        let s2a = Snapshot {
            id: SnapshotId(String::new()),
            parents: vec![id1.clone()],
            files: BTreeMap::from([("a.txt".to_string(), BlobHash::of(b"a"))]),
            provenance: BTreeMap::new(),
            timestamp: 2,
            message: Some("branch-a".to_string()),
        };
        let id2a = store.put_snapshot(&s2a).await.unwrap();

        let s2b = Snapshot {
            id: SnapshotId(String::new()),
            parents: vec![id1.clone()],
            files: BTreeMap::from([("b.txt".to_string(), BlobHash::of(b"b"))]),
            provenance: BTreeMap::new(),
            timestamp: 3,
            message: Some("branch-b".to_string()),
        };
        let id2b = store.put_snapshot(&s2b).await.unwrap();

        // Common ancestor should be s1
        let result = store.common_ancestor(&id2a, &id2b).await.unwrap();
        assert_eq!(result, Some(id1));
    }

    #[tokio::test]
    async fn common_ancestor_no_common() {
        let store = make_store();

        let s1 = Snapshot {
            id: SnapshotId(String::new()),
            parents: vec![],
            files: BTreeMap::new(),
            provenance: BTreeMap::new(),
            timestamp: 1,
            message: Some("root-a".to_string()),
        };
        let id1 = store.put_snapshot(&s1).await.unwrap();

        let s2 = Snapshot {
            id: SnapshotId(String::new()),
            parents: vec![],
            files: BTreeMap::from([("x".to_string(), BlobHash::of(b"x"))]),
            provenance: BTreeMap::new(),
            timestamp: 2,
            message: Some("root-b".to_string()),
        };
        let id2 = store.put_snapshot(&s2).await.unwrap();

        let result = store.common_ancestor(&id1, &id2).await.unwrap();
        assert_eq!(result, None);
    }
}
