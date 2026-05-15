//! Generic agent building-block crate — zero hick pipeline knowledge.
//!
//! `hick-agent-sdk` provides the foundational primitives for building AI coding
//! agents. It deliberately has **no** dependency on Hickory XML, hick:file,
//! hick:copy/paste, or any other hick pipeline concept, making it suitable for
//! building agents with completely different output formats.
//!
//! # Key types
//!
//! | Type | Purpose |
//! |------|---------|
//! | [`AgentRuntime`] | Handle for writing an agent while loop |
//! | [`Step`] | Enum dispatched in the while loop body |
//! | [`LlmClient`] | Trait for LLM chat completion clients |
//! | [`AnthropicClient`] | Anthropic Messages API client |
//! | [`OpenAiCompatClient`] | OpenAI-compatible API client |
//! | [`FilteredLlmClient`] | Network-filtered LLM wrapper |
//! | [`NetworkFilter`] | Configurable outbound network filter |
//! | [`SinkGuard`] | API sink call validator |
//! | [`ProjectContextLoader`] | Loads AGENTS.md / CLAUDE.md context |
//! | [`CommandExecutor`] | Trait for sandboxed command execution |
//! | [`WasmSandboxExecutor`] | WASM container executor (backend-agnostic) |

pub mod command;
pub mod llm;
pub mod llm_anthropic;
pub mod llm_openai_compat;
pub mod network_filter;
pub mod project_context;
pub mod runtime;
pub mod security_config;
pub mod sink_guard;

// ---------------------------------------------------------------------------
// Runtime
// ---------------------------------------------------------------------------

/// The handle for writing an agent while loop (history, streaming, sandbox).
pub use runtime::AgentRuntime;
/// Event enum that the agent while loop dispatches on.
pub use runtime::Step;
/// A streaming token chunk from the LLM.
pub use runtime::StreamChunk;
/// A structured tool call requested by the LLM.
pub use runtime::ToolCall;

// ---------------------------------------------------------------------------
// LLM clients
// ---------------------------------------------------------------------------

/// Trait for LLM chat completion clients (streaming and non-streaming).
pub use llm::LlmClient;
/// A chat message with a role and content string.
pub use llm::Message;
/// Chat message role: System, User, or Assistant.
pub use llm::Role;
/// A chunk of streamed LLM output containing a text delta and optional finish reason.
pub use llm::ChatChunk;
/// Anthropic Messages API client with streaming support.
pub use llm_anthropic::AnthropicClient;
/// OpenAI-compatible chat completion client (works with OpenAI, xAI, Groq, Ollama, etc.).
pub use llm_openai_compat::OpenAiCompatClient;

// ---------------------------------------------------------------------------
// Network filtering
// ---------------------------------------------------------------------------

/// Filters outbound network requests by host against configurable rules.
pub use network_filter::NetworkFilter;
/// An [`LlmClient`] wrapper that checks a [`NetworkFilter`] before each API call.
pub use network_filter::FilteredLlmClient;
/// Request sent to the UI to prompt the user for network access approval.
pub use network_filter::NetworkApprovalRequest;
/// Verdict for a filtered network request: Allow or Block.
pub use network_filter::NetworkVerdict;

// ---------------------------------------------------------------------------
// Sandbox execution
// ---------------------------------------------------------------------------

/// Trait for executing shell commands in a sandboxed environment.
pub use command::CommandExecutor;
/// Structured output (stdout/stderr and exit code) from a sandboxed command execution.
pub use command::CommandOutput;
/// Low-level container backend providing raw start and execute primitives.
pub use command::ContainerBackend;
/// Configuration for WASM sandbox execution.
pub use command::WasmSandboxConfig;
/// Executes commands in Hick WASM Linux containers with persistent state.
pub use command::WasmSandboxExecutor;

// ---------------------------------------------------------------------------
// Security
// ---------------------------------------------------------------------------

/// Tracks API sink call counts and validates calls against capabilities.
pub use sink_guard::SinkGuard;
/// Top-level agent security configuration loaded from `.hick/agent-security.toml`.
pub use security_config::AgentSecurityConfig;

// ---------------------------------------------------------------------------
// Project context
// ---------------------------------------------------------------------------

/// Loads agent context from standard project locations (AGENTS.md, CLAUDE.md, etc.).
pub use project_context::ProjectContextLoader;
