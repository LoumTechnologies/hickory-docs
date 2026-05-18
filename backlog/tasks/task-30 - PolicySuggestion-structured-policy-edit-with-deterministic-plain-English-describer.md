---
id: TASK-30
title: >-
  PolicySuggestion: structured policy edit with deterministic plain-English
  describer
status: Done
assignee: []
created_date: '2026-05-17 10:48'
updated_date: '2026-05-18 12:04'
labels:
  - security
  - policy
  - ux
milestone: m-4
dependencies: []
references:
  - hick-security/crates/hick-policy/src/lib.rs
  - hick-agent/crates/hick-agent-sdk/src/network_filter.rs
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
When the user is prompted about a capability request, they may choose "Always Allow." Always Allow must produce one or more concrete policy edits — rules added to config files — rather than hard-coding the exact request. The UI should present a short list of suggestions at different generalization levels (e.g. exact host, wildcard subdomain, any host) and let the user pick one.

This task adds the foundational `PolicySuggestion` type to `hick-policy` — a structured representation of a single proposed policy edit — along with two deterministic methods on it:

**`PolicySuggestion` fields:**
- `rule_type: SuggestionRuleType` — enum: `Network`, `McpTool`, `ApiSink`
- `pattern: String` — the pattern to allow (e.g. `"api.example.com"`, `"*.example.com"`, `"fetch_webpage"`)
- `scope: SuggestionScope` — enum: `Exact`, `WildcardPath`, `WildcardDomain`, `WildcardAll` (used to describe generalization level)

**`fn describe(&self) -> String`** — deterministic plain-English description of the proposed edit. Pure template fill, no LLM. Examples:
- Network/Exact: `"Allow connections to api.example.com"`
- Network/WildcardDomain: `"Allow connections to any subdomain of example.com"`
- McpTool/Exact: `"Always allow the fetch_webpage tool"`
- ApiSink/Exact: `"Always allow calls to the email sink"`

**`fn to_toml_snippet(&self) -> String`** — the TOML rule entry to insert into the relevant config file. Examples:
- Network: `[[rules]]\npattern = \"api.example.com\"\naction = \"allow\"`
- McpTool (future, when tool-rules.toml exists): `[[rules]]\ntool = \"fetch_webpage\"\naction = \"allow\"`

**`fn config_file_key(&self) -> &'static str`** — returns the config file name the snippet belongs in (e.g. `"network-rules.toml"`, `"tool-rules.toml"`).

No LLM is involved in this task — everything is deterministic. The LLM-backed suggestion generator that produces `Vec<PolicySuggestion>` from a capability request is a separate task.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 PolicySuggestion, SuggestionRuleType, and SuggestionScope are defined in hick-policy and pub-use'd.
- [ ] #2 describe() returns a human-readable sentence for every combination of SuggestionRuleType and SuggestionScope.
- [ ] #3 to_toml_snippet() returns a valid TOML [[rules]] entry for Network and McpTool rule types.
- [ ] #4 config_file_key() returns the correct filename for each rule type.
- [ ] #5 Tests cover describe() and to_toml_snippet() for all SuggestionRuleType variants.
- [ ] #6 No LLM calls, subprocess calls, or async code in this module.
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added `PolicySuggestion` struct with `SuggestionRuleType` and `SuggestionScope` enums in new `hick-policy/src/suggestion.rs`. Implements `describe()`, `to_toml_snippet()`, and `config_file_key()`. Full test suite included. Re-exported from `hick-policy/src/lib.rs`.
<!-- SECTION:FINAL_SUMMARY:END -->
