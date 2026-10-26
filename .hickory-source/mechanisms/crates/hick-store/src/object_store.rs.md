<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-store/src/object_store.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7755adce83bb"></a>
## `crates/hick-store/src/object_store.rs`

<a id="f-7755adce83bb-ObjectStore"></a>
### `ObjectStore`

ObjectStore trait: low-level key-value storage for blobs and metadata.

<storycode:copy id="f-7755adce83bb-ObjectStore">
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
</storycode:copy>

### Assemble `crates/hick-store/src/object_store.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-store/src/object_store.rs"><storycode:paste select="#f-7755adce83bb-ObjectStore"/></storycode:file>

</storycode:doc>
