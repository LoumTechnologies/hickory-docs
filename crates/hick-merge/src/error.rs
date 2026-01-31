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
