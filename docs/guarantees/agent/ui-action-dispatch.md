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

### Search, references and Problems — 2026-10-19

Mounted lists expose explicit JSON UTF-8 evidence reads (8192 byte maximum),
current result indices, shared navigation and close. Private query/diagnostic/
location bytes remain outside discovery revisions. Changed rows or query state
invalidate prior handles, so an old index cannot silently select a new row.
Ranked search exposes the same query and selection handlers as keyboard/pointer
controls, limits its response to the existing 20 hits, and reports lexical/semantic
scope. Query input clears the old response immediately; the existing cancellation
of stale debounce responses remains active. Source freshness beyond the displayed
LSP/search response is not certified by list navigation.

Executed focused checks: uiResultLists (2), SearchPanel (2), ProblemsPanel (4),
all passed. Targeted strict TypeScript checking of these panels and the new test
passed. Tests cover exact human/agent row routing, explicit Unicode evidence,
changed-result refusal and unregistration; they do not witness native LSP startup.

### Snapshot export — 2026-10-19

Human and agent export capture the existing host snapshot, hash the exact JSON
UTF-8 bytes used for the browser download and expose bounded explicit reads of
that capture. Download requests do not attest that the browser wrote a file.
No publication or new verification is implied. Changed document identity retires
the prior artifact read command. Abort observed after capture suppresses the
browser download; an already requested download cannot be rolled back.

Executed focused checks: SnapshotExport.test.tsx (1 passed), including exact
Unicode/failed-result bytes, actor routing, truthful download/publication scope
and document-change retirement; targeted strict TypeScript checking passed.
The browser download and host snapshot are mocked; actual filesystem save and
reviewed publication remain separate witnesses.

### Conversation submission and supervision — 2026-10-19

Registered conversation controls read current history, draft, output and UI status
only through explicit JSON UTF-8 pages of at most 8192 bytes. Opaque revisions
cover branch, draft, running turn, readiness and current evidence. Draft input
uses the same recovery edit handler as typing; it does not submit. Send invokes
the existing conversation handler, preserving its selected context/provider and
recovery semantics, and reports the actual returned turn ID as queued. A local
slash command or failed/no submission returns no invented job. Stop awaits the
existing host request and reports whether it was requested; only the actual turn
state can establish stopped. Branch selection rejects unknown identities and is
disabled while sending/running. Request cancellation is not a rollback of a
submitted provider turn; its actual job must be stopped separately. UI receipts
retain the initiating agent actor; agent invocation is not human attestation.

Executed focused checks: uiConversationActions (1), selected ChatDock provider/
Python/ACP/Stop/registered-Send cases (5), selected WorkspaceChat startup/preview
cases (2), all passed after correcting stale context-status fixtures. The new
ChatDock case drives registry draft/Send into the actual Python conversation
handler and checks its queued host ID. Strict targeted TypeScript checks passed.
Provider responses and host APIs are mocked; these are not real provider/native
witnesses. The initial focused ChatDock runs exposed missing items/updates in
test mocks; those failures are not counted as passing checks.

### Conversation settings disclosure — 2026-10-19

Agent selection, catalogue refresh/read, provider/model input and collapsed-lineage
preferences mount only with the actual settings panel. Selection accepts current
catalogue IDs and uses the same backend/preference/new-thread handler as the human
select. Availability is not a configured login; current connection status remains
a separate host result. Catalogue evidence reads are bounded JSON UTF-8 pages.
Provider/model controls use their existing local setters and describe next-turn
configuration only. Running/starting conversations disable agent/model selection.

Executed focused checks: selected ChatDock provider/Python/ACP/registered-Send/
settings-disclosure cases (5 passed); strict targeted ChatDock checking passed.
The new case drives registered settings opening, model and ACP selection, observes
the existing connection handler, closes settings and checks commands retire,
without any model call. Actual login/provider behavior is mocked.

### Conversation proposal review — 2026-10-19

Proposal state, original/proposed bytes and actual returned statuses have explicit
bounded JSON UTF-8 reads. Review navigation validates current proposal identity.
An agent decision returns a retained requires_input request keyed by document,
backend and proposal, with no edit or host decision call. Enabling auto-accept is
also human-only; selecting review mode can reduce that authority. Human decisions
share the actual apply-before-acknowledge controller; failure after local apply
reports that effects may already have occurred. Changed proposal state invalidates
old handles. A host accepted/applied status is not a verification result. This
registry policy does not sandbox an external ACP process or replace host approval
checks; independent local HTTP clients retain the existing trust model.

Executed focused checks: AgentChanges.test.tsx (9 passed), covering existing
accept/reject, stale-buffer feedback, revised bytes, authorized ACP auto-apply,
Python human review and the new structured read/request/actor guard/stale handle
path. Targeted strict AgentChanges checking passed. Host APIs and editor actions
are mocked in the new request-path case; actual native/provider approval remains
a separate witness.

### UI handshake before context capture — 2026-10-19

Context capture snapshots the current connection and awaits its host-minted window
identity before buffer reads. Pending startup is bounded to three seconds; timeout
or failed attached connection prevents this submission, preserving response
recovery. A closed origin rejects capture rather than adopting a new connection.
No provider/host call to start a turn occurs during this wait. Hostless embedding
without a UI provider retains the prior no-window context behavior. Successful
capture does not guarantee the window remains connected through a later provider
call; the host broker still rejects stale/deleted windows.

Executed focused checks: uiActionConnection (4 passed), selected WorkspaceChat
startup/preview cases (2 passed), strict targeted bridge/context checking passed.
The connection tests exercise pending, timeout, closed/replacement and failed
handshakes with mocked host endpoints; native startup remains a separate witness.

### ACP permission and tool evidence — 2026-10-19

An agent can explicitly read the actual permission request/options and request a
human decision keyed by document/backend/request identity. It cannot answer any
option on its own behalf. Human buttons use the same current-choice handler and
actual host acknowledgement. `answered: false` keeps the request visible and
reports an error; acknowledgement does not certify tool completion. Tool details
use explicit JSON UTF-8 byte reads capped at 8192, with opaque revisions. Closed
adapter tool details retire their reading commands while preserving the mounted
view; reopening requires current handles. This local registry actor policy does
not authenticate independent HTTP callers or sandbox ACP processes.

Executed focused checks: four selected AcpControls permission/disclosure cases
passed, including human exact-choice routing, agent self-answer refusal, actual
host refusal staying visible, and hidden tool-reading retirement. The host-false
case first exposed missing inline error feedback in the hostless adapter; that
feedback was fixed before rerun. Strict targeted AcpControls checking passed.
Endpoints are mocked; real adapter permission/provider behavior remains unwitnessed.

### Adapter setup and configuration — 2026-10-19

Connection controllers now return actual host state/error, including whether an
async result was superseded, rather than only swallowing errors into UI state.
Registered runtime check, reconnect, install and typed configuration use these
controllers. Connection reads are bounded and exclude undisclosed command help
and separate tool/permission payloads. Configuration revisions include document,
backend and adapter session. Sign-in/session-mode changes require human input;
known permission/sandbox/approval/mode fields are also guarded. Opaque external
configuration names are not an authentication or sandbox boundary: the external
adapter retains its independent authority and may have unknown semantics.

Executed focused checks: AcpControls (11 passed), then four selected configuration,
failed-operation, sign-in and superseded-result cases passed after adding the
scope guard. Strict targeted AcpControls, its test and ChatDock checking passed.
Actual APIs are mocked; no adapter package was installed and no login/provider
call was made by these tests.
