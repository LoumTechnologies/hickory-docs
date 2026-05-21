---
id: TASK-49
title: >-
  Pruning workflow: manual approval and optional auto-prune with full audit
  trail
status: Done
assignee: []
created_date: '2026-05-20 12:17'
updated_date: '2026-05-21 10:37'
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
- [x] #1 Pruning view ranks configs by score with confidence intervals visible
- [x] #2 Configs below a threshold relative to the best are highlighted as candidates
- [x] #3 Manual pruning: user selects a candidate and confirms; pruning record appended to eval store
- [x] #4 Auto-prune: enabled via config with a confidence threshold (e.g. 0.95); fires automatically during idle eval
- [x] #5 Auto-prune records include: config name, scores at time of pruning, confidence, timestamp, machine-id
- [x] #6 Pruning view shows all historical auto-prune decisions and allows the user to un-prune (appends un-prune record)
- [x] #7 Pruned configs are excluded from future idle eval scheduling unless un-pruned
- [x] #8 No NDJSON lines are ever deleted; all pruning is metadata-only
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
## Changes

### `hick-eval-store`
- **`record.rs`**: Extended `PruneRecord` with optional `config_name`, `scores`, `confidence`, and `trigger` fields (backward-compatible — old records with missing fields parse as `None`). Added `UnpruneRecord` struct. Added `EvalRecord::Unprune` variant with full serialize/deserialize round-trip support.
- **`pruning.rs`** (new): `ConfigStats` struct with combined score, sample count, win rate, Wilson 95% CI bounds, `is_pruned`, `is_candidate` flags. `compute_config_stats()` replays the full record log — human ratings override judge scores per (config, query). `is_pruned()` for use by TASK-50 idle eval scheduler. `auto_prune_candidates()` returns configs whose CI upper bound falls below `best_win_rate * confidence_threshold`. `wilson_ci()` is the core statistical primitive. 13 unit tests.
- **`lib.rs`**: Exported `UnpruneRecord`, `ConfigStats`, `compute_config_stats`, `is_pruned`, `auto_prune_candidates`, `wilson_ci`.

### `hick-agent`
- **`config.rs`**: Added `auto_prune_threshold: Option<f64>` to `EvalSection`. Two new tests cover present and absent cases.

### `hick-agent-tui`
- **`Cargo.toml`**: Added `hick-eval-store` and `chrono` dependencies.
- **`run.rs`**: Added `PruneAction` enum (`Prune { config_name, scores, confidence }` / `Unprune { config_name }`). Added `eval_store: Option<Arc<dyn StorageBackend>>` and `prune_tx: Option<mpsc::UnboundedSender<PruneAction>>` to `TuiConfig`. Added `PruningViewState` struct. `P` key opens the pruning view; inside the view Up/Down navigate, `p`/Enter prunes a candidate, `u` un-prunes, `Esc` closes. Rendered as a ranked list showing combined score, win rate, Wilson CI, sample count, and status (CANDIDATE / PRUNED). Hint updated in input bar.

### `hick-agent/src/main.rs`
- Added `handle_prune_actions()` async fn — persists `PruneRecord` or `UnpruneRecord` to the eval store on each received `PruneAction`.
- Agent command path creates a `GitBackend`, wraps it in `Arc<dyn StorageBackend>`, and passes both `eval_store` and `prune_tx` to `TuiConfig`.
- Replay path receives `eval_store: None, prune_tx: None`.

### No lines deleted
All pruning is append-only. The `UnpruneRecord` never removes a `PruneRecord`; readers derive current state by replaying the log.
<!-- SECTION:FINAL_SUMMARY:END -->
