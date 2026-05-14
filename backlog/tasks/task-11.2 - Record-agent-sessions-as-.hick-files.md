---
id: TASK-11.2
title: Record agent sessions as .hick files
status: Done
assignee: []
created_date: '2026-05-14 17:13'
updated_date: '2026-05-14 19:53'
labels: []
dependencies:
  - TASK-11.1
parent_task_id: TASK-11
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the current plain-text agent log (`.hick/agent-YYYYMMDD-HHMMSS.log`) with a `.hick` session file that records every turn of the agent conversation in the new session node format defined in TASK-11.1.

**What TASK-11.1 provides:** Parser support for `hick:user`, `hick:assistant`, `hick:observation`, and `hick:command` node types.

**What this task does:** Wire the agent's event stream into a session writer that appends to a `.hick` file in real time as each turn completes.

**Agent event flow (from hick-agent-engine):**
- `AgentEvent` enum carries: user message, LLM response chunks, script execution, stdout observation, errors
- The session writer listens on the event channel (or a tap of it) and serializes each event as the appropriate `.hick` node
- File is written incrementally — each turn appended as it completes, so a crashed session is still partially recoverable

**Key files:**
- `hick-agent/crates/hick-agent-engine/` — ReactLoop, AgentEvent, AgentCommand
- `hick-agent/crates/hick-agent/src/main.rs` — where the log path is currently determined (`agent-{ts}.log`)
- Session file goes to `.hick/agent-YYYYMMDD-HHMMSS.hick`

**Format sketch:**
```xml
<hick:session xmlns:hick="...">
  <hick:user>add a login page</hick:user>
  <hick:assistant>
    I'll create the login page now.
    <hick:action lang="python">
      # ... python script ...
    </hick:action>
  </hick:assistant>
  <hick:observation source="action-1">Files created: src/login.py</hick:observation>
</hick:session>
```
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The agent writes a .hick session file to .hick/agent-YYYYMMDD-HHMMSS.hick for every run
- [x] #2 The old .log format is no longer written (or is written alongside as a transitional measure, clearly marked deprecated)
- [x] #3 Each user message is appended as a hick:user node when received
- [x] #4 Each LLM response is appended as a hick:assistant node (with embedded hick:action blocks) when the response completes
- [x] #5 Each script execution output is appended as a hick:observation node
- [x] #6 The session file is valid XML parseable by hick-lang after each complete turn
- [x] #7 If the agent crashes mid-session, the file up to the last complete turn is valid and parseable
<!-- AC:END -->
