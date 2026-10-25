<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-agent/src/lib.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../contract.md)

Captured SHA-256: `dffefe1d3c0feecfa01cd5df29be29e356cabd1ad563c313bbe76f1d8540d8a3`.

<book:entry path="crates/hickory-agent/src/lib.rs">
//! Shared provider transports, document tools, evidence and session rendering.
//! Agent orchestration lives exclusively in apps/agent.

pub mod audit;
pub mod context;
mod events;
pub mod harness;
mod invocation;
mod key_store;
mod llm;
mod llm_anthropic;
mod llm_batch;
mod llm_openai;
mod provider;
mod receipt;
mod review;
mod scripted;
mod session;
pub mod session_view;
mod tools;
mod usage;

/// The `exec_id` agent sessions stream under on the WS run channel.
pub use events::AGENT_EXEC_ID;
/// Structured events emitted by the agent loop (serde; WS run channel).
pub use events::AgentEvent;
pub use invocation::{ToolInvocation, parse_tool_invocation};
/// Provider keys stored on disk by the desktop app (0600; Debug-redacted).
pub use key_store::KeyStore;
/// A displayable key fragment: first 4 + last 2 chars at most, never the key.
pub use key_store::masked_key;
/// A chunk of streamed LLM output.
pub use llm::ChatChunk;
/// A pinned, boxed stream of chat chunks.
pub use llm::ChatStream;
/// Trait for LLM chat completion clients.
pub use llm::LlmClient;
/// Chat message role.
pub use llm::Role;
/// A chat message with a role and content string.
pub use llm::{Image, Message};
/// Anthropic Messages API client (streaming + non-streaming).
pub use llm_anthropic::AnthropicClient;
/// Default Anthropic model id (current Sonnet-class alias).
pub use llm_anthropic::DEFAULT_ANTHROPIC_MODEL;
/// Output effort level (`output_config.effort`).
pub use llm_anthropic::Effort;
/// Result of a cacheable-prefix measurement (via `count_tokens`).
pub use llm_anthropic::PrefixCheck;
/// Message Batches API client (50%-discounted, latency-insensitive runs).
pub use llm_batch::AnthropicBatchClient;
/// One request in a message batch (keyed by `custom_id`).
pub use llm_batch::BatchEntry;
/// Terminal result of one batch entry.
pub use llm_batch::BatchResult;
/// Chat client for any OpenAI-compatible provider.
pub use llm_openai::OpenAiCompatClient;
/// An OpenAI-compatible vendor (OpenAI, DeepSeek, xAI/Grok, OpenRouter).
pub use llm_openai::Provider;
/// Build the client for a provider selector (`anthropic`/`openai`/…). The
/// `_with_store` variants consult a [`KeyStore`] before the environment.
pub use provider::{
    ProviderSelection, client_for, client_for_action_with_limit, client_for_with_limit,
    client_for_with_store, resolve_selector, resolve_selector_with_store,
};
/// Cancellation status shared by the process supervisors and UI.
pub const STOPPED_BY_USER: &str = "stopped by you";
/// Persisted exchanges supplied as data to the Python agent.
#[derive(Debug, Clone)]
pub struct PriorTurn {
    pub prompt: String,
    pub answer: String,
}
pub use receipt::ScriptResult;
/// A canned LLM client replaying scripted responses (tests, offline dev).
pub use scripted::ScriptedLlmClient;
/// Incremental `hick:session` document writer.
pub use session::HickSessionLog;
/// A no-op session log.
pub use session::NullSessionLog;
/// A single event recorded into a session log.
pub use session::SessionEvent;
/// Sink for structured agent session events.
pub use session::SessionLog;
pub use session::record_outcome;
/// Conventional session file path: `sessions/<timestamp>-<slug>.md`.
pub use session::session_file_path;
/// Single-writer edit session over one primary hick document.
pub use tools::EditSession;
/// The result of one tool invocation (a `<hick:tool-result>` observation).
pub use tools::ToolOutcome;
/// Execute one tool invocation against an edit session.
pub use tools::execute_tool;
/// Hashline rendering and anchor resolution primitives.
pub use tools::hashline;
pub use tools::{ContextRead, Wrote};
/// Four-way token usage for one LLM call or an accumulated total.
pub use usage::Usage;
/// USD cost of a usage record on a model (cache multipliers applied).
pub use usage::cost_usd;
/// Minimum cacheable prefix tokens for a model (below it: silent no-cache).
pub use usage::min_cacheable_prefix_tokens;
/// Per-MTok price lookup for known models.
pub use usage::price_for_model;
</book:entry>

</book:doc>
