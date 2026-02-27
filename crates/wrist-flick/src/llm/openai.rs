use std::pin::Pin;

use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs,
};
use async_trait::async_trait;
use futures::{Stream, StreamExt};

use super::{ChatChunk, LlmClient, Message, Role};

pub struct OpenAiStreamingClient {
    client: async_openai::Client<async_openai::config::OpenAIConfig>,
    model: String,
}

impl OpenAiStreamingClient {
    pub fn new() -> Self {
        Self::with_model("gpt-4o-mini".to_string())
    }

    pub fn with_model(model: String) -> Self {
        let config = async_openai::config::OpenAIConfig::new();
        Self {
            client: async_openai::Client::with_config(config),
            model,
        }
    }
}

impl Default for OpenAiStreamingClient {
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
impl LlmClient for OpenAiStreamingClient {
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
            .build()?;

        let stream = self.client.chat().create_stream(request).await?;

        let mapped = stream.map(|result| {
            let response = result.map_err(|e| anyhow::anyhow!("{}", e))?;
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
        "openai"
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}
