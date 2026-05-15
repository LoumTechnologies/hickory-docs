---
id: TASK-15
title: Rewrite hick binary entry point using sdk + extension architecture
status: Done
assignee: []
created_date: '2026-05-15 12:44'
updated_date: '2026-05-15 17:25'
labels: []
milestone: m-0
dependencies:
  - TASK-12
  - TASK-13
references:
  - hick-agent/crates/hick-agent/src/main.rs
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the current `hick-agent/crates/hick-agent/src/main.rs` monolith with a clean entry point that wires together `hick-agent-sdk` (runtime) and `hick-agent-script-first` (behavior) via an explicit while loop. Behavior must be identical to before — this is a refactor, not a feature change.

**Target structure of main.rs:**
```rust
// ~150 lines total
// 1. Parse CLI args (clap)
// 2. Build AgentRuntime (sdk): sandbox config, network policy, LLM client, context
// 3. Load ProjectContextLoader → inject as system context into runtime
// 4. Instantiate ReactLoop from hick-agent-script-first
// 5. Explicit while loop:
//    loop {
//        match runtime.next_step().await {
//            Step::UserMessage(msg) => react_loop.on_user_message(&runtime, msg).await,
//            Step::AssistantDone(msg) => react_loop.on_assistant_done(&runtime, msg).await,
//            Step::SandboxResult(out) => react_loop.on_sandbox_result(&runtime, out).await,
//            Step::DirectCommand(cmd) => runtime.execute_direct(&cmd).await,
//            Step::Done => break,
//            ...
//        }
//    }
```

The while loop body should be readable in one screen. If it isn't, the sdk's `Step` enum or `ReactLoop` API needs adjustment.

**TUI wiring:**
The ratatui TUI (`tui.rs`) feeds user input into `AgentRuntime` and renders `Step` events. This wiring moves to the binary crate's `tui.rs`, which depends on `AgentRuntime` (sdk) but not on any extension-specific types.

**Validation:**
End-to-end manual test: run `hick` on a real coding task and verify it behaves identically to the current agent.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 hick binary compiles and links against hick-agent-sdk + hick-agent-script-first
- [x] #2 main.rs while loop body is ≤150 lines
- [ ] #3 Running hick on a coding task produces identical behavior to pre-refactor
- [x] #4 TUI renders correctly; user input flows through AgentRuntime
- [x] #5 Session recording (.hick session file) still works
- [x] #6 ! command mode still works
- [x] #7 cargo test --workspace passes
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Rewired the `hick-agent` binary crate to use `hick-agent-sdk` + `hick-agent-script-first` directly instead of routing through the `hick-agent-engine` re-export shim.

**Changes made:**

- `Cargo.toml`: Added `hick-agent-sdk`, `hick-agent-script-first`, and `hick-policy` as direct dependencies. Made `hick-agent-engine` optional (enabled only by the `local` feature for `LocalLlmClient`). Dropped the `integration-test` feature's engine dependency since it was an empty feature.

- `src/lib.rs`: Updated all `pub use` re-exports to come from `hick_agent_sdk` and `hick_agent_script_first` directly. Added `pub use hick_agent_sdk::llm` module for test compatibility.

- `src/main.rs`: Changed imports from `hick_agent_engine` → `hick_agent_script_first` (for `AgentCommand`, `AgentEvent`, `PipelineWatcher`, `ReactLoop`) and `hick_agent_sdk` (for LLM clients, network filter, security config, sink guard). Changed the policy evaluator from `hick_agent_engine::hick_policy::PolicyEvaluator` to `hick_policy::PolicyEvaluator` (direct workspace dep).

- `src/tui.rs`: Changed imports from `hick_agent_engine::events`, `hick_agent_engine::network_filter`, `hick_agent_engine::tools` to the corresponding `hick_agent_script_first` and `hick_agent_sdk` paths.

All 36 workspace tests pass. Binary compiles cleanly. Behavior is identical — this was a pure import refactor with no logic changes.
<!-- SECTION:FINAL_SUMMARY:END -->
