<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-agent/src/llm_openai.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0150d11c8f60"></a>
## `crates/hickory-agent/src/llm_openai.rs`

Related contracts: [An Empty Response Never Kills a Session](../../../../guarantees/agent/an-empty-response-never-kills-a-session/contract.md), [Keep model reasoning distinct from the answer](../../../../guarantees/agent/reasoning-is-shown-apart-from-the-answer/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../guarantees/verification/predictions-precede-managed-observation/contract.md), [Private edit proposals open review before writing](../../../../guarantees/verification/private-proposals-wait-for-review/contract.md).

<a id="f-0150d11c8f60-Provider"></a>
### `Provider`

OpenAI-compatible chat-completions client: OpenAI, DeepSeek, xAI
(Grok), and OpenRouter.
All four speak the same wire protocol — `POST {base}/chat/completions`
with a bearer token, `choices[0].message.content` for the answer and
`data: {...}` SSE frames terminated by `data: [DONE]` when streaming — so
they share one client and differ only in [`Provider`]: base URL, key
environment variable, default model, and the two places the dialects
genuinely diverge (see below). A separate struct per vendor would have
been four copies of the same SSE parser.
## Where the dialects actually differ
- **Output-length field.** OpenAI's newer models reject `max_tokens` and
require `max_completion_tokens`; DeepSeek and xAI take `max_tokens`.
[`Provider::max_tokens_field`] picks per provider — the one thing that
turns a working request into a 400 if you assume wrongly.
- **Reasoning text.** DeepSeek's reasoner returns chain-of-thought in
`delta.reasoning_content`, a sibling of `delta.content`. We deliberately
drop it: the agent's `<hick:next>` protocol is parsed from the answer,
and folding reasoning into the answer would corrupt that parse.
## Caching is implicit here
None of these providers take Anthropic's explicit `cache_control`
breakpoints; they cache a common prefix automatically. The frozen
system-prefix discipline in the Python agent still pays off — an
automatic cache keys on the prefix being byte-identical, which is exactly
what that layout guarantees — there is simply nothing to mark.
## Cost is reported as unknown
[`crate::cost_usd`] returns `None` for these models: the price table
carries only prices we have verified, and its cache multipliers are
Anthropic's. Reporting an unknown price as unknown is the rule (see
`usage.rs`); a guessed number in a cost report is worse than no number.
Token counts are still captured in full.

<storycode:copy id="f-0150d11c8f60-Provider">
//! OpenAI-compatible chat-completions client: OpenAI, DeepSeek, xAI
//! (Grok), and OpenRouter.
//!
//! All four speak the same wire protocol — `POST {base}/chat/completions`
//! with a bearer token, `choices[0].message.content` for the answer and
//! `data: {...}` SSE frames terminated by `data: [DONE]` when streaming — so
//! they share one client and differ only in [`Provider`]: base URL, key
//! environment variable, default model, and the two places the dialects
//! genuinely diverge (see below). A separate struct per vendor would have
//! been four copies of the same SSE parser.
//!
//! ## Where the dialects actually differ
//!
//! - **Output-length field.** OpenAI's newer models reject `max_tokens` and
//!   require `max_completion_tokens`; DeepSeek and xAI take `max_tokens`.
//!   [`Provider::max_tokens_field`] picks per provider — the one thing that
//!   turns a working request into a 400 if you assume wrongly.
//! - **Reasoning text.** DeepSeek's reasoner returns chain-of-thought in
//!   `delta.reasoning_content`, a sibling of `delta.content`. We deliberately
//!   drop it: the agent's `<hick:next>` protocol is parsed from the answer,
//!   and folding reasoning into the answer would corrupt that parse.
//!
//! ## Caching is implicit here
//!
//! None of these providers take Anthropic's explicit `cache_control`
//! breakpoints; they cache a common prefix automatically. The frozen
//! system-prefix discipline in the Python agent still pays off — an
//! automatic cache keys on the prefix being byte-identical, which is exactly
//! what that layout guarantees — there is simply nothing to mark.
//!
//! ## Cost is reported as unknown
//!
//! [`crate::cost_usd`] returns `None` for these models: the price table
//! carries only prices we have verified, and its cache multipliers are
//! Anthropic's. Reporting an unknown price as unknown is the rule (see
//! `usage.rs`); a guessed number in a cost report is worse than no number.
//! Token counts are still captured in full.

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};

use crate::llm::{ChatChunk, ChatStream, LlmClient, Message, Role};
use crate::usage::Usage;

/// An OpenAI-compatible vendor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    OpenAi,
    DeepSeek,
    /// xAI, whose models are the Grok family.
    XAi,
    /// OpenRouter, a gateway that routes to many vendors behind one key.
    OpenRouter,
    /// Gab AI: an OpenAI-compatible gateway (`https://gab.ai/v1`) with its
    /// own Arya model and many vendors' models behind one key.
    Gab,
}

</storycode:copy>
<a id="f-0150d11c8f60-impl-Provider"></a>
### `impl-Provider`

<storycode:copy id="f-0150d11c8f60-impl-Provider">
impl Provider {
    /// Parse a provider selector (`openai`, `deepseek`, `grok`/`xai`,
    /// `openrouter`).
    ///
    /// `grok` is accepted alongside `xai` because the model is the name
    /// people reach for; the vendor is xAI.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "openai" => Some(Self::OpenAi),
            "deepseek" => Some(Self::DeepSeek),
            "xai" | "grok" => Some(Self::XAi),
            "openrouter" => Some(Self::OpenRouter),
            "gab" | "gab-ai" | "gabai" => Some(Self::Gab),
            _ => None,
        }
    }

    /// Display name (also the `provider_name()` the session log records).
    pub fn name(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::DeepSeek => "deepseek",
            Self::XAi => "xai",
            Self::OpenRouter => "openrouter",
            Self::Gab => "gab",
        }
    }

    /// Default chat-completions endpoint.
    pub fn default_base_url(self) -> &'static str {
        match self {
            Self::OpenAi => "https://api.openai.com/v1/chat/completions",
            Self::DeepSeek => "https://api.deepseek.com/chat/completions",
            Self::XAi => "https://api.x.ai/v1/chat/completions",
            Self::OpenRouter => "https://openrouter.ai/api/v1/chat/completions",
            Self::Gab => "https://gab.ai/v1/chat/completions",
        }
    }

    /// Environment variable holding the API key.
    pub fn key_env(self) -> &'static str {
        match self {
            Self::OpenAi => "OPENAI_API_KEY",
            Self::DeepSeek => "DEEPSEEK_API_KEY",
            Self::XAi => "XAI_API_KEY",
            Self::OpenRouter => "OPENROUTER_API_KEY",
            // Gab's own docs name both spellings; `GAB_API_KEY` is primary
            // and `GAB_AI_API_KEY` is honoured by the key lookup as an alias.
            Self::Gab => "GAB_API_KEY",
        }
    }

    /// Environment variable overriding the base URL (gateways, proxies, and
    /// the recording harness the tests drive).
    pub fn base_url_env(self) -> &'static str {
        match self {
            Self::OpenAi => "OPENAI_BASE_URL",
            Self::DeepSeek => "DEEPSEEK_BASE_URL",
            Self::XAi => "XAI_BASE_URL",
            Self::OpenRouter => "OPENROUTER_BASE_URL",
            Self::Gab => "GAB_BASE_URL",
        }
    }

    /// Environment variable overriding the model id.
    pub fn model_env(self) -> &'static str {
        match self {
            Self::OpenAi => "OPENAI_MODEL",
            Self::DeepSeek => "DEEPSEEK_MODEL",
            Self::XAi => "XAI_MODEL",
            Self::OpenRouter => "OPENROUTER_MODEL",
            Self::Gab => "GAB_MODEL",
        }
    }

    /// Default model id when neither the builder nor the environment names
    /// one.
    pub fn default_model(self) -> &'static str {
        match self {
            Self::OpenAi => "gpt-5",
            Self::DeepSeek => "deepseek-chat",
            Self::XAi => "grok-4",
            // The router's own auto-selector: works on every OpenRouter
            // account without naming a downstream vendor's model.
            Self::OpenRouter => "openrouter/auto",
            // Gab's own model; `GET https://gab.ai/v1/models` (public) lists
            // the rest, vendor models included.
            Self::Gab => "arya",
        }
    }

    /// The request field carrying the output-length cap.
    ///
    /// OpenAI's newer models reject `max_tokens`; the others do not know
    /// `max_completion_tokens` (OpenRouter normalizes `max_tokens` for
    /// whatever it routes to).
    fn max_tokens_field(self) -> &'static str {
        match self {
            Self::OpenAi => "max_completion_tokens",
            Self::DeepSeek | Self::XAi | Self::OpenRouter | Self::Gab => "max_tokens",
        }
    }
}

</storycode:copy>
<a id="f-0150d11c8f60-OpenAiCompatClient"></a>
### `OpenAiCompatClient`

A chat client for any OpenAI-compatible provider.

<storycode:copy id="f-0150d11c8f60-OpenAiCompatClient">
/// A chat client for any OpenAI-compatible provider.
pub struct OpenAiCompatClient {
    client: reqwest::Client,
    provider: Provider,
    api_key: String,
    model: String,
    max_tokens: u32,
    json_object: bool,
    base_url: String,
}

</storycode:copy>
<a id="f-0150d11c8f60-impl-OpenAiCompatClient"></a>
### `impl-OpenAiCompatClient`

<storycode:copy id="f-0150d11c8f60-impl-OpenAiCompatClient">
impl OpenAiCompatClient {
    /// Create a client for `provider`, resolving the key, base URL, and
    /// model from the provider's environment variables with the provider
    /// defaults as fallbacks.
    pub fn new(provider: Provider) -> Self {
        Self {
            client: reqwest::Client::new(),
            provider,
            api_key: std::env::var(provider.key_env()).unwrap_or_default(),
            model: std::env::var(provider.model_env())
                .unwrap_or_else(|_| provider.default_model().to_string()),
            max_tokens: 16384,
            json_object: false,
            base_url: std::env::var(provider.base_url_env())
                .unwrap_or_else(|_| provider.default_base_url().to_string()),
        }
    }

    /// Override the model id.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Override the API key (BYO-key plans construct one client per
    /// request).
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = api_key.into();
        self
    }

    /// Override the max output tokens (default 16384).
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Request the provider's JSON-object transport for an action protocol.
    /// Callers must still validate the returned object and authorize every tool.
    pub fn with_json_object(mut self) -> Self {
        self.json_object = true;
        self
    }

    /// Point this client at a different chat-completions endpoint.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Which vendor this client talks to.
    pub fn provider(&self) -> Provider {
        self.provider
    }

    fn request_body(&self, messages: &[Message], stream: bool) -> serde_json::Value {
        let msgs: Vec<WireMessage> = messages
            .iter()
            // Empty content is rejected upstream and carries nothing; see the
            // matching backstop in `llm_anthropic`.
            .filter(|m| !m.content.trim().is_empty() || !m.images.is_empty())
            .map(|m| WireMessage {
                role: match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                },
                content: if m.images.is_empty() { serde_json::json!(m.content) } else {
                    let mut parts = vec![];
                    if !m.content.trim().is_empty() { parts.push(serde_json::json!({"type":"text","text":m.content})); }
                    for image in &m.images { parts.push(serde_json::json!({"type":"image_url","image_url":{"url":format!("data:{};base64,{}", image.media_type, image.data)}})); }
                    serde_json::json!(parts)
                },
            })
            .collect();

        let mut body = serde_json::json!({
            "model": self.model,
            "messages": msgs,
            "stream": stream,
        });
        body[self.provider.max_tokens_field()] = serde_json::json!(self.max_tokens);
        if self.json_object {
            body["response_format"] = serde_json::json!({"type":"json_object"});
        }
        if stream {
            // Without this the stream reports no usage at all and every run
            // would record zero tokens.
            body["stream_options"] = serde_json::json!({ "include_usage": true });
        }
        body
    }

    /// The exact serialized bytes this client would send for `messages`.
    /// The prefix-stability tests compare these across sessions.
    pub fn request_body_bytes(&self, messages: &[Message], stream: bool) -> Vec<u8> {
        serde_json::to_vec(&self.request_body(messages, stream))
            .expect("request serialization cannot fail")
    }

    async fn send(&self, messages: &[Message], stream: bool) -> anyhow::Result<reqwest::Response> {
        let resp = self
            .client
            .post(&self.base_url)
            .bearer_auth(&self.api_key)
            .header("content-type", "application/json")
            .json(&self.request_body(messages, stream))
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("{} API error {}: {}", self.provider.name(), status, text);
        }
        Ok(resp)
    }
}

</storycode:copy>
<a id="f-0150d11c8f60-impl-LlmClient-for-OpenAiCompatClient"></a>
### `impl-LlmClient for OpenAiCompatClient`

<storycode:copy id="f-0150d11c8f60-impl-LlmClient-for-OpenAiCompatClient">
#[async_trait]
impl LlmClient for OpenAiCompatClient {
    async fn complete(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        Ok(self.complete_with_usage(messages).await?.0)
    }

    async fn complete_with_usage(&self, messages: Vec<Message>) -> anyhow::Result<(String, Usage)> {
        let resp = self.send(&messages, false).await?;
        let parsed: ChatResponse = resp.json().await?;
        let text = parsed
            .choices
            .into_iter()
            .filter_map(|c| c.message.and_then(|m| m.content))
            .collect::<Vec<_>>()
            .join("");
        Ok((
            text,
            parsed.usage.map(WireUsage::into_usage).unwrap_or_default(),
        ))
    }

    async fn complete_stream(&self, messages: Vec<Message>) -> anyhow::Result<ChatStream> {
        let resp = self.send(&messages, true).await?;
        Ok(Box::pin(SseParser::new(resp.bytes_stream())))
    }

    fn provider_name(&self) -> &str {
        self.provider.name()
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

</storycode:copy>
<a id="f-0150d11c8f60-WireMessage"></a>
### `WireMessage`

<storycode:copy id="f-0150d11c8f60-WireMessage">
// ---------------------------------------------------------------------------
// Wire types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct WireMessage {
    role: &'static str,
    content: serde_json::Value,
}

</storycode:copy>
<a id="f-0150d11c8f60-ChatResponse"></a>
### `ChatResponse`

<storycode:copy id="f-0150d11c8f60-ChatResponse">
#[derive(Deserialize)]
struct ChatResponse {
    #[serde(default)]
    choices: Vec<Choice>,
    #[serde(default)]
    usage: Option<WireUsage>,
}

</storycode:copy>
<a id="f-0150d11c8f60-Choice"></a>
### `Choice`

<storycode:copy id="f-0150d11c8f60-Choice">
#[derive(Deserialize)]
struct Choice {
    #[serde(default)]
    message: Option<ChoiceMessage>,
    #[serde(default)]
    delta: Option<ChoiceMessage>,
    #[serde(default)]
    finish_reason: Option<String>,
}

</storycode:copy>
<a id="f-0150d11c8f60-ChoiceMessage"></a>
### `ChoiceMessage`

<storycode:copy id="f-0150d11c8f60-ChoiceMessage">
#[derive(Deserialize)]
struct ChoiceMessage {
    #[serde(default)]
    content: Option<String>,
    /// The model's reasoning, when the vendor streams it: OpenRouter and
    /// most gateways say `reasoning`, DeepSeek says `reasoning_content`.
    #[serde(default)]
    reasoning: Option<String>,
    #[serde(default)]
    reasoning_content: Option<String>,
}

</storycode:copy>
<a id="f-0150d11c8f60-WireUsage"></a>
### `WireUsage`

<storycode:copy id="f-0150d11c8f60-WireUsage">
#[derive(Deserialize)]
struct WireUsage {
    #[serde(default)]
    prompt_tokens: u64,
    #[serde(default)]
    completion_tokens: u64,
    #[serde(default)]
    prompt_tokens_details: Option<PromptDetails>,
}

</storycode:copy>
<a id="f-0150d11c8f60-PromptDetails"></a>
### `PromptDetails`

<storycode:copy id="f-0150d11c8f60-PromptDetails">
#[derive(Deserialize)]
struct PromptDetails {
    #[serde(default)]
    cached_tokens: u64,
}

</storycode:copy>
<a id="f-0150d11c8f60-impl-WireUsage"></a>
### `impl-WireUsage`

<storycode:copy id="f-0150d11c8f60-impl-WireUsage">
impl WireUsage {
    /// Map onto the four-way split.
    ///
    /// `prompt_tokens` counts cached and uncached input together, so the
    /// cached part is subtracted out — otherwise cached tokens would be
    /// billed twice in every report. These providers do not distinguish a
    /// cache *write*, so `cache_creation_input_tokens` stays zero.
    fn into_usage(self) -> Usage {
        let cached = self
            .prompt_tokens_details
            .map(|d| d.cached_tokens)
            .unwrap_or(0);
        Usage {
            input_tokens: self.prompt_tokens.saturating_sub(cached),
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: cached,
            output_tokens: self.completion_tokens,
        }
    }
}

</storycode:copy>
<a id="f-0150d11c8f60-SseParser"></a>
### `SseParser`

<storycode:copy id="f-0150d11c8f60-SseParser">
// ---------------------------------------------------------------------------
// SSE parsing
// ---------------------------------------------------------------------------

struct SseParser<S> {
    inner: S,
    buffer: String,
    done: bool,
}

</storycode:copy>
<a id="f-0150d11c8f60-impl-SseParser-S"></a>
### `impl-SseParser<S>`

<storycode:copy id="f-0150d11c8f60-impl-SseParser-S">
impl<S> SseParser<S> {
    fn new(inner: S) -> Self {
        Self {
            inner,
            buffer: String::new(),
            done: false,
        }
    }
}

</storycode:copy>
<a id="f-0150d11c8f60-impl-Stream-for-SseParser-S"></a>
### `impl-Stream for SseParser<S>`

<storycode:copy id="f-0150d11c8f60-impl-Stream-for-SseParser-S">
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
            if this.done {
                return std::task::Poll::Ready(None);
            }
            if let Some(chunk) = parse_next_event(&mut this.buffer, &mut this.done) {
                return std::task::Poll::Ready(Some(Ok(chunk)));
            }

            match Pin::new(&mut this.inner).poll_next(cx) {
                std::task::Poll::Ready(Some(Ok(bytes))) => {
                    this.buffer.push_str(&String::from_utf8_lossy(&bytes));
                }
                std::task::Poll::Ready(Some(Err(e))) => {
                    return std::task::Poll::Ready(Some(Err(anyhow::anyhow!("{}", e))));
                }
                std::task::Poll::Ready(None) => {
                    if let Some(chunk) = parse_next_event(&mut this.buffer, &mut this.done) {
                        return std::task::Poll::Ready(Some(Ok(chunk)));
                    }
                    return std::task::Poll::Ready(None);
                }
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }
        }
    }
}

</storycode:copy>
<a id="f-0150d11c8f60-parse_next_event"></a>
### `parse_next_event`

Consume complete SSE events from `buffer`, returning a chunk for the
first one that carries text, a finish reason, or usage. Returns `None`
when no complete event remains buffered.

<storycode:copy id="f-0150d11c8f60-parse_next_event">
/// Consume complete SSE events from `buffer`, returning a chunk for the
/// first one that carries text, a finish reason, or usage. Returns `None`
/// when no complete event remains buffered.
fn parse_next_event(buffer: &mut String, done: &mut bool) -> Option<ChatChunk> {
    loop {
        let boundary = buffer.find("\n\n")?;
        let raw = buffer[..boundary].to_string();
        buffer.drain(..boundary + 2);

        let Some(payload) = raw
            .lines()
            .find_map(|l| l.strip_prefix("data:").map(|d| d.trim()))
        else {
            continue;
        };

        if payload == "[DONE]" {
            *done = true;
            return None;
        }

        let Ok(parsed) = serde_json::from_str::<ChatResponse>(payload) else {
            // Keep-alives and comment frames are not errors.
            continue;
        };

        let text: String = parsed
            .choices
            .iter()
            .filter_map(|c| c.delta.as_ref().and_then(|d| d.content.clone()))
            .collect();
        let reasoning: String = parsed
            .choices
            .iter()
            .filter_map(|c| {
                c.delta
                    .as_ref()
                    .and_then(|d| d.reasoning.clone().or_else(|| d.reasoning_content.clone()))
            })
            .collect();
        let finish_reason = parsed.choices.iter().find_map(|c| c.finish_reason.clone());
        let usage = parsed.usage.map(WireUsage::into_usage);

        // The usage-only frame that follows the last delta has no text and
        // no finish reason; it must still be emitted or its tokens are lost.
        if text.is_empty() && reasoning.is_empty() && finish_reason.is_none() && usage.is_none() {
            continue;
        }
        return Some(ChatChunk {
            delta: text,
            reasoning,
            finish_reason,
            usage,
        });
    }
}

</storycode:copy>
<a id="f-0150d11c8f60-tests"></a>
### `tests`

<storycode:copy id="f-0150d11c8f60-tests">
#[cfg(test)]
mod tests {
    use super::*;

    // Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
    #[test]
    fn json_action_transport_keeps_the_cap_and_ordinary_requests_unchanged() {
        let ordinary = OpenAiCompatClient::new(Provider::DeepSeek).with_max_tokens(2048);
        assert!(
            ordinary
                .request_body(&[], true)
                .get("response_format")
                .is_none()
        );
        let action = ordinary.with_json_object();
        for stream in [false, true] {
            let body = action.request_body(&[], stream);
            assert_eq!(
                body["response_format"],
                serde_json::json!({"type":"json_object"})
            );
            assert_eq!(body["max_tokens"], 2048);
            assert_eq!(body["stream"], stream);
        }
    }

    #[test]
    fn providers_parse_from_selectors() {
        assert_eq!(Provider::parse("openai"), Some(Provider::OpenAi));
        assert_eq!(Provider::parse("DeepSeek"), Some(Provider::DeepSeek));
        // Both the vendor and the model family name resolve to xAI.
        assert_eq!(Provider::parse("grok"), Some(Provider::XAi));
        assert_eq!(Provider::parse("xai"), Some(Provider::XAi));
        assert_eq!(Provider::parse("OpenRouter"), Some(Provider::OpenRouter));
        assert_eq!(Provider::parse("gemini"), None);
    }

    #[test]
    fn openai_caps_output_with_max_completion_tokens() {
        // Sending `max_tokens` to a current OpenAI model is a 400, and
        // sending `max_completion_tokens` to DeepSeek/xAI is an unknown
        // field — the split has to be per provider.
        let openai = OpenAiCompatClient::new(Provider::OpenAi).with_max_tokens(99);
        let body = openai.request_body(&[], false);
        assert_eq!(body["max_completion_tokens"], 99);
        assert!(body.get("max_tokens").is_none());

        let deepseek = OpenAiCompatClient::new(Provider::DeepSeek).with_max_tokens(99);
        let body = deepseek.request_body(&[], false);
        assert_eq!(body["max_tokens"], 99);
        assert!(body.get("max_completion_tokens").is_none());
    }

    #[test]
    fn system_messages_stay_separate_and_in_order() {
        // The frozen-prefix cache layout depends on the system messages
        // arriving as distinct blocks in order; merging them would break
        // automatic prefix caching on every provider here.
        let client = OpenAiCompatClient::new(Provider::XAi);
        let body = client.request_body(
            &[
                Message::new(Role::System, "frozen"),
                Message::new(Role::System, "per-session"),
                Message::new(Role::User, "hi"),
            ],
            false,
        );
        let msgs = body["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0]["role"], "system");
        assert_eq!(msgs[0]["content"], "frozen");
        assert_eq!(msgs[1]["content"], "per-session");
        assert_eq!(msgs[2]["role"], "user");
    }

    #[test]
    fn streaming_requests_ask_for_usage() {
        let client = OpenAiCompatClient::new(Provider::OpenAi);
        let body = client.request_body(&[], true);
        assert_eq!(body["stream_options"]["include_usage"], true);
        // Non-streaming reports usage unconditionally; the option is
        // rejected there by some gateways.
        assert!(
            client
                .request_body(&[], false)
                .get("stream_options")
                .is_none()
        );
    }

    #[test]
    fn cached_prompt_tokens_are_not_counted_twice() {
        let usage = WireUsage {
            prompt_tokens: 1000,
            completion_tokens: 50,
            prompt_tokens_details: Some(PromptDetails { cached_tokens: 800 }),
        }
        .into_usage();
        assert_eq!(usage.input_tokens, 200, "cached tokens double-counted");
        assert_eq!(usage.cache_read_input_tokens, 800);
        assert_eq!(usage.output_tokens, 50);
    }

    #[test]
    fn sse_stream_yields_text_then_usage_then_stops() {
        let mut buf = String::from(
            "data: {\"choices\":[{\"delta\":{\"content\":\"he\"}}]}\n\n\
             data: {\"choices\":[{\"delta\":{\"content\":\"llo\"},\"finish_reason\":\"stop\"}]}\n\n\
             data: {\"choices\":[],\"usage\":{\"prompt_tokens\":10,\"completion_tokens\":2}}\n\n\
             data: [DONE]\n\n",
        );
        let mut done = false;

        let a = parse_next_event(&mut buf, &mut done).unwrap();
        assert_eq!(a.delta, "he");
        let b = parse_next_event(&mut buf, &mut done).unwrap();
        assert_eq!(b.delta, "llo");
        assert_eq!(b.finish_reason.as_deref(), Some("stop"));
        let c = parse_next_event(&mut buf, &mut done).unwrap();
        assert_eq!(c.delta, "");
        assert_eq!(c.usage.unwrap().output_tokens, 2);
        assert!(parse_next_event(&mut buf, &mut done).is_none());
        assert!(done, "[DONE] must terminate the stream");
    }

    #[test]
    fn reasoning_content_never_leaks_into_the_answer() {
        // DeepSeek's reasoner emits `reasoning_content` beside `content`.
        // The `<hick:next>` protocol is parsed from the answer, so folding
        // reasoning in would corrupt every turn.
        let mut buf = String::from(
            "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"thinking...\"}}]}\n\n\
             data: {\"choices\":[{\"delta\":{\"content\":\"<hick:next>done</hick:next>\"}}]}\n\n",
        );
        let mut done = false;
        // Reasoning arrives as its own chunk, with no answer text in it…
        let first = parse_next_event(&mut buf, &mut done).unwrap();
        assert_eq!(first.delta, "");
        assert_eq!(first.reasoning, "thinking...");
        // …and the answer is only ever the `content`.
        let second = parse_next_event(&mut buf, &mut done).unwrap();
        assert_eq!(second.delta, "<hick:next>done</hick:next>");
        assert_eq!(second.reasoning, "");
    }
    #[test]
    fn selected_images_are_native_openai_compatible_parts() {
        let message = Message {
            role: Role::User,
            content: String::new(),
            images: vec![crate::Image {
                media_type: "image/png".into(),
                data: "CAPTURED_BYTES".into(),
            }],
        };
        let client = OpenAiCompatClient::new(Provider::OpenAi);
        let value: serde_json::Value =
            serde_json::from_slice(&client.request_body_bytes(&[message], false)).unwrap();
        assert_eq!(
            value["messages"][0]["content"][0],
            serde_json::json!({"type":"image_url","image_url":{"url":"data:image/png;base64,CAPTURED_BYTES"}})
        );
    }
}
</storycode:copy>

### Assemble `crates/hickory-agent/src/llm_openai.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/src/llm_openai.rs"><storycode:paste select="#f-0150d11c8f60-Provider"/><storycode:paste select="#f-0150d11c8f60-impl-Provider"/><storycode:paste select="#f-0150d11c8f60-OpenAiCompatClient"/><storycode:paste select="#f-0150d11c8f60-impl-OpenAiCompatClient"/><storycode:paste select="#f-0150d11c8f60-impl-LlmClient-for-OpenAiCompatClient"/><storycode:paste select="#f-0150d11c8f60-WireMessage"/><storycode:paste select="#f-0150d11c8f60-ChatResponse"/><storycode:paste select="#f-0150d11c8f60-Choice"/><storycode:paste select="#f-0150d11c8f60-ChoiceMessage"/><storycode:paste select="#f-0150d11c8f60-WireUsage"/><storycode:paste select="#f-0150d11c8f60-PromptDetails"/><storycode:paste select="#f-0150d11c8f60-impl-WireUsage"/><storycode:paste select="#f-0150d11c8f60-SseParser"/><storycode:paste select="#f-0150d11c8f60-impl-SseParser-S"/><storycode:paste select="#f-0150d11c8f60-impl-Stream-for-SseParser-S"/><storycode:paste select="#f-0150d11c8f60-parse_next_event"/><storycode:paste select="#f-0150d11c8f60-tests"/></storycode:file>

</storycode:doc>
