---
id: TASK-62
title: 'Context compaction: auto-summarise conversation history under window pressure'
status: To Do
assignee: []
created_date: '2026-05-23 02:27'
labels: []
milestone: m-7
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add automatic context compaction to the agent runtime loop so that long sessions don't silently degrade when the context window fills. Without this, the LLM drops earlier context, produces inconsistent decisions, and loses track of prior attempts. This is the single highest-impact missing feature for session quality.

Behaviour: when the accumulated message history exceeds a configurable token threshold (e.g. 80% of the model's context limit), the agent pauses, makes a cheap summarisation call ("summarise what has been accomplished and what remains, preserving all open decisions and file paths"), replaces the older portion of history with the summary as a system-role message, and resumes. The active turn and recent N turns are always preserved verbatim.

The summarisation call should use the `smol` model role once role-based routing (TASK for model routing) is implemented; until then it can reuse the current model.

Key files:
- `hick-agent-sdk/src/agent_engine.rs` — compaction hook in the turn loop
- `hick-agent-sdk/src/llm.rs` — token counting or estimation (character proxy until exact counts are available)
- `agent.toml` schema — `[compaction]` section: `threshold_fraction`, `keep_recent_turns`
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 agent.toml accepts a [compaction] section with threshold_fraction (0.0–1.0, default 0.80) and keep_recent_turns (default 6)
- [ ] #2 When message history token estimate exceeds threshold the agent emits a UiEvent::Info noting compaction is running
- [ ] #3 A summarisation LLM call is made with a fixed continuity prompt; the result replaces older history as a System message
- [ ] #4 The most recent keep_recent_turns user+assistant pairs are always preserved verbatim after the summary
- [ ] #5 The original uncompacted history is written to the session log before replacement so replay is lossless
- [ ] #6 Compaction can be disabled with compaction.enabled = false in agent.toml
- [ ] #7 An integration test simulates a long conversation and verifies compaction fires and the agent continues coherently
- [ ] #8 No compaction occurs during an in-flight tool call (only between turns)
<!-- AC:END -->
