# Explicit UI action dispatch checks scope, state, policy and retries

Given a command explicitly registered with `UiActions`, observing its parent
returns a bounded descriptor with subject, schema, current state and an opaque
action handle. A structured invocation through that registry checks workspace,
window, current actor policy, current revision, enabled state and exact typed
arguments before calling its registered semantic handler. Quoted text, source
location handles and forged handles do not execute commands. Observation identity
is distinct from executable identity and does not grant an edit baseline.

Within one registry lifetime, identical request retries share the same result.
Conflicting key reuse and concurrent execution on a subject are rejected.
Intent is retained before side effects; state and policy are checked again after
that asynchronous boundary. Actual outcomes are recorded separately. The dispatcher
does not infer completed execution from intent or a model declaration.

Cancellation propagates an owned request's AbortSignal. The executor determines
the actual outcome; cancellation does not assert rollback. Failed result retention
reports uncertain side effects instead of retrying execution.

The application registers cell rail actions, More actions menu toggles,
environment disclosure, dependency setup and manager selection, debugger control,
runtime/profile reads, terminal supervision, editor reading/selection/folds,
source-guarded document runs, review disclosure and anchored coach controls.
Human and agent
adapters use those controllers. The Python supervisor and ACP private MCP bridge
forward structured UI tools to the window captured at Send. They supply the actor
from the host session/turn, and accept replies only for that window's outstanding
request. Caller abandonment requests cancellation in the same window. A closed
environment group unregisters its children until explicitly opened.

The host retains immutable intent/result files in WorkspaceStore. A result must
match its intent's action, arguments, actor, scope and request ID. Python captures
UI tool output into the existing context store and records host-observed tool
results; request delivery remains governed by the existing lineage protocol.
An environment setup action returns its real terminal job identity; a cell rail
dispatch does not claim runtime completion or verification.

**Universal UI parity is incomplete.** Native operations, most remaining controls,
full job inspection/cancellation and the coach/launch witnesses remain outstanding.

---

Last LLM verification:

- Date: 2026-10-19
- Reviewer: Codex
- Origin: #72, #73, #74 and #75.
- Implementation: `apps/web/src/lib/{uiActions.ts,uiActionBridge.tsx}`,
  `editor/CardRail.tsx`, `components/MoreActions.tsx`,
  `environments/{EnvironmentPanel.tsx,useEnvironments.ts}`,
  `crates/hickory-cli/src/serve/ui_actions.rs`, Python Review and ACP MCP adapters.
- Focused coverage: `apps/web/src/lib/uiActions.test.ts`; disclosure/invocation,
  duplicate labels, Unicode, stale source/stop, disabled actions, policy denial,
  forged/quoted/cross-window/unregistered handles, typed parameters, retries,
  cancellation ownership, concurrent actions, argument capture, changed state
  during intent persistence and failed intent persistence.
- Evidence limits: dispatcher fixtures, real React controls with injected
  callbacks, real host state/receipt storage, and a real confined Python worker
  over HTTP with scripted provider/UI responses. No paid-provider, real dependency
  installation, native, release or deployment witness is claimed.
- Assumptions: trusted code registers commands and host adapters supply
  actor/scope. Host commands must independently validate current authoritative
  revisions and policy atomically. Parameter schemas here are intentionally
  primitive required fields; resource/upload/selection schemas need adapters.
- Exclusions: exactly-once execution across crashes, cross-window coordination, atomic host writes,
  job cancellation after the dispatch result, general UI/source correspondence,
  opaque ACP provider delivery, token
  accounting beyond observation payload bytes, general read-surface integration and exhaustive UI registration remain
  integration work. Retired references require observing the parent again.
- Human judgment: no human correctness or launch-readiness approval recorded.
- Broad regressions and post-commit monitoring are left to the user.

Executed focused checks (2026-10-19):

- Dispatcher, real control/transport and environment adapter tests: results below.
- Strict TypeScript checking of only `uiActions.ts` and `uiActions.test.ts`:
  passed with ES2022/DOM, bundler resolution and unused-code checks.
- `node scripts/inventory-ui-actions.mjs --check`: passed. The declaration index
  is reproducible for 434 source files, 907 controls and 457 imperative call sites.
- Real confined-Python integration:
  `python_agent_uses_originating_window_ui_tools_with_retained_context`: passed
  with `HICKORY_AGENT_TEST_PYTHON=apps/agent/.venv/bin/python`. The loopback server
  and confinement required execution outside the outer filesystem sandbox.
- Initial four focused Rust host-broker tests passed, including immutable
  receipt persistence. Follow-up cancellation and disclosure checks are below.
- No real provider, native or deployed flow was executed.

Follow-up focused checks (2026-10-19):

- Dispatcher/React bridge/environment action tests: 24 passed across
  `uiActions.test.ts`, `uiActionBridge.test.tsx` and `environmentActions.test.tsx`.
  Includes actual menu disclosure, shared rail callbacks, returned setup job IDs,
  disabled/stale setup, closed-group child retirement and window disposal.
- Existing affected environment controller/panel checks: 5 passed.
- `cargo test -p hickory-cli --lib serve::ui_actions::tests --quiet`:
  5 passed. Includes window reply isolation, immutable receipts, caller cleanup,
  bounded arguments, closed windows and cancellation notification attribution.
- The confined-Python HTTP test above passed with the broker and context-reference
  integration. Its scripted endpoint is explicitly separate from component tests;
  no claim of a witnessed full native/provider customer workflow is made.
- Focused strict TypeScript checking of the dispatcher, bridge, environment
  adapters/tests and WorkspaceChat entry passed with ES2022, DOM/DOM.Iterable,
  bundler resolution and unused-code checks.

Current adapter verification (2026-10-19):

- Dispatcher: 18 focused tests, including bounded arrays and integer inputs.
- Terminal adapter: two tests for real controller job identity and prompt/turbo
  human-input boundaries; injected controller, not a native terminal witness.
- Debug/runtime/editor adapters: five focused tests use actual controllers or
  connected CodeMirror with injected runtime data; no provider assertion.
- Saved execution compares the expected root SHA-256 before parsing/execution
  and retains captured source identity separately from requested identity.
  The focused Rust source guard test passed.
- Changed application entry points passed strict TypeScript checking with the
  repository's interpreter declaration included.
- Review agent comments/investigations now carry agent-declared origin.
  Agent resolution is refused by both the adapter and backend; local declarations
  remain unauthenticated. This supersedes the earlier user-only write limitation.

Disclosure and controller follow-up (2026-10-19):

- UiActionsVisibility retires hidden workspace/spatial-pane registrations without
  destroying editor state. The focused bridge test requires fresh handles after
  reopening. All five bridge tests passed.
- Debugger discovery carries compact state only; explicit detail reads are bounded
  to 8 KiB. Runtime run/source indexes are paged; unknown run/invocation identities
  are refused. Three debug tests and one runtime test passed after these changes.
- FeatureSettings shares its actual open/close controller with the registry;
  LineageOptions only mounts inside the open portal. The real persistence/disclosure
  test and affected feature-settings test passed. App navigation shares its
  native-menu handler; focused strict TypeScript checking passed.
- The combined focused App navigation run initially reported an unhandled async
  teardown rejection after both assertions passed. Each selected test then passed
  separately with no reported unhandled error. The combined-run timing cause was
  not established; these results do not certify broad regression behavior.

Private revision and prompt correction (2026-10-19):

- Retrospective finding: initial debug/runtime/terminal/review/context/config/tree
  adapters serialized private state into revision strings. This could disclose
  hidden patch text through metadata despite bounded explicit reads. Those adapters
  now use useUiRevision's opaque tokens. Earlier tests did not challenge this leak.
  The new context test asserts hidden original/candidate text is absent from
  discovery. No actual paid-provider exposure was performed in this session.
- The terminal host now rechecks its current recognized prompt before agent input,
  bounds agent input bytes and preserves refusal as requires_input. A real PTY
  test passed: stale-UI-style agent input was refused, then actual human input
  answered the waiting program. Prompt recognition remains best effort, not a
  general external-process authorization protocol. Enabling turbo always requires
  human input through the registered adapter, independent of cached UI state.
- Current focused follow-up: two context tests, three terminal adapter tests, three
  existing agent-settings tests plus one new shared-handler test passed. Three
  selected setting/title tests passed. Command-search callback/policy and selected
  existing behavior tests passed (three); source-guarded folder replacement and
  spatial navigation tests passed. Broader regressions remain unrun.

### Meeting import adapter — 2026-10-19

The disclosed importer accepts transcript name/text through the same ingest
controller used by the selected local file. No chooser impersonation is needed.
Both routes enforce extension, UTF-8 decoding and a 1 MiB byte limit before host
submission. A synchronous in-flight guard prevents overlapping imports. Abort
before submission prevents upload; later cancellation does not roll back retained
evidence. Discovery contains opaque revisions, not transcript text. Evidence is
read explicitly in JSON UTF-8 byte pages of at most 8192 bytes; navigation uses
current retained utterance identities. This does not grant microphone consent or
remote transcription authorization.

Executed focused checks: MeetingImport.test.tsx (2 passed), including Unicode
retained evidence/navigation and invalid/multibyte oversized input with no host
write. These component checks mock the host parser and microphone component;
they are not a native capture or real-provider witness.

### Context projection controls — 2026-10-19

The disclosed projection panel provides current document identities in pages of
at most 64, off/expanded/pinned selection, capture, and preview reads in JSON UTF-8
byte pages of at most 8192. Hidden choices unregister. Preview text stays out of
opaque discovery revisions. Human checkboxes use the same selection command;
selection changes clear the previous preview and invalidate in-flight captures.
Closed document identities are rejected. The UI window is captured at context
assembly entry, before awaiting file reads. Neither selection nor preview is a
provider delivery attestation, and neither initiates a model call.

Executed focused checks: uiContextProjectionActions.test.tsx (1 passed) and the
WorkspaceChat explicitly-selected/pinned preview case (1 passed). The first full
WorkspaceChat file run exposed a stale context-status mock and the disclosure
click/toggle timing gap; the mock and immediate shared disclosure handler were
corrected before rerunning that focused case. Targeted strict TypeScript checks
for WorkspaceChat and the adapter test passed. Native/provider flow remains
unwitnessed.
