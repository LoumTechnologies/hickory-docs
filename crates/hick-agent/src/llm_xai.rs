//! xAI API implementation of the LLM client.
//!
//! Uses the xAI API (https://api.x.ai), which is OpenAI-compatible.
//! Set `XAI_API_KEY` for authentication. Default model is `grok-beta`.

use async_trait::async_trait;
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs,
};
use async_openai::config::OpenAIConfig;

use crate::llm::{LlmClient, Message, Role};

/// xAI API base URL (OpenAI-compatible).
pub const XAI_API_BASE: &str = "https://api.x.ai/v1";

/// xAI chat completion client.
pub struct XaiClient {
    client: async_openai::Client<OpenAIConfig>,
    model: String,
}

impl XaiClient {
    /// Create a new client using XAI_API_KEY from env.
    /// Default model: grok-beta.
    pub fn new() -> Self {
        Self::with_model("grok-beta".to_string())
    }

    /// Create with a specific model (e.g. grok-beta, grok-2).
    pub fn with_model(model: String) -> Self {
        let api_key = std::env::var("XAI_API_KEY").unwrap_or_default();
        Self::with_config(model, api_key)
    }

    /// Create with explicit API key.
    pub fn with_api_key(model: String, api_key: impl Into<String>) -> Self {
        Self::with_config(model, api_key.into())
    }

    fn with_config(model: String, api_key: String) -> Self {
        let config = OpenAIConfig::new()
            .with_api_base(XAI_API_BASE)
            .with_api_key(api_key);
        Self {
            client: async_openai::Client::with_config(config),
            model,
        }
    }
}

impl Default for XaiClient {
    fn default() -> Self {
        Self::new()
    }
}

fn to_openai_message(m: Message) -> anyhow::Result<ChatCompletionRequestMessage> {
    Ok(match m.role {
        Role::System => ChatCompletionRequestSystemMessageArgs::default()
            .content(m.content)
            .build()?
            .into(),
        Role::User => ChatCompletionRequestUserMessageArgs::default()
            .content(m.content)
            .build()?
            .into(),
        Role::Assistant => ChatCompletionRequestAssistantMessageArgs::default()
            .content(m.content)
            .build()?
            .into(),
    })
}

#[async_trait]
impl LlmClient for XaiClient {
    async fn complete(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        let api_messages: Vec<ChatCompletionRequestMessage> = messages
            .into_iter()
            .map(to_openai_message)
            .collect::<anyhow::Result<_>>()?;

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(api_messages)
            .build()?;

        let response = self.client.chat().create(request).await?;

        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(content)
    }
}
