---
id: TASK-26
title: 'Structured performance log format: per-turn token usage in all agent frontends'
status: To Do
assignee: []
created_date: '2026-05-16 12:34'
updated_date: '2026-05-20 12:18'
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
Extend the structured performance log to capture all data needed by the cost/token/rate-limit UI (TASK-51). Every agent turn must emit a structured XML log record containing: input tokens, cached tokens (prompt cache hits), output tokens, finish reason, model name, config name, and wall-clock latency. Anthropic/OpenAI-compat turns additionally capture rate limit headers from each HTTP response: requests-limit, requests-remaining, requests-reset, tokens-limit, tokens-remaining, tokens-reset, and retry-after (when throttled). Dollar costs are computed at log time using the pricing tables introduced in TASK-51 and written into the record so the UI never has to recompute them. Local model turns emit token counts and context-window size but omit cost and rate limit fields.
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

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Scope expanded (2026-05-20): original task covered token counts only. Now also covers prompt cache hit/miss, dollar cost at log time, context window tokens, and rate limit header capture from HTTP responses. This data feeds TASK-51 (cost/token/rate-limit UI).
<!-- SECTION:NOTES:END -->
