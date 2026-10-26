<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-agent/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a448ddea64c9"></a>
## `crates/hickory-agent/src/lib.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../contract.md).

<a id="f-a448ddea64c9-audit"></a>
### `audit`

Shared provider transports, document tools, evidence and session rendering.
Agent orchestration lives exclusively in apps/agent.

<storycode:copy id="f-a448ddea64c9-audit">
//! Shared provider transports, document tools, evidence and session rendering.
//! Agent orchestration lives exclusively in apps/agent.

pub mod audit;
</storycode:copy>
<a id="f-a448ddea64c9-context"></a>
### `context`

<storycode:copy id="f-a448ddea64c9-context">
pub mod context;
</storycode:copy>
<a id="f-a448ddea64c9-events"></a>
### `events`

<storycode:copy id="f-a448ddea64c9-events">
mod events;
</storycode:copy>
<a id="f-a448ddea64c9-harness"></a>
### `harness`

<storycode:copy id="f-a448ddea64c9-harness">
pub mod harness;
</storycode:copy>
<a id="f-a448ddea64c9-invocation"></a>
### `invocation`

<storycode:copy id="f-a448ddea64c9-invocation">
mod invocation;
</storycode:copy>
<a id="f-a448ddea64c9-key_store"></a>
### `key_store`

<storycode:copy id="f-a448ddea64c9-key_store">
mod key_store;
</storycode:copy>
<a id="f-a448ddea64c9-llm"></a>
### `llm`

<storycode:copy id="f-a448ddea64c9-llm">
mod llm;
</storycode:copy>
<a id="f-a448ddea64c9-llm_anthropic"></a>
### `llm_anthropic`

<storycode:copy id="f-a448ddea64c9-llm_anthropic">
mod llm_anthropic;
</storycode:copy>
<a id="f-a448ddea64c9-llm_batch"></a>
### `llm_batch`

<storycode:copy id="f-a448ddea64c9-llm_batch">
mod llm_batch;
</storycode:copy>
<a id="f-a448ddea64c9-llm_openai"></a>
### `llm_openai`

<storycode:copy id="f-a448ddea64c9-llm_openai">
mod llm_openai;
</storycode:copy>
<a id="f-a448ddea64c9-provider"></a>
### `provider`

<storycode:copy id="f-a448ddea64c9-provider">
mod provider;
</storycode:copy>
<a id="f-a448ddea64c9-receipt"></a>
### `receipt`

<storycode:copy id="f-a448ddea64c9-receipt">
mod receipt;
</storycode:copy>
<a id="f-a448ddea64c9-review"></a>
### `review`

<storycode:copy id="f-a448ddea64c9-review">
mod review;
</storycode:copy>
<a id="f-a448ddea64c9-scripted"></a>
### `scripted`

<storycode:copy id="f-a448ddea64c9-scripted">
mod scripted;
</storycode:copy>
<a id="f-a448ddea64c9-session"></a>
### `session`

<storycode:copy id="f-a448ddea64c9-session">
mod session;
</storycode:copy>
<a id="f-a448ddea64c9-session_view"></a>
### `session_view`

<storycode:copy id="f-a448ddea64c9-session_view">
pub mod session_view;
</storycode:copy>
<a id="f-a448ddea64c9-tools"></a>
### `tools`

<storycode:copy id="f-a448ddea64c9-tools">
mod tools;
</storycode:copy>
<a id="f-a448ddea64c9-usage"></a>
### `usage`

<storycode:copy id="f-a448ddea64c9-usage">
mod usage;

</storycode:copy>
<a id="f-a448ddea64c9-use_declaration"></a>
### `use_declaration`

The `exec_id` agent sessions stream under on the WS run channel.

<storycode:copy id="f-a448ddea64c9-use_declaration">
/// The `exec_id` agent sessions stream under on the WS run channel.
pub use events::AGENT_EXEC_ID;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-2"></a>
### `use_declaration`

Structured events emitted by the agent loop (serde; WS run channel).

<storycode:copy id="f-a448ddea64c9-use_declaration-2">
/// Structured events emitted by the agent loop (serde; WS run channel).
pub use events::AgentEvent;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-a448ddea64c9-use_declaration-3">
pub use invocation::{ToolInvocation, parse_tool_invocation};
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-4"></a>
### `use_declaration`

Provider keys stored on disk by the desktop app (0600; Debug-redacted).

<storycode:copy id="f-a448ddea64c9-use_declaration-4">
/// Provider keys stored on disk by the desktop app (0600; Debug-redacted).
pub use key_store::KeyStore;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-5"></a>
### `use_declaration`

A displayable key fragment: first 4 + last 2 chars at most, never the key.

<storycode:copy id="f-a448ddea64c9-use_declaration-5">
/// A displayable key fragment: first 4 + last 2 chars at most, never the key.
pub use key_store::masked_key;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-6"></a>
### `use_declaration`

<storycode:copy id="f-a448ddea64c9-use_declaration-6">
/// A chunk of streamed LLM output.
pub use llm::ChatChunk;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-7"></a>
### `use_declaration`

<storycode:copy id="f-a448ddea64c9-use_declaration-7">
/// A pinned, boxed stream of chat chunks.
pub use llm::ChatStream;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-8"></a>
### `use_declaration`

<storycode:copy id="f-a448ddea64c9-use_declaration-8">
/// Trait for LLM chat completion clients.
pub use llm::LlmClient;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-9"></a>
### `use_declaration`

<storycode:copy id="f-a448ddea64c9-use_declaration-9">
/// Chat message role.
pub use llm::Role;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-10"></a>
### `use_declaration`

A chat message with a role and content string.

<storycode:copy id="f-a448ddea64c9-use_declaration-10">
/// A chat message with a role and content string.
pub use llm::{Image, Message};
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-11"></a>
### `use_declaration`

Anthropic Messages API client (streaming + non-streaming).

<storycode:copy id="f-a448ddea64c9-use_declaration-11">
/// Anthropic Messages API client (streaming + non-streaming).
pub use llm_anthropic::AnthropicClient;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-12"></a>
### `use_declaration`

Default Anthropic model id (current Sonnet-class alias).

<storycode:copy id="f-a448ddea64c9-use_declaration-12">
/// Default Anthropic model id (current Sonnet-class alias).
pub use llm_anthropic::DEFAULT_ANTHROPIC_MODEL;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-13"></a>
### `use_declaration`

Output effort level (`output_config.effort`).

<storycode:copy id="f-a448ddea64c9-use_declaration-13">
/// Output effort level (`output_config.effort`).
pub use llm_anthropic::Effort;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-14"></a>
### `use_declaration`

Result of a cacheable-prefix measurement (via `count_tokens`).

<storycode:copy id="f-a448ddea64c9-use_declaration-14">
/// Result of a cacheable-prefix measurement (via `count_tokens`).
pub use llm_anthropic::PrefixCheck;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-15"></a>
### `use_declaration`

Message Batches API client (50%-discounted, latency-insensitive runs).

<storycode:copy id="f-a448ddea64c9-use_declaration-15">
/// Message Batches API client (50%-discounted, latency-insensitive runs).
pub use llm_batch::AnthropicBatchClient;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-16"></a>
### `use_declaration`

One request in a message batch (keyed by `custom_id`).

<storycode:copy id="f-a448ddea64c9-use_declaration-16">
/// One request in a message batch (keyed by `custom_id`).
pub use llm_batch::BatchEntry;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-17"></a>
### `use_declaration`

<storycode:copy id="f-a448ddea64c9-use_declaration-17">
/// Terminal result of one batch entry.
pub use llm_batch::BatchResult;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-18"></a>
### `use_declaration`

Chat client for any OpenAI-compatible provider.

<storycode:copy id="f-a448ddea64c9-use_declaration-18">
/// Chat client for any OpenAI-compatible provider.
pub use llm_openai::OpenAiCompatClient;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-19"></a>
### `use_declaration`

An OpenAI-compatible vendor (OpenAI, DeepSeek, xAI/Grok, OpenRouter).

<storycode:copy id="f-a448ddea64c9-use_declaration-19">
/// An OpenAI-compatible vendor (OpenAI, DeepSeek, xAI/Grok, OpenRouter).
pub use llm_openai::Provider;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-20"></a>
### `use_declaration`

Build the client for a provider selector (`anthropic`/`openai`/…). The
`_with_store` variants consult a [`KeyStore`] before the environment.

<storycode:copy id="f-a448ddea64c9-use_declaration-20">
/// Build the client for a provider selector (`anthropic`/`openai`/…). The
/// `_with_store` variants consult a [`KeyStore`] before the environment.
pub use provider::{
    ProviderSelection, client_for, client_for_action_with_limit, client_for_with_limit,
    client_for_with_store, resolve_selector, resolve_selector_with_store,
};
</storycode:copy>
<a id="f-a448ddea64c9-STOPPED_BY_USER"></a>
### `STOPPED_BY_USER`

Cancellation status shared by the process supervisors and UI.

<storycode:copy id="f-a448ddea64c9-STOPPED_BY_USER">
/// Cancellation status shared by the process supervisors and UI.
pub const STOPPED_BY_USER: &str = "stopped by you";
</storycode:copy>
<a id="f-a448ddea64c9-PriorTurn"></a>
### `PriorTurn`

Persisted exchanges supplied as data to the Python agent.

<storycode:copy id="f-a448ddea64c9-PriorTurn">
/// Persisted exchanges supplied as data to the Python agent.
#[derive(Debug, Clone)]
pub struct PriorTurn {
    pub prompt: String,
    pub answer: String,
}
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-21"></a>
### `use_declaration`

<storycode:copy id="f-a448ddea64c9-use_declaration-21">
pub use receipt::ScriptResult;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-22"></a>
### `use_declaration`

A canned LLM client replaying scripted responses (tests, offline dev).

<storycode:copy id="f-a448ddea64c9-use_declaration-22">
/// A canned LLM client replaying scripted responses (tests, offline dev).
pub use scripted::ScriptedLlmClient;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-23"></a>
### `use_declaration`

Incremental `hick:session` document writer.

<storycode:copy id="f-a448ddea64c9-use_declaration-23">
/// Incremental `hick:session` document writer.
pub use session::HickSessionLog;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-24"></a>
### `use_declaration`

<storycode:copy id="f-a448ddea64c9-use_declaration-24">
/// A no-op session log.
pub use session::NullSessionLog;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-25"></a>
### `use_declaration`

A single event recorded into a session log.

<storycode:copy id="f-a448ddea64c9-use_declaration-25">
/// A single event recorded into a session log.
pub use session::SessionEvent;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-26"></a>
### `use_declaration`

Sink for structured agent session events.

<storycode:copy id="f-a448ddea64c9-use_declaration-26">
/// Sink for structured agent session events.
pub use session::SessionLog;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-27"></a>
### `use_declaration`

<storycode:copy id="f-a448ddea64c9-use_declaration-27">
pub use session::record_outcome;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-28"></a>
### `use_declaration`

Conventional session file path: `sessions/<timestamp>-<slug>.md`.

<storycode:copy id="f-a448ddea64c9-use_declaration-28">
/// Conventional session file path: `sessions/<timestamp>-<slug>.md`.
pub use session::session_file_path;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-29"></a>
### `use_declaration`

Single-writer edit session over one primary hick document.

<storycode:copy id="f-a448ddea64c9-use_declaration-29">
/// Single-writer edit session over one primary hick document.
pub use tools::EditSession;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-30"></a>
### `use_declaration`

The result of one tool invocation (a `<hick:tool-result>` observation).

<storycode:copy id="f-a448ddea64c9-use_declaration-30">
/// The result of one tool invocation (a `<hick:tool-result>` observation).
pub use tools::ToolOutcome;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-31"></a>
### `use_declaration`

Execute one tool invocation against an edit session.

<storycode:copy id="f-a448ddea64c9-use_declaration-31">
/// Execute one tool invocation against an edit session.
pub use tools::execute_tool;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-32"></a>
### `use_declaration`

Hashline rendering and anchor resolution primitives.

<storycode:copy id="f-a448ddea64c9-use_declaration-32">
/// Hashline rendering and anchor resolution primitives.
pub use tools::hashline;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-33"></a>
### `use_declaration`

<storycode:copy id="f-a448ddea64c9-use_declaration-33">
pub use tools::{ContextRead, Wrote};
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-34"></a>
### `use_declaration`

Four-way token usage for one LLM call or an accumulated total.

<storycode:copy id="f-a448ddea64c9-use_declaration-34">
/// Four-way token usage for one LLM call or an accumulated total.
pub use usage::Usage;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-35"></a>
### `use_declaration`

USD cost of a usage record on a model (cache multipliers applied).

<storycode:copy id="f-a448ddea64c9-use_declaration-35">
/// USD cost of a usage record on a model (cache multipliers applied).
pub use usage::cost_usd;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-36"></a>
### `use_declaration`

Minimum cacheable prefix tokens for a model (below it: silent no-cache).

<storycode:copy id="f-a448ddea64c9-use_declaration-36">
/// Minimum cacheable prefix tokens for a model (below it: silent no-cache).
pub use usage::min_cacheable_prefix_tokens;
</storycode:copy>
<a id="f-a448ddea64c9-use_declaration-37"></a>
### `use_declaration`

<storycode:copy id="f-a448ddea64c9-use_declaration-37">
/// Per-MTok price lookup for known models.
pub use usage::price_for_model;
</storycode:copy>

### Assemble `crates/hickory-agent/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/src/lib.rs"><storycode:paste select="#f-a448ddea64c9-audit"/><storycode:paste select="#f-a448ddea64c9-context"/><storycode:paste select="#f-a448ddea64c9-events"/><storycode:paste select="#f-a448ddea64c9-harness"/><storycode:paste select="#f-a448ddea64c9-invocation"/><storycode:paste select="#f-a448ddea64c9-key_store"/><storycode:paste select="#f-a448ddea64c9-llm"/><storycode:paste select="#f-a448ddea64c9-llm_anthropic"/><storycode:paste select="#f-a448ddea64c9-llm_batch"/><storycode:paste select="#f-a448ddea64c9-llm_openai"/><storycode:paste select="#f-a448ddea64c9-provider"/><storycode:paste select="#f-a448ddea64c9-receipt"/><storycode:paste select="#f-a448ddea64c9-review"/><storycode:paste select="#f-a448ddea64c9-scripted"/><storycode:paste select="#f-a448ddea64c9-session"/><storycode:paste select="#f-a448ddea64c9-session_view"/><storycode:paste select="#f-a448ddea64c9-tools"/><storycode:paste select="#f-a448ddea64c9-usage"/><storycode:paste select="#f-a448ddea64c9-use_declaration"/><storycode:paste select="#f-a448ddea64c9-use_declaration-2"/><storycode:paste select="#f-a448ddea64c9-use_declaration-3"/><storycode:paste select="#f-a448ddea64c9-use_declaration-4"/><storycode:paste select="#f-a448ddea64c9-use_declaration-5"/><storycode:paste select="#f-a448ddea64c9-use_declaration-6"/><storycode:paste select="#f-a448ddea64c9-use_declaration-7"/><storycode:paste select="#f-a448ddea64c9-use_declaration-8"/><storycode:paste select="#f-a448ddea64c9-use_declaration-9"/><storycode:paste select="#f-a448ddea64c9-use_declaration-10"/><storycode:paste select="#f-a448ddea64c9-use_declaration-11"/><storycode:paste select="#f-a448ddea64c9-use_declaration-12"/><storycode:paste select="#f-a448ddea64c9-use_declaration-13"/><storycode:paste select="#f-a448ddea64c9-use_declaration-14"/><storycode:paste select="#f-a448ddea64c9-use_declaration-15"/><storycode:paste select="#f-a448ddea64c9-use_declaration-16"/><storycode:paste select="#f-a448ddea64c9-use_declaration-17"/><storycode:paste select="#f-a448ddea64c9-use_declaration-18"/><storycode:paste select="#f-a448ddea64c9-use_declaration-19"/><storycode:paste select="#f-a448ddea64c9-use_declaration-20"/><storycode:paste select="#f-a448ddea64c9-STOPPED_BY_USER"/><storycode:paste select="#f-a448ddea64c9-PriorTurn"/><storycode:paste select="#f-a448ddea64c9-use_declaration-21"/><storycode:paste select="#f-a448ddea64c9-use_declaration-22"/><storycode:paste select="#f-a448ddea64c9-use_declaration-23"/><storycode:paste select="#f-a448ddea64c9-use_declaration-24"/><storycode:paste select="#f-a448ddea64c9-use_declaration-25"/><storycode:paste select="#f-a448ddea64c9-use_declaration-26"/><storycode:paste select="#f-a448ddea64c9-use_declaration-27"/><storycode:paste select="#f-a448ddea64c9-use_declaration-28"/><storycode:paste select="#f-a448ddea64c9-use_declaration-29"/><storycode:paste select="#f-a448ddea64c9-use_declaration-30"/><storycode:paste select="#f-a448ddea64c9-use_declaration-31"/><storycode:paste select="#f-a448ddea64c9-use_declaration-32"/><storycode:paste select="#f-a448ddea64c9-use_declaration-33"/><storycode:paste select="#f-a448ddea64c9-use_declaration-34"/><storycode:paste select="#f-a448ddea64c9-use_declaration-35"/><storycode:paste select="#f-a448ddea64c9-use_declaration-36"/><storycode:paste select="#f-a448ddea64c9-use_declaration-37"/></storycode:file>

</storycode:doc>
