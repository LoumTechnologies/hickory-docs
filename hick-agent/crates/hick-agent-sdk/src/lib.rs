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
//! | [`McpClient`] | Trait for MCP server clients |
//! | [`StdioMcpClient`] | MCP client over child-process stdio |
//! | [`HttpMcpClient`] | MCP client over HTTP |
//! | [`McpCredentialStore`] | Holds resolved MCP auth credentials |
//! | [`ToolGate`] | Classification-aware MCP tool call gating |
//! | [`ShapeFormatConfig`] | Shape-aware MCP result serialization config |
//! | [`format_mcp_result`] | Detect shape and serialize an MCP result |

pub mod command;
pub mod llm;
pub mod llm_anthropic;
pub mod llm_openai_compat;
pub mod mcp_client;
pub mod mcp_config;
pub mod mcp_format;
pub mod network_filter;
pub mod project_context;
pub mod runtime;
pub mod security_config;
pub mod sink_guard;
pub mod tool_gate;

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
// MCP client
// ---------------------------------------------------------------------------

/// Trait for a connected MCP server client (stdio or HTTP).
pub use mcp_client::McpClient;
/// MCP client that communicates with a server process over stdin/stdout.
pub use mcp_client::StdioMcpClient;
/// MCP client that communicates with a server over HTTP (JSON-RPC 2.0 POST).
pub use mcp_client::HttpMcpClient;
/// Description of a single tool exposed by an MCP server.
pub use mcp_client::McpToolDef;

// ---------------------------------------------------------------------------
// MCP configuration
// ---------------------------------------------------------------------------

/// Transport kind for an MCP server connection (`stdio` or `http`).
pub use mcp_config::McpTransportKind;
/// Trust level assigned to an MCP server.
pub use mcp_config::McpServerTrust;
/// Configuration for a single MCP server (safe to commit — no secrets).
pub use mcp_config::McpServerConfig;
/// Top-level server configuration file (`.hick/mcp-servers.toml`).
pub use mcp_config::McpServersConfig;
/// A resolved credential ready to inject as HTTP headers at call time.
pub use mcp_config::ResolvedCredential;
/// Holds resolved MCP auth credentials; never serialized, never written to disk.
pub use mcp_config::McpCredentialStore;

// ---------------------------------------------------------------------------
// Tool gate
// ---------------------------------------------------------------------------

/// Trust level for an MCP server, used by `ToolGate` to determine gating.
pub use tool_gate::ToolTrust;
/// Verdict returned by [`ToolGate::check`]: `Allow` or `Block`.
pub use tool_gate::ToolVerdict;
/// Request sent to the UI when `ToolGate` needs the user's decision.
pub use tool_gate::ToolApprovalRequest;
/// Configuration for [`ToolGate`], produced by `AgentSecurityConfig::to_tool_gate_config`.
pub use tool_gate::ToolGateConfig;
/// Classification-aware MCP tool call gating with per-session verdict caching.
pub use tool_gate::ToolGate;

// ---------------------------------------------------------------------------
// MCP response format
// ---------------------------------------------------------------------------

/// Structural shape of an MCP tool response value.
pub use mcp_format::ResponseShape;
/// Detect the structural shape of a JSON value.
pub use mcp_format::detect_shape;
/// Output serialization format for MCP tool results.
pub use mcp_format::McpResponseFormat;
/// Per-shape format overrides.
pub use mcp_format::ByShapeConfig;
/// Shape-aware format configuration for MCP tool results.
pub use mcp_format::ShapeFormatConfig;
/// Detect shape and serialize an MCP result using the configured format.
pub use mcp_format::format_mcp_result;

// ---------------------------------------------------------------------------
// Project context
// ---------------------------------------------------------------------------

/// Loads agent context from standard project locations (AGENTS.md, CLAUDE.md, etc.).
pub use project_context::ProjectContextLoader;
