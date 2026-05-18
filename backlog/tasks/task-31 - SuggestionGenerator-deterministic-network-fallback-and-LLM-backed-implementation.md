---
id: TASK-31
title: >-
  SuggestionGenerator: deterministic network fallback and LLM-backed
  implementation
status: Done
assignee: []
created_date: '2026-05-17 10:48'
updated_date: '2026-05-18 12:04'
labels:
  - security
  - policy
  - llm
  - ux
milestone: m-4
dependencies:
  - TASK-30
references:
  - hick-security/crates/hick-policy/src/lib.rs
  - hick-agent/crates/hick-agent-sdk/src/network_filter.rs
  - hick-agent/crates/hick-agent-sdk/src/security_config.rs
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
When a prompt fires (network request, MCP tool call, or API sink), the host needs to generate 2–4 `PolicySuggestion` values at different generalization levels to present to the user alongside the Allow Once / Always Allow choice.

This task implements the suggestion generation layer in `hick-policy`.

## `CapabilityRequest` type

A structured description of what was requested — the input to a suggestion generator:

```rust
pub enum CapabilityRequest {
    Network { host: String },
    McpTool { server_name: String, tool_name: String, args_summary: Option<String> },
    ApiSink { sink_type: String, slot_name: String },
}
```

## `SuggestionGenerator` trait

```rust
#[async_trait]
pub trait SuggestionGenerator: Send + Sync {
    async fn suggestions(&self, req: &CapabilityRequest) -> Vec<PolicySuggestion>;
}
```

## `DeterministicSuggestionGenerator`

No LLM. Produces suggestions mechanically from the request:

- `CapabilityRequest::Network { host }`:
  - Exact: `"api.example.com"` → `PolicySuggestion { rule_type: Network, pattern: "api.example.com", scope: Exact }`
  - WildcardDomain: `"*.example.com"` (strip subdomain if present)
  - WildcardAll: `"*"`
- `CapabilityRequest::McpTool { tool_name, .. }`:
  - Exact: allow this specific tool name
  - WildcardAll: allow all tools on this server
- `CapabilityRequest::ApiSink { sink_type, .. }`:
  - Exact: allow this sink type

## `LlmSuggestionGenerator`

Uses the `LlmClient` from `hick-agent-sdk` to generate suggestions for cases where the deterministic approach is too coarse (primarily MCP tool calls with complex args).

**Critical constraint**: The LLM call must carry NO conversation history and NO agent task context — only the `CapabilityRequest` serialized as JSON. This ensures the suggestion is based solely on the capability being requested, not on what the agent is trying to accomplish. This is both a privacy property and a correctness property.

The prompt asks the LLM to return a JSON array of `{ pattern, scope }` objects at different generalization levels. The generator deserializes these into `PolicySuggestion` values. Falls back to `DeterministicSuggestionGenerator` output if the LLM call fails or returns invalid JSON.

## Eager background generation

Suggestion generation should be kicked off in the background as soon as a prompt fires, so results are ready if the user chooses Always Allow. The result is discarded if the user chooses Allow Once. Wire this timing up in the binary crates (copper-ledge, iron-peak) in the Always Allow task — this task only provides the generator types.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 CapabilityRequest enum is defined and pub-use'd from hick-policy.
- [ ] #2 SuggestionGenerator trait is defined and pub-use'd.
- [ ] #3 DeterministicSuggestionGenerator produces 3 suggestions for a Network request: exact host, wildcard domain, wildcard all.
- [ ] #4 DeterministicSuggestionGenerator produces 2 suggestions for an McpTool request: exact tool, all tools on server.
- [ ] #5 LlmSuggestionGenerator fires an LLM call with only the CapabilityRequest JSON — no conversation history, no agent context.
- [ ] #6 LlmSuggestionGenerator falls back to DeterministicSuggestionGenerator output on LLM error or parse failure.
- [ ] #7 Tests for DeterministicSuggestionGenerator cover all CapabilityRequest variants.
- [ ] #8 LlmSuggestionGenerator has an integration test using a mock LlmClient that returns valid and invalid JSON.
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added `SuggestionGenerator` async trait, `CapabilityRequest` enum, and `DeterministicSuggestionGenerator` in new `hick-policy/src/generator.rs`. Added `LlmSuggestionGenerator` with deterministic fallback in new `hick-agent-sdk/src/suggestion_gen.rs`. All re-exported from their respective `lib.rs` facades.
<!-- SECTION:FINAL_SUMMARY:END -->
