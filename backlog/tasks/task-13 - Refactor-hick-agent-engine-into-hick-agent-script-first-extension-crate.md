---
id: TASK-13
title: Refactor hick-agent-engine into hick-agent-script-first extension crate
status: To Do
assignee: []
created_date: '2026-05-15 12:43'
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
- [ ] #1 hick-agent-script-first crate exists at hick-agent/crates/hick-agent-script-first/
- [ ] #2 ReactLoop is reimplemented using AgentRuntime + Step from hick-agent-sdk
- [ ] #3 AgentState enum is preserved (Idle/Thinking/Executing/Observing/Done/Error)
- [ ] #4 SYSTEM_PROMPT lives in this crate, not in the sdk
- [ ] #5 Code block extraction logic lives in this crate
- [ ] #6 hick-agent-engine is either removed from the workspace or gutted to a thin re-export shim (decide during implementation)
- [ ] #7 cargo test --workspace passes
<!-- AC:END -->
