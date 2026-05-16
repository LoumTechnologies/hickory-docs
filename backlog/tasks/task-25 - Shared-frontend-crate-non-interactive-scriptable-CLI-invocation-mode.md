---
id: TASK-25
title: 'Shared frontend crate: non-interactive (scriptable) CLI invocation mode'
status: To Do
assignee: []
created_date: '2026-05-16 12:33'
labels:
  - frontend
  - cli
  - evaluation-prereq
milestone: m-2
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Create a shared Rust crate that all agent frontend binaries depend on to support non-interactive (scriptable) invocation. This is the interface the eval runner (TASK-23.3) uses to drive agents programmatically.

## Where it lives

This is a **frontend concern**, not an SDK concern. The SDK (`hick-agent-sdk`) is the core runtime with no presentation knowledge. This crate sits one layer above it — a shared library that each binary frontend links, similar to how extension crates work. Suggested name: `hick-agent-frontend` or `hick-frontend-common`.

All existing binary crates (copper-ledge, iron-peak, etc.) must adopt it. Future binaries are expected to adopt it by default.

## Why a shared crate, not per-binary

Each binary has its own ReAct loop wiring, so a shared crate cannot own the loop itself. What it CAN own is the argument parsing and invocation contract so the eval runner has a single stable interface regardless of which binary it is calling.

## Invocation contract

When invoked with `--non-interactive` (or equivalent), the agent binary must:

1. Accept a prompt via `--prompt <file>` or stdin
2. Accept a workspace path via `--workspace <dir>` (pre-populated with task seed files)
3. Run to completion (no user input required)
4. Exit 0 on success, non-zero on failure or timeout
5. Write its performance log to `--perf-log <file>` (format defined in TASK-24)

The interactive (TUI) mode remains the default when `--non-interactive` is absent.

## CLI sketch

```
copper-ledge --non-interactive \
             --prompt eval/corpus/fix-off-by-one/prompt.md \
             --workspace /tmp/eval-workspace-xyz \
             --perf-log /tmp/run-abc/perf.xml \
             --max-turns 20 \
             --timeout-secs 300
```

## Shared crate responsibilities

- Arg parsing for the non-interactive flags above (via clap or similar)
- Workspace setup helper: copy seed files into a temp dir, hand path to the agent loop
- Exit-code contract enforcement
- Wiring the perf log sink (from TASK-24) into the agent session

The crate exposes a `run_non_interactive(config, agent_loop_fn)` entry point that each binary calls with its own loop implementation.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A shared crate exists and is linked by all existing agent binary crates.
- [ ] #2 Each binary accepts --non-interactive, --prompt, --workspace, --perf-log, --max-turns, --timeout-secs flags when the crate is linked.
- [ ] #3 Running a binary in non-interactive mode with a prompt file drives the agent to completion without any stdin interaction.
- [ ] #4 Exit code is 0 on clean completion and non-zero on agent error or timeout.
- [ ] #5 Interactive (TUI) mode is unaffected when --non-interactive is absent.
- [ ] #6 The crate is documented well enough that a new binary crate author can adopt it with a single function call.
<!-- AC:END -->
