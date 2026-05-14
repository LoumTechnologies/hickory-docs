---
id: TASK-11
title: Agent session recording as replayable .hick files
status: To Do
assignee: []
created_date: '2026-05-14 17:12'
labels: []
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The AI agent currently writes a plain-text log to `.hick/agent-YYYYMMDD-HHMMSS.log`. Replace this with a `.hick` session file that records the full conversation AND is replayable: running `hick run session.hick` re-executes all side-effecting steps (Python scripts embedded in LLM responses, direct `!` commands) without calling the LLM, producing the same output files.

**Background — the agent's ReAct loop:**
The agent works in turns: user message → LLM responds with a Python script → script runs in a WASM container → stdout observed → repeat. The LLM response IS the executable content. During replay, we skip the "call the LLM" step and use the stored response instead, then execute its embedded script.

**New `.hick` node types needed:**

| Node | Content | During replay |
|------|---------|---------------|
| `hick:user` | User message text | Inert (context only) |
| `hick:assistant` | LLM response, contains embedded action/script blocks | Execute the embedded scripts |
| `hick:observation` | stdout/stderr from a script execution | Inert (already captured) |
| `hick:command` | A `!`-prefixed direct shell command typed by the user | Execute |

**`!` command mode in the TUI:**
Users can prefix input with `!` to run a shell command directly in the WASM container, bypassing the LLM. The output is shown inline and added to the session file as a `hick:command` + `hick:observation` pair. The LLM is NOT called afterward (silent by default). This is analogous to the `! <command>` shortcut in Claude Code.

**Two supported flows:**
1. Normal repo: agent writes code, runs tests, edits files — session captures everything
2. Hick-file repo: agent modifies `.hick` files, runs `hick run`, tests output — session is itself a `.hick` file in the same format as the source
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The .hick format parser (hick-lang) accepts hick:user, hick:assistant, hick:observation, and hick:command nodes without error
- [ ] #2 The agent writes a .hick session file to .hick/agent-YYYYMMDD-HHMMSS.hick during each run, replacing the current .log format
- [ ] #3 The session file contains all turns: user messages, full LLM responses (including embedded scripts), command outputs
- [ ] #4 Running `hick run <session.hick>` re-executes the session: embedded scripts in hick:assistant nodes and hick:command nodes are executed, hick:user and hick:observation nodes are skipped
- [ ] #5 Replay produces the same output files as the original session (verified by a test with a known session fixture)
- [ ] #6 Typing `! <cmd>` in the agent TUI runs the command in the WASM container immediately without calling the LLM
- [ ] #7 `!` command stdout is displayed inline in the TUI and appended to the session file as hick:command + hick:observation
- [ ] #8 The LLM is not invoked after a `!` command (silent mode)
<!-- AC:END -->
