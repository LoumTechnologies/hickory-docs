---
id: TASK-11.4
title: 'Session replay: hick run re-executes a .hick session file without LLM'
status: In Progress
assignee: []
created_date: '2026-05-14 17:14'
updated_date: '2026-05-14 19:55'
labels: []
dependencies:
  - TASK-11.1
  - TASK-11.2
parent_task_id: TASK-11
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Enable `hick run <session.hick>` to replay a recorded agent session deterministically: re-execute all side-effecting steps (Python scripts embedded in `hick:assistant` nodes, `hick:command` nodes) without making any LLM API calls, producing the same output files as the original session.

**What TASK-11.1 provides:** Parser support for session node types.
**What TASK-11.2 provides:** Real session files written by the agent to replay against.

**Replay semantics — walk the session file top to bottom:**
- `hick:user` → skip (inert context)
- `hick:assistant` → extract and execute each embedded `hick:action` script, exactly as the agent would have
- `hick:observation` → skip (output already captured; shown for context if verbose)
- `hick:command` → execute in the WASM container

**How `hick run` detects a session file:**
Either by the presence of a `hick:session` root element, or by encountering session nodes during parse. The existing pipeline execution path should not be affected when no session nodes are present.

**Key files:**
- `crates/hick/src/main.rs` — `run_single_shot()` is where to branch into replay mode
- `hick-agent/crates/hick-agent-engine/` — the script execution logic to reuse for replay
- `hick-lang/crates/hick-lang/` — the parser (extended in TASK-11.1)

**Goal:** A session file committed to git serves as a reproducible build artifact. `hick run session.hick` is cheap (no LLM), fast, and deterministic.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 hick run <session.hick> detects the session format and enters replay mode
- [ ] #2 hick:assistant embedded scripts are executed in the WASM container during replay
- [ ] #3 hick:command nodes are executed in the WASM container during replay
- [ ] #4 hick:user and hick:observation nodes are skipped without error
- [ ] #5 No LLM API calls are made during replay
- [ ] #6 Replay produces the same output files as the original session (verified by a test fixture: record a session, replay it, diff the outputs)
- [ ] #7 Normal pipeline .hick files (no session nodes) are unaffected — hick run still works as before
- [ ] #8 hick run --verbose during replay prints each node being executed or skipped
<!-- AC:END -->
