<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Keep model reasoning distinct from the answer

For: Authors using an agent to change documents while retaining review control.

Inspect a recorded turn without mistaking a reasoning block for the final response or an executed check.

## What must happen

The conversation distinguishes reasoning, final response and tool observations using their recorded roles. The current Python runtime owns prompt/history construction; the shared session and UI retain the separate recorded content. Reasoning text does not execute tools, certify completion or establish a check result.

## Limits of the evidence

Provider-specific availability of reasoning is not a promise to expose hidden model state.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/reasoning-is-shown-apart-from-the-answer.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain role separation while applying the Python-runtime amendment. Earlier Rust-loop test claims remain historical.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-982c781ee838"></a>
## `crates/hickory-agent/src/llm.rs`

Related contracts: [Keep model reasoning distinct from the answer](contract.md).

<a id="f-982c781ee838-Message"></a>
### `Message`

LLM client abstraction: chat messages, streaming chunks, and the
[`LlmClient`] trait every provider implements.

<storycode:copy id="f-982c781ee838-Message">
//! LLM client abstraction: chat messages, streaming chunks, and the
//! [`LlmClient`] trait every provider implements.

use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};

use crate::usage::Usage;

/// A chat message with text and explicitly selected host-validated images.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<Image>,
}

</storycode:copy>
<a id="f-982c781ee838-Image"></a>
### `Image`

Host-validated image bytes encoded for a provider request, never a remote URL.

<storycode:copy id="f-982c781ee838-Image">
/// Host-validated image bytes encoded for a provider request, never a remote URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub media_type: String,
    pub data: String,
}

</storycode:copy>
<a id="f-982c781ee838-impl-Message"></a>
### `impl-Message`

<storycode:copy id="f-982c781ee838-impl-Message">
impl Message {
    /// Convenience constructor.
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            images: vec![],
        }
    }
}

</storycode:copy>
<a id="f-982c781ee838-Role"></a>
### `Role`

Chat message role: System, User, or Assistant.

<storycode:copy id="f-982c781ee838-Role">
/// Chat message role: System, User, or Assistant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

</storycode:copy>
<a id="f-982c781ee838-ChatChunk"></a>
### `ChatChunk`

A chunk of streamed LLM output containing a text delta and optional
finish reason.

<storycode:copy id="f-982c781ee838-ChatChunk">
/// A chunk of streamed LLM output containing a text delta and optional
/// finish reason.
#[derive(Debug, Clone, Default)]
pub struct ChatChunk {
    /// The text fragment in this chunk.
    pub delta: String,
    /// A fragment of the model's REASONING, when the provider streams it
    /// (Anthropic `thinking_delta`; OpenAI-compatible `reasoning` /
    /// `reasoning_content` deltas). Shown folded, recorded in the session as
    /// `<hick:reasoning>`, never part of the answer.
    pub reasoning: String,
    /// Set when the stream is complete (e.g. "stop", "end_turn").
    pub finish_reason: Option<String>,
    /// Partial usage carried by this chunk (Anthropic reports input +
    /// cache counters on `message_start` and output tokens on
    /// `message_delta`). Sum the usages of every chunk in a stream to get
    /// the call's total — see [`crate::Usage::add`].
    pub usage: Option<Usage>,
}

</storycode:copy>
<a id="f-982c781ee838-impl-ChatChunk"></a>
### `impl-ChatChunk`

<storycode:copy id="f-982c781ee838-impl-ChatChunk">
impl ChatChunk {
    /// A plain text delta with no finish reason or usage.
    pub fn text(delta: impl Into<String>) -> Self {
        Self {
            delta: delta.into(),
            ..Self::default()
        }
    }

    /// A reasoning delta with no answer text.
    pub fn reasoning(delta: impl Into<String>) -> Self {
        Self {
            reasoning: delta.into(),
            ..Self::default()
        }
    }
}

</storycode:copy>
<a id="f-982c781ee838-ChatStream"></a>
### `ChatStream`

<storycode:copy id="f-982c781ee838-ChatStream">
/// A pinned, boxed stream of [`ChatChunk`]s.
pub type ChatStream = Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>;

</storycode:copy>
<a id="f-982c781ee838-LlmClient"></a>
### `LlmClient`

Trait for LLM chat completion clients (streaming and non-streaming).

<storycode:copy id="f-982c781ee838-LlmClient">
/// Trait for LLM chat completion clients (streaming and non-streaming).
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Complete a chat conversation (returns full response text).
    async fn complete(&self, messages: Vec<Message>) -> anyhow::Result<String>;

    /// Complete a chat conversation, returning the text plus the call's
    /// four-way token [`Usage`]. Default: delegates to
    /// [`LlmClient::complete`] with zero usage (providers that bill should
    /// override).
    async fn complete_with_usage(&self, messages: Vec<Message>) -> anyhow::Result<(String, Usage)> {
        Ok((self.complete(messages).await?, Usage::default()))
    }

    /// Stream a chat completion. Returns a stream of chunks.
    ///
    /// Default implementation collects the full response from
    /// [`LlmClient::complete`] and emits it as a single chunk. Providers that
    /// support streaming should override this for incremental output.
    async fn complete_stream(&self, messages: Vec<Message>) -> anyhow::Result<ChatStream> {
        let (response, usage) = self.complete_with_usage(messages).await?;
        let stream = futures::stream::once(async move {
            Ok(ChatChunk {
                delta: response,
                reasoning: String::new(),
                finish_reason: Some("stop".into()),
                usage: Some(usage),
            })
        });
        Ok(Box::pin(stream))
    }

    /// Check that this client's credentials are accepted by the provider.
    ///
    /// For bring-your-own-key: a key is validated when it is *entered*, so a
    /// typo is one message in a form rather than a 401 surfacing three layers
    /// down in a streamed agent run, after the user has already described a
    /// task and waited.
    ///
    /// The default asks for a one-token completion — the smallest request
    /// every provider understands. Providers with a free credential-checking
    /// endpoint should override (Anthropic uses `count_tokens`).
    ///
    /// A network failure is reported like a rejection, because from here the
    /// two are indistinguishable and both mean "not usable right now".
    async fn validate_credentials(&self) -> anyhow::Result<()> {
        self.complete(vec![Message::new(Role::User, "ping")])
            .await
            .map(|_| ())
    }

    /// Provider name for display (e.g. "anthropic").
    fn provider_name(&self) -> &str;

    /// Model name for display (e.g. "claude-sonnet-5").
    fn model_name(&self) -> &str;
}

</storycode:copy>
<a id="f-982c781ee838-impl-so-an-Arc-dyn-LlmClient-can-itself-be-used-where-an"></a>
### `impl-so an Arc<dyn LlmClient> can itself be used where an`

Blanket impl so an `Arc<dyn LlmClient>` can itself be used where an
`LlmClient` is expected.

<storycode:copy id="f-982c781ee838-impl-so-an-Arc-dyn-LlmClient-can-itself-be-used-where-an">
/// Blanket impl so an `Arc<dyn LlmClient>` can itself be used where an
/// `LlmClient` is expected.
#[async_trait]
impl LlmClient for Arc<dyn LlmClient> {
    async fn complete(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        (**self).complete(messages).await
    }

    async fn complete_with_usage(&self, messages: Vec<Message>) -> anyhow::Result<(String, Usage)> {
        (**self).complete_with_usage(messages).await
    }

    async fn complete_stream(&self, messages: Vec<Message>) -> anyhow::Result<ChatStream> {
        (**self).complete_stream(messages).await
    }

    async fn validate_credentials(&self) -> anyhow::Result<()> {
        (**self).validate_credentials().await
    }

    fn provider_name(&self) -> &str {
        (**self).provider_name()
    }

    fn model_name(&self) -> &str {
        (**self).model_name()
    }
}
</storycode:copy>

### Assemble `crates/hickory-agent/src/llm.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/src/llm.rs"><storycode:paste select="#f-982c781ee838-Message"/><storycode:paste select="#f-982c781ee838-Image"/><storycode:paste select="#f-982c781ee838-impl-Message"/><storycode:paste select="#f-982c781ee838-Role"/><storycode:paste select="#f-982c781ee838-ChatChunk"/><storycode:paste select="#f-982c781ee838-impl-ChatChunk"/><storycode:paste select="#f-982c781ee838-ChatStream"/><storycode:paste select="#f-982c781ee838-LlmClient"/><storycode:paste select="#f-982c781ee838-impl-so-an-Arc-dyn-LlmClient-can-itself-be-used-where-an"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/ChatDock.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-57557cac78c8)
- [apps/web/src/components/SessionTurns.tsx](../../../mechanisms/apps/web/src/components/SessionTurns.tsx.md#output-b05fd3f7cf29)
- [crates/hickory-agent/src/llm.rs](contract.md#output-982c781ee838)
- [crates/hickory-agent/src/llm_anthropic.rs](../../../mechanisms/crates/hickory-agent/src/llm_anthropic.rs.md#output-ad06d96de78a)
- [crates/hickory-agent/src/llm_openai.rs](../../../mechanisms/crates/hickory-agent/src/llm_openai.rs.md#output-0150d11c8f60)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
