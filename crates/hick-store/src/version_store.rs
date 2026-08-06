//! VersionStore trait: the core abstraction for snapshot-based version tracking.

use async_trait::async_trait;

use crate::error::StoreError;
use crate::types::{BlobHash, Snapshot, SnapshotId};

/// Trait for storing and retrieving versioned pipeline state.
///
/// Implementations manage blobs (file contents), snapshots (point-in-time state),
/// and branches (named pointers to snapshots).
#[async_trait]
pub trait VersionStore: Send + Sync {
    /// Store a blob and return its content-addressed hash.
    async fn put_blob(&self, content: &[u8]) -> Result<BlobHash, StoreError>;

    /// Retrieve a blob by its hash.
    async fn get_blob(&self, hash: &BlobHash) -> Result<Vec<u8>, StoreError>;

    /// Store a snapshot and return its ID.
    async fn put_snapshot(&self, snapshot: &Snapshot) -> Result<SnapshotId, StoreError>;

    /// Retrieve a snapshot by its ID.
    async fn get_snapshot(&self, id: &SnapshotId) -> Result<Snapshot, StoreError>;

    /// Find the most recent common ancestor of two snapshots.
    ///
    /// Returns `None` if the snapshots share no common history.
    async fn common_ancestor(
        &self,
        a: &SnapshotId,
        b: &SnapshotId,
    ) -> Result<Option<SnapshotId>, StoreError>;

    /// Get the snapshot ID that a branch points to.
    async fn get_branch(&self, name: &str) -> Result<Option<SnapshotId>, StoreError>;

    /// Set a branch to point to a snapshot.
    async fn set_branch(&self, name: &str, id: &SnapshotId) -> Result<(), StoreError>;

    /// List all branch names.
    async fn list_branches(&self) -> Result<Vec<String>, StoreError>;
}
