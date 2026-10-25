<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-store/src/object_store.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `512010866630faf3e0a28126213cb11a8825b3d144c15856358993a9b67e7fbf`.

<book:entry path="crates/hick-store/src/object_store.rs">
//! ObjectStore trait: low-level key-value storage for blobs and metadata.

use async_trait::async_trait;

use crate::error::StoreError;

/// Low-level key-value store for binary objects.
///
/// Used by `BuiltinVersionStore` to persist blobs, snapshots, and branches.
#[async_trait]
pub trait ObjectStore: Send + Sync {
    /// Write data at a key. Idempotent for content-addressed keys.
    async fn put(&self, key: &str, data: &[u8]) -> Result<(), StoreError>;

    /// Read data at a key.
    async fn get(&self, key: &str) -> Result<Vec<u8>, StoreError>;

    /// Check if a key exists.
    async fn exists(&self, key: &str) -> Result<bool, StoreError>;

    /// List keys with a given prefix.
    async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError>;

    /// Delete a key.
    async fn delete(&self, key: &str) -> Result<(), StoreError>;
}
</book:entry>

</book:doc>
