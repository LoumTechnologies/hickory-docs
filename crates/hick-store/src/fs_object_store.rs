//! Filesystem-backed ObjectStore.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use log::debug;

use crate::error::StoreError;
use crate::object_store::ObjectStore;

/// Filesystem implementation of `ObjectStore`.
///
/// Keys map to file paths under a root directory. Directory separators
/// in keys (e.g., `blobs/abc`) create subdirectories.
pub struct FsObjectStore {
    root: PathBuf,
}

impl FsObjectStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    fn key_path(&self, key: &str) -> PathBuf {
        self.root.join(key)
    }
}

#[async_trait]
impl ObjectStore for FsObjectStore {
    async fn put(&self, key: &str, data: &[u8]) -> Result<(), StoreError> {
        let path = self.key_path(key);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Atomic write: write to temp file then rename
        let tmp_path = path.with_extension("tmp");
        tokio::fs::write(&tmp_path, data).await?;
        tokio::fs::rename(&tmp_path, &path).await?;

        debug!("Stored object: {key} ({} bytes)", data.len());
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, StoreError> {
        let path = self.key_path(key);
        tokio::fs::read(&path).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => StoreError::Other(format!("key not found: {key}")),
            _ => StoreError::Io(e),
        })
    }

    async fn exists(&self, key: &str) -> Result<bool, StoreError> {
        let path = self.key_path(key);
        Ok(path.exists())
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        let dir = self.key_path(prefix);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut keys = Vec::new();
        let mut read_dir = tokio::fs::read_dir(&dir).await?;

        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if let Ok(relative) = path.strip_prefix(&self.root) {
                keys.push(relative.to_string_lossy().into_owned());
            }
        }

        keys.sort();
        Ok(keys)
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        let path = self.key_path(key);
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StoreError::Io(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fs_put_get_roundtrip() {
        let dir = std::env::temp_dir().join("hick-store-fs-test");
        let _ = std::fs::remove_dir_all(&dir);

        let store = FsObjectStore::new(&dir);
        store.put("blobs/abc", b"hello").await.unwrap();
        let result = store.get("blobs/abc").await.unwrap();
        assert_eq!(result, b"hello");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn fs_atomic_write() {
        let dir = std::env::temp_dir().join("hick-store-fs-atomic-test");
        let _ = std::fs::remove_dir_all(&dir);

        let store = FsObjectStore::new(&dir);
        store.put("data/file.json", b"content").await.unwrap();

        // No temp files should remain
        let tmp_path = dir.join("data/file.tmp");
        assert!(!tmp_path.exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn fs_exists_and_delete() {
        let dir = std::env::temp_dir().join("hick-store-fs-exists-test");
        let _ = std::fs::remove_dir_all(&dir);

        let store = FsObjectStore::new(&dir);
        assert!(!store.exists("key").await.unwrap());
        store.put("key", b"val").await.unwrap();
        assert!(store.exists("key").await.unwrap());
        store.delete("key").await.unwrap();
        assert!(!store.exists("key").await.unwrap());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
