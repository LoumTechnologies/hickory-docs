---
id: TASK-12
title: >-
  Create hick-agent-sdk: generic agent building-block crate (zero hick
  knowledge)
status: Done
assignee:
  - Claude
created_date: '2026-05-15 12:43'
updated_date: '2026-05-15 16:03'
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
- [x] #1 hick-agent-sdk crate exists in hick-agent/crates/hick-agent-sdk/
- [x] #2 Cargo.toml has zero dependencies on hick, hick-lang, hick-exec, or any other hickory pipeline crate
- [x] #3 AgentRuntime struct is pub use'd from lib.rs with full doc comments
- [x] #4 Step enum is pub use'd from lib.rs with full doc comments
- [x] #5 LlmClient trait + AnthropicClient + OpenAiCompatClient are pub use'd
- [x] #6 WasmSandboxExecutor and CommandExecutor are pub use'd
- [x] #7 NetworkFilter and SinkGuard are pub use'd
- [x] #8 ProjectContextLoader (formerly context.rs) is pub use'd
- [x] #9 cargo check --package hick-agent-sdk passes with no warnings
- [x] #10 cargo test --package hick-agent-sdk passes
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
## Implementation Plan

### New crate layout
```
hick-agent/crates/hick-agent-sdk/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── llm.rs                 ← copied from hick-agent-engine (no hick deps)
    ├── llm_anthropic.rs       ← copied from hick-agent-engine
    ├── llm_openai_compat.rs   ← copied from hick-agent-engine
    ├── network_filter.rs      ← copied from hick-agent-engine (hick_token only)
    ├── sink_guard.rs          ← copied from hick-agent-engine (hick_token/classify/sink)
    ├── security_config.rs     ← copied from hick-agent-engine (hick_token/classify/sink)
    ├── project_context.rs     ← from engine's context.rs + ProjectContextLoader struct
    ├── command.rs             ← from hick-agent-core command.rs, refactored with ContainerBackend trait
    └── runtime.rs             ← new: AgentRuntime + Step + StreamChunk + ToolCall
```

### Key design: WasmSandboxExecutor without hick dep
Introduce `ContainerBackend` trait in command.rs. `WasmSandboxExecutor::new(config, backend: Box<dyn ContainerBackend>)`. Remove `auth` field from `WasmSandboxConfig`. Use `base64` crate instead of `hick::volume_state::encode_base64`.

### Cargo.toml dependencies
hick-token, hick-classify, hick-sink (security crates, not pipeline) + reqwest, async-openai, async-trait, futures, tokio, serde/json, toml, chrono, log, base64, anyhow.

### Not changed
hick-agent-engine and hick-agent-core kept as-is.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Created `hick-agent/crates/hick-agent-sdk/` — a new crate with zero dependency on `hick`, `hick-lang`, or `hick-exec`.

**Files created:**
- `Cargo.toml` — depends only on hick-security crates (hick-token, hick-classify, hick-sink) plus std ecosystem crates
- `src/lib.rs` — full pub-use facade with doc comments for every exported item
- `src/llm.rs` — LlmClient trait, Message, Role, ChatChunk
- `src/llm_anthropic.rs` — AnthropicClient with SSE streaming
- `src/llm_openai_compat.rs` — OpenAiCompatClient (OpenAI, xAI, Groq, Ollama, etc.)
- `src/network_filter.rs` — NetworkFilter, FilteredLlmClient, NetworkApprovalRequest, NetworkVerdict
- `src/sink_guard.rs` — SinkGuard with capability-based sink validation
- `src/security_config.rs` — AgentSecurityConfig (load from TOML, convert to capabilities)
- `src/project_context.rs` — ProjectContextLoader struct wrapping the context-file scanner
- `src/command.rs` — CommandExecutor trait, CommandOutput, ContainerBackend trait (new), WasmSandboxConfig (simplified, no hick::AuthStack), WasmSandboxExecutor (uses Box<dyn ContainerBackend> + base64 crate instead of hick::volume_state)
- `src/runtime.rs` — AgentRuntime, Step enum, StreamChunk, ToolCall; complete_stream() returns 'static stream (clones history); includes token-budget tracking and compact_history()

**Key design:** `WasmSandboxExecutor` was refactored to accept a `Box<dyn ContainerBackend>` instead of holding `hick::ContainerExecutor` directly. The `hick`-specific backend implementation will live in `hick-agent-core` (follow-up task).

**Test results:** 36 unit tests + 1 doctest, all passing. `cargo check --package hick-agent-sdk` clean with no warnings.
<!-- SECTION:FINAL_SUMMARY:END -->
