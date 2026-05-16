---
id: TASK-27
title: >-
  Config-driven agent runtime: compose extensions from a TOML file without
  writing Rust
status: To Do
assignee: []
created_date: '2026-05-16 12:57'
labels:
  - agent
  - config-driven
  - path-1
milestone: m-1
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement a "universal" agent binary and supporting runtime that lets users enable or disable extensions via a TOML config file, without writing Rust or recompiling. This is Path 1 of the three agent-building paths.

## The three paths

1. **Config file** (this task) — drop a TOML in your repo, run `hick-agent --config agent.toml`. No Rust required.
2. **Custom SDK** — import `hick-agent-sdk` + chosen extension crates, write a minimal ReAct loop (~50 lines of Rust). Full customization.
3. **Published binary** — install the pre-built binary that ships the winning extension combination from A/B evaluation.

This task implements Path 1 only. Paths 2 and 3 already exist in the architecture.

## Why a universal binary, not dynamic plugin loading

Dynamic `.so` loading is complex and fragile. Instead: compile a single "kitchen sink" binary (`hick-agent`) that links every first-party extension crate, and use a runtime composition mechanism to activate only the extensions listed in the config. New first-party extensions get added to this binary as they're promoted from evaluation. Third-party extensions remain a Path 2 concern (they require writing Rust).

## Config format

```toml
# agent.toml — place at repo root or pass via --config
[agent]
model = "claude-sonnet-4-6"
extended_thinking = false

[extensions]
script_first = true
hick_edit = true
# future extensions listed here as they are added

[runtime]
max_turns = 30
timeout_secs = 600
```

Unrecognised extension keys are an error (not silently ignored), so typos are caught at startup.

## Runtime composition

The SDK's extension trait system must support building a composed agent from a list of enabled extension identifiers rather than requiring each extension to be hard-wired at compile time. The binary calls something like:

```rust
hick_agent_sdk::Builder::new()
    .with_extensions_from_config(&config.extensions)
    .build()
    .run()
```

Each extension crate registers itself with a string identifier. The builder activates only those matching the config.

## Eval system integration

Once this binary exists, TASK-23.5 (eval coordinator) can use `extensions = [...]` in `eval-plan.toml` to test arbitrary combinations without compiling a new binary crate per combination. This is what enables the coordinator to handle combinatorial explosion gracefully — the search space is no longer gated by the build system.

## What this does NOT do

- Does not support third-party/user-written extensions (that's Path 2)
- Does not replace the named binary crates (copper-ledge, iron-peak) — those remain for Path 3 and for precise reproducible A/B runs
- Does not expose every SDK tuning knob — advanced users who need that should use Path 2
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A `hick-agent` binary exists that links all first-party extension crates.
- [ ] #2 Running `hick-agent --config agent.toml` starts an agent with exactly the extensions listed in the config enabled.
- [ ] #3 An unrecognised extension key in the config produces a clear error at startup, not a silent no-op.
- [ ] #4 Enabling zero extensions produces a functional baseline agent (sdk only).
- [ ] #5 The config format is documented in a README or man page shipped with the binary.
- [ ] #6 TASK-23.5 eval coordinator can specify `extensions = ["script-first", "hick-edit"]` in eval-plan.toml and have the coordinator invoke `hick-agent --config <generated-config>` without requiring a purpose-built binary.
- [ ] #7 Adding a new first-party extension to the universal binary requires changes in exactly two places: the extension crate itself and the `hick-agent` binary's Cargo.toml.
<!-- AC:END -->
