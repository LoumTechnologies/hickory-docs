<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-store/src/error.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d85e4c8717a8"></a>
## `crates/hick-store/src/error.rs`

<a id="f-d85e4c8717a8-StoreError"></a>
### `StoreError`

<storycode:copy id="f-d85e4c8717a8-StoreError">
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
</storycode:copy>

### Assemble `crates/hick-store/src/error.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-store/src/error.rs"><storycode:paste select="#f-d85e4c8717a8-StoreError"/></storycode:file>

</storycode:doc>
