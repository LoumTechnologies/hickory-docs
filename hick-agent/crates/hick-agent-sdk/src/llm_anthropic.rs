//! Anthropic Messages API client.
//!
//! Supports both streaming and non-streaming completions via the
//! Anthropic Messages API (https://api.anthropic.com/v1/messages).

use std::pin::Pin;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};

use crate::llm::{ChatChunk, LlmClient, Message, Role};
use crate::token_usage::TokenUsage;

const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic chat completion client with streaming support.
pub struct AnthropicClient {
    client: reqwest::Client,
    api_key: String,
    model: String,
    max_tokens: u32,
    last_usage: Arc<Mutex<Option<TokenUsage>>>,
}

impl AnthropicClient {
    /// Create with `ANTHROPIC_API_KEY` from env. Default model: `claude-sonnet-4-5-20250929`.
    pub fn new() -> Self {
        let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        Self {
            client: reqwest::Client::new(),
            api_key,
            model: "claude-sonnet-4-5-20250929".to_string(),
            max_tokens: 16384,
            last_usage: Arc::new(Mutex::new(None)),
        }
    }

    /// Create with explicit model and API key.
    pub fn with_config(model: String, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            max_tokens: 16384,
            last_usage: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for AnthropicClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    stream: bool,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContentBlock>,
    usage: AnthropicUsage,
}

#[derive(Deserialize)]
struct AnthropicContentBlock {
    text: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: u64,
    output_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
}

fn build_request(
    model: &str,
    max_tokens: u32,
    messages: Vec<Message>,
    stream: bool,
) -> AnthropicRequest {
    let mut system = None;
    let mut api_messages = Vec::new();

    for m in messages {
        match m.role {
            Role::System => {
                system = Some(m.content);
            }
            Role::User => {
                api_messages.push(AnthropicMessage {
                    role: "user".into(),
                    content: m.content,
                });
            }
            Role::Assistant => {
                api_messages.push(AnthropicMessage {
                    role: "assistant".into(),
                    content: m.content,
                });
            }
        }
    }

    AnthropicRequest {
        model: model.to_string(),
        max_tokens,
        messages: api_messages,
        system,
        stream,
    }
}

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn complete(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        let body = build_request(&self.model, self.max_tokens, messages, false);

        let resp = self
            .client
            .post(ANTHROPIC_API_BASE)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await?;
            anyhow::bail!("Anthropic API error {}: {}", status, text);
        }

        let parsed: AnthropicResponse = resp.json().await?;

        *self.last_usage.lock().unwrap() = Some(TokenUsage {
            input_tokens: parsed.usage.input_tokens,
            output_tokens: parsed.usage.output_tokens,
            cache_creation_tokens: parsed.usage.cache_creation_input_tokens,
            cache_read_tokens: parsed.usage.cache_read_input_tokens,
            provider: "anthropic".into(),
            model: self.model.clone(),
        });

        let text = parsed
            .content
            .into_iter()
            .filter_map(|b| b.text)
            .collect::<Vec<_>>()
            .join("");
        Ok(text)
    }

    async fn complete_stream(
        &self,
        messages: Vec<Message>,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>> {
        let body = build_request(&self.model, self.max_tokens, messages, true);

        let resp = self
            .client
            .post(ANTHROPIC_API_BASE)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await?;
            anyhow::bail!("Anthropic API error {}: {}", status, text);
        }

        let byte_stream = resp.bytes_stream();
        let stream = SseParser::new(byte_stream, self.model.clone(), Arc::clone(&self.last_usage));
        Ok(Box::pin(stream))
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn last_usage(&self) -> Option<TokenUsage> {
        self.last_usage.lock().unwrap().clone()
    }
}

struct SseParser<S> {
    inner: S,
    buffer: String,
    model: String,
    usage_out: Arc<Mutex<Option<TokenUsage>>>,
    // Partial accumulators — input tokens arrive in message_start,
    // output tokens arrive in message_delta.
    input_acc: u64,
    cache_create_acc: u64,
    cache_read_acc: u64,
}

impl<S> SseParser<S> {
    fn new(inner: S, model: String, usage_out: Arc<Mutex<Option<TokenUsage>>>) -> Self {
        Self {
            inner,
            buffer: String::new(),
            model,
            usage_out,
            input_acc: 0,
            cache_create_acc: 0,
            cache_read_acc: 0,
        }
    }

    fn flush_usage(&mut self, output_tokens: u64) {
        *self.usage_out.lock().unwrap() = Some(TokenUsage {
            input_tokens: self.input_acc,
            output_tokens,
            cache_creation_tokens: self.cache_create_acc,
            cache_read_tokens: self.cache_read_acc,
            provider: "anthropic".into(),
            model: self.model.clone(),
        });
    }

    fn parse_next_event(&mut self) -> Option<ChatChunk> {
        loop {
            let boundary = self.buffer.find("\n\n")?;
            let event_block = self.buffer[..boundary].to_string();
            self.buffer = self.buffer[boundary + 2..].to_string();

            let mut event_type = String::new();
            let mut data = String::new();

            for line in event_block.lines() {
                if let Some(rest) = line.strip_prefix("event: ") {
                    event_type = rest.trim().to_string();
                } else if let Some(rest) = line.strip_prefix("data: ") {
                    data = rest.to_string();
                }
            }

            match event_type.as_str() {
                "message_start" => {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                        if let Some(usage) = parsed
                            .get("message")
                            .and_then(|m| m.get("usage"))
                        {
                            self.input_acc = usage
                                .get("input_tokens")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);
                            self.cache_create_acc = usage
                                .get("cache_creation_input_tokens")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);
                            self.cache_read_acc = usage
                                .get("cache_read_input_tokens")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);
                        }
                    }
                    continue;
                }
                "content_block_delta" => {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                        let text = parsed
                            .get("delta")
                            .and_then(|d| d.get("text"))
                            .and_then(|t| t.as_str())
                            .unwrap_or_default();
                        if !text.is_empty() {
                            return Some(ChatChunk {
                                delta: text.to_string(),
                                finish_reason: None,
                            });
                        }
                    }
                }
                "message_delta" => {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                        let output_tokens = parsed
                            .get("usage")
                            .and_then(|u| u.get("output_tokens"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        if output_tokens > 0 {
                            self.flush_usage(output_tokens);
                        }
                        let stop_reason = parsed
                            .get("delta")
                            .and_then(|d| d.get("stop_reason"))
                            .and_then(|r| r.as_str())
                            .map(|s| s.to_string());
                        if stop_reason.is_some() {
                            return Some(ChatChunk {
                                delta: String::new(),
                                finish_reason: stop_reason,
                            });
                        }
                    }
                }
                "message_stop" => {
                    return Some(ChatChunk {
                        delta: String::new(),
                        finish_reason: Some("end_turn".into()),
                    });
                }
                _ => {
                    continue;
                }
            }
        }
    }
}

impl<S> Stream for SseParser<S>
where
    S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin + Send,
{
    type Item = anyhow::Result<ChatChunk>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            if let Some(chunk) = this.parse_next_event() {
                return std::task::Poll::Ready(Some(Ok(chunk)));
            }

            let inner = Pin::new(&mut this.inner);
            match inner.poll_next(cx) {
                std::task::Poll::Ready(Some(Ok(bytes))) => {
                    this.buffer.push_str(&String::from_utf8_lossy(&bytes));
                }
                std::task::Poll::Ready(Some(Err(e))) => {
                    return std::task::Poll::Ready(Some(Err(anyhow::anyhow!("{}", e))));
                }
                std::task::Poll::Ready(None) => {
                    if let Some(chunk) = this.parse_next_event() {
                        return std::task::Poll::Ready(Some(Ok(chunk)));
                    }
                    return std::task::Poll::Ready(None);
                }
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }
        }
    }
}
