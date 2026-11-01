//! Per-call LLM token usage, used by the telemetry extension for billing.

use serde::{Deserialize, Serialize};

/// Token consumption from a single LLM API call.
///
/// No prompt content is stored here — only counts needed to estimate the
/// user's provider bill. Transmitted to the billing server by the telemetry
/// extension crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens in the prompt (excluding cache reads).
    pub input_tokens: u64,
    /// Tokens in the generated response.
    pub output_tokens: u64,
    /// Tokens written to the prompt cache this call (Anthropic-specific; zero otherwise).
    pub cache_creation_tokens: u64,
    /// Tokens served from the prompt cache this call (Anthropic-specific; zero otherwise).
    pub cache_read_tokens: u64,
    /// Provider name, e.g. "anthropic", "openai".
    pub provider: String,
    /// Model identifier, e.g. "claude-opus-4-7-20251101".
    pub model: String,
}
