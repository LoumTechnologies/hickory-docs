//! OpenAI API implementation of the LLM client.

use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs,
};
use async_trait::async_trait;

use crate::llm::{LlmClient, Message, Role};

/// OpenAI chat completion client.
pub struct OpenAiClient {
    client: async_openai::Client<async_openai::config::OpenAIConfig>,
    model: String,
}

impl OpenAiClient {
    /// Create a new client using the default config (reads OPENAI_API_KEY from env).
    pub fn new() -> Self {
        Self::with_model("gpt-4o-mini".to_string())
    }

    /// Create with a specific model.
    pub fn with_model(model: String) -> Self {
        let config = async_openai::config::OpenAIConfig::new();
        Self {
            client: async_openai::Client::with_config(config),
            model,
        }
    }
}

impl Default for OpenAiClient {
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
impl LlmClient for OpenAiClient {
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
