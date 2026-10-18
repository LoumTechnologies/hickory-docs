# A running agent can be stopped

Given a hosted Python turn, Stop cancels in-flight inference or review and
terminates the confined process group. A pending proposal is not approved by
cancellation. The session records the failure/cancellation, the turn has status
stopped and no successful answer, and the existing WebSocket terminal event
restores the Send control. Standalone Python also accepts an injectable cancel
callback and its development CLI handles Ctrl-C.

Partial provider usage received before a cancelled inference is not currently
emitted as a completed usage receipt. This does not promise that a provider
immediately stops billing when its HTTP stream is dropped.

---

Last LLM implementation review (2026-10-18): partially verified. Requirement,
choices, limitations and executable witnesses are in
[the migration audit](../audits/2026-10-18-python-agent-ownership.md).
Evidence: Python `core.py`, `runtime.py`, `hosted.py`; Rust `python_agent.rs`,
`serve/python_agent.rs`, existing changeset verifier and confinement profile.
Executed checks: see the audit. Human correctness judgment: not recorded.

## Historical evidence for the retired implementation

The following notes predate the migration. Deleted Rust-runtime paths and old
check results do not verify the current implementation.


Last LLM verification:
- Date: 2026-08-25
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence:
  - Loop: `crates/hickory-agent/src/react_loop.rs` — `AgentConfig::cancel`
    (an `Arc<AtomicBool>`), checked per streamed chunk in
    `stream_completion`, at each turn top, and after every script and tool;
    a cancelled run records `Usage` + `End` in the session, emits
    `AgentEvent::Error` with the exported sentinel `STOPPED_BY_USER`, and
    fails with that exact message — the contract callers match on.
  - Server: `crates/hickory-cli/src/serve/agent.rs` — `AgentHub::cancels`
    (flag registered before the spawn, so a stop cannot race the start and
    find nothing), `AgentHub::stop`, the `stop_turn` route
    (`serve/mod.rs`: `POST /docs/{id}/agent/stop`), `finish` mapping the
    sentinel to turn status `"stopped"`, and the terminal WS frame carrying
    `"stopped"`.
  - Dock: `apps/web/src/components/ChatDock.tsx` — the Stop button
    (`StopMark` icon, `components/icons.tsx`) replaces Send while `running`,
    is live until pressed, shows "Stopping…" once pulled, and a `"stopped"`
    turn renders as muted words (`.chat-stopped`), never `.chat-error`.
    Client: `api.agentStop` (`api/client.ts`); `RunStatus` gained
    `"stopped"` (`api/types.ts`); route documented in
    `docs/specs/freeform/api.md`.
- Caveats — what LLM review could NOT establish:
  - What the provider bills for an aborted stream is the provider's ledger:
    dropping the connection stops further generation, but tokens already
    generated are billed, and the run's own usage record for the cut turn is
    lost with the stream — the recorded spend can under-report the final
    partial turn.
  - The CLI (`hick agent`) has no stop button; Ctrl+C killing the process is
    its cord, unchanged here.
- Test coverage: `crates/hickory-agent/tests/stop_button.rs`
  (`stop_cuts_a_runaway_stream_mid_generation` — an endless
  `<sh:exec></sh:exec>` stream, the real incident's shape, cut within a
  bounded time; `a_stop_before_the_first_call_never_reaches_the_model`);
  `crates/hickory-cli/tests/serve_agent.rs`
  (`the_stop_route_halts_a_runaway_turn_and_records_a_stop` full-stack over
  HTTP, `stopping_an_idle_document_names_the_situation` for the 409);
  `apps/web/src/components/ChatDock.test.tsx` ("stopping a run": the button
  replaces Send, stays live, pulls the cord once, and a stopped turn renders
  quietly).
