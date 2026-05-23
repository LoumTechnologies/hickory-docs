---
id: TASK-63
title: 'Role-based model routing: smol/slow/plan/commit roles in agent.toml'
status: To Do
assignee: []
created_date: '2026-05-23 02:27'
labels: []
milestone: m-7
dependencies: []
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a model routing layer to `hick-agent-sdk` so that different agent activities use appropriately sized models rather than always using the same model. Multi-provider support already exists (`llm_anthropic.rs`, `llm_openai_compat.rs`); this task adds a routing layer on top.

Four roles:
- `default` — normal interactive turns (current behaviour)
- `smol` — cheap/fast model for subagent fan-out and context compaction summarisation
- `slow` — deep reasoning model for hard problems (user-switchable mid-session)
- `plan` — model used during plan-mode reasoning (see plan-mode task)
- `commit` — cheap model for session summary / changelog generation

Each role maps to a `(provider, model)` pair in `agent.toml`. Unspecified roles fall back to `default`. The `LlmClient` abstraction already exists; a `ModelRouter` sits above it and selects the right client per role.

`agent.toml` example:
```toml
[models]
default = { provider = "anthropic", model = "claude-opus-4-7" }
smol    = { provider = "anthropic", model = "claude-haiku-4-5-20251001" }
slow    = { provider = "xai",       model = "grok-3" }
commit  = { provider = "anthropic", model = "claude-haiku-4-5-20251001" }
```
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 agent.toml [models] section accepts role keys (default, smol, slow, plan, commit) each with provider and model fields
- [ ] #2 ModelRouter struct in hick-agent-sdk selects the correct LlmClient for a given role, falling back to default for unspecified roles
- [ ] #3 The agent engine passes the role to ModelRouter at each call site (default for normal turns, smol for compaction summarisation)
- [ ] #4 Provider values map to the existing client constructors: anthropic → AnthropicClient, openai/xai/deepseek/openrouter/etc → OpenAiCompatClient with the appropriate base URL
- [ ] #5 An unknown provider name returns a clear configuration error at startup, not a runtime panic
- [ ] #6 If [models] is absent the agent behaves identically to current behaviour (single model from the existing config)
- [ ] #7 Unit tests cover: role lookup, fallback to default, unknown provider error
<!-- AC:END -->
