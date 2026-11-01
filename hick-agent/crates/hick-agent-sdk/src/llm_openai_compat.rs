//! Generic OpenAI-compatible LLM client.
//!
//! Works with OpenAI, xAI, Groq, Together, Ollama, OpenRouter, and any other
//! API that implements the OpenAI chat completions interface.

use std::pin::Pin;
use std::sync::{Arc, Mutex};

use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    ChatCompletionStreamOptions, CreateChatCompletionRequestArgs,
};
use async_trait::async_trait;
use futures::{Stream, StreamExt};

use crate::llm::{ChatChunk, LlmClient, Message, Role};
use crate::token_usage::TokenUsage;

/// OpenAI-compatible chat completion client (works with OpenAI, xAI, Groq, Ollama, etc.).
///
/// Configurable base URL allows use with any OpenAI-compatible API.
pub struct OpenAiCompatClient {
    client: async_openai::Client<OpenAIConfig>,
    model: String,
    provider: String,
    last_usage: Arc<Mutex<Option<TokenUsage>>>,
}

impl OpenAiCompatClient {
    /// Standard OpenAI API. Reads `OPENAI_API_KEY` from env.
    pub fn openai(model: String) -> Self {
        let config = OpenAIConfig::new();
        Self {
            client: async_openai::Client::with_config(config),
            model,
            provider: "openai".into(),
            last_usage: Arc::new(Mutex::new(None)),
        }
    }

    /// xAI (Grok) API. Reads `XAI_API_KEY` from env.
    pub fn xai(model: String) -> Self {
        let api_key = std::env::var("XAI_API_KEY").unwrap_or_default();
        Self::custom("https://api.x.ai/v1", api_key, model, "xai")
    }

    /// DeepSeek API. Reads `DEEPSEEK_API_KEY` from env.
    pub fn deepseek(model: String) -> Self {
        let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();
        Self::custom("https://api.deepseek.com", api_key, model, "deepseek")
    }

    /// Any OpenAI-compatible endpoint.
    pub fn custom(
        api_base: impl Into<String>,
        api_key: impl Into<String>,
        model: String,
        provider_name: impl Into<String>,
    ) -> Self {
        let config = OpenAIConfig::new()
            .with_api_base(api_base)
            .with_api_key(api_key);
        Self {
            client: async_openai::Client::with_config(config),
            model,
            provider: provider_name.into(),
            last_usage: Arc::new(Mutex::new(None)),
        }
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
impl LlmClient for OpenAiCompatClient {
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

        if let Some(usage) = response.usage {
            *self.last_usage.lock().unwrap() = Some(TokenUsage {
                input_tokens: usage.prompt_tokens as u64,
                output_tokens: usage.completion_tokens as u64,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
                provider: self.provider.clone(),
                model: self.model.clone(),
            });
        }

        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();
        Ok(content)
    }

    async fn complete_stream(
        &self,
        messages: Vec<Message>,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>> {
        let api_messages: Vec<ChatCompletionRequestMessage> = messages
            .into_iter()
            .map(to_openai_message)
            .collect::<anyhow::Result<_>>()?;

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(api_messages)
            .stream_options(ChatCompletionStreamOptions { include_usage: true })
            .build()?;

        let stream = self.client.chat().create_stream(request).await?;

        let usage_out = Arc::clone(&self.last_usage);
        let provider = self.provider.clone();
        let model = self.model.clone();

        let mapped = stream.map(move |result| {
            let response = result.map_err(|e| anyhow::anyhow!("{}", e))?;

            // The last stream chunk carries usage when include_usage=true.
            if let Some(usage) = &response.usage {
                *usage_out.lock().unwrap() = Some(TokenUsage {
                    input_tokens: usage.prompt_tokens as u64,
                    output_tokens: usage.completion_tokens as u64,
                    cache_creation_tokens: 0,
                    cache_read_tokens: 0,
                    provider: provider.clone(),
                    model: model.clone(),
                });
            }

            let choice = response.choices.first();
            let delta = choice
                .and_then(|c| c.delta.content.clone())
                .unwrap_or_default();
            let finish = choice.and_then(|c| {
                c.finish_reason
                    .as_ref()
                    .map(|r| format!("{:?}", r).to_lowercase())
            });
            Ok(ChatChunk {
                delta,
                finish_reason: finish,
            })
        });

        Ok(Box::pin(mapped))
    }

    fn provider_name(&self) -> &str {
        &self.provider
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn last_usage(&self) -> Option<TokenUsage> {
        self.last_usage.lock().unwrap().clone()
    }
}
