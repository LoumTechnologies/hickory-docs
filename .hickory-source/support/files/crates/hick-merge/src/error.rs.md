<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-merge/src/error.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `b2311bfc25397f7f990f471456ff83f70b2921cf3ef1d1581fbd478ac971d44e`.

<book:entry path="crates/hick-merge/src/error.rs">
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
</book:entry>

</book:doc>
