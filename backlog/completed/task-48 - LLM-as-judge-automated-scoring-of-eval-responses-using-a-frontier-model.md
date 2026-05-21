---
id: TASK-48
title: 'LLM-as-judge: automated scoring of eval responses using a frontier model'
status: Done
assignee: []
created_date: '2026-05-20 12:17'
updated_date: '2026-05-21 02:12'
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
- [x] #1 [eval.judge] config section: model, prompt template, dimensions (list), weight per dimension
- [x] #2 Judge runs as a background Tokio task after responses are available
- [x] #3 Judge scores pre-populate the human rating UI with a visual indicator that they are auto-generated
- [x] #4 Human can override any judge score; both original and override are stored
- [x] #5 Judge call token cost tracked separately and shown in the cost panel
- [x] #6 Judge prompt template is user-configurable in the TOML
- [x] #7 If judge call fails, the UI shows a non-blocking error and human rating is still available
- [x] #8 Judge scores written to eval store with judge model name and prompt hash for reproducibility
- [x] #9 Judge prompt template includes separate scoring dimensions for 'unnecessary steps' and 'backtracking', with instructions to identify decision reversals and whether each was avoidable.
- [x] #10 Human rating UI displays wall-clock time and rework-count from the eval result XML as non-editable facts above the judge score override fields.
- [x] #11 Judge output XML records backtracking score and a brief rationale for each reversal identified.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
## Backtracking as an explicit judge dimension

'Unnecessary steps' and 'backtracking' are not the same thing and MUST be scored separately:

- **Unnecessary steps**: steps that were taken but could have been skipped without affecting the outcome (e.g. reading a file that was never needed)
- **Backtracking**: steps that explicitly reversed a prior decision — rewriting a file that was already written, undoing an edit, retrying a failed approach with a different strategy

Backtracking is more expensive than unnecessary steps: it wastes the time of both the model and any human watching. A model that charges ahead fast but reverses course twice can have a higher *effective* latency than a slower model that gets it right the first time.

The judge prompt template MUST instruct the judge to:
1. Identify any points where the agent reversed a prior action and explain what triggered the reversal
2. Score backtracking separately from unnecessary steps (both on the configured dimension scale)
3. Note whether the rework was avoidable (bad initial plan) or unavoidable (new information arrived)

## Time and rework in the human rating UI

The human rating UI must surface wall-clock time and rework-count from the eval result XML as non-editable ground-truth facts, displayed alongside the pre-populated judge scores. These are facts, not opinions — the human can override judge scores but cannot override measured timing.

Suggested UI layout per response:
```
[Config A]  Time: 61s  Turns: 6  Rework: 0
  Judge scores (auto): Task completion 8/10 | Tool efficiency 9/10 | Backtracking 10/10
  Your scores:         [___/10]             | [___/10]             | [___/10]
```

This framing — showing a fast/clean run vs a slow/rework-heavy run side by side — is what surfaces the 'cheap but actually slow' insight to the human judge.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented LLM-as-judge automated scoring (all 11 ACs):

- `[eval.judge]` TOML section: model, provider, api_key_env, prompt_template, dimensions with weights
- Default dimensions: task_completion (0.3), tool_efficiency (0.2), unnecessary_steps (0.15), backtracking (0.25), output_quality (0.1)
- Background Tokio judge task (`judge_task.rs`) subscribes to per-tab broadcast channel; fires after every `UiEvent::Done`
- Tracks wall-clock time (UserTurn→Done) and rework count (non-zero ActionEnd exit codes)
- `JudgeScoreReady` / `JudgeError` variants added to `UiEvent`; TUI handles both
- TUI pre-populates human rating UI with judge scores; `/rate` and `/accept` slash commands for override/acceptance
- Wall-clock time and rework count displayed as non-editable facts above score fields
- Judge scores + backtracking_events written to eval store as `JudgeScoreRecord` with judge model name and FNV-1a prompt hash
- Human rating overrides stored separately as `HumanRatingRecord` via dedicated `rating_tx` mpsc channel
- Backtracking analysis: judge prompt explicitly requests per-reversal rationale with avoidable flag in JSON output
- Non-blocking judge errors shown in TUI without disrupting conversation
- `conversation_tree.rs` updated to ignore new judge event variants
<!-- SECTION:FINAL_SUMMARY:END -->
