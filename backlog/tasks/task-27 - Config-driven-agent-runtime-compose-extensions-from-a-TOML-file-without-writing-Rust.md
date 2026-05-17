---
id: TASK-27
title: >-
  Config-driven agent runtime: compose extensions from a TOML file without
  writing Rust
status: To Do
assignee:
  - Nate Loum
created_date: '2026-05-16 12:57'
updated_date: '2026-05-17 10:50'
labels:
  - agent
  - config
  - search-space
  - eval
milestone: m-1
dependencies:
  - TASK-33
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Design and implement the unified agent config file format (`.hick/agent.toml`) that maps the configurable knobs of the SDK v2 loop architecture (TASK-33) to a TOML schema. A single file serves two purposes without modification:

1. **Point config** — used directly by `hick-agent --config` to start a single concrete agent run.
2. **Search space** — read by the eval coordinator (TASK-23.5) to generate the trial matrix for A/B evaluation, with no separate per-extension or per-combination files needed.

## Why one file for both

TASK-23.5's current design requires a `baseline.toml` plus N separate extension config files, all listed in `eval-plan.toml`. That requires enumeration as N grows. The search-axis approach puts the variability *inside* the config file itself, so adding a new extension to the search space is one line, not a new file + a list edit.

## Config schema

Fields accept either a **scalar** (concrete value) or a **search axis** (`{ search = [...] }`). A file with no search axes is a point config. A file with one or more search axes plus a `[search]` section is a search space config.

```toml
config_version = 1

# ── LLM ────────────────────────────────────────────────────────
[agent]
model    = "claude-sonnet-4-6"          # scalar
provider = "anthropic"                  # scalar

# ── Intent parsers (MatchIntent impls — from TASK-33) ──────────
[intent_parsers]
script_first = true                               # scalar: always on
hick_edit    = { search = [true, false] }         # search axis: 2 values

# ── Data classifiers (ClassifyData impls — from TASK-33) ───────
[classifiers]
secrets = true                                    # scalar: always on

# ── Runtime knobs ───────────────────────────────────────────────
[runtime]
max_turns    = 30
timeout_secs = 600

# ── Search section — omit entirely for a concrete point config ──
[search]
strategy = "ofat"    # grid | ofat | fractional-factorial
metric   = "task_completion_rate"

# Constraints: invalid combinations are pruned before trial generation
[[search.constraints]]
if   = { "intent_parsers.hick_edit" = true }
then = { "intent_parsers.script_first" = true }
```

## Combinatorial expansion without enumeration

The `strategy` field under `[search]` controls how trials are generated from the declared axes:

- **`grid`** — full Cartesian product of all search axis values, minus constrained combinations.
- **`ofat`** (one-factor-at-a-time) — baseline (all axes at their first value) plus one axis varied at a time. Gives `1 + sum(len(axis) - 1)` trials — linear in the number of axes.
- **`fractional-factorial`** — Plackett-Burman or similar design matrix; evaluates all main effects with ~2n trials.

At n=10 extensions, `ofat` requires 11 trials vs 1024 for `grid`. The coordinator chooses which to use; the config just declares the space.

## CLI surface

```
hick-agent --config .hick/agent.toml            # run as point config (error if [search] present)
hick-agent --config .hick/agent.toml --expand   # print all concrete trial configs to stdout as TOML
```

`--expand` is what the eval coordinator calls to get the trial matrix; it replaces the separate `extensions = [list]` in TASK-23.5's `eval-plan.toml`.

## Unknown key policy

- Unknown keys in `[intent_parsers]` or `[classifiers]` are validated against the registered extension IDs at startup — unknown keys are a hard error.
- Unknown keys elsewhere (e.g., in `[agent]`) follow serde's `deny_unknown_fields` behaviour.

## What this does NOT do

- Does not implement the search strategies themselves (that is TASK-23.5's job).
- Does not implement the `MatchIntent` / `ClassifyData` traits (that is TASK-33).
- Does not change existing binary crates (`copper-ledge`, `iron-peak`).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A config file with no `[search]` section and all-scalar fields is parsed as a `PointConfig`; a file with `[search]` and at least one search axis is parsed as a `SearchSpaceConfig`. Both are represented by the same `AgentConfig` type with a discriminated field.
- [ ] #2 Unknown keys in `[intent_parsers]` or `[classifiers]` produce a startup error naming the unrecognised key.
- [ ] #3 A `SearchSpaceConfig` with `strategy = "grid"` and two binary search axes produces the correct 4-combination trial list via `AgentConfig::expand_trials()`.
- [ ] #4 A `SearchSpaceConfig` with `strategy = "ofat"` and n axes produces exactly `1 + sum(len(axis_i) - 1)` trials.
- [ ] #5 Constraints prune invalid combinations: a constraint `if hick_edit=true then script_first=true` removes `{hick_edit=true, script_first=false}` from the trial set.
- [ ] #6 `hick-agent --config <file> --expand` prints all concrete trial configs as TOML to stdout and exits without starting an agent.
- [ ] #7 `hick-agent --config <file>` (no `--expand`) starts an agent when the file is a point config and emits a clear error when the file contains a `[search]` section.
- [ ] #8 Adding a new extension to the search space requires exactly one line in the config file (`new_ext = { search = [true, false] }`) and no other file changes.
<!-- AC:END -->
