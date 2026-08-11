//! LLM client abstraction: chat messages, streaming chunks, and the
//! [`LlmClient`] trait every provider implements.

use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};

use crate::usage::Usage;

/// A chat message with a role and content string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    /// Convenience constructor.
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }
}

/// Chat message role: System, User, or Assistant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// A chunk of streamed LLM output containing a text delta and optional
/// finish reason.
#[derive(Debug, Clone)]
pub struct ChatChunk {
    /// The text fragment in this chunk.
    pub delta: String,
    /// Set when the stream is complete (e.g. "stop", "end_turn").
    pub finish_reason: Option<String>,
    /// Partial usage carried by this chunk (Anthropic reports input +
    /// cache counters on `message_start` and output tokens on
    /// `message_delta`). Sum the usages of every chunk in a stream to get
    /// the call's total — see [`crate::Usage::add`].
    pub usage: Option<Usage>,
}

impl ChatChunk {
    /// A plain text delta with no finish reason or usage.
    pub fn text(delta: impl Into<String>) -> Self {
        Self {
            delta: delta.into(),
            finish_reason: None,
            usage: None,
        }
    }
}

/// A pinned, boxed stream of [`ChatChunk`]s.
pub type ChatStream = Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>;

/// Trait for LLM chat completion clients (streaming and non-streaming).
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Complete a chat conversation (returns full response text).
    async fn complete(&self, messages: Vec<Message>) -> anyhow::Result<String>;

    /// Complete a chat conversation, returning the text plus the call's
    /// four-way token [`Usage`]. Default: delegates to
    /// [`LlmClient::complete`] with zero usage (providers that bill should
    /// override).
    async fn complete_with_usage(&self, messages: Vec<Message>) -> anyhow::Result<(String, Usage)> {
        Ok((self.complete(messages).await?, Usage::default()))
    }

    /// Stream a chat completion. Returns a stream of chunks.
    ///
    /// Default implementation collects the full response from
    /// [`LlmClient::complete`] and emits it as a single chunk. Providers that
    /// support streaming should override this for incremental output.
    async fn complete_stream(&self, messages: Vec<Message>) -> anyhow::Result<ChatStream> {
        let response = self.complete(messages).await?;
        let stream = futures::stream::once(async move {
            Ok(ChatChunk {
                delta: response,
                finish_reason: Some("stop".into()),
                usage: None,
            })
        });
        Ok(Box::pin(stream))
    }

    /// Check that this client's credentials are accepted by the provider.
    ///
    /// For bring-your-own-key: a key is validated when it is *entered*, so a
    /// typo is one message in a form rather than a 401 surfacing three layers
    /// down in a streamed agent run, after the user has already described a
    /// task and waited.
    ///
    /// The default asks for a one-token completion — the smallest request
    /// every provider understands. Providers with a free credential-checking
    /// endpoint should override (Anthropic uses `count_tokens`).
    ///
    /// A network failure is reported like a rejection, because from here the
    /// two are indistinguishable and both mean "not usable right now".
    async fn validate_credentials(&self) -> anyhow::Result<()> {
        self.complete(vec![Message::new(Role::User, "ping")])
            .await
            .map(|_| ())
    }

    /// Provider name for display (e.g. "anthropic").
    fn provider_name(&self) -> &str;

    /// Model name for display (e.g. "claude-sonnet-5").
    fn model_name(&self) -> &str;
}

/// Blanket impl so an `Arc<dyn LlmClient>` can itself be used where an
/// `LlmClient` is expected.
#[async_trait]
impl LlmClient for Arc<dyn LlmClient> {
    async fn complete(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        (**self).complete(messages).await
    }

    async fn complete_with_usage(&self, messages: Vec<Message>) -> anyhow::Result<(String, Usage)> {
        (**self).complete_with_usage(messages).await
    }

    async fn complete_stream(&self, messages: Vec<Message>) -> anyhow::Result<ChatStream> {
        (**self).complete_stream(messages).await
    }

    async fn validate_credentials(&self) -> anyhow::Result<()> {
        (**self).validate_credentials().await
    }

    fn provider_name(&self) -> &str {
        (**self).provider_name()
    }

    fn model_name(&self) -> &str {
        (**self).model_name()
    }
}
