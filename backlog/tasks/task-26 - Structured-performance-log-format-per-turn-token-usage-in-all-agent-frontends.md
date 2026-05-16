---
id: TASK-26
title: 'Structured performance log format: per-turn token usage in all agent frontends'
status: To Do
assignee: []
created_date: '2026-05-16 12:34'
updated_date: '2026-05-16 12:34'
labels:
  - frontend
  - logging
  - evaluation-prereq
milestone: m-2
dependencies:
  - TASK-25
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Define and implement a standardized XML performance log format that every agent frontend writes during a run. This is separate from the session log (which varies per implementation) and is specifically designed for the eval system to extract token usage and timing data.

## Why not use the session log

Session log formats differ between agent implementations — hick-edit writes different content than script-first. The eval runner (TASK-23.3) needs a single stable source of token usage truth regardless of which agent it ran. A dedicated perf log is the right separation.

## Log format

Written to the path passed via `--perf-log` (from TASK-23's scriptable CLI, TASK-24). One file per run.

```xml
<perf-log agent="copper-ledge" model="claude-sonnet-4-6"
           extended-thinking="false" started-at="2026-05-16T12:00:00Z">

  <turn index="1" timestamp="2026-05-16T12:00:03Z" duration-secs="4.1">
    <token-usage cache-write-input="1200" cache-read-input="0"
                 uncached-input="840" output="312" thinking="0"/>
  </turn>

  <turn index="2" timestamp="2026-05-16T12:00:07Z" duration-secs="6.3">
    <token-usage cache-write-input="0" cache-read-input="1180"
                 uncached-input="94" output="541" thinking="0"/>
  </turn>

  <!-- ... -->

  <summary total-turns="8" wall-secs="47.2"
           total-cache-write-input="1200" total-cache-read-input="9440"
           total-uncached-input="1203" total-output="2847" total-thinking="0"/>

</perf-log>
```

## Token usage source

Token counts come from the API response metadata returned by the SDK after each LLM call. The SDK already receives this data; the frontend just needs to write it to the perf log after each turn rather than discarding it.

## Shared crate

Implement the perf log writer in the same shared frontend crate as TASK-24 (the non-interactive CLI crate), so frontends get both for the price of one dependency. The writer is usable in interactive mode too — always emit the perf log when `--perf-log` is passed, regardless of interactive vs non-interactive.

## All frontends must adopt

Both existing binary crates and any future binaries must write this log. This is the contract the eval runner depends on — if a frontend doesn't emit it, it cannot be evaluated.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Perf log XML schema is defined and checked in (as a doc or Rust type) in the shared frontend crate.
- [ ] #2 All existing agent binary crates write a valid perf log when --perf-log <path> is passed.
- [ ] #3 Per-turn token usage values match the API response metadata (verified by comparing log totals against a known API call).
- [ ] #4 The <summary> element totals match the sum of per-turn values.
- [ ] #5 The perf log is written incrementally (one <turn> appended after each LLM call), not buffered and written at exit — a crashed run still has partial data.
- [ ] #6 TASK-23.3 (eval runner) can parse the perf log without ambiguity to extract total token usage.
<!-- AC:END -->
