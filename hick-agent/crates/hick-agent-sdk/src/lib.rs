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
//! | [`UiEvent`] | Canonical UI event stream (TUI, web, relay) |
//! | [`UiCommand`] | Commands from any UI toward the agent core |
//! | [`ConversationTree`] | Conversation tree derived from [`UiEvent`]s |
//!
//! With the **`web`** feature ([`WebServer`]), expose a localhost HTTP shell + `/ws` JSON bridge.
pub mod agent_memory;
pub mod agent_skills;
pub mod agentignore;
pub mod command;
pub mod conversation_tree;
pub mod llm;
pub mod llm_anthropic;
pub mod llm_openai_compat;
pub mod mcp_client;
pub mod mcp_config;
pub mod mcp_format;
pub mod mcp_install;
pub mod mcp_project_config;
pub mod network_filter;
pub mod project_context;
pub mod runtime;
pub mod security_config;
pub mod session_log;
pub mod sink_guard;
pub mod suggestion_gen;
pub mod tool_gate;
pub mod xml_session_log;
pub mod session_bundle_reader;
pub mod ui_event;
pub mod pane_watcher;
pub mod pane_csp;

#[cfg(feature = "web")]
pub mod web_server;

#[cfg(feature = "relay")]
pub mod relay_client;

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
/// Response returned through the approval channel (verdict + optional always-allow rule).
pub use network_filter::ApprovalResponse;

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
/// MCP client that runs a stdio server with a sentinel credential and sandbox isolation.
pub use mcp_client::SandboxedStdioMcpClient;
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
/// Per-server sandbox configuration for stdio MCP servers.
pub use mcp_config::McpSandboxConfig;
/// A host entry in the per-server sandbox network allow-list.
pub use mcp_config::McpSandboxAllowedHost;
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
// Suggestion generation
// ---------------------------------------------------------------------------

/// LLM-backed suggestion generator for the Always Allow flow.
pub use suggestion_gen::LlmSuggestionGenerator;

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

// ---------------------------------------------------------------------------
// .agentignore (task 38)
// ---------------------------------------------------------------------------

/// Gitignore-style path filter loaded from `.agentignore` at the project root.
pub use agentignore::AgentIgnoreFilter;

// ---------------------------------------------------------------------------
// Project-local MCP server discovery (task 39)
// ---------------------------------------------------------------------------

/// MCP server configuration discovered from `mcp.json` / `.mcp.json` at the project root.
pub use mcp_project_config::ProjectMcpConfig;
/// A single MCP server entry from the project's `mcp.json` / `.mcp.json`.
pub use mcp_project_config::ProjectMcpServerEntry;

// ---------------------------------------------------------------------------
// Agent skills (task 40)
// ---------------------------------------------------------------------------

/// A single skill loaded from `.agents/skills/<name>.md`.
pub use agent_skills::Skill;
/// Loads and provides access to project skills from `.agents/skills/`.
pub use agent_skills::SkillLoader;

// ---------------------------------------------------------------------------
// Agent memory bank (task 41)
// ---------------------------------------------------------------------------

/// Persistent agent memory bank backed by `.agents/memory/*.md` files.
pub use agent_memory::MemoryBank;

// ---------------------------------------------------------------------------
// MCP install
// ---------------------------------------------------------------------------

/// Parsed MCP server install specification (stdio command or HTTP URL).
pub use mcp_install::McpInstallSpec;
/// Package manager detected from a stdio command's first token.
pub use mcp_install::PackageManager;
/// Network policy covering all supported package registries (npm, PyPI, crates.io).
pub use mcp_install::PackageManagerNetworkPolicy;
/// Drives the MCP server install sequence (package install, verify, write config).
pub use mcp_install::McpInstaller;
/// Credential input request sent from an install task to the UI.
pub use mcp_install::McpCredentialRequest;
/// Parse a raw spec string into an [`McpInstallSpec`].
pub use mcp_install::parse_spec;
/// Infer a server name from an install spec; returns `None` when ambiguous.
pub use mcp_install::infer_name;
/// Detect the package manager used by a stdio command.
pub use mcp_install::detect_package_manager;
/// Read all configured MCP servers and return a formatted table.
pub use mcp_install::list_servers;
/// Remove a named server from `.hick/mcp-servers.toml`.
pub use mcp_install::remove_server;

// ---------------------------------------------------------------------------
// Session logging
// ---------------------------------------------------------------------------

/// Trait for structured agent session event sinks.
pub use session_log::SessionLog;
/// Typed event enum recorded by [`SessionLog::record`].
pub use session_log::LogEvent;
/// Binary attachment written into a session bundle directory.
pub use session_log::LogAttachment;
/// Severity level for [`LogEvent::DiagnosticLog`] records.
pub use session_log::LogLevel;
/// No-op [`SessionLog`] for tests and unconfigured contexts.
pub use session_log::NullSessionLog;
/// Writes Agent Log XML to a session directory bundle.
pub use xml_session_log::XmlSessionLog;
/// Reads a completed session bundle and maps it back to [`UiEvent`]s for replay.
pub use session_bundle_reader::SessionBundleReader;
/// [`log::Log`] adapter that forwards diagnostic records to a [`SessionLog`].
pub use xml_session_log::XmlDiagnosticLogger;
/// Wraps an [`McpClient`] to emit [`LogEvent::McpCall`] / [`LogEvent::McpResponse`] events.
pub use mcp_client::LoggingMcpClient;

// ---------------------------------------------------------------------------
// Agent UI model (shared TUI / web / relay)
// ---------------------------------------------------------------------------

/// Incrementally maintained conversation tree for UI rendering.
pub use conversation_tree::ConversationNode;
/// Category of node stored in a [`ConversationTree`].
pub use conversation_tree::ConversationNodeKind;
/// Opaque payload attached to a [`ConversationNode`].
pub use conversation_tree::ConversationPayload;
/// Append-only conversation tree built from [`UiEvent`]s.
pub use conversation_tree::ConversationTree;
/// Thread-safe [`NodeId`] allocator.
pub use ui_event::IdGen;
/// Stable vertex id for UI trees and transports.
pub use ui_event::NodeId;
/// Clone-safe network approval bubble for [`UiEvent::NetworkApproval`].
pub use ui_event::UiNetworkApproval;
/// Command from a UI toward the agent core (and optional tree chrome).
pub use ui_event::UiCommand;
/// UI-facing event emitted by bridges from extension cores.
pub use ui_event::UiEvent;
/// Emit an Adaptive Card as a structured XML log node.
pub use ui_event::emit_adaptive_card;

// ---------------------------------------------------------------------------
// Agent panes
// ---------------------------------------------------------------------------

/// Polls `.agents/panes/*/pane.yaml` and emits [`UiEvent::PaneContent`] on changes.
pub use pane_watcher::PaneWatcher;
/// Parsed schema for a `pane.yaml` file.
pub use pane_watcher::PaneYaml;
/// Build a Content-Security-Policy header value for an HTML pane iframe.
pub use pane_csp::generate_pane_csp;

// ---------------------------------------------------------------------------
// Local web UI (optional)
// ---------------------------------------------------------------------------

#[cfg(feature = "web")]
/// Configuration for [`WebServer::bind`] (defaults + [`WebServerConfig::from_env`]).
pub use web_server::WebServerConfig;
#[cfg(feature = "web")]
/// Local SPA + `/ws` bridge for [`UiEvent`] / [`UiCommand`] JSON.
pub use web_server::WebServer;

// ---------------------------------------------------------------------------
// Cloud relay (optional)
// ---------------------------------------------------------------------------

#[cfg(feature = "relay")]
/// Outbound WebSocket bridge to `relay.hickory.dev` for remote UI access.
pub use relay_client::RelayClient;
