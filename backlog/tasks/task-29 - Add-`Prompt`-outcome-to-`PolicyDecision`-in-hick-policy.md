---
id: TASK-29
title: Add `Prompt` outcome to `PolicyDecision` in hick-policy
status: To Do
assignee: []
created_date: '2026-05-17 10:47'
updated_date: '2026-05-17 10:51'
labels:
  - security
  - policy
milestone: m-4
dependencies: []
references:
  - hick-security/crates/hick-policy/src/lib.rs
  - hick-agent/crates/copper-ledge/src/main.rs
  - hick-agent/crates/iron-peak/src/main.rs
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The `PolicyDecision` enum in `hick-security/crates/hick-policy/src/lib.rs` currently has two variants: `Allow` and `Deny { reason }`. Python policy scripts return one of these as JSON to the `PolicyEvaluator`.

Add a third variant `Prompt { reason: String }` so that a policy script can signal "I cannot make a binary decision — surface this to the user." This is distinct from `Deny`: Deny is a hard block, Prompt suspends execution and waits for human input.

The JSON representation should be `{"decision": "prompt", "reason": "..."}` — consistent with the existing `serde(tag = "decision")` on the enum.

All call sites that match on `PolicyDecision` (in `copper-ledge/src/main.rs` and `iron-peak/src/main.rs`) must handle the new variant. For now, treat `Prompt` the same as `Deny` at those call sites with a TODO comment — the full prompt flow is wired up in a later task (Always Allow task).

This is a public API addition (new enum variant), so hick-policy needs a minor version bump per the repo's semver policy in `hick-agent/CLAUDE.md`.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 PolicyDecision has a Prompt { reason: String } variant.
- [ ] #2 JSON roundtrip: {"decision": "prompt", "reason": "needs approval"} deserializes to Prompt and re-serializes correctly.
- [ ] #3 evaluate_flow and evaluate_access return Prompt when the Python script returns that decision.
- [ ] #4 All existing match sites on PolicyDecision handle Prompt (no non-exhaustive match warnings or errors).
- [ ] #5 hick-policy version is bumped (minor).
- [ ] #6 Tests cover: policy script returning prompt is parsed and returned correctly.
<!-- AC:END -->
