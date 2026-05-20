---
id: TASK-27
title: >-
  Config-driven agent runtime: compose extensions from a TOML file without
  writing Rust
status: To Do
assignee:
  - Nate Loum
created_date: '2026-05-16 12:57'
updated_date: '2026-05-20 12:18'
labels:
  - agent
  - config
  - search-space
  - eval
milestone: m-1
dependencies:
  - TASK-33
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend the existing security/MCP config TOML (e.g. `mcp-servers.toml`) with new sections rather than introducing a separate config file. New sections to add: `[extensions]` (which extension crates are active, keyed by extension name), `[model]` (provider, model ID, API key env var, context window, max tokens), `[machine]` (see TASK-45), and `[eval]` (idle timeout, judge config, auto-prune threshold). The existing security policy sections are unchanged. Config resolution order: `--config` flag > `.hick/agent.toml` in project root > `~/.hick/agent.toml` > built-in default. Named example configs (`configs/hick-edit.toml`, `configs/standard.toml`) ship with the repo and reproduce the former iron-peak and copper-ledge combinations.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A config file with no `[search]` section and all-scalar fields is parsed as a `PointConfig`; a file with `[search]` and at least one search axis is parsed as a `SearchSpaceConfig`. Both are represented by the same `AgentConfig` type with a discriminated field.
- [ ] #2 Unknown keys in `[intent_parsers]` or `[classifiers]` produce a startup error naming the unrecognised key.
- [ ] #3 A `SearchSpaceConfig` with `strategy = "grid"` and two binary search axes produces the correct 4-combination trial list via `AgentConfig::expand_trials()`.
- [ ] #4 A `SearchSpaceConfig` with `strategy = "ofat"` and n axes produces exactly `1 + sum(len(axis_i) - 1)` trials.
- [ ] #5 Constraints prune invalid combinations: a constraint `if hick_edit=true then script_first=true` removes `{hick_edit=true, script_first=false}` from the trial set.
- [ ] #6 `hick-agent --config <file> --expand` prints all concrete trial configs as TOML to stdout and exits without starting an agent.
- [ ] #7 `hick-agent --config <file>` (no `--expand`) starts an agent when the file is a point config and emits a clear error when the file contains a `[search]` section.
- [ ] #8 Adding a new extension to the search space requires exactly one line in the config file (`new_ext = { search = [true, false] }`) and no other file changes.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Scope clarified (2026-05-20): extend the existing security TOML specifically — do not introduce a new config file format. Sections [machine] and [eval] are defined here but their consumers are TASK-45 and TASK-50. Named example configs replace the iron-peak and copper-ledge binaries (see TASK-43).
<!-- SECTION:NOTES:END -->
