//! The Hickory Docs AI agent harness.
//!
//! A script-first ReAct loop whose durable output is **literate
//! programming**: every session is written incrementally as a `hick:session`
//! document that round-trips through `hick-lang`'s `SessionDocument` parser
//! and can be promoted into a clean `hick:doc` pipeline with
//! `hickory promote`.
//!
//! The loop: the LLM responds with `<hick:next>code</hick:next>` plus one
//! fenced code block (shell or python) → the script executes through the
//! [`hickory_executor::Executor`] boundary in a workspace container →
//! stdout/stderr feeds back as the next observation. A
//! `<hick:next>done</hick:next>` response ends the run.
//!
//! Ported (minimally) from the `hick-agent` monorepo's `hick-agent-sdk` and
//! `hick-agent-script-first` crates; the security stack, MCP layers,
//! conversation trees, TUI, relay, and local-llama backends were deliberately
//! left behind — see `docs/developers/vendoring-notes.md`.

mod events;
mod llm;
mod llm_anthropic;
mod protocol;
mod react_loop;
mod script;
mod scripted;
mod session;

/// The `exec_id` agent sessions stream under on the WS run channel.
pub use events::AGENT_EXEC_ID;
/// Structured events emitted by the agent loop (serde; WS run channel).
pub use events::AgentEvent;
/// A chunk of streamed LLM output.
pub use llm::ChatChunk;
/// A pinned, boxed stream of chat chunks.
pub use llm::ChatStream;
/// Trait for LLM chat completion clients.
pub use llm::LlmClient;
/// A chat message with a role and content string.
pub use llm::Message;
/// Chat message role.
pub use llm::Role;
/// Anthropic Messages API client (streaming + non-streaming).
pub use llm_anthropic::AnthropicClient;
/// Default Anthropic model id (current Sonnet-class alias).
pub use llm_anthropic::DEFAULT_ANTHROPIC_MODEL;
/// Default system prompt for the script-first strategy.
pub use protocol::SYSTEM_PROMPT;
/// One parsed turn of the `<hick:next>` protocol.
pub use protocol::Turn;
/// Build the re-prompt correction message for a malformed response.
pub use protocol::correction_message;
/// Parse an LLM response against the `<hick:next>` protocol.
pub use protocol::parse_response;
/// Configuration for one agent run.
pub use react_loop::AgentConfig;
/// The result of a completed agent run.
pub use react_loop::AgentOutcome;
/// Run the script-first ReAct loop to completion.
pub use react_loop::run_agent;
/// The container name the agent runs in.
pub use script::AGENT_CONTAINER;
/// A code block extracted from an LLM response.
pub use script::CodeBlock;
/// Execution backend for an extracted code block (shell / python).
pub use script::Language;
/// Result of executing a script.
pub use script::ScriptResult;
/// Extract supported fenced code blocks from an LLM response.
pub use script::extract_code_blocks;
/// Run one code block through an executor and capture the result.
pub use script::run_script;
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
/// Conventional session file path: `sessions/<timestamp>-<slug>.hick`.
pub use session::session_file_path;
