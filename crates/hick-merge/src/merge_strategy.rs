//! MergeStrategy trait for resolving three-way conflicts.

use async_trait::async_trait;

use crate::error::MergeError;

/// Strategy for resolving three-way merge conflicts.
///
/// Given the base version (common ancestor), the generated version (pipeline output),
/// and the edited version (what the user may have modified on disk), produce a
/// merged result.
#[async_trait]
pub trait MergeStrategy: Send + Sync {
    /// Merge three versions of a file.
    ///
    /// - `path`: the file path (for context in LLM-based strategies)
    /// - `base`: the version from the last snapshot (common ancestor)
    /// - `generated`: the new pipeline output
    /// - `edited`: the current disk version (possibly user-modified)
    async fn merge(
        &self,
        path: &str,
        base: &[u8],
        generated: &[u8],
        edited: &[u8],
    ) -> Result<Vec<u8>, MergeError>;
}
