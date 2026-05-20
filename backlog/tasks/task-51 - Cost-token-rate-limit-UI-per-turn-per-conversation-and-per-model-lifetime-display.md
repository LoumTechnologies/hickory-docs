---
id: TASK-51
title: >-
  Cost, token, rate limit UI: per-turn, per-conversation, and per-model lifetime
  display
status: To Do
assignee: []
created_date: '2026-05-20 12:17'
labels:
  - ui
  - cost
  - tokens
dependencies:
  - TASK-26
  - TASK-37
  - TASK-44
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend the UI (TUI status panel and browser UI footer/sidebar) with a comprehensive cost and rate limit view. Per-turn: input tokens, cached tokens (prompt cache hits), output tokens, cache savings in dollars, turn cost. Per-conversation: running total cost, context window percentage with a visual indicator (warn at 80%), total tokens used. Per-model lifetime: cumulative spend across all sessions and eval runs, pulled from the eval store. Rate limits (Anthropic/OpenAI-compat only): requests remaining/limit, tokens remaining/limit, reset countdown timer, throttled state warning with retry-after countdown. Local models show token counts and context % but no dollar cost or rate limits. Pricing tables live in a TOML file that can be updated without recompiling. All cost figures use accurate per-model input/output/cache pricing.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Per-turn breakdown shown after each assistant response: input tokens, cached tokens, output tokens, cost
- [ ] #2 Cache hit savings shown explicitly in dollars (not just token count)
- [ ] #3 Context window % shown as a progress bar or percentage; turns amber at 80%, red at 95%
- [ ] #4 Running conversation total cost shown persistently in status area
- [ ] #5 Per-model lifetime spend loaded from eval store and shown in a model info panel
- [ ] #6 Rate limit state (requests remaining, tokens remaining, reset countdown) shown per model
- [ ] #7 Throttled state shown as a prominent warning with retry-after countdown in seconds
- [ ] #8 Local model tabs show token counts and context % but omit cost and rate limit sections
- [ ] #9 Pricing tables in a versioned TOML file (e.g. ~/.hick/pricing.toml) updateable without recompile
- [ ] #10 TUI and browser UI have feature parity for all fields above
- [ ] #11 Judge call costs tracked and displayed separately from conversation costs
<!-- AC:END -->
