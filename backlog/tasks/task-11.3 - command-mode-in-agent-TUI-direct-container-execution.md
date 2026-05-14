---
id: TASK-11.3
title: '! command mode in agent TUI: direct container execution'
status: Done
assignee: []
created_date: '2026-05-14 17:13'
updated_date: '2026-05-14 19:55'
labels: []
dependencies:
  - TASK-11.1
  - TASK-11.2
parent_task_id: TASK-11
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a `!`-prefix command mode to the agent TUI. When the user types `! <cmd>` in the input field, the command runs directly in the WASM container without calling the LLM. The output is shown inline in the TUI and recorded in the session file. The LLM is not invoked afterward (silent by default).

**What TASK-11.1 provides:** The `hick:command` and `hick:observation` node types.
**What TASK-11.2 provides:** The session writer that appends nodes to the `.hick` session file.

**Behavior:**
- User types `! ls src/` and presses Enter
- Command runs immediately in the WASM sandbox (using the existing `CommandExecutor`)
- stdout/stderr displayed inline in the TUI conversation view
- A `hick:command` node + `hick:observation` node appended to the session file
- Cursor returns to input — no LLM call, no waiting
- The output becomes part of conversation history (so a subsequent LLM message can reference what was seen)

**Analogy:** This is the same as `! <command>` in Claude Code, which runs a shell command and adds output to context without triggering a model response.

**Key files:**
- `hick-agent/crates/hick-agent/src/tui/` — input handling, event dispatch
- `hick-agent/crates/hick-agent-engine/` — AgentCommand enum, ReactLoop
- The existing `WasmSandboxExecutor` / `CommandExecutor` trait handles the actual container execution
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Typing `! <cmd>` in the TUI input and pressing Enter runs the command in the WASM container without calling the LLM
- [x] #2 Command stdout/stderr is displayed inline in the TUI conversation view
- [x] #3 A hick:command node and hick:observation node are appended to the session file
- [x] #4 No LLM call is made after a ! command (verified by checking no AgentEvent::AssistantChunk is emitted)
- [x] #5 A subsequent user message to the LLM includes the ! command output in its context
- [x] #6 Typing `!` alone (no command) shows a usage hint rather than erroring
- [x] #7 Normal (non-! prefixed) input continues to work as before
<!-- AC:END -->
