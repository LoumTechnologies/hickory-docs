---
id: TASK-24.2
title: 'MCP tool gate: classification-aware allow/block/prompt gating for tool calls'
status: To Do
assignee: []
created_date: '2026-05-16 12:22'
updated_date: '2026-05-16 13:02'
labels:
  - mcp
  - security
milestone: m-3
dependencies:
  - TASK-24.1
references:
  - hick-agent/crates/hick-agent-sdk/src/network_filter.rs
  - hick-agent/crates/hick-agent-sdk/src/sink_guard.rs
  - hick-agent/crates/hick-agent-sdk/src/security_config.rs
parent_task_id: TASK-24
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a `ToolGate` to `hick-agent-sdk` that governs every MCP tool call across all four exposure modes. This is the security enforcement layer — all extension crates (TASK-24.3–24.6) must route every dispatch through `ToolGate` before the call reaches `McpClient`.

`ToolGate` is deliberately modelled on the existing `NetworkFilter` in `hick-agent-sdk/src/network_filter.rs`. The same allow/block/prompt pattern, the same approval channel model (`mpsc::UnboundedSender<ToolApprovalRequest>`), the same per-session verdict cache.

## Trust levels and classification interaction

Per-server and per-tool trust levels in `.hick/mcp-servers.toml`:

```toml
[[servers]]
name = "filesystem"
trust = "trusted"      # trusted | untrusted | prompt (default: prompt)

[[servers]]
name = "external-api"
trust = "untrusted"
```

Classification interaction rules:
1. If any slot of the current WASM container scope carries a classification that matches a rejection criterion configured for the `mcp-tool-call` sink type (analogous to existing `script-execution` sink in `AgentSecurityConfig`), the call is **blocked** regardless of trust level.
2. If the tool server is `untrusted` AND classified data matching any rejection criterion is present in scope, call is **blocked**.
3. If trust level is `prompt`, the user is prompted regardless of classification (like `RuleAction::Prompt` in `NetworkFilter`).
4. Verdicts (allow/block) are cached per tool name + server name for the session, same as `NetworkFilter`.

## New types (mirror of `network_filter.rs` pattern)

```rust
pub enum ToolTrust { Trusted, Untrusted, Prompt }
pub enum ToolVerdict { Allow, Block }
pub struct ToolApprovalRequest {
    pub server_name: String,
    pub tool_name: String,
    pub response_tx: oneshot::Sender<ToolVerdict>,
}
pub struct ToolGate { /* config, cache, approval_tx, sink_guard ref */ }
impl ToolGate {
    pub async fn check(&self, server: &str, tool: &str, scope_classifications: &[Classification]) -> ToolVerdict;
}
```

## `AgentSecurityConfig` extension

- Add `[[mcp_tool_sinks]]` analogous to existing `[[sinks]]` so users can configure which classification labels block MCP calls.
- Default: `sensitivity: secret` blocks all MCP tool calls (same default as script-execution sink).
- `to_tool_gate_config()` method on `AgentSecurityConfig` produces a `ToolGateConfig`.

## What this task does NOT do
- Does not dispatch tool calls (that's each extension crate's job).
- Does not implement any of the four LLM exposure modes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 ToolGate::check returns Block when scope_classifications contains a rejection criterion, regardless of tool trust level.
- [ ] #2 ToolGate::check returns Block (not Prompt) for untrusted tools when any rejection classification is present in scope.
- [ ] #3 ToolGate::check sends a ToolApprovalRequest and waits when trust = Prompt and no rejection classification is present.
- [ ] #4 Verdicts are cached per (server_name, tool_name) pair for the session duration.
- [ ] #5 AgentSecurityConfig gains a mcp_tool_sinks stanza; default config rejects sensitivity:secret from all MCP calls.
- [ ] #6 ToolGate is pub-use'd from hick-agent-sdk with full doc comments.
- [ ] #7 Tests cover: block-on-secret, block-untrusted-with-classification, prompt-for-unknown, cache-reuse, channel-closed-blocks-safely.
- [ ] #8 ToolGate compiles and passes tests in isolation (does not require any extension crate to exist).
<!-- AC:END -->
