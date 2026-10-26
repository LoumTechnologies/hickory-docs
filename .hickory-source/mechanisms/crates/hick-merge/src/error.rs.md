<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-merge/src/error.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3122fdde27a8"></a>
## `crates/hick-merge/src/error.rs`

<a id="f-3122fdde27a8-MergeError"></a>
### `MergeError`

<storycode:copy id="f-3122fdde27a8-MergeError">
//! Error types for the merge system.

#[derive(Debug, thiserror::Error)]
pub enum MergeError {
    #[error("conflict on file '{path}': {reason}")]
    Conflict { path: String, reason: String },

    #[error("store error: {0}")]
    Store(#[from] hick_store::StoreError),

    #[error("LLM merge API error: {0}")]
    LlmApi(String),

    #[error("merge strategy failed: {0}")]
    StrategyFailed(String),

    #[error("{0}")]
    Other(String),
}
</storycode:copy>

### Assemble `crates/hick-merge/src/error.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-merge/src/error.rs"><storycode:paste select="#f-3122fdde27a8-MergeError"/></storycode:file>

</storycode:doc>
