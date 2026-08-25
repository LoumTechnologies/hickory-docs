# A Running Agent Can Be Stopped

Given an agent turn running on a document, when the reader presses the Stop
button in the chat dock (a stop-square icon, shown in place of Send for the
whole time a turn is live and never disabled while one is), then
`POST /api/docs/:id/agent/stop` sets the run's cancel flag and the loop halts
at its next seam — **between streamed chunks**, dropping the provider stream,
which is what stops the token spend on a model looping mid-generation; between
turns; and after a script or tool finishes — the session file still closes
honestly (spend recorded, end marked), the turn finishes with status
`"stopped"` and a terminal frame on the run channel, and the dock renders it
as the user's own quiet act, never as a red error. A stopped turn keeps no
answer, so the next message replays the branch as if it never ran.

The reason: the incident that forced this was a model degenerating into an
endless `<sh:exec></sh:exec>` stream, and the only cord was killing the whole
program. A tool that bills by the token owes its user a hand on that cord —
and the mid-stream check is the load-bearing part, because a runaway spends
between turns' checkpoints, not at them. Scripts are deliberately not killed
half-way (they are bounded by their own limits, and a half-run script leaves
a workspace the session file does not describe); the stop lands right after,
with the observation recorded.

---

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
