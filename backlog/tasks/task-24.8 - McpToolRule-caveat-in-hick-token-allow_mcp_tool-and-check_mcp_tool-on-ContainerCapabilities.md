---
id: TASK-24.8
title: >-
  McpToolRule caveat in hick-token: allow_mcp_tool and check_mcp_tool on
  ContainerCapabilities
status: To Do
assignee: []
created_date: '2026-05-17 10:49'
labels:
  - mcp
  - security
milestone: m-3
dependencies:
  - TASK-24.1
references:
  - hick-security/crates/hick-token/src/lib.rs
parent_task_id: TASK-24
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add `McpToolRule` as a caveat type in `hick-security/crates/hick-token/src/lib.rs`, giving the capability token system awareness of which MCP tools a container is permitted to call. This is the static allow/deny layer at the token level — separate from the runtime `ToolGate` (TASK-24.2), which handles prompt/trust-level logic. A container's token can be attenuated to restrict which tools it may call; the `ToolGate` then adds a second, runtime layer on top.

The pattern follows the existing `ApiSinkRule` exactly:
- Caveat string format: `"mcp-tool = <tool_name>"`
- `McpToolRule { tool_name: String }` with `to_caveat()` and `from_caveat()`
- `allow_mcp_tool(tool_name: &str) -> Self` builder on `ContainerCapabilities`
- `check_mcp_tool(tool_name: &str) -> bool` check method (closed-by-default: returns false when no rules defined)
- Wire into `intersect()` and `attenuate()` the same way `api_sink_rules` is handled (lines 461–464 and 549–551 in the current lib.rs)
- Wire into the caveat serialization/deserialization loop (same pattern as `ApiSinkRule::from_caveat` at line 595)

Minor version bump for hick-token (new public API).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 McpToolRule is defined with to_caveat() and from_caveat() following the ApiSinkRule pattern.
- [ ] #2 allow_mcp_tool() adds an McpToolRule to ContainerCapabilities.
- [ ] #3 check_mcp_tool() returns true only when a matching rule exists; returns false when no mcp_tool_rules are defined.
- [ ] #4 McpToolRule is correctly preserved through attenuate() and intersect().
- [ ] #5 Caveat roundtrip: to_caveat() output parses back to the same McpToolRule via from_caveat().
- [ ] #6 hick-token version is bumped (minor).
- [ ] #7 Tests cover: allow specific tool, deny absent tool, closed-by-default, attenuation preserves rules, intersect keeps only shared rules.
<!-- AC:END -->
