---
id: TASK-32
title: >-
  Always Allow: extend approval requests to carry policy edits and apply them to
  config files
status: To Do
assignee: []
created_date: '2026-05-17 10:48'
updated_date: '2026-05-17 10:51'
labels:
  - security
  - policy
  - ux
  - network
milestone: m-4
dependencies:
  - TASK-30
  - TASK-31
  - TASK-24.2
references:
  - hick-agent/crates/hick-agent-sdk/src/network_filter.rs
  - hick-agent/crates/hick-agent-sdk/src/sink_guard.rs
  - hick-agent/crates/copper-ledge/src/main.rs
  - hick-agent/crates/iron-peak/src/main.rs
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The approval channel for both `NetworkFilter` (`NetworkApprovalRequest`) and `ToolGate` (`ToolApprovalRequest`, from TASK-24.2) currently only supports Allow/Block verdicts. There is no way for the user to signal "allow permanently" — the verdict is session-only and caches in memory.

This task adds the Always Allow path to both filters.

## Changes to `NetworkApprovalRequest` and `ToolApprovalRequest`

Extend the response side of both approval channels to carry an optional policy edit:

```rust
pub struct ApprovalResponse {
    pub verdict: NetworkVerdict,            // Allow or Block
    pub always_allow: Option<PolicySuggestion>,  // if Some, persist this rule before proceeding
}
```

The `oneshot::Sender` in both request types changes from `oneshot::Sender<NetworkVerdict>` / `oneshot::Sender<ToolVerdict>` to `oneshot::Sender<ApprovalResponse>`. (Or introduce a shared `ApprovalResponse` type — either is fine.)

## Config file application

When `always_allow` is `Some(suggestion)`, the filter must:
1. Append `suggestion.to_toml_snippet()` to the config file named by `suggestion.config_file_key()`, relative to the project directory.
2. Reload the filter's rules from the updated config file so subsequent requests use the new rule without restarting.
3. Then send `Allow` as the in-session verdict.

If writing the config file fails, log a warning and fall back to session-only allow (do not block the in-flight request).

## Binary wiring (copper-ledge and iron-peak)

The approval channel receivers in `copper-ledge/src/main.rs` and `iron-peak/src/main.rs` need to:
1. On receiving an approval request: kick off `SuggestionGenerator::suggestions()` in the background immediately (eager pre-computation).
2. Present the prompt to the user: "Allow Once" / "Always Allow" (with suggestion list) / "Block".
3. If Always Allow: wait for background suggestions (or use deterministic fallback if not ready), let the user pick one, and send `ApprovalResponse { verdict: Allow, always_allow: Some(chosen_suggestion) }`.
4. If Allow Once: send `ApprovalResponse { verdict: Allow, always_allow: None }`.
5. If Block: send `ApprovalResponse { verdict: Block, always_allow: None }`.

The UI representation of suggestions comes from `PolicySuggestion::describe()` — the plain-English strings from TASK-30.

## What this task does NOT do
- Does not implement the actual UI/TUI rendering — that depends on each binary's UI layer. This task adds the data plumbing; the UI layer can be stubbed with a CLI prompt.
- Does not add suggestion generation for `PolicyDecision::Prompt` from the Python policy evaluator path — that is future work.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 NetworkApprovalRequest response channel carries ApprovalResponse (verdict + optional PolicySuggestion), not a bare NetworkVerdict.
- [ ] #2 ToolApprovalRequest response channel carries the same ApprovalResponse type.
- [ ] #3 When always_allow is Some, the filter appends the TOML snippet to the correct config file before caching the Allow verdict.
- [ ] #4 After a successful always_allow write, a second request to the same host/tool bypasses the prompt (rule is live in the filter).
- [ ] #5 If config file write fails, the request is still allowed for the session and the failure is logged as a warning (not a hard error).
- [ ] #6 copper-ledge and iron-peak kick off SuggestionGenerator in the background when a prompt fires.
- [ ] #7 Tests: always_allow writes the expected line to a temp config file; reload causes subsequent check to return Allow without prompting; write failure allows session-only.
<!-- AC:END -->
