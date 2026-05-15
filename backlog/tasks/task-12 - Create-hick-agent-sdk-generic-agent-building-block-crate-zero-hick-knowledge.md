---
id: TASK-12
title: >-
  Create hick-agent-sdk: generic agent building-block crate (zero hick
  knowledge)
status: To Do
assignee: []
created_date: '2026-05-15 12:43'
labels: []
milestone: m-0
dependencies: []
references:
  - hick-agent/crates/hick-agent-engine/src/lib.rs
  - hick-agent/crates/hick-agent-core/src/lib.rs
  - hick-agent/crates/hick-agent-engine/src/context.rs
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extract all generic, hick-agnostic building blocks from `hick-agent-engine` and `hick-agent-core` into a new `hick-agent-sdk` crate. This crate is the foundation for building hundreds of different agent behaviors — it must have **zero knowledge of Hickory XML**, hick:file, hick:copy/paste, or any hick pipeline concept.

**What moves into hick-agent-sdk:**

From `hick-agent-engine`:
- `LlmClient` trait + `AnthropicClient` + `OpenAiCompatClient` implementations
- `FilteredLlmClient` (network-filtered LLM wrapper)
- `NetworkFilter` (capability-based outbound filter)
- `SinkGuard` (file write tracking)
- `context.rs` — loads AGENTS.md / CLAUDE.md / .agents/*.md project context (rename to `ProjectContextLoader`)
- Token authority / budget tracking

From `hick-agent-core`:
- `CommandExecutor` trait
- `WasmSandboxConfig` + `WasmSandboxExecutor` (the WASM VM + sandbox lifecycle)

**New types added in hick-agent-sdk:**

`AgentRuntime` — the handle for writing an agent while loop. Manages:
- conversation history (add/remove/compact messages)
- streaming LLM calls (returns an async stream of tokens/events)
- sandbox lifecycle (start/stop/reset)
- firewall policy (allow/deny rules)
- project context loading

`Step` enum — what a while loop dispatches on:
```rust
pub enum Step {
    UserMessage(String),
    AssistantChunk(StreamChunk),   // streaming token
    AssistantDone(Message),        // full response assembled
    ToolCall(ToolCall),
    SandboxResult(CommandOutput),
    DirectCommand(String),         // ! cmd from user
    ContextWindowFull,
    Stuck { repeated_turns: usize },
    Done,
}
```

The runtime does NOT parse LLM responses or decide what to do with them — that is entirely the while loop body's responsibility. Different agents can interpret the same `AssistantDone` message completely differently (extract Python scripts, extract hick:copy XML, ask the user, etc.).

**Key constraint:** `hick-agent-sdk/Cargo.toml` must not depend on the `hick` crate, `hick-lang`, or any other hickory-specific crate. A `cargo check` on the crate in isolation must succeed without those crates present.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 hick-agent-sdk crate exists in hick-agent/crates/hick-agent-sdk/
- [ ] #2 Cargo.toml has zero dependencies on hick, hick-lang, hick-exec, or any other hickory pipeline crate
- [ ] #3 AgentRuntime struct is pub use'd from lib.rs with full doc comments
- [ ] #4 Step enum is pub use'd from lib.rs with full doc comments
- [ ] #5 LlmClient trait + AnthropicClient + OpenAiCompatClient are pub use'd
- [ ] #6 WasmSandboxExecutor and CommandExecutor are pub use'd
- [ ] #7 NetworkFilter and SinkGuard are pub use'd
- [ ] #8 ProjectContextLoader (formerly context.rs) is pub use'd
- [ ] #9 cargo check --package hick-agent-sdk passes with no warnings
- [ ] #10 cargo test --package hick-agent-sdk passes
<!-- AC:END -->
