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

Conversation edit policy lives in `acp::edits`. `read_buffer`/`edit_buffer`
address the editor snapshots sent with a turn; `read_doc`/`edit_doc` recognize
those same buffers. These names never change with Review versus Auto-accept.
A live-buffer write waits for the UI to apply its ordinary editor transaction
and acknowledge it. Existing file/output tools preview through the shared
hashline resolver, wait for approval, recheck the reviewed surface, then run
the existing tool. Do not hold workspace, server, or recorder locks while
waiting for review. Stop cancels pending decisions. `acp-edit-mode` in the
conversation record restores the policy; it is not a global UI preference.

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

The UI uses `ConversationEditor` over the shared `DocumentEditor`, with a
transaction-protected prefix and an editable response suffix. Its reading
selects the current branch from recorded bytes and registers remapped receipts
with `lensSources`. Owner updates preserve the draft and bypass the edit filter;
ordinary keyboard and widget transactions cannot change the protected prefix.
`preserveBytes` disables paragraph unwrapping in immutable and live readings.

`AgentChanges` publishes proposals to the local reading registry. Review tabs
use `DocumentComparison` and `comparisonField`; acceptance applies only to the
identified original editor before acknowledging the tool. Commit readings use
that same editor over immutable Git blobs from `GET /api/git/reading`. Neither
kind of reading joins a document room or has a save/execution binding.
