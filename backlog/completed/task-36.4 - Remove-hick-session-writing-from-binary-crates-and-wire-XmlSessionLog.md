---
id: TASK-36.4
title: 'Remove hick: session writing from binary crates and wire XmlSessionLog'
status: Done
assignee: []
created_date: '2026-05-19 11:01'
updated_date: '2026-05-19 11:32'
labels:
  - agent
  - logging
dependencies:
  - TASK-36.3
parent_task_id: TASK-36
priority: high
ordinal: 4000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Remove the `hick:`-namespaced session file writing from `iron-peak` and `copper-ledge`, and wire `XmlSessionLog` + `XmlDiagnosticLogger` in all four agent binary crates.

## Depends on

TASK-36.3 — `ReactLoop` and `McpClient` now emit via `SessionLog`; no session writing should remain in the TUI layer.

## Binary crates in scope

- `iron-peak` — has TUI session recording (`<hick:session>`, `<hick:command>`, etc. in `src/tui.rs`)
- `copper-ledge` — same pattern in `src/tui.rs`
- `hick-agent` — uses `env_logger::init()`
- `hick-plan-runner` — uses `env_logger::init()`

## Changes per binary

**iron-peak and copper-ledge:**
1. Remove all `log_*` methods and the `log`/`log_display`/`session_action_index` fields from the TUI struct
2. Remove `env_logger` dependency
3. In `main()`: construct `XmlSessionLog` from the `--log-path` (or equivalent) argument, wrap it in `Arc`, call `log::set_boxed_logger(Box::new(XmlDiagnosticLogger::new(log.clone())))`, pass `log` to `ReactLoop::with_session_log()`
4. The TUI struct no longer writes any XML — it only renders the terminal

**hick-agent and hick-plan-runner:**
1. Replace `env_logger::init()` with `XmlDiagnosticLogger` wired to an `XmlSessionLog` (or `NullSessionLog` if no log path is configured)
2. Remove `env_logger` dependency

## xml_escape

The current `xml_escape` helper in the TUI files can be deleted — XML escaping is now `XmlSessionLog`'s responsibility.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 iron-peak TUI struct has no log fields, no log_* methods, and no xml_escape helper
- [ ] #2 copper-ledge TUI struct has no log fields, no log_* methods, and no xml_escape helper
- [ ] #3 All four binary crates replace env_logger::init() with XmlDiagnosticLogger wired to XmlSessionLog
- [ ] #4 env_logger removed from Cargo.toml in all four crates
- [ ] #5 Session XML files produced by iron-peak and copper-ledge use log: namespace and log:session root element, not hick: namespace
- [ ] #6 cargo test --workspace passes
<!-- AC:END -->
