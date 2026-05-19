# hick-agent-sdk

Foundational, pipeline-agnostic core that all agent crates depend on.

Contains zero Hickory XML knowledge — it is the shared infrastructure layer that
extensions and binary crates build on top of.

## Provided capabilities

- **LLM clients** — `LlmClient` trait with implementations for Anthropic and
  OpenAI-compatible APIs
- **WASM sandbox** — `WasmSandboxExecutor` for isolated command execution
- **Network filtering** — `NetworkFilter` / `FilteredLlmClient` for gating
  outbound requests
- **MCP client layer** — `StdioMcpClient`, `HttpMcpClient`, `McpCredentialStore`,
  `ToolGate` for calling MCP servers
- **Security** — `SinkGuard` for API sink validation; `AgentSecurityConfig`
  loaded from `.hick/agent-security.toml`
- **Agent runtime** — `AgentRuntime`, the agent while-loop handle

Extend this crate (rather than forking it) when adding new foundational
capabilities.
