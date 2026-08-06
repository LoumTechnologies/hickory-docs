//! Anthropic Messages API client.
//!
//! Supports both streaming and non-streaming completions via the Anthropic
//! Messages API (`https://api.anthropic.com/v1/messages`).
//!
//! The API key resolves, in order: an explicit per-client override (BYO-key
//! plans construct one client per request via [`AnthropicClient::with_api_key`])
//! then the `ANTHROPIC_API_KEY` environment variable.

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};

use crate::llm::{ChatChunk, ChatStream, LlmClient, Message, Role};

const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Default model: the current Sonnet-class alias (no date suffix).
pub const DEFAULT_ANTHROPIC_MODEL: &str = "claude-sonnet-5";

/// Anthropic chat completion client with streaming support.
pub struct AnthropicClient {
    client: reqwest::Client,
    api_key: String,
    model: String,
    max_tokens: u32,
}

impl AnthropicClient {
    /// Create with `ANTHROPIC_API_KEY` from env and the default model
    /// ([`DEFAULT_ANTHROPIC_MODEL`]).
    pub fn new() -> Self {
        let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        Self {
            client: reqwest::Client::new(),
            api_key,
            model: DEFAULT_ANTHROPIC_MODEL.to_string(),
            max_tokens: 16384,
        }
    }

    /// Override the model id (e.g. `claude-haiku-4-5`).
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Override the API key for this client (per-call/BYO-key plans:
    /// construct one client per request).
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = api_key.into();
        self
    }

    /// Override the max output tokens (default 16384).
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    fn request_builder(&self, body: &AnthropicRequest) -> reqwest::RequestBuilder {
        self.client
            .post(ANTHROPIC_API_BASE)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(body)
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
            Role::System => system = Some(m.content),
            Role::User => api_messages.push(AnthropicMessage {
                role: "user".into(),
                content: m.content,
            }),
            Role::Assistant => api_messages.push(AnthropicMessage {
                role: "assistant".into(),
                content: m.content,
            }),
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
        let resp = self.request_builder(&body).send().await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await?;
            anyhow::bail!("Anthropic API error {}: {}", status, text);
        }

        let parsed: AnthropicResponse = resp.json().await?;
        Ok(parsed
            .content
            .into_iter()
            .filter_map(|b| b.text)
            .collect::<Vec<_>>()
            .join(""))
    }

    async fn complete_stream(&self, messages: Vec<Message>) -> anyhow::Result<ChatStream> {
        let body = build_request(&self.model, self.max_tokens, messages, true);
        let resp = self.request_builder(&body).send().await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await?;
            anyhow::bail!("Anthropic API error {}: {}", status, text);
        }

        let byte_stream = resp.bytes_stream();
        Ok(Box::pin(SseParser::new(byte_stream)))
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

// ---------------------------------------------------------------------------
// SSE parsing
// ---------------------------------------------------------------------------

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
            if let Some(chunk) = parse_next_event(&mut this.buffer) {
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

/// Consume one complete SSE event from `buffer`, returning a chunk for the
/// event types we care about (`content_block_delta`, `message_delta`,
/// `message_stop`). Returns `None` when no complete event is buffered.
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
            _ => continue,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_parser_extracts_text_deltas() {
        let mut buf = String::from(
            "event: content_block_delta\ndata: {\"delta\":{\"text\":\"hi\"}}\n\n\
             event: message_stop\ndata: {}\n\n",
        );
        let c1 = parse_next_event(&mut buf).unwrap();
        assert_eq!(c1.delta, "hi");
        let c2 = parse_next_event(&mut buf).unwrap();
        assert_eq!(c2.finish_reason.as_deref(), Some("end_turn"));
        assert!(parse_next_event(&mut buf).is_none());
    }

    #[test]
    fn build_request_splits_system() {
        let req = build_request(
            "m",
            10,
            vec![
                Message::new(Role::System, "sys"),
                Message::new(Role::User, "hi"),
            ],
            false,
        );
        assert_eq!(req.system.as_deref(), Some("sys"));
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, "user");
    }
}
