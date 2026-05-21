---
id: TASK-23.3
title: 'Eval runner: run one agent configuration against one task and emit XML result'
status: To Do
assignee: []
created_date: '2026-05-16 12:13'
updated_date: '2026-05-21 01:49'
labels:
  - evaluation
  - runner
milestone: m-2
dependencies:
  - TASK-23.1
  - TASK-23.2
  - TASK-25
  - TASK-26
parent_task_id: TASK-23
priority: high
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Build the primitive eval unit: a CLI tool that runs a single agent configuration on a single benchmark task, captures all observable outcomes, and writes a structured XML result file.

## Dependencies

- Task format defined in TASK-23.1 (must be done first)
- Cost model crate from TASK-23.2

## Agent configuration vs compiled binary

Because extension combinations will explode combinatorially, the runner must support **declarative configs** in addition to compiled binaries:

```toml
# eval/configs/sonnet-script-hick-edit.toml
[agent]
binary = "copper-ledge"          # compiled binary name, OR:
# extensions = ["script-first", "hick-edit"]  # dynamic wiring (future)
model = "claude-sonnet-4-6"
extended_thinking = false

[runtime]
max_turns = 20
timeout_secs = 300
```

The runner invokes the binary with the task's `prompt.md` content as input, sets up the `seed/` directory as the workspace, enforces the wall-clock timeout, and captures:
- stdout/stderr logs
- workspace diff (files added/modified/deleted)
- token usage from agent logs (the SDK logs usage per turn in XML)

## Output format

One XML result file per run, e.g. `results/<run-id>/result.xml`:

```xml
<eval-result run-id="..." task="fix-off-by-one" config="sonnet-script-hick-edit" timestamp="...">
  <agent binary="copper-ledge" model="claude-sonnet-4-6" extended-thinking="false"/>
  <timing wall-secs="47.2" turns="8"/>
  <token-usage cache-write-input="1200" cache-read-input="8400"
               uncached-input="340" output="2100" thinking="0"/>
  <cost-breakdown cache-write-usd="0.0180" cache-read-usd="0.0042"
                  uncached-input-usd="0.0034" output-usd="0.0630" total-usd="0.0886"/>
  <workspace-diff>...</workspace-diff>
  <exit-status>0</exit-status>
</eval-result>
```

## CLI interface

```
hick-eval run --config eval/configs/sonnet-script-hick-edit.toml \
              --task eval/corpus/fix-off-by-one \
              --out results/
```
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 CLI runs an agent binary against a corpus task directory and writes a valid XML result file.
- [ ] #2 Token usage is captured from agent SDK logs (not estimated); the XML matches actual API response metadata.
- [ ] #3 Cost breakdown in the result uses the cost model crate (TASK-23.2).
- [ ] #4 Wall-clock timeout is enforced; a timed-out run records exit-status='timeout' in the XML.
- [ ] #5 Agent configuration is read from a TOML file, not hard-coded — swapping configs requires no recompilation.
- [ ] #6 Result XML is well-formed and contains all fields shown in the spec above.
- [ ] #7 Result XML <timing> element includes a rework-count attribute counting tool calls that reversed a prior action in the same run; the field is 0 (not absent) when no rework occurred.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
## Rework / backtracking tracking

The timing block must include a `rework-count` attribute: the number of tool calls that explicitly reversed a prior action (e.g. file overwrites that undo a previous write, reverted edits, repeated identical calls after a failure). This is distinct from turn count — a model that makes 6 turns cleanly is categorically different from one that makes 6 turns including 2 backtracks.

Updated `<timing>` spec:

```xml
<timing wall-secs="47.2" turns="8" rework-count="2"/>
```

The runner infers `rework-count` from the agent's tool call log: any write to a path that was already written in the same run, or any edit that touches lines already edited, increments the counter. This is a conservative lower bound — the judge may surface additional backtracking the log does not make explicit.
<!-- SECTION:NOTES:END -->
