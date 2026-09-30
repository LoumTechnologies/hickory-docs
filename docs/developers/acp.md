# Work on the ACP agent extension

For engineers changing Hickory's local agent pane or adding an ACP adapter.

ACP lives in `crates/hickory-cli/src/serve/acp`, above the existing agent hub.
It is independent of `LlmClient`, the language parser, and element registration.
The built-in agent remains the `builtin` backend.

`POST /api/docs/:id/agent/acp` launches or loads the selected agent. The existing
`POST /api/docs/:id/agent` starts turns with a `backend` field. The existing
run WebSocket channel carries tokens, reasoning, ACP updates, and terminal
status. REST snapshots recover state if a terminal frame is missed.

The transport has one stdout reader. After initialization it delivers updates
and responses through one ordered queue, so a prompt response cannot overtake
the last token. Reverse requests run separately so waiting for permission does
not block later updates. A workspace gate serializes MCP and generic ACP file
operations; a separate recorder gate prevents two session writers.

The private MCP bridge reuses `mcp::Server`, an explicit workspace root, and
the conversation's recorder. Capture the live CRDT revision before tools run,
then apply their source changes as operations against that revision. Do not
replace the live room with a stale disk snapshot after a whole turn.

Codex's `session/fork` creates an inactive session. **Resume it before prompting.**
The fork point is the recorded ACP message id, carried in the adapter's AIR
metadata. Advertising generic fork support does not establish exact rewind.

Run `just test-acp` for deterministic protocol, MCP, built-in-agent, and UI
regressions. The Rust ACP fixture is an explicit protocol peer for races,
permissions, and cancellation; it is never a production login fallback.
`just clippy-all` checks examples and test targets as CI does.

For an authenticated live Codex test, set `HICKORY_ACP_LIVE_COMMAND` to the
absolute path of an installed `codex-acp` executable and run
`just test-acp-live`. This test uses the actual account and buys real model
turns. It edits only a temporary fixture, then verifies exact rewind and
restart loading. Normal tests never invoke that adapter.

Adapter catalogue versions are pinned in `acp::install`. Upgrade a pin only
with a live smoke test. The current conformance surface uses ACP v1;
unsupported protocol versions produce an actionable connection error.
