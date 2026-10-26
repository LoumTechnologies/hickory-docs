<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-merge/src/merge_strategy.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4c9131267e18"></a>
## `crates/hick-merge/src/merge_strategy.rs`

<a id="f-4c9131267e18-MergeStrategy"></a>
### `MergeStrategy`

MergeStrategy trait for resolving three-way conflicts.

<storycode:copy id="f-4c9131267e18-MergeStrategy">
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
</storycode:copy>

### Assemble `crates/hick-merge/src/merge_strategy.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-merge/src/merge_strategy.rs"><storycode:paste select="#f-4c9131267e18-MergeStrategy"/></storycode:file>

</storycode:doc>
