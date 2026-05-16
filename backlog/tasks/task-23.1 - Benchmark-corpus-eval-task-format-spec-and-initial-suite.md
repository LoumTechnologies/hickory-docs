---
id: TASK-23.1
title: 'Benchmark corpus: eval task format spec and initial suite'
status: To Do
assignee: []
created_date: '2026-05-16 12:12'
labels:
  - evaluation
  - corpus
milestone: m-2
dependencies: []
parent_task_id: TASK-23
priority: high
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Define the on-disk format for evaluation tasks and create an initial suite that covers representative coding scenarios. Every other eval subtask depends on this format being stable.

## Format requirements

Each eval task is a self-contained directory under `eval/corpus/<task-slug>/`:

```
eval/corpus/fix-off-by-one/
  task.toml          # metadata: title, description, model hints, tags
  prompt.md          # the coding prompt given to the agent verbatim
  seed/              # files placed in the workspace before the agent runs
  expected/          # (optional) reference output for diffing
  check.sh           # acceptance script: exits 0 = pass, non-zero = fail
```

`task.toml` fields at minimum:
- `title`, `description`, `tags` (e.g. ["bug-fix", "rust", "algorithmic"])
- `difficulty` (easy/medium/hard) — used by the coordinator for stratified sampling
- `timeout_secs` — wall-clock budget for the agent

`check.sh` receives the workspace path as `$1`. It must be hermetic (no network, no side effects). Typical checks: `cargo build`, `cargo test`, custom assertions.

## Initial suite

Create at least 8 tasks spanning:
- Bug fix (2): straightforward logic error, subtle off-by-one
- Feature addition (2): add a function, add a CLI flag
- Refactor (2): extract a function, rename + move a module
- Test writing (2): write unit tests for an existing function

## Combinatorial explosion note

Tasks should have `tags` and `difficulty` so the coordinator (TASK-23.5) can build a stratified sample instead of running the full corpus against every combination.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A `task.toml` schema (as a Rust struct or TOML schema doc) is defined and checked into `eval/`.
- [ ] #2 At least 8 eval tasks exist under `eval/corpus/`, covering bug-fix, feature-addition, refactor, and test-writing categories.
- [ ] #3 Every task has a `check.sh` that exits 0 on a correct solution and non-zero on the unmodified seed.
- [ ] #4 The format is documented in `eval/README.md` with enough detail for TASK-23.3 and TASK-23.4 to build against it without further clarification.
<!-- AC:END -->
