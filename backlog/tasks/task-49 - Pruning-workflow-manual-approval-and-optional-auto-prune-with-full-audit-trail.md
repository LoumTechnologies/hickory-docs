---
id: TASK-49
title: >-
  Pruning workflow: manual approval and optional auto-prune with full audit
  trail
status: To Do
assignee: []
created_date: '2026-05-20 12:17'
labels:
  - eval
  - ui
dependencies:
  - TASK-44
  - TASK-48
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
A dedicated pruning view (in TUI and browser UI) shows configs ranked by combined human + judge score with a computed confidence level. Configs that are statistically clearly worse than the current best are flagged as pruning candidates. The user must explicitly approve pruning each candidate, or can enable auto-prune with a configurable confidence threshold. Auto-prune writes a record to the eval store explaining which config was pruned, what scores triggered pruning, and the confidence level at the time — nothing is deleted. The pruning view shows the full history of auto-prune decisions for review.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Pruning view ranks configs by score with confidence intervals visible
- [ ] #2 Configs below a threshold relative to the best are highlighted as candidates
- [ ] #3 Manual pruning: user selects a candidate and confirms; pruning record appended to eval store
- [ ] #4 Auto-prune: enabled via config with a confidence threshold (e.g. 0.95); fires automatically during idle eval
- [ ] #5 Auto-prune records include: config name, scores at time of pruning, confidence, timestamp, machine-id
- [ ] #6 Pruning view shows all historical auto-prune decisions and allows the user to un-prune (appends un-prune record)
- [ ] #7 Pruned configs are excluded from future idle eval scheduling unless un-pruned
- [ ] #8 No NDJSON lines are ever deleted; all pruning is metadata-only
<!-- AC:END -->
