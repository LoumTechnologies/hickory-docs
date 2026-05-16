---
id: TASK-23.4
title: 'Quality scorer: automated acceptance checking for eval runs'
status: To Do
assignee: []
created_date: '2026-05-16 12:13'
labels:
  - evaluation
  - quality-scoring
milestone: m-2
dependencies:
  - TASK-23.1
parent_task_id: TASK-23
priority: high
ordinal: 4000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Build a tool that takes an agent's output workspace and a corpus task directory, runs the task's acceptance criteria, and emits a structured quality score. This is the "did it work?" half of each eval run.

## Dependency

Task format defined in TASK-23.1 (must be done first).

## Acceptance checking pipeline

Each task's `check.sh` is the ground truth, but the scorer also runs a standard pre-check sequence before invoking it:

1. **Compile check** — if the workspace contains a `Cargo.toml`, run `cargo build`. Fail fast if it doesn't compile.
2. **Test check** — if a `Cargo.toml` is present, run `cargo test --test-threads=1`. Record pass/fail and number of tests.
3. **Task-specific check** — run `check.sh <workspace_path>`. Capture stdout, stderr, and exit code.

Each step is recorded independently so partial credit is possible (e.g., "compiled but tests failed").

## Output format

```xml
<quality-result run-id="..." task="fix-off-by-one">
  <check name="compile" passed="true" duration-secs="3.1"/>
  <check name="test" passed="true" tests-run="12" tests-failed="0" duration-secs="8.4"/>
  <check name="acceptance" passed="true" duration-secs="0.3">
    <stdout>All assertions passed</stdout>
  </check>
  <score value="1.0"/>   <!-- 0.0–1.0; partial credit if some checks pass -->
</quality-result>
```

Score formula (adjust as implementation matures): `(checks_passed / checks_total)` weighted by severity (compile failure = 0.0 always, regardless of other checks).

## CLI interface

```
hick-eval score --task eval/corpus/fix-off-by-one \
                --workspace results/<run-id>/workspace \
                --run-id <run-id> \
                --out results/<run-id>/quality.xml
```

The coordinator (TASK-23.5) calls this after each eval runner invocation.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Scorer runs compile, test, and acceptance checks in order and records each independently.
- [ ] #2 A compile failure forces score=0.0 regardless of other check outcomes.
- [ ] #3 Output is a valid XML file matching the schema above.
- [ ] #4 Scorer is hermetic: no network access, no side effects outside the results directory.
- [ ] #5 Partial-credit scoring is demonstrated by a test where compile passes but tests fail, yielding a score between 0 and 1.
<!-- AC:END -->
