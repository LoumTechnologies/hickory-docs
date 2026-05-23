---
id: TASK-65
title: 'LSP diagnostics extension: real-time code intelligence as agent tools'
status: To Do
assignee: []
created_date: '2026-05-23 02:28'
labels: []
milestone: m-7
dependencies: []
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a `hick-agent-lsp` extension crate that exposes Language Server Protocol operations as first-class agent tools. After the agent writes code it currently has no way to verify correctness without running a full build. LSP diagnostics close this loop: write → get diagnostics → fix → write, without a build round-trip.

This is a new extension crate following the existing pattern (`hick-agent-hick-edit`, `hick-agent-mcp-xml`, etc.). Active when `[extensions] lsp = true` in `agent.toml`.

Phase 1 (this task): diagnostics and references — the highest-value 20% of LSP surface.
- `lsp_diagnostics(file)` → errors, warnings, hints for a file
- `lsp_references(file, line, col)` → all reference locations for the symbol at position

The crate manages a pool of LSP server processes (one per language, lazily started, idle-reaped). Language → server binary resolved from PATH or project config (e.g. `rust-analyzer`, `pyright`, `typescript-language-server`).

Key reference: `tower-lsp` or direct `lsp-types` + `tokio::process` for the server pool.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 hick-agent-lsp compiles as a new extension crate following the existing extension pattern
- [ ] #2 LspServerPool manages one LSP process per detected language; processes start lazily on first use and are reaped after a configurable idle timeout
- [ ] #3 lsp_diagnostics(path) opens the file in the LSP server (textDocument/didOpen if needed), requests diagnostics, and returns them as structured XML observations
- [ ] #4 lsp_references(path, line, col) calls textDocument/references and returns a list of (file, line, col) locations
- [ ] #5 Both tools are registered in the agent and documented in the system prompt fragment
- [ ] #6 Language server binary resolution: check project .lsp-config or agent.toml [lsp.servers] first, then PATH
- [ ] #7 If no LSP server is available for a file's language the tools return a clear 'no LSP server configured for <language>' message rather than an error
- [ ] #8 Integration test with rust-analyzer (if available in CI): write a Rust file with a type error, call lsp_diagnostics, verify the error is reported
- [ ] #9 agent.toml [extensions] lsp = true/false gates the extension at startup
<!-- AC:END -->
