---
id: TASK-38
title: Implement .agentignore support
status: To Do
assignee: []
created_date: '2026-05-19 17:28'
labels:
  - agent-standards
  - security
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Support the cross-agent `.agentignore` standard for controlling which files and directories the agent is allowed to read or act on.

When loading project context or responding to file-read/edit requests, the agent should check for a `.agentignore` file at the project root and skip any paths that match. This prevents agents from reading secrets (`.env`), large binaries, generated output, or any other files the project owner wants excluded.

**Scope:**
- Parse `.agentignore` using gitignore-style glob patterns
- Apply the filter in `ProjectContextLoader` when scanning context files/dirs
- Apply the filter in any file-read tool paths (tool gate layer)
- Treat absence of `.agentignore` as no restrictions (current behaviour preserved)

**References:**
- `hick-agent/crates/hick-agent-sdk/src/project_context.rs` — context loader
- `hick-agent/crates/hick-agent-sdk/src/tool_gate.rs` — tool access control
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Files matching .agentignore patterns are not included in project context
- [ ] #2 File-read tool calls are blocked for paths matching .agentignore
- [ ] #3 Absence of .agentignore imposes no restrictions
- [ ] #4 gitignore-style glob patterns are supported (wildcards, negation, directory patterns)
- [ ] #5 Tests cover: secret files excluded, generated dirs excluded, negation patterns
<!-- AC:END -->
