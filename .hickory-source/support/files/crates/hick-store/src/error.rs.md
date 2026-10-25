<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-store/src/error.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `d0d6bc7fe2b02fbbd974f5563361de2189698aedae927f0cd09ab35f35015611`.

<book:entry path="crates/hick-store/src/error.rs">
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
</book:entry>

</book:doc>
