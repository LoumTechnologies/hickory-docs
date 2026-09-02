# One Language Server Session Per Workspace

Given the local server with several documents and plain files open across
any number of windows and panes, when any of them asks a language question,
then there is exactly one `hick-lsp` session for the workspace — and so one
rust-analyzer, one pyright — shared by every socket: each connection's
requests are answered under its own ids, notifications reach every
connection, a later connection is told the server's capabilities on
subscribing, a file two panes hold open is opened once with the server and
closed only when the last of them lets go, and a connection that drops
closes only what it was the last to hold.

The session used to be per WebSocket connection, and a document has a
connection each. That was invisible while only documents asked, since a
document's code is staged in its own directory; it becomes a machine on its
knees the moment plain files ask, because each rust-analyzer indexes the
whole repository.

## Boundary

The session lives as long as the server process. Nothing shuts it down when
the last window closes; the desktop app ends the process, and `hick up` is
the person's own session.

---

Last LLM verification:
- Date: 2026-09-02
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hickory-cli/src/serve/lsp_bridge.rs` — `LspHub`
  (`subscribe`, `send`, `unsubscribe`; `HubState::dispatch` returns replies
  by remapped id and broadcasts the rest; `did_open_as_change` for a second
  opener); `crates/hickory-cli/src/serve/socket.rs` — `forward_lsp`
  subscribes on the first `0x02` frame, `run_workspace_socket` serves
  `?doc=workspace`; `crates/hickory-cli/src/serve/mod.rs` holds the hub on
  `LocalState.lsp`.
- Test coverage: `crates/hickory-cli/src/serve/lsp_bridge.rs::tests`
  (`two_windows_share_one_session_and_each_gets_its_own_replies`,
  `the_bridge_answers_a_real_document_with_diagnostics`).
- Caveats: a server-initiated request (`window/showMessageRequest`) is
  broadcast to every connection and each may answer; `tower-lsp` tolerates
  the extra replies. None of the servers hick spawns sends one today.
