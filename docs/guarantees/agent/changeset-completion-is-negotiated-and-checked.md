# Changeset completion is negotiated and checked

Given an ACP v1 peer that advertises `hickorydocs.com/changesets` version 1,
and a turn on an existing declared project, when work begins, Hickory freezes
starting output tree A and binds it to that turn's boundary. The peer can inspect
A and current output B and submit a literate `.md` closure over MCP or the
negotiated ACP method. Both routes use the same verifier. The harness owns A;
the peer cannot reset it halfway through a turn.

Submission evaluates the supplied closure entirely in memory over A. Acceptance
requires equality of the complete evaluated output inventory with B, including
file bytes, kinds and executable bits. Missing or extra outputs fail. A stale
fingerprint, invalid closure, or wrong boundary refuses submission. Diagnostics
are returned to the agent for repair. Before completion, any observed later output change invalidates
acceptance. A completed artifact remains a record of its captured B. Failed submissions do not preserve an earlier acceptance.

A negotiated peer ending a turn without a current accepted artifact receives
up to three repair prompts. Exhaustion is an error; cancellation stops the turn
without requesting an artifact. Acceptance and the completed capture are checked
again before reporting success. The artifact and repair evidence are recorded
in the conversation. Peers without the capability retain ordinary completion
and may voluntarily call the MCP tools. Such completion never claims acceptance.
A turn without a declared project cannot negotiate a usable output boundary.

Standalone MCP exposes `begin_changeset` to freeze one task's starting project
before edits, followed by the same status/submission tools. Reset requires a new
MCP client/server lifetime. Standalone MCP does not own an external harness's
completion lifecycle.

Accepted artifacts appear automatically in the Agent pane. Their patch views
use ordinary document rendering with the supplied closure and captured A;
project language tools and debugging receive evaluated full inventories. Accepted
records are read-only. Intermediate proposed-change reviews have no captured
project tools. Native tool materialization retains its existing lazy behavior.

Acceptance checks source-output equality. It does not verify explanatory prose,
runtime behavior, approval, or exclusive authorship. Existing ambiguous-write
resolution remains required before incorporation. No protocol capability claims
to control native filesystem writes.

---

Last LLM verification:

- Date: 2026-10-17
- Reviewer: Codex
- Result: partially verified; implementation inspected, executed evidence below.
- Requirement: the user approved a negotiated ACP changeset contract, implemented
  through MCP first, with repair feedback and existing-agent compatibility.
- Evidence: `changeset.rs::{Contract,verify_sources}`, `mcp.rs`, `acp/mcp.rs`,
  `acp/client.rs`, `acp/mod.rs`, `agent_projects.rs`, `ChangesetDocument`,
  `AgentProject`, and `elements/patch/Editor`.
- Assumptions: the declared inventory defines the captured output boundary;
  source-document incorporation is a separate equality. A fingerprint describes
  an observed tree, not a filesystem transaction or exclusive writer identity.
- Witnesses: `changeset::tests`, real HTTP/subprocess `agent_changesets.rs`,
  `literate_agent_projects.rs`, and web patch/changeset tests.
- Challenges: stale status, wrong turn, missing output, later write, cancellation,
  repeated failed completion, and legacy peers. A peer completing successfully
  with a mismatching artifact would falsify this guarantee.
- Executed checks: `just test-agent-changesets` passed three verifier unit tests,
  six real ACP/MCP process flows and four existing capture/incorporation flows.
  `just test-agent-changesets-web` passed typechecking and 65 tests;
  `just test-agent-audit-web` passed typechecking and 77 tests.
  `just check-agent-changesets` passed formatting, the source-length ratchet
  and focused Clippy with warnings denied. The broader ACP regression result
  is recorded in the continuation below. No paid-model or desktop visual
  acceptance is inferred from these checks.
- Dependencies: the named verifier, capture, bridge and renderer paths; edits to
  those paths invalidate this review. No provider behavior beyond the fixture is
  established.
- Human judgment: the design was approved; implementation has not been reviewed
  by the user. Built-in/Python harness finalization loops and third-party adapter
  adoption remain outside this first ACP/MCP implementation.

## Inspected dependency hashes

SHA-256 of the implementation checked in this continuation:

- `crates/hickory-cli/src/changeset.rs`: `a51e7cf9a59cc24789e2de3ef8068bd6fcd85a7b7fd73a808625bcf8790968a6`
- `crates/hickory-cli/src/changeset_mcp.rs`: `0007303e90f51069e27b1724a0f423a83593816bdfb64307d1b3a34d719e5221`
- `crates/hickory-cli/src/serve/acp/client.rs`: `810190c65423b1001ae5efa3ffc7ee72c90a578fe2a9a460e9fed687d42351b0`
- `crates/hickory-cli/src/serve/acp/mcp.rs`: `166aab078db42a303b71500a43f7fc6db9e9c2746824de49de718db966d0279e`
- `crates/hickory-cli/src/serve/acp/mod.rs`: `601cebead21db3648063b5483dcfa732467b6393639732ffa9a44e4af3f422f2`
- `crates/hickory-cli/src/serve/agent_projects.rs`: `29851fb56ceb1d633e095b795abeeb2991de47037e155c6a2006615199589ff6`
- `apps/web/src/components/ChangesetDocument.tsx`: `046c725bfaa0f1950b26806d15649ada0c665d1fcf616d89b965576ac6bdecbc`
- `apps/web/src/elements/patch/Editor.tsx`: `a9f8cbf6038507860bd8aa50a5263d3b22160b259c401c899e6e643b2bde89fc`

## Final regression results

The native stages of `just test-acp` passed 44 tests; its two paid/live Codex
tests were intentionally ignored. An earlier full-web development run reported
an included-document debug-source mismatch while that routing was being corrected.
The corrected displayed-document identity and asynchronous assertion passed
`just test-agent-changesets-web`, then final `just test-web` passed typechecking
and all 2,180 tests in 236 files. No live-provider or desktop visual acceptance
is claimed.
