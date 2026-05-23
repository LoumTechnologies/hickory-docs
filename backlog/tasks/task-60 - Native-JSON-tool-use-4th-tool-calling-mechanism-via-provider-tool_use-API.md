---
id: TASK-60
title: 'Native JSON tool use: 4th tool-calling mechanism via provider tool_use API'
status: To Do
assignee: []
created_date: '2026-05-23 02:27'
labels: []
milestone: m-3
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add native structured tool use (OpenAI/Anthropic `tool_use` content blocks) as a fourth tool-calling mechanism alongside the existing three: XML elements (`hick-agent-mcp-xml`), GraphQL fenced blocks (`hick-agent-mcp-graphql`), and generated SDK stubs (`hick-agent-mcp-sdk`).

With native tool use the model API returns a structured `tool_use` content block with typed JSON — no text parsing required. The agent loop detects these blocks, dispatches to `ToolGate` + `McpClient`, and injects `tool_result` blocks back into the message history. This is how every major modern agent works (Claude Code, Amp, etc.) and is the most reliable path for MCP calls because the API guarantees schema compliance.

The four mechanisms are complementary: native tool use should be the default for models/providers that support it; the text-embedded formats remain available for script-first contexts and models that don't support structured tool use.

Key files:
- `hick-agent-sdk/src/llm_anthropic.rs` — extend to pass tool schemas and decode `tool_use` blocks
- `hick-agent-sdk/src/llm_openai_compat.rs` — extend to pass tool schemas and decode `tool_calls` delta chunks
- `hick-agent-sdk/src/llm.rs` — add `ToolSchema` and `ToolCall` types to the shared `LlmClient` trait
- `hick-agent-sdk/src/mcp_client.rs` — `McpToolDef.input_schema` is already available; reuse it
- React loop / agent engine — detect tool calls in the completion, dispatch via ToolGate, loop
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 LlmClient trait gains optional tool schema input and tool call output so providers that don't support it can return None
- [ ] #2 AnthropicClient passes McpToolDef schemas as tools[] in the request and decodes tool_use content blocks in the response stream
- [ ] #3 OpenAiCompatClient passes McpToolDef schemas as tools[] and decodes tool_calls delta chunks
- [ ] #4 Agent loop detects tool calls from LlmClient, dispatches each through ToolGate (respecting block/allow/prompt verdicts), injects tool_result messages, and continues the loop
- [ ] #5 ToolGate classification checks apply identically to native tool calls as they do to XML/GraphQL/SDK calls
- [ ] #6 An integration test exercises at least one round-trip: agent turn → tool_use block → ToolGate allow → mock McpClient → tool_result injected → next turn
- [ ] #7 Existing XML, GraphQL, and SDK tool-calling paths are unaffected
<!-- AC:END -->
