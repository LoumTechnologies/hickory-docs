---
id: TASK-58
title: 'feat: load API keys from ~/.hick/keys.toml user config file'
status: Done
assignee: []
created_date: '2026-05-21 11:21'
updated_date: '2026-05-21 11:25'
labels: []
dependencies: []
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Allow users to store LLM API keys in `~/.hick/keys.toml` so they don't need to set environment variables. Keys in the file act as fallbacks — environment variables and `--api-key` CLI flags still take priority.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 ~/.hick/keys.toml is parsed at startup with fields: anthropic_api_key, openai_api_key, xai_api_key
- [x] #2 Keys from the file are used when the corresponding env var is not set
- [x] #3 Environment variables take priority over keys.toml
- [x] #4 --api-key CLI flag continues to take priority over both
- [x] #5 Missing file is silently ignored; malformed file emits a warning
- [x] #6 Startup status message shows which providers were loaded from keys.toml
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added `UserKeys` struct to `config.rs` with `load()` and `key_for()` methods. At startup in `main()`, keys from `~/.hick/keys.toml` are injected into the process environment (via `set_var`) only when the corresponding env var is absent or empty — same pattern as `dotenvy`. A status message is emitted on the agent info channel listing which env vars were populated from the file. Three new unit tests cover the key cases: empty-string filtering, missing file, and valid file parsing.
<!-- SECTION:FINAL_SUMMARY:END -->
