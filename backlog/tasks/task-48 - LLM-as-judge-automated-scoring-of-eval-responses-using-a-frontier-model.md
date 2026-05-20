---
id: TASK-48
title: 'LLM-as-judge: automated scoring of eval responses using a frontier model'
status: To Do
assignee: []
created_date: '2026-05-20 12:17'
labels:
  - eval
  - scoring
dependencies:
  - TASK-44
  - TASK-46
  - TASK-47
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
After a multi-config or retroactive comparison produces responses, a configurable judge model (typically a frontier model, configured in the agent TOML under `[eval.judge]`) scores each response on: task completion, tool call efficiency, unnecessary steps, and output quality. Scoring runs as a background Tokio task so it does not block the UI. The judge's scores are pre-populated in the human rating UI and can be overridden by the user. Judge scores and human overrides are both persisted to the eval store. The judge prompt is configurable. Cost of judge calls is tracked separately in the eval store and surfaced in the cost UI.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 [eval.judge] config section: model, prompt template, dimensions (list), weight per dimension
- [ ] #2 Judge runs as a background Tokio task after responses are available
- [ ] #3 Judge scores pre-populate the human rating UI with a visual indicator that they are auto-generated
- [ ] #4 Human can override any judge score; both original and override are stored
- [ ] #5 Judge call token cost tracked separately and shown in the cost panel
- [ ] #6 Judge prompt template is user-configurable in the TOML
- [ ] #7 If judge call fails, the UI shows a non-blocking error and human rating is still available
- [ ] #8 Judge scores written to eval store with judge model name and prompt hash for reproducibility
<!-- AC:END -->
