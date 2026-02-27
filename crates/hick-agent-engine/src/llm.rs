//! LLM client abstraction for the coding agent.

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
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

/// A chunk of streamed LLM output.
#[derive(Debug, Clone)]
pub struct ChatChunk {
    /// The text fragment in this chunk.
    pub delta: String,
    /// Set when the stream is complete (e.g. "stop", "end_turn").
    pub finish_reason: Option<String>,
}

/// Client for LLM completions.
///
/// Implementations may use OpenAI, Anthropic, or any OpenAI-compatible provider.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Complete a chat conversation (returns full response).
    async fn complete(&self, messages: Vec<Message>) -> anyhow::Result<String>;

    /// Stream a chat completion. Returns a stream of chunks.
    ///
    /// Default implementation collects the full response from [`complete`] and
    /// emits it as a single chunk. Providers that support streaming should
    /// override this for incremental output.
    async fn complete_stream(
        &self,
        messages: Vec<Message>,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>> {
        let response = self.complete(messages).await?;
        let stream = futures::stream::once(async move {
            Ok(ChatChunk {
                delta: response,
                finish_reason: Some("stop".into()),
            })
        });
        Ok(Box::pin(stream))
    }

    /// Provider name for display.
    fn provider_name(&self) -> &str;

    /// Model name for display.
    fn model_name(&self) -> &str;
}
