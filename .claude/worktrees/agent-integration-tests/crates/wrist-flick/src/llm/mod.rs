pub mod anthropic;
pub mod openai;

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone)]
pub struct ChatChunk {
    pub delta: String,
    pub finish_reason: Option<String>,
}

#[async_trait]
pub trait StreamingLlmClient: Send + Sync {
    async fn complete(&self, messages: Vec<Message>) -> anyhow::Result<String>;
    async fn complete_stream(
        &self,
        messages: Vec<Message>,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>>;
}

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
impl StreamingLlmClient for MockLlmClient {
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
}
