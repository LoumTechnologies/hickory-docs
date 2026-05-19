---
id: TASK-41
title: Implement agent memory bank via .agents/memory/
status: To Do
assignee: []
created_date: '2026-05-19 17:28'
labels:
  - agent-standards
  - memory
dependencies: []
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Support a persistent memory bank where the agent can read and write notes about the project across sessions. Files live under `.agents/memory/` at the project root and are agent-maintained — the agent updates them as it learns things about the project, similar to Cline's `cline_docs/` pattern.

**Convention:**
- Memory files live in `.agents/memory/*.md`
- The agent reads all memory files at startup and includes them in context
- The agent can create, update, and append to memory files during a session
- Memory is project-scoped (committed to the repo) — distinct from user-scoped memory

**Scope:**
- Load `.agents/memory/*.md` in `ProjectContextLoader` alongside other context files
- Expose a `write_memory` / `update_memory` capability in the runtime so the agent can persist notes
- Memory writes should be atomic (write to temp file, rename) to avoid corruption
- Include a brief summary of loaded memory files in the system prompt context section

**Suggested initial memory files the agent may create:**
- `project.md` — high-level project description, tech stack, conventions
- `decisions.md` — architectural decisions and their rationale
- `progress.md` — current work state, what's done, what's next

**References:**
- `hick-agent/crates/hick-agent-sdk/src/project_context.rs` — context loader
- `hick-agent/crates/hick-agent-sdk/src/runtime.rs` — runtime loop
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Memory files in .agents/memory/*.md are loaded at startup and included in system prompt context
- [ ] #2 Agent can write new memory files and update existing ones during a session
- [ ] #3 Memory writes are atomic (no partial writes on crash)
- [ ] #4 Absence of .agents/memory/ directory is handled gracefully (no error)
- [ ] #5 Tests cover: loading existing memories, writing a new memory file, updating existing memory, missing directory no-op
<!-- AC:END -->
