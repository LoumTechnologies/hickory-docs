pub mod anthropic;
pub mod openai;

// Re-export the unified LLM types from the engine crate.
pub use hick_agent_engine::llm::{ChatChunk, LlmClient, Message, Role};

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;

/// Mock LLM client for testing.
pub struct MockLlmClient {
    pub response: String,
}

impl MockLlmClient {
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
        }
    }
}

#[async_trait]
impl LlmClient for MockLlmClient {
    async fn complete(&self, _messages: Vec<Message>) -> anyhow::Result<String> {
        Ok(self.response.clone())
    }

    async fn complete_stream(
        &self,
        _messages: Vec<Message>,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>> {
        let response = self.response.clone();
        let stream = futures::stream::once(async move {
            Ok(ChatChunk {
                delta: response,
                finish_reason: Some("stop".into()),
            })
        });
        Ok(Box::pin(stream))
    }

    fn provider_name(&self) -> &str {
        "mock"
    }

    fn model_name(&self) -> &str {
        "mock"
    }
}
