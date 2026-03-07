//! Error types for the version store.

use crate::types::{BlobHash, SnapshotId};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("blob not found: {0}")]
    BlobNotFound(BlobHash),

    #[error("snapshot not found: {0}")]
    SnapshotNotFound(SnapshotId),

    #[error("branch not found: {0}")]
    BranchNotFound(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("git command failed: {0}")]
    GitCommand(String),

    #[error("concurrent modification on branch '{0}'")]
    ConcurrentModification(String),

    #[error("{0}")]
    Other(String),
}
