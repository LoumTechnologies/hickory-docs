//! In-memory ObjectStore for testing.

use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::error::StoreError;
use crate::object_store::ObjectStore;

/// In-memory implementation of `ObjectStore`.
///
/// Useful for unit tests and as a reference implementation.
pub struct InMemoryObjectStore {
    data: RwLock<HashMap<String, Vec<u8>>>,
}

impl InMemoryObjectStore {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryObjectStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ObjectStore for InMemoryObjectStore {
    async fn put(&self, key: &str, data: &[u8]) -> Result<(), StoreError> {
        self.data
            .write()
            .await
            .insert(key.to_string(), data.to_vec());
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, StoreError> {
        self.data
            .read()
            .await
            .get(key)
            .cloned()
            .ok_or_else(|| StoreError::Other(format!("key not found: {key}")))
    }

    async fn exists(&self, key: &str) -> Result<bool, StoreError> {
        Ok(self.data.read().await.contains_key(key))
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        let data = self.data.read().await;
        let mut keys: Vec<String> = data
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        keys.sort();
        Ok(keys)
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        self.data.write().await.remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn put_get_roundtrip() {
        let store = InMemoryObjectStore::new();
        store.put("key1", b"value1").await.unwrap();
        let result = store.get("key1").await.unwrap();
        assert_eq!(result, b"value1");
    }

    #[tokio::test]
    async fn get_missing_key() {
        let store = InMemoryObjectStore::new();
        assert!(store.get("missing").await.is_err());
    }

    #[tokio::test]
    async fn exists_check() {
        let store = InMemoryObjectStore::new();
        assert!(!store.exists("key").await.unwrap());
        store.put("key", b"val").await.unwrap();
        assert!(store.exists("key").await.unwrap());
    }

    #[tokio::test]
    async fn list_with_prefix() {
        let store = InMemoryObjectStore::new();
        store.put("blobs/abc", b"1").await.unwrap();
        store.put("blobs/def", b"2").await.unwrap();
        store.put("snapshots/123", b"3").await.unwrap();

        let blobs = store.list("blobs/").await.unwrap();
        assert_eq!(blobs.len(), 2);

        let snaps = store.list("snapshots/").await.unwrap();
        assert_eq!(snaps.len(), 1);
    }

    #[tokio::test]
    async fn delete_removes_key() {
        let store = InMemoryObjectStore::new();
        store.put("key", b"val").await.unwrap();
        store.delete("key").await.unwrap();
        assert!(!store.exists("key").await.unwrap());
    }
}
