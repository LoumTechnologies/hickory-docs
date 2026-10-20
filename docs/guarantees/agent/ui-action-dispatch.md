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

Conversation backend/model selector revisions also bind document, selected branch
and session. A focused ChatDock follow-up (2 selected cases passed) verifies that
same-looking controls cannot reuse old handles after a document change, alongside
existing settings disclosure. Strict targeted checking passed. This guards local
UI retargeting; concurrent independent host configuration remains outside this
client-only check.

### Recorded ACP evidence — 2026-10-19

Recorded message/reasoning/plan/tool/status/filesystem evidence has actual details
disclosure and explicit JSON UTF-8 reads capped at 8192 bytes. Closed items retain
mounted view state while retiring their reading and nested tool-reading commands.
Opaque revision tokens include the retained value without disclosing it. Invalid
or non-object JSON remains inspectable as text. A recorded stop is labelled
neutrally; an adapter report is not an independent execution/byte check.

Executed focused checks: AcpRecord (2 passed), including nested reader retirement,
explicit Unicode evidence, malformed payload and neutral stopped attribution;
strict targeted checking passed. These retained-record component checks do not
witness live provider output or certify an adapter's claims.

### Desktop command installer — 2026-10-19

Install/remove/refresh use the existing desktop endpoint and report returned
availability, installation, removal, conflict and message fields. Bounded explicit
reads expose the full status/error; revision tokens keep private source paths
opaque. A synchronous pending guard prevents overlapping local operations, and
status generations reject superseded initial reads. Client enabled predicates
preserve existing conflict/platform constraints; the native host remains the
filesystem authority. No CLI execution or supported packaging witness is implied.

Executed focused checks: CommandPathSettings (5 passed), including human install/
remove, conflict, failure/retry, headless UI and registered install/read with actual
mocked host status; targeted strict checking passed. These tests did not modify
PATH or shell profiles on a real platform.

### Dataset previews and read-only tables — 2026-10-19

Dataset refresh/remote load/paging use their existing fetch/state handlers and
report requested transitions rather than claiming a completed read. S3 remains
explicit; no remote read occurs before its Load action. Responses are bound to
source, format, document, page, refresh and remote-selection identity. A changed
request cannot display old rows under the new source/page. Current read-only
tables expose bounded JSON UTF-8 schema/visible-row reads and shared filter/sort
setters. Scope includes loaded count, source offset, total/has-more and column
truncation; sorting/filtering is page-local and never writes the dataset.

Executed focused checks: uiDatasetActions (2 passed), covering exact Unicode
read/sort/filter, invalid column, explicit S3 load, actual next-page response,
old-page handle retirement and truncation scope; targeted strict checking passed.
Fetch responses are mocked; real S3 credentials/storage were not exercised.

### Recording and transcription supervision — 2026-10-19

The actual recording panel now registers typed configuration/draft/save,
retention/recovery/delete/download, current-state reads, capture supervision and
local transcription. Microphone start, remote consent/upload and human-reviewed
transcript import return addressable human requests to an agent. Remote upload
is never authorized solely by a cached checked consent value. Original audio and
transcription remain separate from correction drafts and linked note import.

Capture returns a real recording job ID after MediaRecorder starts. Stop reports
its request while final chunks/hash remain pending. Transcription returns actual
audio/transcript identity and byte count after the host response. Explicit cancel
and owned AbortSignal cancellation use the existing host cancellation route; a
request is not rollback or proof a remote provider avoided processing/charges.
Status, visibility and cancel subjects stay usable during a pending local
transcription. Read output is bounded JSON UTF-8 bytes. Active capture must stop
before its retained audio can be recovered/deleted. Download is a browser request,
not an observed saved file. Independent host/device/platform policy still applies.

Executed focused checks: MeetingRecording (4 passed), plus its pending-job
supervision case rerun after adding close/reopen coverage (passed); strict targeted
component/test checking passed. Cases cover actual human remote-consent failure,
OS capture refusal, agent self-consent/remote-upload refusal, and local request
status/cancel while pending. Devices, provider and retention APIs are mocked;
no real microphone, remote upload or provider billing was exercised.

### Cell, diagram and math readings — 2026-10-19

A shared code-owned reader explicitly pages JSON UTF-8 data in at most 8192 bytes,
with opaque revision/snapshot identity and no edit-baseline claim. Cell readings
reflect visible status, diffs and output; collapsed transcript is omitted until
the real replay state changes. Private full block state guards revisions even
when visible values happen to match. Diagram readings separate authored input
from captured renderer input, mark stale SVG previews, and treat assertion state
as reported rather than independently proven. Math reports renderer/fallback
state and stages output before promotion; obsolete async renders cannot overwrite
a newer equation. Source/SVG representations do not certify visual image delivery
or mathematical/requirement correctness.

Executed focused checks: DiagramPanel (6), CellPanel (7), MathPanel (1), all passed;
strict targeted checking passed. New cases verify transcript disclosure and stale
handles, exact Unicode reading, real fallback state, and out-of-order math
completion. The renderer engines are mocked in the renderer-specific tests; these
are not real native layout, model vision, provider or screenshot witnesses.

Rendered-reading evidence fingerprints for that check (historical after later source/inventory changes; SHA-256 binds listed bytes, not semantic parity):

- `docs/developers/ui-control-inventory.json`: `35e65d1f10642d2fde37f42f327eb1aef637697d411b9f48d3f7728f42656054`
- `scripts/inventory-ui-actions.mjs`: `fd2cf4881b5c22659ccb67e06c8f61e1b5be77d5bd6b7e4f7a9c59df38c369d8`
- `apps/web/src/lib/uiReading.ts`: `cc37d833882855bcaf17e8fecbc04830f8304a048efbaebbd7e949b43af0acc0`
- `apps/web/src/components/MathPanel.test.tsx`: `966bf2b6d29c52c98308815d7af30abd2b4e0910969cda83d59ba71d41652fb5`
- `apps/web/src/components/CellPanel.test.tsx`: `51bec8f7426917ef4b6e287195a3beab45450428c2bb9961143d79d9bd894b9e`

### Shared in-app prompt controller — 2026-10-19

Each code-owned prompt has a unique ID, typed draft/option-index input, bounded
JSON UTF-8 reads and cancellation. Prompt settlement uses that expected identity;
identical titles cannot retarget old actions. Caller policy defaults to a human
request. Ordinary rename/code-action/Save a copy/watch callers explicitly enable
agent input; source-sensitive callers provide a captured live revision getter.
Changed source disables input/settlement until a new action is requested. Choice
labels are data; invocation selects only an existing code-owned option value.
Replacing or unmounting a prompt resolves its old promise as cancelled rather
than leaving an operation awaiting forever. Prompt settlement is not an operation
success or human attestation.

Executed focused checks: PromptPanel (4 passed), including draft/resolver routing,
human-only choice, changed-source refusal and identical-title replacement/unmount.
The first run had an ambiguous test label query (dialog and input); the query was
corrected and all cases rerun. Strict targeted WorkspaceView/prompt checking passed
with the repository's existing js-interpreter declaration explicitly included;
the initial ad-hoc command omitted that declaration and failed on library types.
These tests do not execute an actual LSP refactor, native chooser or file copy.

### Scoped workspace menu controller — 2026-10-19

Native menu and agent controls now share Save/Save a copy/Save all/Print/Insert/
Edit element handlers. Live private source/focus/selection/open-buffer guards
reject stale handles even without a React render. The controller copies target
identity before async formatting; subsequent focus changes cannot retarget an
unnamed save. Source-changing/closed-buffer formatting failure prevents save
submission. Native destination/print dialogs require real human input; in-app
copy prompts use the separately scoped prompt interface. Known empty unnamed
buffers with formatting off create no file and need no chooser.

Outcomes preserve returned document save booleans, copy/insert/editor requests,
and unconfirmed self-saving-pane flush intent. Prompt/print requests do not
certify copied files, physical output or verification. Current command reads
include actual document banner/sync state and the last adapter outcome. This
client guard does not coordinate independent external writers or strengthen the
existing document-save endpoint into a filesystem transaction.

Executed focused checks: workspaceMenuActions (5), uiWorkspaceMenuActions (1),
existing useUntitledSave (2), all passed; strict targeted WorkspaceView/menu
checking passed with the existing interpreter declaration included. Cases cover
focus changes during formatting, real acknowledgement false, chooser/print
policy, empty saves, source changes without rerender and shared human/agent
routing. Native dialogs, printing, full app menus and external filesystem races
remain unwitnessed.

### Core editable table controls — 2026-10-19

Tables now register current controller reads and explicitly requested cell regions
(maximum 64 by 64 cells, then JSON UTF-8 pages capped at 8192 bytes), shared cell
commit, selection/ranges/rows/columns, structural/menu/fill and layout/disclosure
commands. Reads distinguish raw/displayed values, calculation pending/current/
failed and owning writable-scope currency. Discovery revisions keep cell/source
bytes opaque. Structural commands require pending edit commit/cancel first.

Authored CSV and Markdown adapters bind block bytes and their actual immutable
CodeMirror document. Writes reject a changed/closed/readonly owner even before
React rerenders. Source identity is disclosed explicitly. Agent actor travels
through the source callback and its UI transaction name; these are recorded
agent operations, not human approval or verified correctness. Independent host
operations retain local UI permissions; no model code receives direct filesystem
access. Layout dispatch/measurement is not an acknowledged persistence result.
Nested formula/format/resize and remaining gestures are not certified by this slice.

Executed focused checks: 13 selected TablePanel/MarkdownTable/new adapter cases
passed; new adapter cases rerun after metadata correction (2 passed), then a third
real-CodeMirror CSV write case passed with the two earlier cases. Strict targeted
checking passed after removing an unused test import. Coverage includes real
CSV write-through, Markdown callback/actor routing, stale owner before rerender,
cell write/fill/menu, existing CSV normalization and formula result/failure cases.
These checks do not witness native pointer geometry, a real formula interpreter,
provider behavior or whole-table UI parity.

### Nested table controls and trace replay — 2026-10-19

Size drafts, validation/destructive-loss state, apply/cancel, numeric formats and
formula-trace navigation/read now share the actual nested controllers. Owner IDs
associate duplicate labels with their table. Resize/format source writes retain
the owning source guard and actor. Human callback arity remains compatible with
existing CSV/Markdown writers. Prose-wrap input mounts only through its real
settings disclosure. Known source changes invalidate old nested action handles.

Formula traces are keyed to their supplied language/table revision before any
read/navigation: a prior trace cannot be displayed as the new one while awaiting
its host response. Bounded current-step reads retain expressions, bindings,
errors, index/total and the captured-order-only scope. Navigation is replay and
never claims DAP reverse execution, inner-expression stepping or verification.

Executed focused checks: 8 selected table/format/trace/new-adapter cases passed;
strict targeted checking passed. The preceding selected run caught extra actor
arguments on an ordinary human resize callback; normal human arity was restored
and the affected cases rerun. Trace source-key/type checking also caught adapter
hooks placed in the effect cleanup scope; they were moved to component scope
before any passing execution was claimed. The tests mock the formula host and do
not establish native pointer geometry, interpreter execution or provider evidence.

Git history uses the same human navigation handlers for expanding/collapsing commits, opening an actual commit reading or current working file, loading older history, and opening story/literate views. Bounded JSON byte reads include only loaded summaries plus explicitly expanded bodies/files. The opaque revision includes loaded history and disclosure state; changed revisions refuse old handles and removed controls become unavailable. File navigation opens the current file, not its historical contents or an edit baseline. A successful history refresh clears a previous displayed fetch error. Focused evidence: strict TypeScript and the history disclosure/navigation/changed-and-removed-handle test. Working-tree mutations remain separately outstanding.

The Git working-tree controller registers bounded current-state/diff reads, current-file navigation, stage/unstage (one/all), stash/pop, fast-forward pull, branch disclosure/create/checkout, message/amend drafts, commit and editable-review navigation. Human buttons share those handlers; command results retain actual host replies (including the commit identity), host refusals are unavailable rather than success, and failed commits preserve the draft. A synchronous in-flight guard prevents overlapping Git host mutations even before React renders busy state. The selected diff clears before a new selection/request. Push publication, rewriting the previous commit, and the existing human two-click discard decision return requires_input to agents; they do not impersonate the human decision. These are addressable operation results, not an OS limitation or a claim that the agent published/discarded. UI revisions cover the fetched status, selection, drafts, branches, notice and busy state; fetched porcelain status is not an atomic file-content snapshot, and external Git/file changes can race the existing host APIs. Focused checks: seven selected GitPane staging/diff/commit/discard/branch/failure/agent-boundary cases passed (the branch case was rerun after preserving the existing one-argument checkout callback); strict controller and test TypeScript passed. No real checkout or remote publication was performed for these checks.

Visual bisect registers only its visibility launcher while closed. Once disclosed, bounded session/graph/host-output readings and boundary drafts, refresh/start, candidate-window open, skip, experiment-preserve/restore and finish use the actual host controller. Remaining commit text requires a separate evidence disclosure. Closing retires the inner controls. Good/bad are explicit human classifications and agent requests return requires_input; a host reply is not independent test evidence. Modified candidates retain the existing refusal of verdict/end until preservation/restoration. A synchronous operation lock covers the pre-render interval, and generation-ordered polling refuses older responses. Restore replies retain the actual patch path and finish replies retain inspection-worktree information. Focused checks: two real-component mocked-host disclosure/authority/preserve checks and strict TypeScript passed. Native candidate-window opening and real worktree execution remain unwitnessed in this environment; no outcome is presented as a completed native workflow.

Comparison controls share actual layout/source/fork handlers and bounded source-choice metadata; hidden compact options retire their child controls through the margin disclosure. Visible CodeMirror sides expose scoped editor line/selection/fold reading and a current-source guarded draft-range edit. The live guard includes connection/read-only state, draft text and selection, editable permission, source identity, target, kind and captured URI. Immutable sides remain disabled; changed draft state refuses a prior handle before a second edit. Draft writes invoke the existing model update, which resets equality evidence to unverified and drops now-incompatible captured project tooling. Agent transaction tags distinguish the UI event but do not authenticate source authorship or approve a named edit proposal. Replies explicitly establish no save, approval or verification. Rendered diffs expose bounded text reads; binary placeholders expose no binary data. Focused checks: four comparison component cases and three rendered-diff cases passed (the added case rerun after correcting its expected disabled status), plus strict affected TypeScript and file-length/whitespace checks. Patch gutter breakpoint actions, captured-code tool navigation and other comparison-adjacent flows still require separate coverage.

Captured comparison references expose bounded current-list reads and exact-index navigation through the same human handler. Navigation awaits the captured-source host request, records the actual target URI/line, keeps a failed list available and reports failure rather than success. A live source/connection guard expires prior handles even without React rendering. Reference callbacks are adopted only for the owning captured context and unchanged visible source; replaced-source results do not become a new view's references. Captured-code setup state has bounded reads and a retry action only while its actual error is displayed; retry reports registration requested, not runnable/verified success. Five focused checks passed: real captured navigation, setup retry, late context cleanup, exact navigation failure/live-source guard, and delayed-reference rejection after source replacement. Strict affected TypeScript passed. Patch-gutter controls and native debugger/provider witnesses remain separate.

Merge views expose bounded current regions/answers/preview reading and shared conflict choice, cancel and accept handlers. Replacing path/base/ours/theirs mounts a new controller and drops old answers and handles. Acceptance awaits the owning callback, reports actual failure and establishes neither a save nor proposal approval nor verification. A live owner predicate refuses stale acceptance; synchronous pending ownership prevents duplicate accepts and disables concurrent choices. Plain-file acceptance captures the owning buffer and merge, refetches the disk side, refuses a changed/closed buffer or owner, and reopens a new unanswered merge if the disk text changed again. Only a matching observed disk side becomes the save baseline before the merged draft enters the buffer. The normal hash-guarded saver handles subsequent writes; this is not an atomic read/write filesystem transaction. Plain-file editor controls retire while the merge replaces that editor. At merge-only commit `e479ec8`, generated-output acceptance awaited its existing queued host reply and retained failed merges; the following held-output resolution integration supersedes that queued acknowledgement. Six selected merge/restore checks passed, including live-owner refusal, changed-input answer retirement, current-disk acceptance and changed-disk re-review; strict affected TypeScript passed. The plain-file checks wait for the asynchronously mounted owning editor before clicking.

Held-output merge requests now carry both exact observed UTF-8 sides and retain the actual owning callback reply in the UI result. The loop checks current held/tracked ownership, unchanged produced bytes, canonical in-folder target and disk bytes before writing. It removes the hold only after a successful write. The HTTP endpoint waits for the loop's applied/refused result; success establishes the output-file write and explicitly leaves source propagation unconfirmed. Closed waiting receivers are skipped before starting queued work. A stale side, missing hold or symlink retarget is refused without replacing the output; after success a replay is refused because the hold retired. Engine-owned writes retain their serialization, but unrelated external processes can still race the check/rename interval; there is no filesystem compare-and-swap, exactly-once crash guarantee or timeout rollback. The timeout result instructs inspection before retry. Focused evidence: two real-filesystem state checks (both-side staleness/hold preservation/success/replay and out-of-folder symlink refusal), one actual HTTP test proving no enqueue-only success and applied/refused replies, three merge/restore UI checks, strict affected TypeScript, file-length and whitespace checks passed. The HTTP listener was rerun outside the sandbox after its initial bind was refused.

The visible merge-driver notice exposes bounded current configuration/failure/repair-state reading and shared initialize/dismiss handlers. Initialization awaits the existing host reply and retains its changed-files/status fields; a host refusal is unavailable, not a successful repair. A synchronous lock prevents pre-render duplicate initialization. The initialization command retires when the actual result replaces the warning, and all notice controls retire after dismissal. Dismissal reports no repair. Four selected behavior/authority checks passed (human repair/failure/dismissal and agent awaited-repair/pending-stale-handle/retirement), plus strict affected TypeScript. No actual repository initialization or merge execution was performed by the mocked-host UI checks.

Divergence banners expose bounded current cause/effect/failure/pending reads and only their actual available keep/take/merge handlers. Opaque live revisions include owning file/buffer/source state, so typing invalidates an observed decision without requiring React to render first. Shared handlers serialize pending choices, await promise-returning callbacks, preserve their actual replies and report asynchronous failure. Void callbacks explicitly establish only request dispatch, not a completed overwrite/save. Plain-file merge opening and disk reload now recheck the owning buffer across the fetch; reload retains newer edits instead of adopting a late disk response. At divergence-only commit `0bb59df`, regeneration retained a queued host reply; the following regeneration integration supersedes that contract. Six focused divergence/merge/reload cases and strict affected TypeScript passed, including a real CodeMirror typing/save-conflict/deferred-reload flow. The test's edits use the actual input transaction event, matching the saver boundary.

Held-output regeneration now requires loaded disk bytes and observed produced bytes, checks both current hold and tracked output content, and shares the actual applied/refused resolution acknowledgement. The engine-controlled regenerate mode accepts only the observed produced bytes and restores generated/read-only permissions before retiring the hold. The old unguarded queued regeneration command was removed from both watch adapters. UI receipts retain the actual application reply; they do not infer source propagation or verification. Focused validation passed: both earlier resolution state tests, a regeneration bytes/permission check, actual HTTP merge/refusal/regeneration acknowledgement, strict affected TypeScript and file-length/whitespace checks. Compilation caught and corrected the now-explicit result type in the command adapter before these checks passed. External-writer comparison/rename races and partial I/O failures remain subject to the prior inspection/no-rollback caveats.

Plain-file menus disclose only their actual available literate-view/review/adoption handlers, with explicit owning path/save/pending metadata and live private buffer/hash revision checks. Actions await their host/navigation handler and retain actual replies; ownership is updated only from actual adoption success. A later navigation exception is a separate field/notice, not a fabricated adoption failure. The owner's bounded status reading remains outside the menu and distinguishes a last-loaded hash from current source/save state. Menu children retire when closed. Human button auto-dismissal snapshots requests started by that click and waits for their retained results, preserving the controller across asynchronous intent persistence. It ignores older completions after a new menu epoch; this changes no actor authorization or request deduplication. Focused evidence: delayed intent/native execution/retirement, scoped file menu navigation/adoption/live-buffer refusal, three existing ownership-display cases, and adopted-success/navigation-failure handling passed; strict affected TypeScript and file-length/whitespace checks passed. These mocked-host UI checks establish no real file adoption, source verification or packaged native witness.
