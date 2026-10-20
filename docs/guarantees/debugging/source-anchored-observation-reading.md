# Observations read beside their captured source

Given a native DAP or browser interpreter session with captured UTF-8 source,
when its reading surface is expanded, then source excerpts and typed observations
share immutable revision references and evidence IDs. Requested breakpoints,
adapter-bound breakpoints and the actual paused statement are distinct facts.
Selecting a caller never moves the actual paused statement to that caller.
Values and explicit expression receipts identify their captured stop and frame;
invocation identity remains unknown when the provider cannot establish it.

The native UI and MCP debugger reads use `Live::capture_reading` and
`Reading::agent_projection`. Browser execution produces the same reading schema.
Workspace context includes explicitly expanded readings with those same evidence
IDs and captured byte ranges, rather than all captured events or visible pixels.
Source aliases come from the workspace root, never a basename/suffix guess.
Changed source and historical captures are labelled separately from captured
stops. Closing the reading removes its context selection; another mounted view
cannot clear an expanded view's selection.

Given a location handle and stop/frame/run scope, when a native located action
is requested, then Rust looks up the captured document coordinate and checks the
current source bytes, stop scope and actual top frame before movement. Unknown,
changed, wrong-frame and stale references fail. Display labels and added view
rows are never parsed as coordinates. Literal source text such as BREAKPOINT
HERE cannot create evidence or authorize an action. Observations never modify
Markdown. Reading evidence does not expand the Python agent's tool allowlist or
authorize evaluation. Existing evaluation paths retain their existing authority
and browser expressions continue to reject calls and getters.

Debugger records reuse `hick-observe::Run`, not another archival store. Logs,
profiles, output, exceptions and debugger records can project through a common
provider envelope with evidence/location references. Measurement semantics and
uncertainty are independent of source precision. Runtime stop/frame identities
are optional for non-debugger providers. Review evidence has PR/thread and
base/head/diff-side identity, without a fabricated runtime identity. Unmapped
and outdated review anchors remain explicit.

Historical captures, ordinary stepping, restart-frame and actual reverse
execution are distinct. Reverse execution is advertised only from runtime
capabilities. An older captured line never becomes a live reverse stop.

---

Last LLM verification:

- Date: 2026-10-18
- Reviewer: Codex
- Result: partially verified
- Originating requirement: GitHub #64 under #47; source/location contract #48,
  selected context #52, runtime store #54, output presentation #57, evaluation
  #53, profiling #65 and external review provider #66 remain separate concerns.
- Interpretation and assumptions: the companion reading contains captured code
  and observations together; existing editor gutters/inline values remain in
  place. DAP reports attribution, rather than proving semantic causality. A
  stop-bound frame is not assumed to identify a recursive invocation across
  stops. Scope checks are made when Rust resolves an action; external filesystem
  edits are not locked for the duration of an adapter request.
- Evidence: `crates/hick-observe/src/reading.rs` (`Reading`, `Scope`,
  `ProviderAnnotation`, `Run::provider_annotations`); `hick-dap/src/mapping.rs`
  captures source before launch; `hickory-cli/src/debug_reading.rs` shares the
  native producer and resolution; `serve/debug_bridge.rs` and `mcp.rs` consume
  it. `apps/web/src/debug/ObservationReading.tsx`, `observations.ts`,
  `browser/runtime.ts`, `browser/transport.ts` and `views/WorkspaceChat.tsx`
  implement rendering, explicit selection, capture and context projection.
- Witnesses/challenges: Unicode/BOM source and multiple generated-file fragments;
  a real debugpy-relocated blank-line breakpoint; repeated recursive frame scopes;
  unknown/fabricated marker handles; edited source and old stop/frame references;
  expression receipts surviving a reread of the same stop; browser calls/getters
  and unsupported reverse execution remain refused. The shared JSON fixture is
  consumed by Rust and the rendered browser component.
- Exclusions and unsupported data: no profiler instrumentation (#65), review
  ingestion/actions (#66), output-history renderer (#57), unrestricted Python
  debugger tools, automatic debugger-run archival, historical-run browser or
  reverse simulation is added. Native exception descriptions are retained when
  reported; full exception details and invocation identity remain explicitly
  unsupported. Runtime provider version/measurement uncertainty can be unknown.
  Selection is at the expanded reading level, without a new pinning policy.
- Executed checks: `cargo test -p hick-observe reading:: --quiet` (5 passed);
  `cargo test -p hick-dap --lib observation_tests --quiet` (1 passed);
  `cargo test -p hickory-cli --lib serve::debug_bridge::tests --quiet` (11 passed,
  including the real local debugpy adapter);
  `cargo test -p hickory-cli --lib observation_capability_tests --quiet` (1 passed);
  `cargo clippy -p hick-observe -p hick-dap -p hickory-cli --lib --no-deps -- -D warnings`
  passed. Web `npx tsc -b --noEmit` passed; focused `ObservationReading`,
  `browser/runtime`, `useDebugger` and `DebugStrip` Vitest checks passed.
  `npx playwright test -c playwright.embed.config.ts --project chromium -g
  'source observations share' --workers=1` passed (1 flow). `git diff --check`
  passed. Commands were run against the implementation content recorded below;
  the #64 handoff names the resulting commit. Broad regression checks and their
  status monitoring are left to the user.
- Human judgment: not recorded. The automated browser flow verifies interaction
  outcomes, not a human usability study. Agent request/action fixtures verify
  handle resolution and refusals, not a paid model task or token/latency benchmark.
  #53 can measure those outcomes without equating token savings with usability.
- Alternatives considered: separate UI/agent string reconstruction would lose
  trusted coordinates; parsing marker words would let source impersonate evidence;
  a new archive would compete with #54. These are rejected in favor of typed
  derivation over existing run records. This audit cannot certify that every
  hidden assumption or language-provider behavior has been examined.
- Falsifiers: conflating requested/bound/paused sites; accepting a stale or
  wrong-frame action; presenting old values as current; source text becoming a
  tool annotation; losing evidence IDs between the expanded view and context;
  or granting Python evaluation capability from received evidence.
- Dependencies: implementation is in the same commit as this guarantee. The #64
  handoff records its commit and checks. Changes to the named modules or shared
  fixture require renewed review; earlier verification is historical evidence.

Inspected implementation content hashes (SHA-256; stale on change):

- `crates/hick-observe/src/reading.rs`: `2b2ecc217e0cb4e9409a2e0c623c30e7a0fa23c68a38b4220d9f0abc9e890e5d`
- `crates/hick-dap/src/mapping.rs`: `64f555988f10f82dfa8d88cf4cfda012e06ffcff06218d25f02fd6f640ff9ab2`
- `crates/hickory-cli/src/debug_reading.rs`: `b0721ee5010fcbfa63d4086a24c12af34f92e6f95e8f79529f04aab0e050b115`
- `crates/hickory-cli/src/serve/debug_bridge.rs`: `29ad51c7aa1c9049a9231450a6667d0c7d0020359e75e69edeae380110870a56`
- `apps/web/src/debug/observations.ts`: `ecfaac8be5e2effd4de381ceca98c0ba2c09663bae4dbdce61a14d583cc616b7`
- `apps/web/src/debug/browser/runtime.ts`: `3f0c070cae57064f055772b2aff7728332cb0120ffbe4ad91f8adf66c5c343d7`
- `apps/web/src/debug/fixtures/reading.json`: `6b82f85eda88e3984aa2e3f10b3437343a7b694934981e836b023eef1ffa551d`

### Runtime and external-provider integration, 2026-10-19

The browser runtime history and debugger reading share provider evidence IDs with
selected context. Runtime output/log/exception/profile/debugger payloads retain
their native typed payloads. Nondebugger envelopes have no invented stop/frame;
source precision remains independent of profile aggregation and uncertainty.
Unknown provider versions remain unknown. External provider readings retain
repository/PR/thread/base/head/side identity without inventing runtime scope.
Expanded or pinned readings select context; receiving them grants no actions.
Authored bytes and debugger location resolution are unchanged.

The general provider view and context serializer support unavailable source,
unmapped evidence, historical and changed-source states. The runtime compatibility
view labels a runtime batch separately from a captured debugger stop. The
legacy debugger scope remains solely for revision-bound debugger actions.

Executed focused checks for this integration: five Rust reading/handle/provider
contract tests; real local debugpy relocated-breakpoint and stale-action test;
two frontend runtime-history tests and one external-review context contract test.
TypeScript passed. Browser interaction results are recorded in the issue handoff.
No human usability or paid model benchmark is claimed; #53 owns measurements.
The preceding implementation hashes are historical, not hashes of this update.

### Shared observation disclosure/pinning (issue #72)

Source-observation visibility and pinning now use explicit revision-bound
commands over the same state as native details/checkbox controls and the existing
selected-context projection. The root reader contains disclosure/capture state,
not annotation payloads. Expanded content mounts its own bounded UTF-8 JSON
reader for rendered location lines, annotations, unmapped evidence, provider
precision/measurement summaries and unsupported fields. It does not dump
unrendered whole-source bytes. Collapsing retires the detailed reader; pinning
continues to select captured evidence while folded, as the native view already
did. Stop/source/reading changes invalidate old handles. Selection replies do
not assert actual provider delivery, current edit authority or independent
verification; historical/changed flags are retained.

Five existing selected-context/pinning cases and one new mixed native/model
progressive-reading case pass. The latter checks closed discovery, real selected
context state, displayed checkbox/details state, exact rendered source line,
old-stop refusal, collapsed-reader retirement and native unpin cleanup. Strict
TypeScript/file-length/diff checks pass. These tests use a captured JSON fixture,
not a live debugger/provider/native package.
