# The LSP Channel Degrades To Structural Or Null Answers, Never Errors

Given a doc WebSocket with the 0x02 LSP bridge (api.md v0.3), when a child
language server (pyright, rust-analyzer, …) or the hick-lsp binary itself is
unavailable, then every client request still receives a JSON-RPC response
with a `result` (structural copy/paste answers where the parser can provide
them, `null` otherwise) — never an `error` frame, a dropped request, or a
closed channel. Definition/reference targets inside generated outputs are
translated through the last run's provenance to `hick:///` source
coordinates when byte-mappable, else returned as `hick-output:///` targets.

---

Last LLM verification:
- Date: 2026-08-06
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `crates/hick-lsp/src/backend.rs` — `child_request` swallows
  missing/failed children (`None`, logged), falling back to
  `crates/hick-lsp/src/structural.rs` copy/paste answers;
  `apps/server/src/ws.rs` `handle_lsp_frame` answers requests with
  `result: null` when the session cannot start or the child died;
  `apps/server/src/lsp.rs` `OutputTranslator` does the provenance mapping
  with an explicit `hick-output:///` fallback.
- Test coverage: `apps/server/tests/integration.rs`
  (`ws_lsp_channel_degrades_gracefully_without_child_servers`,
  `ws_lsp_references_round_trip_in_same_virtual_file`,
  `outputs_nav_definition_maps_back_to_copy_block`) plus
  `crates/hick-lsp` unit tests for reference/position translation
  (`backend::tests`, `structural::tests`) and
  `apps/server/src/lsp.rs::tests` for the provenance fallback.
