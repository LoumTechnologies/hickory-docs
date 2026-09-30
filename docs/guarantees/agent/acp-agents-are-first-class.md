# ACP Agents Work In The Agent Pane And Leave A Hickory Record

Given an installed ACP v1 agent, when it is selected in the Agent pane, then
Hickory launches its executable directly in the workspace, negotiates its
capabilities, and connects a private loopback MCP server containing Hickory's
existing document tools. The downloaded desktop executable also supplies the
stdio proxy; a separate `hick` installation is not required.

The agent owns login, available models, access modes, and its command sandbox.
Hickory does not inject its stored provider keys. The pane offers the adapter's
agent-owned authentication methods, configuration options, tool activity, and
exact permission choices. Permission replies are validated against that request
and are consumed once. Changing configuration while a turn runs is refused.

A missing known adapter offers installation when npm is available. Catalogue
installs pin Codex ACP 2.0.1 and Claude Agent ACP 0.81.2. A custom command has an
executable and an argument array; no shell evaluates that array.

Each turn records its prompt, parent, actual ACP backend, selected model,
answer, exposed reasoning, reported activity, and permission decisions in one
closed Hickory session document. Stream checkpoints preserve interrupted prose
and are compacted after the canonical assistant is saved. ACP shell activity
is evidence, never an executable cell. Unreported provider usage has no invented
price. The adapter's remote session id is recorded locally and used to load a
conversation after restart when loading is supported.

When the user stops a turn, pending permission requests are cancelled and a
`session/cancel` notification is sent. If the process does not finish within
three seconds, it is killed. The persisted turn remains stopped after restart.
An app-interrupted turn is recovered as stopped, with its saved evidence.

Hickory MCP edits reach the live editor. They are applied against a captured
CRDT revision so concurrent typing outside the replaced region survives.
Generic ACP file access is confined to canonical workspace paths. Generic writes
refuse generated paths and require a preceding read of an open document.
Hickory-handled file reads/writes produce the existing context evidence.

Codex rewind uses its reported message id and AIR fork-point metadata. The
forked session is resumed before prompting. The abandoned branch remains in the
same Hickory conversation. A generic fork capability alone never implies exact
rewind; unsupported adapters must start a new thread.

---

Last LLM verification:

- Date: 2026-09-30
- Reviewer: Codex
- Result: partially verified
- Evidence: `serve/acp/{client,transport,config,record,mcp,workspace}.rs`,
  `serve/agent.rs`, `mcp.rs`, desktop and CLI proxy entry points;
  `AcpControls.tsx`, `ChatDock.tsx`, `AcpRecord.tsx`, `AgentSettings.tsx`.
- Tests: `tests/serve_acp.rs` covers final-token ordering, authentication,
  configuration, permissions, stop, restart, exact fork activation, and live
  document edits through the real MCP handlers. `record` and `workspace` unit
  tests cover interrupted text including closing-tag collisions, non-ASCII
  coordinates, and concurrent typing. `AcpControls.test.tsx` covers sign-in,
  configuration, rejecting permission, and the install affordance. Existing
  built-in-agent and external-MCP integration tests still pass.
- Live evidence: authenticated published Codex ACP 2.0.1 read, reverse-edited,
  and verified generated code, continued a conversation, forked before a later
  marker, returned NO_MARKER, and loaded the fork after an engine restart.
  The real installation route completed. Workspace check and all-target Clippy
  pass; the full web suite passes.
- Caveats: authenticated Claude, Windows/Linux desktop packaging, terminal-only
  authentication, native adapter shell edits, overlapping concurrent changes,
  and upstream model-service failures are not established by this test run.
  Native commands follow the adapter's own boundary; the ACP file confinement
  is not a sandbox for the adapter process. Terminal client capabilities are
  deliberately not advertised. Exact rewind depends on the Codex adapter's
  fork-point extension. Repository-wide codegen/file-length checks have
  pre-existing failures outside this change.
