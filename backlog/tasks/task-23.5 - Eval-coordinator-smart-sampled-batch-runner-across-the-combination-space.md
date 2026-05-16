---
id: TASK-23.5
title: 'Eval coordinator: smart-sampled batch runner across the combination space'
status: To Do
assignee: []
created_date: '2026-05-16 12:13'
labels:
  - evaluation
  - coordinator
  - sampling
milestone: m-2
dependencies:
  - TASK-23.3
  - TASK-23.4
parent_task_id: TASK-23
priority: high
ordinal: 5000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Build the orchestration layer that runs the eval runner and quality scorer across many agent configurations and corpus tasks, using a sampling strategy that remains useful as the number of extensions grows.

## Dependencies

- TASK-23.3 (eval runner)
- TASK-23.4 (quality scorer)

## The combinatorial explosion problem

With n extension crates, the number of possible combinations is 2^n. At n=5 that's 32; at n=10 it's 1024. Exhaustive search is impractical. The coordinator MUST support non-exhaustive strategies:

### Strategy 1: One-factor-at-a-time (OFAT) — implement first
Start from the baseline binary (sdk only, no extensions). Add each extension individually and measure delta quality and delta cost. This requires only `1 + n` runs per task rather than `2^n`, and gives per-extension attribution.

### Strategy 2: Fractional factorial (implement later, design for it now)
Use a Plackett-Burman or similar design matrix to evaluate all main effects with `~2n` runs. Design the config schema now to support this.

### Strategy 3: Top-k refinement (implement later)
After OFAT identifies the most impactful extensions, do full enumeration over the top-k only.

## Configuration

```toml
# eval/eval-plan.toml
strategy = "ofat"      # ofat | fractional-factorial | exhaustive
baseline = "eval/configs/baseline.toml"
extensions = [
  "eval/configs/ext-script-first.toml",
  "eval/configs/ext-hick-edit.toml",
  # add new extension configs here — no code changes needed
]
corpus = "eval/corpus"
sample_n_tasks = 8    # stratified sample per difficulty level
results_dir = "results"
parallel_runs = 2     # run this many agent processes concurrently
```

Adding a new extension requires only adding a config file and listing it here.

## CLI interface

```
just eval                        # run with eval/eval-plan.toml
hick-eval plan --config eval/eval-plan.toml --dry-run   # show what would run
hick-eval plan --config eval/eval-plan.toml             # execute
```

The coordinator logs its plan (which configs × which tasks) to `results/<run-set-id>/plan.xml` before starting.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 OFAT strategy runs baseline + one-extension-at-a-time and records results correctly.
- [ ] #2 Adding a new extension config file and listing it in eval-plan.toml causes it to be included in the next eval run without any code changes.
- [ ] #3 Dry-run mode prints the full run matrix (config × task) without executing anything.
- [ ] #4 Coordinator writes a plan.xml listing all scheduled runs before execution begins.
- [ ] #5 Parallel execution of up to N agent runs is supported and configurable.
- [ ] #6 Results directory structure is documented so TASK-23.6 (reporter) can parse it reliably.
<!-- AC:END -->
