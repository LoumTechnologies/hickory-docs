//! LLM client abstraction for the coding agent.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// Client for LLM completions.
///
/// Implementations may use OpenAI, Anthropic, Ollama, or other providers.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Complete a chat conversation.
    async fn complete(&self, messages: Vec<Message>) -> anyhow::Result<String>;
}
