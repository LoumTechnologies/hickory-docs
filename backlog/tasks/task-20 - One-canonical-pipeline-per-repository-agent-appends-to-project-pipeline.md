---
id: TASK-20
title: 'One canonical pipeline per repository: agent appends to project pipeline'
status: To Do
assignee: []
created_date: '2026-05-15 12:46'
labels: []
milestone: m-1
dependencies:
  - TASK-18
  - TASK-19
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Establish the convention that each repository has one canonical growing pipeline (anchored at `_hick.yml`) and the agent appends to it over time rather than creating ad-hoc files. The pipeline is the repository's declarative history — a running record of every file the agent has ever created or modified.

**Convention:**

- `_hick.yml` is the pipeline root; it lists `.hick` files in order
- Each agent session that produces output appends a new `.hick` file (or appends elements to an existing one via `hick promote`)
- The pipeline grows monotonically; entries are never removed (only superseded by later `hick:copy` replacements or `hick:file` take-overs)
- `hick run` (no args) runs the full pipeline from `_hick.yml`

**Agent behavior change:**

When starting a task:
1. Load the current `_hick.yml` to understand what the pipeline currently produces
2. Use `PasteSlotRegistry` (TASK-17) to discover all named slots in all pipeline-owned files
3. Use this as context when deciding how to make edits

When finishing a task:
1. Either: directly append hick:copy/hick:file elements during the session (TASK-18)
2. Or: run `hick promote` on the session file afterward (TASK-19)

**`_hick.yml` auto-initialization:**
If `_hick.yml` does not exist, `hick init` (or the agent on first run) creates a minimal one. If it does exist, the agent reads it before starting and appends to it on completion.

**Tooling:**
- `hick pipeline show` — print the full pipeline DAG with file ownership
- `hick pipeline status` — show which output files are pipeline-owned vs untracked
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Agent reads _hick.yml on startup and uses it as pipeline context
- [ ] #2 Agent appends new hick:copy / hick:file elements to the pipeline after completing a task
- [ ] #3 _hick.yml is auto-created by hick init if it does not exist
- [ ] #4 hick pipeline show prints the pipeline DAG with file ownership
- [ ] #5 hick pipeline status distinguishes pipeline-owned files from untracked files
- [ ] #6 Running hick run after several agent sessions reproduces all files from all sessions
- [ ] #7 cargo test --workspace passes
<!-- AC:END -->
