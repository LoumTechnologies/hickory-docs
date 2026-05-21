---
id: TASK-23.6
title: 'Results reporting: per-extension attribution, cost-quality Pareto analysis'
status: To Do
assignee: []
created_date: '2026-05-16 12:13'
updated_date: '2026-05-21 01:49'
labels:
  - evaluation
  - reporting
  - analysis
milestone: m-2
dependencies:
  - TASK-23.5
parent_task_id: TASK-23
priority: medium
ordinal: 6000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Build the reporting tool that reads a completed eval run set and produces human-readable summaries for comparing agent configurations, attributing quality/cost deltas to individual extensions, and identifying Pareto-optimal combinations.

## Dependency

Requires a completed results directory from TASK-23.5 (eval coordinator). Must be able to parse the XML result and quality files from TASK-23.3 and TASK-23.4.

## Reports to generate

### 1. Per-configuration summary table (terminal + markdown)
| Config | Avg Quality | Avg Cost (USD) | Avg Turns | Runs |
|---|---|---|---|---|
| baseline | 0.62 | $0.041 | 6.2 | 8 |
| +script-first | 0.81 | $0.089 | 9.4 | 8 |
| +hick-edit | 0.74 | $0.063 | 7.1 | 8 |

### 2. Per-extension attribution (OFAT delta table)
| Extension | ΔQuality | ΔCost | Quality/Cost ratio |
|---|---|---|---|
| script-first | +0.19 | +$0.048 | 3.96 |
| hick-edit | +0.12 | +$0.022 | 5.45 |

Higher ratio = more quality per dollar. This table directly answers "which extension is worth adding next?"

### 3. Pareto frontier (for exhaustive or fractional runs)
List configurations that are not dominated (no other config is both cheaper AND higher quality). Highlight these in the summary table.

### 4. Per-task breakdown (drill-down)
For a given task slug, show all configs side-by-side with quality score and cost.

## CLI interface

```
just eval-report                 # summary from latest results/
hick-eval report --results results/<run-set-id>/ --format markdown
hick-eval report --results results/<run-set-id>/ --task fix-off-by-one
```

Markdown output is written to `results/<run-set-id>/report.md`. Terminal output renders as a table.

## Combinatorial note

The reporter must handle OFAT, fractional-factorial, and exhaustive result sets — the analysis method is inferred from the plan.xml written by the coordinator.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Reporter reads result and quality XML files from a results directory and produces a summary table without manual data entry.
- [ ] #2 Per-extension attribution table (ΔQuality, ΔCost, ratio) is produced for any OFAT run set.
- [ ] #3 Pareto-optimal configurations are identified and flagged when the result set contains enough combinations.
- [ ] #4 Markdown report is written to results/<run-set-id>/report.md.
- [ ] #5 `just eval-report` runs the reporter against the most recent results directory.
- [ ] #6 Reporter handles missing or timed-out runs gracefully (marks them N/A rather than crashing).
- [ ] #7 Summary table includes Avg Time-to-Done (wall seconds) and Avg Rework columns populated from result XML.
- [ ] #8 A cost-vs-time Pareto frontier is produced alongside the cost-vs-quality Pareto, with rework rate annotated on each config.
- [ ] #9 When a human rating UI consumes the report, wall time and rework count are displayed as non-editable ground-truth fields.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
## Speed metrics and time-to-done

The summary table must include Avg Time-to-Done (wall seconds) alongside cost and quality. Turn count alone is misleading: a fast model with 15 turns can complete before a slow model with 6 turns, and a model that backtracks inflates both turn count and wall time.

Updated summary table spec:

| Config | Avg Quality | Avg Cost (USD) | Avg Time (s) | Avg Turns | Avg Rework | Runs |
|---|---|---|---|---|---|---|
| baseline | 0.62 | $0.041 | 61 | 6.2 | 0.4 | 8 |
| +script-first | 0.81 | $0.089 | 94 | 9.4 | 1.1 | 8 |

## Cost-vs-time Pareto frontier

In addition to the cost-vs-quality Pareto, produce a **cost-vs-time** Pareto: configurations not dominated on both cost and wall time. This surfaces the "cheap but slow" vs "expensive but fast" tradeoff directly. A config that is cheaper AND faster than another is strictly better on efficiency grounds regardless of quality.

Label each config on the Pareto charts with its rework rate so humans can see whether speed is coming from clean execution or from brute-force retries.

## Human-as-judge surface

When the report is consumed by a human rating UI, wall time and rework count MUST be shown as non-editable ground-truth facts alongside the pre-populated judge scores. The human should see: 'This run took 94s with 3 reworks' before they decide whether to override a judge score.
<!-- SECTION:NOTES:END -->
