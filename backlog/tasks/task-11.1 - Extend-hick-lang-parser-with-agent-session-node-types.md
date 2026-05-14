---
id: TASK-11.1
title: Extend hick-lang parser with agent session node types
status: Done
assignee: []
created_date: '2026-05-14 17:13'
updated_date: '2026-05-14 19:49'
labels: []
dependencies: []
parent_task_id: TASK-11
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add four new XML node types to the `.hick` format parser in `hick-lang`. These nodes represent turns in an AI agent session and enable the session file to be both a readable transcript and a replayable program.

**New node types:**

| Node | Purpose | Replay behavior |
|------|---------|----------------|
| `hick:user` | User message text | Inert — loaded as context, not executed |
| `hick:assistant` | Full LLM response; may contain embedded `hick:action` script blocks | Execute embedded scripts, skip the LLM call |
| `hick:observation` | Captured stdout/stderr from a prior script or command execution | Inert — already recorded, not re-executed |
| `hick:command` | A direct shell command typed by the user with `!` prefix | Execute in WASM container |

**Where to look:**
- Parser: `hick-lang/crates/hick-lang/` — the XML-based `.hick` format parser
- Existing node types live there as the reference for how to add new ones
- The graph tool `query_graph pattern=file_summary target=hick-lang/crates/hick-lang/src/lib.rs` gives an overview of the parser structure

**Design notes:**
- `hick:assistant` nodes contain free text (the LLM's prose) plus zero or more `hick:action` child elements that hold Python scripts to execute
- `hick:observation` carries a `source` attribute referencing which action produced it (optional but useful for replay verification)
- These nodes should be gracefully ignored when processing a normal pipeline `.hick` file (i.e., no error if a `hick:user` appears in a file that also has pipeline nodes)
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 hick-lang parses hick:user, hick:assistant, hick:observation, and hick:command nodes without error
- [x] #2 hick:assistant nodes can contain hick:action child elements with embedded Python scripts
- [x] #3 A .hick file mixing session nodes and pipeline nodes (hick:exec, hick:file, etc.) parses without error
- [x] #4 The parsed AST exposes the new node types in a way that hick run and hick-agent can pattern-match on them
- [x] #5 Unit tests cover round-trip parse of each new node type
- [x] #6 Existing parser tests all pass
<!-- AC:END -->
