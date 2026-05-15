---
id: TASK-15
title: Rewrite hick binary entry point using sdk + extension architecture
status: To Do
assignee: []
created_date: '2026-05-15 12:44'
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
- [ ] #1 hick binary compiles and links against hick-agent-sdk + hick-agent-script-first
- [ ] #2 main.rs while loop body is ≤150 lines
- [ ] #3 Running hick on a coding task produces identical behavior to pre-refactor
- [ ] #4 TUI renders correctly; user input flows through AgentRuntime
- [ ] #5 Session recording (.hick session file) still works
- [ ] #6 ! command mode still works
- [ ] #7 cargo test --workspace passes
<!-- AC:END -->
