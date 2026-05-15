---
id: TASK-13
title: Refactor hick-agent-engine into hick-agent-script-first extension crate
status: Done
assignee:
  - Claude
created_date: '2026-05-15 12:43'
updated_date: '2026-05-15 16:59'
labels: []
milestone: m-0
dependencies:
  - TASK-12
references:
  - hick-agent/crates/hick-agent-engine/src/react_loop.rs
  - hick-agent/crates/hick-agent-engine/src/state.rs
  - hick-agent/crates/hick-agent-engine/src/lib.rs
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Strip `hick-agent-engine` down to only the opinionated, script-first ReAct behavior and rename/repackage it as `hick-agent-script-first`. This is an **extension crate** that sits on top of `hick-agent-sdk` and provides one specific agent strategy: the LLM is prompted to write Python scripts, those scripts are extracted from its response and executed in the WASM sandbox, stdout is fed back as an observation.

**What stays in this crate (moved from hick-agent-engine):**
- `ReactLoop` — the current script-first state machine driver
- `AgentState` enum: `Idle / Thinking / Executing / Observing / Done / Error`
- The 100+ line `SYSTEM_PROMPT` about writing Python scripts (currently hardcoded in `react_loop.rs`)
- Code block extraction logic (parse triple-backtick Python blocks from LLM responses)
- Tool definitions that write scripts to `.hick/tmp/` and execute them
- `PipelineWatcher` — watches for `hick run` output (hick-specific, but lives here as the first concrete user of the extension pattern)

**What is removed from this crate:**
- Everything now in `hick-agent-sdk` (LLM clients, sandbox executor, network filter, etc.)

**New dependency graph:**
```
hick-agent-script-first → hick-agent-sdk
hick-agent-script-first → hick (allowed: this extension IS hick-aware via PipelineWatcher)
```

**AgentRuntime integration:**
`ReactLoop` should be reimplemented as a while loop using `AgentRuntime` + `Step` from the sdk. The state machine logic is identical — just expressed via the sdk's interface rather than ad-hoc. This validates that the sdk's `Step` enum covers all the state transitions the script-first agent needs.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 hick-agent-script-first crate exists at hick-agent/crates/hick-agent-script-first/
- [x] #2 ReactLoop is reimplemented using AgentRuntime + Step from hick-agent-sdk
- [x] #3 AgentState enum is preserved (Idle/Thinking/Executing/Observing/Done/Error)
- [x] #4 SYSTEM_PROMPT lives in this crate, not in the sdk
- [x] #5 Code block extraction logic lives in this crate
- [x] #6 hick-agent-engine is either removed from the workspace or gutted to a thin re-export shim (decide during implementation)
- [x] #7 cargo test --workspace passes
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
## Implementation Plan

### Summary
Create `hick-agent-script-first` as an extension crate on top of `hick-agent-sdk`. Gut `hick-agent-engine` to a thin re-export shim. Update `hick-agent` binary minimally.

### Files to create: `hick-agent/crates/hick-agent-script-first/`
- `Cargo.toml` — deps: hick-agent-sdk, hick-agent-core, hick-policy, hick-flow, hick, tokio, etc.
- `src/lib.rs` — pub use facade
- `src/tools.rs` — copy from engine (CodeBlock, Language, ScriptResult, extract_code_blocks)
- `src/state.rs` — copy from engine (AgentState enum)
- `src/events.rs` — copy from engine (AgentEvent, AgentCommand)
- `src/react_loop.rs` — NEW impl using AgentRuntime + Step (key change)
- `src/pipeline_watcher.rs` — copy from engine
- `src/file_io.rs` — copy from engine (needed by pipeline_watcher)
- `src/reverse_provenance.rs` — copy from engine (needed by pipeline_watcher)
- `src/backing.rs` — copy from engine (needed by pipeline_watcher)

### ReactLoop redesign (AC #2)
- Constructor: `ReactLoop::new(llm: Box<dyn LlmClient>, project_dir: PathBuf) -> Self`
  - Creates `AgentRuntime::new(llm, Box::new(NoopSandbox), project_dir)` internally
- `run(&mut self, user_message, events, interrupt)` — no history param (runtime manages history)
- `run_direct_command(&mut self, cmd, events)` — no history param
- Inner LLM loop uses `runtime.complete_stream()` → processes `Step::AssistantChunk` / `Step::AssistantDone`
- Script execution via `self.executor` (hick-agent-core CommandExecutor) — unchanged behavior
- Context loading via `self.runtime.load_project_context()` instead of engine's context module

### Gut hick-agent-engine (AC #6)
- `lib.rs` becomes a thin shim: re-exports from hick-agent-script-first + hick-agent-sdk
- Keep module files as pass-through re-exports for path-qualified imports (events, network_filter, tools)
- Remove or stub out modules now in sdk (llm, llm_anthropic, llm_openai_compat, network_filter, sink_guard, security_config)
- Remove or stub out modules now in script-first (state, events, tools, react_loop, pipeline_watcher, backing, file_io, reverse_provenance)
- Keep `context.rs` (or remove if sdk covers it)
- Keep `agent.rs` as a stub/remove (no longer needed since ReactLoop::new takes llm directly)

### Update hick-agent binary
- `main.rs`: Remove `Agent::new(...)` / `AgentConfig` usage; pass `llm` directly to `ReactLoop::new(llm, project_dir)`
- `main.rs`: Remove `let mut history = Vec::new()` in `agent_task`; change run/run_direct_command calls (drop `&mut history` param)
- `main.rs`: Make react_loop `mut`
- `tui.rs`: Update module-qualified imports to use top-level re-exports from engine shim

### Add to workspace Cargo.toml
Workspace glob `hick-agent/crates/*` already covers new crate — no Cargo.toml change needed.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Created `hick-agent-script-first` crate containing the opinionated script-first ReAct strategy (ReactLoop, AgentState, tools, events, PipelineWatcher, file_io, backing, reverse_provenance). ReactLoop now takes `Box<dyn LlmClient>` directly and manages history internally via AgentRuntime. Gutted `hick-agent-engine` to a thin re-export shim (kept llm_local/llama.cpp there). Updated `hick-agent` binary: removed Agent/AgentConfig/TokenAuthority, replaced all `config.*` references with bare local variables, cleaned up unused imports (hick-token, rand). All 36 agent tests pass.
<!-- SECTION:FINAL_SUMMARY:END -->
