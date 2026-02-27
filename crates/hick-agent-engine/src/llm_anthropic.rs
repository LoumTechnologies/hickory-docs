//! Anthropic Messages API client.
//!
//! Supports both streaming and non-streaming completions via the
//! Anthropic Messages API (https://api.anthropic.com/v1/messages).

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};

use crate::llm::{ChatChunk, LlmClient, Message, Role};

const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic chat completion client.
pub struct AnthropicClient {
    client: reqwest::Client,
    api_key: String,
    model: String,
    max_tokens: u32,
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
        }
    }

    /// Create with explicit model and API key.
    pub fn with_config(model: String, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            max_tokens: 16384,
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
}

#[derive(Deserialize)]
struct AnthropicContentBlock {
    text: Option<String>,
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
                // Anthropic uses a top-level system field, not a system message
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
        let stream = SseParser::new(byte_stream);
        Ok(Box::pin(stream))
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

/// Parses Anthropic SSE events from a byte stream into ChatChunk items.
struct SseParser<S> {
    inner: S,
    buffer: String,
}

impl<S> SseParser<S> {
    fn new(inner: S) -> Self {
        Self {
            inner,
            buffer: String::new(),
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
            // Try to extract a complete SSE event from the buffer
            if let Some(chunk) = parse_next_event(&mut this.buffer) {
                return std::task::Poll::Ready(Some(Ok(chunk)));
            }

            // Need more data from the underlying stream
            let inner = Pin::new(&mut this.inner);
            match inner.poll_next(cx) {
                std::task::Poll::Ready(Some(Ok(bytes))) => {
                    this.buffer.push_str(&String::from_utf8_lossy(&bytes));
                }
                std::task::Poll::Ready(Some(Err(e))) => {
                    return std::task::Poll::Ready(Some(Err(anyhow::anyhow!("{}", e))));
                }
                std::task::Poll::Ready(None) => {
                    // Stream ended — emit any remaining buffered event
                    if let Some(chunk) = parse_next_event(&mut this.buffer) {
                        return std::task::Poll::Ready(Some(Ok(chunk)));
                    }
                    return std::task::Poll::Ready(None);
                }
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }
        }
    }
}

fn parse_next_event(buffer: &mut String) -> Option<ChatChunk> {
    loop {
        let boundary = buffer.find("\n\n")?;
        let event_block = buffer[..boundary].to_string();
        *buffer = buffer[boundary + 2..].to_string();

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
                // Skip other event types (message_start, content_block_start, ping, etc.)
                continue;
            }
        }
    }
}
