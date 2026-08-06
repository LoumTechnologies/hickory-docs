//! Anthropic Messages API client.
//!
//! Supports both streaming and non-streaming completions via the Anthropic
//! Messages API (`https://api.anthropic.com/v1/messages`), with prompt
//! caching, four-way usage capture, and configurable effort.
//!
//! ## Cache stability by construction
//!
//! The request body is built by one deterministic function
//! ([`AnthropicClient::request_body_bytes`] exposes its exact bytes for
//! tests): serde struct serialization has a fixed field order, system
//! messages become system blocks in input order, and nothing time- or
//! session-dependent is injected here. Breakpoint placement (max 4):
//!
//! 1. the **last system block** — caches the whole system prefix (the frozen
//!    protocol + tool doctrine live in the first system message; anything
//!    per-session comes as a *later* system message so the frozen part stays
//!    byte-identical across sessions);
//! 2. the **first system block** (only when there are two or more) — gives
//!    the frozen cross-session part its own cache entry;
//! 3. a **rolling breakpoint** on the last content block of the last
//!    message — multi-turn reuse;
//! 4. an **intermediate breakpoint** ~15 blocks before the end on long
//!    histories — a breakpoint only scans back 20 content blocks, so without
//!    it a long tool-heavy turn would leave earlier blocks uncached.
//!
//! We never send `temperature`/`top_p`/`top_k`, `thinking`, or
//! `budget_tokens`: on `claude-sonnet-5` adaptive thinking is on by default
//! when `thinking` is omitted, and non-default sampling params or
//! `budget_tokens` return 400.
//!
//! The API key resolves, in order: an explicit per-client override (BYO-key
//! plans construct one client per request via [`AnthropicClient::with_api_key`])
//! then the `ANTHROPIC_API_KEY` environment variable.

use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};

use crate::llm::{ChatChunk, ChatStream, LlmClient, Message, Role};
use crate::usage::{Usage, min_cacheable_prefix_tokens};

const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_COUNT_TOKENS_URL: &str = "https://api.anthropic.com/v1/messages/count_tokens";
const ANTHROPIC_VERSION: &str = "2023-06-01";
/// Beta header enabling context editing (`clear_tool_uses_20250919`).
const CONTEXT_MANAGEMENT_BETA: &str = "context-management-2025-06-27";

/// Default model: the current Sonnet-class alias (no date suffix).
pub const DEFAULT_ANTHROPIC_MODEL: &str = "claude-sonnet-5";

/// A breakpoint only scans back this many content blocks; long histories
/// get an intermediate breakpoint so nothing falls outside the lookback.
const CACHE_LOOKBACK_BLOCKS: usize = 20;
/// Place the intermediate breakpoint this many blocks before the rolling
/// one (comfortably inside the 20-block lookback).
const INTERMEDIATE_BREAKPOINT_GAP: usize = 15;

/// Output effort level (`output_config.effort`). The API default is
/// `high`; we omit the field unless explicitly configured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    Low,
    Medium,
    High,
    Xhigh,
    Max,
}

impl Effort {
    /// Parse a CLI/config string (`low|medium|high|xhigh|max`).
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            "xhigh" => Some(Self::Xhigh),
            "max" => Some(Self::Max),
            _ => None,
        }
    }
}

/// Anthropic chat completion client with streaming support.
pub struct AnthropicClient {
    client: reqwest::Client,
    api_key: String,
    model: String,
    max_tokens: u32,
    effort: Option<Effort>,
    context_editing: bool,
    prefix_checked: AtomicBool,
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
            effort: None,
            context_editing: false,
            prefix_checked: AtomicBool::new(false),
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

    /// Set `output_config.effort`. Omitted by default (API default: high).
    /// Cheap lookups run fine at [`Effort::Low`]; leave refactors at the
    /// default until the E4 sweep says otherwise.
    pub fn with_effort(mut self, effort: Effort) -> Self {
        self.effort = Some(effort);
        self
    }

    /// Enable context editing (`clear_tool_uses_20250919` via the
    /// `context-management-2025-06-27` beta) for long sessions. Off by
    /// default. Note: it clears API-native `tool_use`/`tool_result` blocks;
    /// while the agent speaks its text `<hick:tool>` protocol, this is
    /// forward-looking plumbing (harmless today, effective the day the loop
    /// moves to native tool blocks).
    pub fn with_context_editing(mut self, enabled: bool) -> Self {
        self.context_editing = enabled;
        self
    }

    fn request_builder(&self, url: &str, body: &impl Serialize) -> reqwest::RequestBuilder {
        let mut builder = self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json");
        if self.context_editing {
            builder = builder.header("anthropic-beta", CONTEXT_MANAGEMENT_BETA);
        }
        builder.json(body)
    }

    fn request_body(&self, messages: &[Message], stream: bool) -> AnthropicRequest {
        build_request(
            &self.model,
            self.max_tokens,
            self.effort,
            self.context_editing,
            messages,
            stream,
        )
    }

    /// The exact serialized bytes of the request this client would send for
    /// `messages`. Used by the cache byte-stability tests: two sessions with
    /// the same system + tools must produce identical bytes, or the prefix
    /// cache is silently dead.
    pub fn request_body_bytes(&self, messages: &[Message], stream: bool) -> Vec<u8> {
        serde_json::to_vec(&self.request_body(messages, stream))
            .expect("request serialization cannot fail")
    }

    /// Count tokens for `messages` via `POST /v1/messages/count_tokens` —
    /// the only valid token measurement (never estimate with a foreign
    /// tokenizer or a character heuristic). Requires an API key.
    pub async fn count_tokens(&self, messages: &[Message]) -> anyhow::Result<u64> {
        let body = self.request_body(messages, false);
        let count_body = CountTokensRequest {
            model: body.model,
            system: body.system,
            messages: body.messages,
        };
        let resp = self
            .request_builder(ANTHROPIC_COUNT_TOKENS_URL, &count_body)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await?;
            anyhow::bail!("Anthropic count_tokens error {}: {}", status, text);
        }
        let parsed: CountTokensResponse = resp.json().await?;
        Ok(parsed.input_tokens)
    }

    /// Measure (via `count_tokens`) whether the system prefix of `messages`
    /// meets the model's minimum cacheable size — below the minimum the API
    /// silently never caches. The probe includes one 1-character user
    /// message (the endpoint needs a non-empty conversation), which inflates
    /// the count by a few tokens; irrelevant at the 512/1024 threshold.
    pub async fn verify_cacheable_prefix(
        &self,
        messages: &[Message],
    ) -> anyhow::Result<PrefixCheck> {
        let mut probe: Vec<Message> = messages
            .iter()
            .filter(|m| m.role == Role::System)
            .cloned()
            .collect();
        probe.push(Message::new(Role::User, "."));
        let tokens = self.count_tokens(&probe).await?;
        let minimum = min_cacheable_prefix_tokens(&self.model);
        Ok(PrefixCheck {
            tokens,
            minimum,
            cacheable: tokens >= minimum,
        })
    }

    /// Once per client, verify the cacheable prefix in the background and
    /// log the result (debug on pass, warn on fail). No-ops without a
    /// tokio runtime.
    fn spawn_prefix_check(&self, messages: &[Message]) {
        if self.prefix_checked.swap(true, Ordering::SeqCst) {
            return;
        }
        let Ok(handle) = tokio::runtime::Handle::try_current() else {
            return;
        };
        let clone = AnthropicClient {
            client: self.client.clone(),
            api_key: self.api_key.clone(),
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            effort: self.effort,
            context_editing: self.context_editing,
            prefix_checked: AtomicBool::new(true),
        };
        let messages: Vec<Message> = messages.to_vec();
        handle.spawn(async move {
            match clone.verify_cacheable_prefix(&messages).await {
                Ok(check) if check.cacheable => log::debug!(
                    "prompt-cache prefix ok: system prefix is {} tokens (minimum {} on {})",
                    check.tokens,
                    check.minimum,
                    clone.model
                ),
                Ok(check) => log::warn!(
                    "prompt-cache prefix TOO SMALL: system prefix is {} tokens but {} needs \
                     {} — the system breakpoint will silently not cache (multi-turn caching \
                     still works once the conversation passes the minimum)",
                    check.tokens,
                    clone.model,
                    check.minimum
                ),
                Err(e) => log::debug!("prompt-cache prefix check skipped: {e}"),
            }
        });
    }
}

/// Result of a cacheable-prefix measurement.
#[derive(Debug, Clone, Copy)]
pub struct PrefixCheck {
    /// Measured tokens of the system prefix (plus a ~1-token probe).
    pub tokens: u64,
    /// The model's minimum cacheable prefix.
    pub minimum: u64,
    /// Whether the prefix meets the minimum.
    pub cacheable: bool,
}

impl Default for AnthropicClient {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Request/response wire types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub(crate) struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<Vec<TextBlock>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_config: Option<OutputConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context_management: Option<serde_json::Value>,
    /// Omitted when false so the same body shape is valid as Batch API
    /// `params` (which reject a `stream` field).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
}

#[derive(Serialize)]
struct CountTokensRequest {
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<Vec<TextBlock>>,
    messages: Vec<AnthropicMessage>,
}

#[derive(Deserialize)]
struct CountTokensResponse {
    input_tokens: u64,
}

#[derive(Serialize)]
struct OutputConfig {
    effort: Effort,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<TextBlock>,
}

#[derive(Serialize)]
struct TextBlock {
    #[serde(rename = "type")]
    kind: &'static str,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_control: Option<CacheControl>,
}

impl TextBlock {
    fn new(text: String) -> Self {
        Self {
            kind: "text",
            text,
            cache_control: None,
        }
    }

    fn mark(&mut self) {
        self.cache_control = Some(CacheControl { kind: "ephemeral" });
    }
}

#[derive(Serialize)]
struct CacheControl {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContentBlock>,
    #[serde(default)]
    usage: Option<WireUsage>,
}

#[derive(Deserialize)]
struct AnthropicContentBlock {
    text: Option<String>,
}

/// Usage as it appears on the wire (all fields optional).
#[derive(Deserialize, Default, Clone, Copy)]
struct WireUsage {
    #[serde(default)]
    input_tokens: Option<u64>,
    #[serde(default)]
    cache_creation_input_tokens: Option<u64>,
    #[serde(default)]
    cache_read_input_tokens: Option<u64>,
    #[serde(default)]
    output_tokens: Option<u64>,
}

impl WireUsage {
    fn to_usage(self) -> Usage {
        Usage {
            input_tokens: self.input_tokens.unwrap_or(0),
            cache_creation_input_tokens: self.cache_creation_input_tokens.unwrap_or(0),
            cache_read_input_tokens: self.cache_read_input_tokens.unwrap_or(0),
            output_tokens: self.output_tokens.unwrap_or(0),
        }
    }
}

/// Build the request deterministically: same inputs, same bytes.
pub(crate) fn build_request(
    model: &str,
    max_tokens: u32,
    effort: Option<Effort>,
    context_editing: bool,
    messages: &[Message],
    stream: bool,
) -> AnthropicRequest {
    let mut system_blocks: Vec<TextBlock> = Vec::new();
    let mut api_messages: Vec<AnthropicMessage> = Vec::new();

    for m in messages {
        match m.role {
            Role::System => system_blocks.push(TextBlock::new(m.content.clone())),
            Role::User | Role::Assistant => {
                let role = if m.role == Role::User {
                    "user"
                } else {
                    "assistant"
                };
                api_messages.push(AnthropicMessage {
                    role: role.into(),
                    content: vec![TextBlock::new(m.content.clone())],
                });
            }
        }
    }

    // Breakpoints (max 4; see the module docs).
    // 1+2: last system block always; first as well when there are several.
    if let Some(last) = system_blocks.last_mut() {
        last.mark();
    }
    if system_blocks.len() > 1 {
        system_blocks[0].mark();
    }
    // 3: rolling breakpoint on the last content block of the last message.
    if let Some(last_msg) = api_messages.last_mut()
        && let Some(last_block) = last_msg.content.last_mut()
    {
        last_block.mark();
    }
    // 4: intermediate breakpoint on long histories so the rolling
    // breakpoint's 20-block lookback always reaches cached territory.
    // (Each message holds one block here.)
    if api_messages.len() > CACHE_LOOKBACK_BLOCKS {
        let idx = api_messages.len() - 1 - INTERMEDIATE_BREAKPOINT_GAP;
        if let Some(block) = api_messages[idx].content.last_mut() {
            block.mark();
        }
    }

    AnthropicRequest {
        model: model.to_string(),
        max_tokens,
        messages: api_messages,
        system: if system_blocks.is_empty() {
            None
        } else {
            Some(system_blocks)
        },
        output_config: effort.map(|effort| OutputConfig { effort }),
        context_management: context_editing.then(|| {
            serde_json::json!({
                "edits": [{"type": "clear_tool_uses_20250919"}]
            })
        }),
        stream,
    }
}

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn complete(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        Ok(self.complete_with_usage(messages).await?.0)
    }

    async fn complete_with_usage(&self, messages: Vec<Message>) -> anyhow::Result<(String, Usage)> {
        self.spawn_prefix_check(&messages);
        let body = self.request_body(&messages, false);
        let resp = self
            .request_builder(ANTHROPIC_API_BASE, &body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await?;
            anyhow::bail!("Anthropic API error {}: {}", status, text);
        }

        let parsed: AnthropicResponse = resp.json().await?;
        let usage = parsed.usage.map(WireUsage::to_usage).unwrap_or_default();
        let text = parsed
            .content
            .into_iter()
            .filter_map(|b| b.text)
            .collect::<Vec<_>>()
            .join("");
        Ok((text, usage))
    }

    async fn complete_stream(&self, messages: Vec<Message>) -> anyhow::Result<ChatStream> {
        self.spawn_prefix_check(&messages);
        let body = self.request_body(&messages, true);
        let resp = self
            .request_builder(ANTHROPIC_API_BASE, &body)
            .send()
            .await?;

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
    /// Cumulative output_tokens reported so far — `message_delta` carries a
    /// running total, chunks carry the increment so summing chunk usages
    /// yields the correct call total.
    output_tokens_seen: u64,
}

impl<S> SseParser<S> {
    fn new(inner: S) -> Self {
        Self {
            inner,
            buffer: String::new(),
            output_tokens_seen: 0,
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
            if let Some(chunk) = parse_next_event(&mut this.buffer, &mut this.output_tokens_seen) {
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
                    if let Some(chunk) =
                        parse_next_event(&mut this.buffer, &mut this.output_tokens_seen)
                    {
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
/// event types we care about (`message_start` for input/cache usage,
/// `content_block_delta` for text, `message_delta` for output usage and the
/// stop reason, `message_stop`). Returns `None` when no complete event is
/// buffered.
fn parse_next_event(buffer: &mut String, output_tokens_seen: &mut u64) -> Option<ChatChunk> {
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
            "message_start" => {
                // Carries input + cache token counts for the whole call.
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data)
                    && let Some(u) = parsed.get("message").and_then(|m| m.get("usage"))
                    && let Ok(wire) = serde_json::from_value::<WireUsage>(u.clone())
                {
                    let mut usage = wire.to_usage();
                    // Any output_tokens here are a running total too.
                    let inc = usage.output_tokens.saturating_sub(*output_tokens_seen);
                    *output_tokens_seen = (*output_tokens_seen).max(usage.output_tokens);
                    usage.output_tokens = inc;
                    return Some(ChatChunk {
                        delta: String::new(),
                        finish_reason: None,
                        usage: Some(usage),
                    });
                }
            }
            "content_block_delta" => {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    let text = parsed
                        .get("delta")
                        .and_then(|d| d.get("text"))
                        .and_then(|t| t.as_str())
                        .unwrap_or_default();
                    if !text.is_empty() {
                        return Some(ChatChunk::text(text));
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
                    let usage = parsed
                        .get("usage")
                        .and_then(|u| serde_json::from_value::<WireUsage>(u.clone()).ok())
                        .map(|wire| {
                            let mut usage = wire.to_usage();
                            let inc = usage.output_tokens.saturating_sub(*output_tokens_seen);
                            *output_tokens_seen = (*output_tokens_seen).max(usage.output_tokens);
                            usage.output_tokens = inc;
                            usage
                        });
                    if stop_reason.is_some() || usage.is_some() {
                        return Some(ChatChunk {
                            delta: String::new(),
                            finish_reason: stop_reason,
                            usage,
                        });
                    }
                }
            }
            "message_stop" => {
                return Some(ChatChunk {
                    delta: String::new(),
                    finish_reason: Some("end_turn".into()),
                    usage: None,
                });
            }
            _ => continue,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_all(buf: &mut String) -> Vec<ChatChunk> {
        let mut seen = 0u64;
        let mut out = Vec::new();
        while let Some(c) = parse_next_event(buf, &mut seen) {
            out.push(c);
        }
        out
    }

    #[test]
    fn sse_parser_extracts_text_deltas() {
        let mut buf = String::from(
            "event: content_block_delta\ndata: {\"delta\":{\"text\":\"hi\"}}\n\n\
             event: message_stop\ndata: {}\n\n",
        );
        let chunks = parse_all(&mut buf);
        assert_eq!(chunks[0].delta, "hi");
        assert_eq!(chunks[1].finish_reason.as_deref(), Some("end_turn"));
        assert_eq!(chunks.len(), 2);
    }

    #[test]
    fn sse_parser_captures_four_way_usage() {
        let mut buf = String::from(
            "event: message_start\ndata: {\"message\":{\"usage\":{\"input_tokens\":7,\
             \"cache_creation_input_tokens\":100,\"cache_read_input_tokens\":900,\
             \"output_tokens\":1}}}\n\n\
             event: content_block_delta\ndata: {\"delta\":{\"text\":\"ok\"}}\n\n\
             event: message_delta\ndata: {\"delta\":{\"stop_reason\":\"end_turn\"},\
             \"usage\":{\"output_tokens\":42}}\n\n\
             event: message_stop\ndata: {}\n\n",
        );
        let chunks = parse_all(&mut buf);
        let mut total = Usage::default();
        for c in &chunks {
            if let Some(u) = &c.usage {
                total.add(u);
            }
        }
        assert_eq!(total.input_tokens, 7);
        assert_eq!(total.cache_creation_input_tokens, 100);
        assert_eq!(total.cache_read_input_tokens, 900);
        // 1 at message_start + increment 41 at message_delta = 42, not 43.
        assert_eq!(total.output_tokens, 42);
    }

    #[test]
    fn cumulative_output_tokens_become_increments() {
        let mut buf = String::from(
            "event: message_delta\ndata: {\"delta\":{},\"usage\":{\"output_tokens\":10}}\n\n\
             event: message_delta\ndata: {\"delta\":{\"stop_reason\":\"end_turn\"},\
             \"usage\":{\"output_tokens\":25}}\n\n",
        );
        let chunks = parse_all(&mut buf);
        let sum: u64 = chunks
            .iter()
            .filter_map(|c| c.usage.as_ref())
            .map(|u| u.output_tokens)
            .sum();
        assert_eq!(sum, 25);
    }

    #[test]
    fn build_request_splits_system_into_blocks_with_breakpoints() {
        let req = build_request(
            "m",
            10,
            None,
            false,
            &[
                Message::new(Role::System, "frozen prompt"),
                Message::new(Role::System, "session doc context"),
                Message::new(Role::User, "hi"),
            ],
            false,
        );
        let system = req.system.as_ref().unwrap();
        assert_eq!(system.len(), 2);
        assert!(system[0].cache_control.is_some(), "first system block");
        assert!(system[1].cache_control.is_some(), "last system block");
        assert_eq!(req.messages.len(), 1);
        assert!(
            req.messages[0].content[0].cache_control.is_some(),
            "rolling breakpoint on last message"
        );
    }

    #[test]
    fn single_system_block_gets_one_breakpoint() {
        let req = build_request(
            "m",
            10,
            None,
            false,
            &[
                Message::new(Role::System, "sys"),
                Message::new(Role::User, "hi"),
                Message::new(Role::Assistant, "yo"),
                Message::new(Role::User, "more"),
            ],
            false,
        );
        let system = req.system.as_ref().unwrap();
        assert_eq!(system.len(), 1);
        assert!(system[0].cache_control.is_some());
        // Only the LAST message block is marked.
        let marked: Vec<usize> = req
            .messages
            .iter()
            .enumerate()
            .filter(|(_, m)| m.content[0].cache_control.is_some())
            .map(|(i, _)| i)
            .collect();
        assert_eq!(marked, vec![2]);
    }

    #[test]
    fn long_history_gets_intermediate_breakpoint_within_budget() {
        let mut messages = vec![Message::new(Role::System, "sys")];
        for i in 0..30 {
            let role = if i % 2 == 0 {
                Role::User
            } else {
                Role::Assistant
            };
            messages.push(Message::new(role, format!("turn {i}")));
        }
        let req = build_request("m", 10, None, false, &messages, false);
        let marked: Vec<usize> = req
            .messages
            .iter()
            .enumerate()
            .filter(|(_, m)| m.content[0].cache_control.is_some())
            .map(|(i, _)| i)
            .collect();
        // Rolling on the last, intermediate 15 before it; ≤4 total
        // including the system breakpoint.
        assert_eq!(marked, vec![14, 29]);
        let system_marks = req
            .system
            .as_ref()
            .unwrap()
            .iter()
            .filter(|b| b.cache_control.is_some())
            .count();
        assert!(system_marks + marked.len() <= 4);
    }

    #[test]
    fn effort_and_context_editing_serialize_only_when_set() {
        let plain = build_request(
            "m",
            10,
            None,
            false,
            &[Message::new(Role::User, "x")],
            false,
        );
        let json = serde_json::to_value(&plain).unwrap();
        assert!(json.get("output_config").is_none());
        assert!(json.get("context_management").is_none());
        // We must never send sampling params or thinking config.
        for forbidden in ["temperature", "top_p", "top_k", "thinking", "budget_tokens"] {
            assert!(json.get(forbidden).is_none(), "{forbidden} must be absent");
        }

        let tuned = build_request(
            "m",
            10,
            Some(Effort::Low),
            true,
            &[Message::new(Role::User, "x")],
            false,
        );
        let json = serde_json::to_value(&tuned).unwrap();
        assert_eq!(json["output_config"]["effort"], "low");
        assert_eq!(
            json["context_management"]["edits"][0]["type"],
            "clear_tool_uses_20250919"
        );
    }

    #[test]
    fn effort_parses_all_levels() {
        for (s, e) in [
            ("low", Effort::Low),
            ("medium", Effort::Medium),
            ("high", Effort::High),
            ("xhigh", Effort::Xhigh),
            ("max", Effort::Max),
        ] {
            assert_eq!(Effort::parse(s), Some(e));
        }
        assert_eq!(Effort::parse("ultra"), None);
    }
}
