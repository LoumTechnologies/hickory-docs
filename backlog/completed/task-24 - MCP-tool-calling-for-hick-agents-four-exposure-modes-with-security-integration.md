---
id: TASK-24
title: >-
  MCP tool calling for hick agents: four exposure modes with security
  integration
status: Done
assignee: []
created_date: '2026-05-16 12:21'
updated_date: '2026-05-18 11:44'
labels:
  - mcp
  - security
  - extensions
  - tool-calling
milestone: m-3
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Allow any hick agent binary to be configured with MCP servers. Expose those tools to the LLM through four pluggable modes, each implemented as a separate `hick-agent-*` extension crate. The entire system must integrate with the existing classification/sink-guard/network-filter security framework — auth secrets must never reach the LLM, and untrusted tools must be blocked or prompt-required when classified data is in scope.

## Architecture summary

**Core** (`hick-agent-sdk`):
- MCP client: server config schema, auth secret management, JSON-RPC 2.0 client (stdio + HTTP transports), tool manifest loading.
- MCP tool gate: per-tool trust levels, classification-aware call blocking/prompting (analogous to `NetworkFilter`), user approval channel.

**Extensions** (`hick-agent-mcp-*`):
- `hick-agent-mcp-wasm` — auto-generated WASI binary per tool, callable from inside the WASM VM like any CLI.
- `hick-agent-mcp-sdk` — auto-generated Python/TypeScript stub SDK, auto-imported into every LLM script.
- `hick-agent-mcp-xml` — `<hick:tool>` XML element dispatcher in the hick pipeline.
- `hick-agent-mcp-graphql` — local GraphQL server over MCP JSON-RPC; LLM field-selects what enters context.

## Security invariants (non-negotiable across all subtasks)
1. Auth credentials (API keys, tokens) are loaded from host environment / secrets file and NEVER written into generated WASI source, SDK files, GraphQL schema, or LLM context.
2. `ToolGate` enforces per-tool allow/block/prompt policy before any dispatch, analogous to `NetworkFilter`.
3. If classified data (`sensitivity: secret` or any configured rejection criterion) is present in the WASM container scope, calls to untrusted tools are blocked; if trust level is unknown, the user is prompted via the approval channel.
4. All four exposure modes funnel through the same `ToolGate` so security properties are mode-independent.

## Subtask order
TASK-24.1 (MCP client core) → TASK-24.2 (tool gate) → TASK-24.3–24.6 (exposure mode extensions, can proceed in parallel once 24.1 and 24.2 are done).
<!-- SECTION:DESCRIPTION:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
All eight subtasks complete. MCP client core + ToolGate in hick-agent-sdk (24.1, 24.2); four extension crates (mcp-wasm, mcp-sdk, mcp-xml, mcp-graphql) for the four exposure modes (24.3–24.6); shared ShapeFormatConfig response serialization in hick-agent-sdk (24.7); McpToolRule caveat in hick-token 0.2.0 (24.8). Universal hick-agent binary wires all extensions from .hick/agent.toml (TASK-28).
<!-- SECTION:FINAL_SUMMARY:END -->
