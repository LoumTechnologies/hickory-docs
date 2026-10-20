# Implementing shared human and agent actions

Audience: engineers connecting Hickory's existing UI handlers to the built-in
Python agent and external agent clients for [#72](https://github.com/LoumTechnologies/hickory-docs/issues/72).

**Status: partial integration.** Registered families now include rail actions,
menu disclosure, environment setup, debugger state/control, bounded runtime/profile
and source reads, terminal supervision/output/input, editor reads/selections/folds,
source-guarded document execution, folder reads/opening, file-operation prompts,
rename columns, actor-attributed review comments/investigations and the anchored
native coach. Application navigation, feature-settings disclosure, appearance/editing choices,
agent runtime configuration, explicit context updates, guarded folder search/replace,
command search and spatial layout also have registered controllers. Subsequent integrations cover conversation/provider/permission controls, meeting import/recording, rendered reading, editable tables/forms, Git history/working tree/bisect, comparison/merge/divergence and file representation/adoption controls. Both Python and
ACP/MCP use the originating window broker. This is not universal UI parity.
No real native/provider recording, packaged-release witness or matching deployed
build witness has been supplied.

In the application, send a task from the workspace conversation, then call
`ui_observe({parent: "root"})`, observe a returned group and submit
`ui_act({handle, arguments: {}, request_id: "operation-1"})`. Python snippets use
`tool("ui_observe", {...})` and `tool("ui_act", {...})`. The tools use the
window captured at Send; their schemas cannot choose a different window or actor.
Standalone CLI agents have no attached UI and do not advertise these tools.

The adapters are `lib/uiActionBridge.tsx` and `serve/ui_actions.rs`, using `/api/v1/ui/windows`. App and engine must support this shared contract; an unsupported/malformed handshake refuses attached Send without legacy fallback. A host-minted
window identity selects a bounded polling queue. Disconnected/stale windows fail
explicitly and never attach another window automatically. Pending caller
cancellation requests cancellation in that same window; it cannot promise rollback
or cancel a host job whose dispatch already returned. Closing the provider aborts
in-flight registry calls and unregisters its controls.

Receipts are retained in the existing per-project WorkspaceStore under
`ui-actions/<window>/`. Intent and result are immutable separate files keyed by
actor and request ID. Result metadata must match its retained intent. Python's
session records host-observed results and captures UI tool output into the context
store; the existing request assembly/delivery protocol then carries those
references. ACP's session records the tool request and actual result/error; its
provider internals retain the existing opacity limits. UI revision tokens are
concurrency identities, not cryptographic source hashes or full edit baselines.

## Connect a real control

1. Identify the semantic command and the predicate currently used by the human
   control. Move them into a shared controller where necessary. Register that
   controller; never dispatch a synthetic DOM click or execute text from a file.
2. Register a `UiCommand` in the host-bound window's `UiActions` registry.
   Use a stable subject identifying the actual document/block/thread/frame,
   a parent disclosure group, typed parameters, prerequisites and a live `read`.
3. Include every relevant revision in `read().revision`: workspace epoch,
   current source, selection/UI state, run, stop and frame as applicable. An
   expanded snapshot is an observation, never a complete edit baseline.
4. Have the human adapter and agent adapter call the same command under the
   current actor policy. The trusted host adapter supplies scope and actor;
   the model cannot choose either. Ordinary accessible labels stay unchanged.
5. Retain request intent and actual result through the host-owned
   session/evidence store. Register an observer for each disclosed reading
   surface as well as its actions. Preserve capture and delivery lineage when
   including observations in the Python-owned request context.

For example, `observe("document:deadline/rail")` could return
`Run [action:opaque-window-namespace:1]`. The model submits a structured
`{handle, arguments: {}, request_id}` through a dedicated tool. A quoted hash
in Markdown, a prompt or a tool result never executes. A source/location handle
cannot be used in its place. Repeated labels on different subjects have different
handles. The current prototype uses a UUID namespace and counter; a host transport
must preserve the distinction between identifiers and authorization.

## Reading and interaction contract

An action descriptor contains its readable label, subject, disclosure parent,
opaque handle, required typed parameter schema, enabled state and reason,
prerequisites, current value where applicable, and relevant revision. Execution
returns completed value, a real queued/running job identity, a required-input
request, an unavailable reason, or an error. A queued request is not a completed
operation. The intent receipt has `stage: requested` and no invented job/result.

`observe(parent, limit, offset)` discloses only that registered group. The broker's
`root` observation lists registered groups without their children. Pages are
bounded to 64 descriptors and include total/disclosed counts and an immutable
observation identity. Reading does not execute or expand anything. Scrolling
does not add hidden context. The adapter must report actual serialized byte/token
cost when assembling context; the dispatcher does not guess provider token costs.
The broker reports `observation_bytes` for the serialized observation payload,
excluding its accounting/transport envelope; action replies similarly report `result_bytes` for the serialized immutable result payload. Provider token cost remains explicitly unmeasured. Opening a menu or expanding a view must invoke its real state transition before
disclosing current children. A parent string alone is not an expansion authority.

Forms need read descriptors for current values, typed input commands and a
distinct submit command. Selections need source revision and exact coordinates;
folds need current range/state. Do not infer a target from its label or latest
selection after invocation. Read-only values, errors, tables, graphs, generated
views, terminal output and disabled controls also need observations. Registering
only buttons does not satisfy the inventory.

An OS chooser must have a scoped file/resource-reference equivalent or a retained
`requires_input` request/result path. Credential entry, microphone consent,
sign-in and human attestations must preserve their real authority. Unsupported
platform operations return a reason and recovery path. No model-authored receipt
may impersonate human approval or imply an unsupported debugger operation.

## Dispatch and lifetime

The in-memory dispatcher binds handles to one registry/workspace/window and
current state. It checks policy, schema and enabled state, serializes in-flight
operations on the same subject, retains intent, then checks current revision and
policy again before invoking. Stale actions return one refreshed descriptor;
retired/unknown handles require observing the parent again. Handlers must still
enforce authoritative host revisions and credentials atomically: a browser
predicate cannot prevent a concurrent host edit.

Retries with the same actor, request ID, handle and arguments share the same
promise/result. Conflicting key reuse is rejected. A new request ID permits an
intentional repeat. Requests are retained for this registry's lifetime; at 4096
requests the dispatcher refuses further requests instead of silently evicting
deduplication records. Restarting the window invalidates handles and loses this
in-memory result cache. Retained host receipt identities refuse changed retries;
old window handles become unavailable after restart. Crash recovery does not
re-execute uncertain operations or reconstruct their result. Inspection and a
new turn are required. This does not guarantee exactly-once execution across
crashes or coordinate another independent human action on the host.

Cancellation checks ownership and propagates an AbortSignal to the executor.
It cannot promise rollback. An executor returning a long-running job must expose
host-owned job observation and cancellation commands; cancelling the completed
dispatcher call does not cancel that job. Failed result persistence reports that
side effects may have occurred. The same request ID will not re-execute them.

Python retains planning, context selection and refresh policy. Rust retains
execution, filesystem/process/credential boundaries, evidence, durable receipts
and authority. The TypeScript registry is an adapter foundation; it does not
grant shell access to confined Python snippets. ACP/MCP and Python need the same
validated host broker interface, including explicit opaque-provider limitations.

This uses Hickory's existing local-only server trust model. The window identity
is routing scope, not authentication or a new capability grant. Registry checks
and host-owned executor/credential boundaries still apply. External ACP processes
retain their independent permissions; this broker does not sandbox those processes.

## Surface inventory and owned integration work

The companion [declaration index](ui-control-inventory.json) enumerates every
non-test TS/TSX file under `apps/web/src`, including files without controls. It
records source hashes, conservative HTML-entry import reachability, JSX controls and callback expressions, event listeners,
imperative DOM creation/dispatch and keymap declarations. Regenerate with
`node scripts/inventory-ui-actions.mjs`; use `--check` to detect drift. It is a
review aid, not an assertion that arbitrary callbacks are safe semantic commands.
The generated index records current file and declaration counts. Declarations can render many instances; these are not capability counts.
Spreads, dynamically constructed controls, canvas gestures, native menus and
rendered values need the manual mapping below and further adapter review.

Version 3 adds `possible_entrypoints` for each web module. It follows literal
imports/re-exports, lazy imports, type references and worker/raw-query imports
from every top-level web HTML script; conditional branches remain potential
paths. This is a conservative import graph, not proof of rendered visibility,
runtime reachability or semantic parity. Review Vite input changes, external
injection and the recorded unresolved paths before excluding a file. In the
current graph, `OutputTree`, `SessionTurns` and several old demo modules have no
HTML-entry caller; they are review candidates rather than missing live controls.
The generated `hick_lang.js` wrapper is explicitly unresolved by this TS/TSX
index. Neither an unimported module nor unresolved generated code is an OS
limitation or an automatic exemption from #72.

Every family still has **remaining shared-interface integration**; the registered
commands above implement portions of the listed families. Some have
working independent agent tools; those do not change this status. Ownership is
tracked in [#74](https://github.com/LoumTechnologies/hickory-docs/issues/74) for
descriptors/disclosure and [#75](https://github.com/LoumTechnologies/hickory-docs/issues/75)
for broker/capability/read adapters. The table's identity and guards are adapter
requirements, not claims that the prototype already validates those host fields.

| Family: actual UI/controller → host boundary | Parameters, identity and guards | Observation, effects and prerequisites |
| --- | --- | --- |
| Workspace/files/navigation: `views/WorkspaceView.tsx`, `shell/FilesystemTreeEditor.tsx`, `shell/treeMenu.ts`, `shell/useDired.ts`, `shell/dragDrop.ts` → `serve/files_ops.rs`, `serve/workspace.rs` | Scoped path, file identity, source/tree epoch; select/open/create/rename/move/delete/drag; overwrite/concurrent-edit checks | Tree, selection, open panes, errors; filesystem writes require existing host policy |
| Tabs/layout/command/menu: `shell/CommandBar.tsx`, `shell/PaneHeader.tsx`, `shell/SpatialController.ts`, `shell/fileMenuActions.tsx`, `components/MoreActions.tsx` | Window, pane/tab/menu identity and UI epoch; command ID, layout/geometry, keyboard/context menu | Current menus, focus, history and pane contents; view mutations; hidden children disclosed explicitly |
| Editor/folds/selections: `editor/DocumentEditor.tsx`, `components/PlainFilePane.tsx`, `lib/keymap.ts` | Exact source revision/range; text, selection, fold, undo/redo and formatting parameters | Saved/unsaved source, selection, folds, diagnostics; document edits; full baseline required for edits |
| Block/rail/generated views: `editor/CardRail.tsx`, `lib/railActions.ts`, `components/{Cell,Table,Dataset,Diagram,Math,Picture}Panel.tsx`, `components/InsertMenu.tsx` | Document/block/source revision, action, cell arguments, renderer state; run status and edit baseline | Rendered values, tables, graph gestures, source/result toggles, replay; execute/convert/edit via existing block handlers |
| Search/references/refactor: `components/SearchPanel.tsx`, `components/ReferencesPanel.tsx`, `components/ProblemsPanel.tsx` → `serve/find.rs`, `serve/refactor.rs` | Query/options, scoped targets and source revisions; replacement preview before apply | Current matches, diagnostic locations, references; navigation and replacement writes |
| Agent controls/context/history: `views/WorkspaceView.tsx`, `components/ContextUpdates.tsx`, `views/HistoryLens.tsx` → `serve/agent.rs`, `serve/python_agent.rs`, `context_updates.rs` | Conversation/branch/request/delivery identity; prompt, provider, context choices, stop/retry; actor policy | Exact selected context, delivery receipts, responses, errors and spend; real provider calls; no silent coach sends |
| Review/diffs/merge/Git: `components/ReviewPanel.tsx`, `components/GithubReviewThreads.tsx`, `components/DiffView.tsx`, `components/MergeView.tsx`, `views/GitPane.tsx`, `views/BisectPane.tsx` → `serve/agent_review.rs`, `serve/git_ops.rs`, `serve/changeset_drafts.rs` | Thread, target quote/range/revision, draft/Git head; comment/investigate/apply/resolve/merge parameters | Threads, actual response/diff/checks, conflicts and credit; writes and Git changes; human-only attestation remains explicit |
| Environment/install: `environments/EnvironmentPanel.tsx`, `environments/useEnvironments.ts`, `components/InstallCommand.tsx` → `serve/environments.rs::{inspect,act,choose}`, `serve/install.rs::install` | Project/finding revision, installation kind/language/action/choice; availability, running-job and credential guards | Findings, setup terminal/job, progress, failures; real package/process operations via existing host executor |
| Terminal: `terminal/TerminalPane.tsx`, `terminal/WatchingTerminal.tsx`, `terminal/useTerminals.ts` → `serve/terminal.rs::{open,input,interrupt,answer,close,resize,set_turbo,anchor}` | Terminal/session/prompt identity; input, answer, resize, turbo and anchors; current actor/execution policy | Exact available transcript and prompt state; process input and interrupt; unavailable/credential requests are observable |
| Execution/verification: `components/CellPanel.tsx`, `debug/RuntimeRuns.tsx` → `serve/test_run.rs`, `serve/runtime_runs.rs` | Document/source/run identity, executor and inputs; save/prerequisite checks, cancellation ID | Queued/running/completed/failed receipts and source hashes; process/container capabilities; completion prose cannot certify a pass |
| Debug: `debug/DebugStrip.tsx`, `debug/DebugRail.tsx`, `debug/useDebugger.ts`, `debug/ObservationReading.tsx` → `serve/debug_bridge.rs`, `serve/debug_sources.rs` | Session/run/stop/frame/source handles; breakpoint, step, evaluate, watch, frame selection; current adapter capabilities | Bound/paused markers, frames, values and output; debugger process effects; reverse/restart only when supported |
| Logs/profiling: `debug/RuntimeLogView.tsx`, `debug/RuntimeRuns.tsx`, `debug/cmRuntime.ts` → `serve/runtime_runs.rs` | Run/source revision and observation ID; profile kind/options; provider availability | Source-linked metrics, units, interval, overhead, stale/failed states; native profile execution or explicitly labelled replay |
| Meetings: `components/MeetingImport.tsx`, `components/MeetingRecording.tsx` → `serve/meeting_import.rs`, `serve/meeting_recording.rs::{capture,cancel,recover,transcribe,link}` | Scoped transcript/audio reference, capture/request ID, source hash, transcription settings; device/model availability | Original bytes, utterance citations, capture/transcription progress and error; microphone OS consent requires user request/result |
| Settings/forms/dialogs: `views/SettingsView.tsx`, `views/AgentSettings.tsx`, `views/CommandPathSettings.tsx`, `components/FeatureSettings.tsx`, `components/LineageSettings.tsx` | Setting key/current UI epoch, typed value, dialog identity; secret/permission boundaries | Current value, enabled reason and validation; local preferences/host configuration; secret values must use existing redaction |
| Export/publication: `components/SnapshotExport.tsx`, `embed/PublishedSnapshot.tsx` → existing `publications/selected.json`, `scripts/publish-spec-results.py` | Reviewed selection, exact source/dependency/asset hashes, artifact reference; publication review guards | Retained results including stale/failure/unavailable; export/publish writes; no automatic private-session upload |
| Native windows/choosers/external links: `apps/desktop/src-tauri/src/{lib,file_open,folder_session,command_path}.rs` → `serve/shell.rs::{save_file_dialog,pick_folder,close_window}`, `serve/reveal.rs::{reveal,open_external}` | Window/workspace, scoped resource reference, destination and OS capability | Open/closed/result/cancelled/unsupported states; new windows, external app and OS dialogs; require addressable equivalent/request path |
| Native filesystem mount: `apps/desktop/fskit/{WorkspaceExtension,WorkspaceVolume}.swift` | Mount/session/resource identity and OS availability/consent | macOS-specific platform boundary; no fabricated Linux/Windows support or extension-consent receipt |
| Coach/homepage/sample/download: `components/NativeWorkflowTour.tsx`, `components/WorkflowTour.tsx`, `tour/scenario.ts`, `views/LandingView.tsx`, `shell/SamplePicker.tsx` → `serve/engineer_tour.rs` | Isolated practice repository/source/run/request identity, coach step and target; evidence gates | Current problem, exact evidence and missing-target recovery; create/open isolated practice; #63 owns coach, #76 owns recording/released/deployed witnesses |

## Remaining acceptance work

- #73: review the declaration index against all dynamic/native controls and
  rendered values; resolve per-command semantic controllers and observation
  schemas. The family inventory is not exhaustive per-command certification.
- #74: extend registration to the remaining controls and add missing
  read/expand/input adapters and authoritative source/run/stop checks. Complete
  crash recovery and host-wide concurrency where needed. Add declaration
  registration enforcement once controls actually register; an all-pass stub
  check would conceal the current gap.
- #75: the Python and ACP/MCP window broker interface is connected. Complete every
  capability family, actual job observation/cancellation and the
  setup→reproduce→debug→edit→verify and profile→remeasure witnesses. The confined
  Python integration test uses a scripted provider and UI endpoint; component
  tests separately exercise the real registered controls. Neither is a native
  or paid-provider witness.
- #63: the resumable anchored coach now delegates first and reconciles host
  evidence. Complete visual/native/provider accessibility and recovery witnesses.
  ACP provider delivery remains opaque to the coach. Profiling is optional.
- #76: link the actual recording and reviewed evidence through #61's pipeline;
  reconcile release/install claims, witness a supported release/provider run and
  matching deployed revision. Keep absent witnesses explicitly absent.

Only focused behavior/safety checks are authorized here. The user monitors broad
regressions using `just checks-status` and
[GitHub Actions](https://github.com/LoumTechnologies/hickory-docs/actions).

Current disclosure refinements:

- Hidden workspace and spatial-pane subtrees unregister their commands while
  retaining mounted editors. Reopening requires fresh handles.
- Debugger details and review collections use explicit UTF-8 byte pages (8 KiB
  maximum). Runtime run/source indexes use explicit pages of at most 64 entries.
- Application New, Files, Agent and Settings navigation shares the native menu
  handler. Focus-sensitive Save/Print/Insert commands still need their scoped
  workspace adapter; navigation registration does not certify those commands.
- Feature settings opens the actual anchored panel before lineage options mount.
  Those options call the same persistence and change-notification handlers as
  their checkbox/select controls.

Later integration details:

- Context updates disclose the real panel and patch before selection, retain exact
  candidate revisions and return actual queue states. Provider confirmation remains
  a later boundary. A changed branch cannot adopt an old response's displayed state.
- Settings expose shared choice/toggle/title and runtime/configuration handlers.
  Credential drafts remain outside observations; credential changes return a
  retained request whose result is read from the actual masked host status.
- Folder find returns source hashes. Replacement sends the observed hashes,
  refuses detected drift, reports skipped/unavailable files and reruns search.
  The UI refuses replacement of truncated results. Outside-editor writes can
  still race the final check/write; multi-file operations remain non-atomic.
- Command search exposes current candidate identities. Command-mode callbacks need
  explicit agent policy; source-sensitive document runs use their existing
  source-bound registered controls. Unknown callbacks cannot impersonate approvals.
- Spatial center/restore/pin/expand/dock/resize/close and drawer controls share actual
  handlers and preserve editor mounts. Hidden/collapsed bodies require explicit
  expansion before their controls register. Pointer gestures, transient hover
  previews, minimap and remaining layout interactions still need complete parity.
- Private controller state is compared inside useUiRevision and represented by
  opaque tokens. Full serialized state is not a revision token or implicit context.

Meeting transcript import now has an explicit disclosure, typed transcript text
input equivalent to the file chooser, shared local ingest, bounded exact UTF-8
JSON evidence pages, and note/original/utterance navigation. Both input routes
validate format and the actual UTF-8 byte limit before the same host request.
Capture, transcription configuration, OS microphone consent and recording
management remain separate integration work; importing text does not attest to
human review or authorize remote transcription.

Context projection now exposes disclosure, paged document choices, off/expanded/
pinned selection, preview capture and bounded JSON UTF-8 preview reads. Human
checkboxes and the agent use the same selection handler. Selection changes clear
old previews; a superseded asynchronous capture cannot replace the newer choice.
The originating UI window is captured before asynchronous buffer reads, rather
than looking up a possibly different window afterward. A preview remains a
capture, not a provider delivery receipt, and Send captures context again.

Ranked search, references and Problems now register current-list observations,
bounded JSON UTF-8 byte pages, navigation and close. Search also exposes query
input and selected-result movement. Query changes clear previous results before
debounced search; a previous query response cannot replace the new one. Navigation
handles are bound to the current result list and selection. These are navigation
observations, not source-edit baselines or proof that an LSP diagnostic is fresh.

Snapshot export now shares its capture/download handler with registered agent
controls. A completed handler reports the actual captured JSON byte count and
SHA-256, a browser download request (without claiming a saved path), and no
publication request. Explicit pages expose those exact captured bytes. Switching
documents retires the old artifact read command; saved/recorded snapshot contents
do not attest to a new verification run or to the current unsaved editor.

Coach children now unregister when guidance closes; phase-specific actions are
only registered where the corresponding controls exist. Current instructions,
skips and observed evidence gates have bounded JSON UTF-8 reads, with private
practice state kept behind opaque revision tokens. Source navigation and popup
Locate/Dismiss share the actual handlers; Locate reports absent/hidden targets
rather than claiming it opened a pane or started a debugger. Dismiss preserves
retained history and never certifies skipped work.

## Native declaration review ownership

The generated inventory also records every Rust source in the desktop host,
FSKit Swift source, and the host `shell.rs`/`reveal.rs` boundaries. It indexes
function declarations, host callback fields, route/menu declarations and compile
guards with source hashes. Web rows now list action adapter hook calls as well as
control declarations. A hook call does not certify its surrounding controls.

| Native declaration → semantic boundary | Parameters, observation and current ownership |
| --- | --- |
| `serve/shell.rs::Shell::{open_folder,pick_folder,save_file,close_window}` → desktop-injected closures | Folder/name/window and `OpenWhere`; pick/save return actual path or cancellation, close/open return dispatch result. #74 owns addressable requests, #75 actual capability adapters and lifecycle results. |
| `serve/shell.rs::{save_file_dialog,pick_folder,close_window}` | Existing HTTP handlers, host availability guards and errors are the authority; a browser-only host cannot claim native dialogs. #75 owns equivalent scoped input/request-result registration. |
| `serve/reveal.rs::{reveal,open_external}` | Host-validated target and process launch result; launching is not proof another app opened or consumed it. #75 owns agent adapter and retained result. |
| `desktop/src-tauri/src/command_path.rs::{status,install,remove}` | Actual current PATH/install status and command request; user profile/filesystem mutations use the existing host API, not a synthetic settings click. #74/#75 own configuration action disclosure and execution adapter. |
| `desktop/src-tauri/src/folder_session.rs::attach_folder` and `file_open.rs` | Native process/window/folder lifecycle and open-document routing; results must identify originating/replacement windows and retained unsaved-buffer decisions. #75 owns cross-window request/result handling. |
| `desktop/src-tauri/src/lib.rs` menu/close/open callbacks | Menu IDs and compile guards are indexed. Shared navigation is implemented; scoped edit/save/print/new-project and close lifecycle remain #74/#75 work. Native callback presence does not certify agent parity. |
| `desktop/fskit/{WorkspaceExtension,WorkspaceVolume}.swift` | Every function and platform guard is indexed. Mount/extension/device consent remains an OS-owned macOS boundary with explicit availability/request/result requirements in #75; no unsupported platform support is inferred. |

Review each generated native declaration against these owned families. Private
helpers and tests may appear; they are not independently exposed capabilities.
Canvas/drag interactions and rendered values remain manual review work in #73;
this broader source index intentionally does not mark them complete.

Conversation controls now register bounded state/history/draft/stream/usage reads,
response draft, discard, Send, Stop, branch selection, new thread, tree and optional
collapsed visibility. Send calls the actual conversation handler and returns its
host turn ID as queued; Stop awaits the existing stop endpoint but requires the
actual terminal turn state before claiming completion. Retried request IDs reuse
the registry result. Response typing and agent draft input share the recovery edit
handler; ordinary keystrokes remain immediate. Provider/model/backend settings,
permissions/edit review and history/project lens controls still need their own
complete disclosure and actor-aware adapters.

Conversation agent/provider/model and collapsed-lineage controls now register
inside the actual feature settings panel. Backend choices use the observed agent
catalogue and the same preference/new-thread callback; actual connection remains
the existing ACP/Python flow. Catalogue reads are explicit bounded byte pages.
Provider/model updates configure the next turn, without claiming a login or
sending a request. Closing settings retires these commands; switching to ACP
retires Python-only model controls. Permissions and edit acceptance remain
separate actor-policy work.

Conversation edit proposals now expose bounded current-state reads, existing
change-review navigation and an addressable human decision request. Agents cannot
accept/reject a pending proposal on the human's behalf or enable auto-accept;
requests retain the proposal identity and later reads show the actual host status.
Human decisions use the same apply-before-acknowledge controller, including stale
editor errors and revised bytes. Existing externally authorized ACP auto-apply
behavior remains separate; Python still requires each approval.

Context assembly now awaits its originating UI handshake for at most three
seconds before reading buffers. A pending/failed attached connection prevents
submission with a retry/reopen reason; it no longer silently drops UI tools.
Closing that origin rejects its capture even if a replacement connects. A
hostless embedding with no UiActionsProvider still captures without a window.
An established capture remains bound to its original window; it never reattaches.

ACP permission cards now have exact bounded reads and addressable human-choice
requests. Agent invocation cannot answer a permission itself. Human choices use
the actual host permission endpoint; cards retire only on `answered: true`, and a
refused answer stays visible with an error. Adapter tool details have explicit
open/close disclosure and bounded byte reads; closed details unregister their
reading commands. Permission and provider authority remain owned by the actual
adapter/host, not by a rendered action reference.

Adapter connection/configuration reads now expose only the current disclosed
settings and counts for separate permission/tool cards. Runtime checking,
reconnect, adapter installation and typed model/configuration choices share the
existing host controller and report its actual ready/error/session result.
Superseded async results report that the originating operation may have completed.
Sign-in and adapter session mode changes expose explicit human request paths.
Known permission/sandbox/approval/mode configuration fields also require human
input; other opaque adapter configuration semantics retain the adapter's own
permission model. Agent command help requires its actual disclosure before bytes
are included in connection reads. Configuration revisions include document,
backend and adapter session, not just the displayed value.

Recorded ACP activity now uses the same progressive evidence disclosure in dock
and session views. Collapsing a recorded item retires its own byte reader and any
nested live-detail readers; expanding exposes exact retained JSON through bounded
pages. Adapter-reported status remains a declaration, and recorded stops use a
neutral label rather than assuming the human initiated them.

Native command-path settings now share install/remove/refresh handlers and bounded
status reads. Enabled predicates retain the existing host availability, conflict
and removable-installation conditions. Actual returned installer state is reported;
it does not claim a CLI run or mutate another installation on conflict. Initial
status responses cannot replace a newer explicit refresh. A headless/missing host
keeps its ordinary hidden UI and exposes unavailable status through the adapter.

Dataset preview now exposes explicit remote S3 load, refresh, current status and
actual previous/next paging. Loading a new source/page invalidates old table
controls; fetched responses carry their request identity before display. Read-only
notebook tables expose current schema/values through bounded pages and the same
filter/sort setters used by their inputs. Filtering/sorting apply only to the
loaded preview, and reads preserve offset/total/has-more/column truncation scope.
Editable table cells, formula debugging, fill/layout and other renderer gestures
remain separate integration work.

Meeting recording now has actual panel disclosure, bounded catalogue/configuration/
retention/transcript reads, configuration drafts/save, recording supervision,
local transcription, recovery/delete/download and explicit cancellation. Start
microphone, remote consent/upload and reviewed-transcript import require actual
human input when requested by an agent. Cached checkbox state cannot authorize an
agent's remote upload. Capture returns its real recording ID; stop/final retention
and transcription/cancellation remain distinct states. Reads, disclosure and
cancellation have separate subjects so a pending transcription does not block its
own supervision. Closing/reopening the panel retains the current job.

Cell, diagram and math panels now expose explicit bounded controller-owned
readings. Cell readings include visible status/diff/output and preserve transcript
collapse; Replay must actually reveal hidden transcript before it is read. Diagram
readings distinguish current authored input from a stale rendered SVG and report
assertion states as declarations. Math reports actual rendered/fallback/pending
state and stages async rendering so an obsolete completion cannot overwrite a
new equation. The shared reader emits no DOM scrape or edit baseline. Image bytes,
visual model delivery, editable table/graph gestures and remaining renderers still
need their own complete adapters.

Shared in-app prompts now expose bounded current text/choice reads, draft input,
apply/choose/cancel and caller-owned actor policy. Rename/code-action and Save a
copy prompts explicitly enable agent input with live source guards; debugger watch
text enables ordinary input. Unclassified prompts, including unsaved-work choices,
retain human decision authority. Prompt IDs prevent identical titles from
retargeting an old handle. Replacing/unmounting settles pending promises as
cancelled. Settling a prompt resumes its caller; it does not certify that a later
save/refactor/debug operation succeeded. Native OS choosers remain separate work.

Workspace Save/Save a copy/Save all/Print/Insert/Edit element now share a scoped
controller with native menu dispatch. Source/focus/selection and open-buffer state
guard handles without disclosing source in discovery. Targets are captured before
awaiting formatting; unnamed saves use that exact tab ID. Native destination and
print dialogs expose human request paths, while the in-app Save a copy prompt is
addressable through the shared prompt controller. Known empty unnamed saves with
formatting off are no-ops. Save acknowledgements, self-saver flush intent, copy
prompt settlement and physical print output remain distinct; none is verification.

Insert/Edit Element forms now share typed catalogue, attribute/body draft, preview,
submit and cancel handlers. Their captured document editor, immutable source and
selection are retained through focus changes; stale/closed/read-only source is
refused. Opening the form does not write. Agent submission carries the actual actor
into the handler and uses an explicit agent UI transaction event; receipts do not
turn that event into human approval, verification or commit authorship. The native
entry points use the document-editor target registry rather than a later arbitrary
focused file/generated buffer. Each reopened form gets a new request identity.

Core editable tables now expose bounded controller/cell-region reads, cell writes,
selection/ranges/rows/columns, pending-edit commit/cancel, add row/column, actual
context-menu disclosure/actions, fill, formula/debugger/size disclosure, prose wrap
and row/column sizing/fitting through the existing controllers. Structural actions
require pending draft commit/cancel first. Authored CSV and Markdown table adapters
bind their owning editor, original document and block bytes; stale/readonly/closed
writes are refused before invoking the source callback. Agent actor is forwarded
through CSV/Markdown writes and recorded in the UI transaction event. Source UI
writes are not independent proposal approval, commit authorship or verification.
Formula computation reports pending/current/failed separately from editable scope.
Nested number-format, size-form and formula-step controls are the next slice;
remaining clipboard/formula-pointing/drag/scroll details still need parity review.

Capability wording now distinguishes independently approved document-edit previews
from routine registered host UI operations, including source changes. UI receipts
record agent actors and do not count as proposal approval or independent correctness.
The confined worker still has no direct filesystem/process access. Human-only
review/attestation, credentials and OS input remain actual human requests.

Nested table size/number-format/formula-trace controls are now registered with an
explicit owner link to the table. Resize drafts use the actual validation/loss
calculation and guard the source before Apply; formats preserve numeric raw values
and use the shared writer. Prose wrap lives behind the actual settings gear rather
than an always-available hidden-setting action. Formula trace navigation/read uses
the actual captured host trace, expires prior-source handles and drops old trace
content immediately when a new supplied revision is pending. Replay is cell order,
not expression stepping or reverse execution. Remaining clipboard/formula-pointing,
scroll/engagement and drag edge cases still require inventory review.

Table grid controls were extracted without changing selection or drag behavior. Focused checks: strict TablePanel TypeScript and 13 selection/resize/fit cases passed. The repository file-length check still reports other oversized files; their cleanup remains required.

Git history now has progressive bounded reading and shared navigation for loaded summaries, expanded bodies/files, commit readings, current files, older history, story and literate views. Working-tree command parity remains outstanding.

Git working-tree reads, drafts, navigation and routine host command handlers now share scoped controls and actual host replies. Push/amend/discard report explicit human-input decisions; fetched status is not an atomic disk baseline. Bisect and the other repository/merge interfaces remain outstanding.

Visual bisect now has progressive scoped session reading and real host command handlers. Good/bad verdicts explicitly require human classification; native-window/worktree execution remains unwitnessed. Remaining-commit evidence is a nested disclosure.

The issue's source changes now satisfy the file-length ratchet. Existing Git history types, ribbon measurement helpers, Python host protocol implementation, session bootstrap, diagnostics and output formatting were extracted into modules with their existing public entry points preserved. Existing inline Rust test modules moved to dedicated test files; fixture literals remain intact and the debugger fixture include path follows the new location. The baseline was regenerated only downward. Focused validation: strict affected frontend TypeScript; six ribbon/history checks; one each Python capability, debugger opaque-handle, MCP schema, parser BOM, agent DAG, adapter capability and volume flush safety check; CLI binary compilation; file-length and whitespace checks all passed. These are focused checks, not broad regression results.

Comparison layouts/source choices/forks now share scoped controls; compact options register only after margin disclosure. Visible draft edits use the existing model-change handler and invalidate equality evidence. Scoped editor and rendered-diff reading are available. This does not constitute proposal approval or a saved/verified correction. Captured-code navigation and patch gutter controls remain separate outstanding work.

Captured comparison references and failed-setup retry now have scoped controls and bounded state reads. Reference navigation awaits the exact captured-source request, retains host failures and rejects live changed-source handles. Late reference results are not adopted after a captured source replacement.

Merge previews/choices/cancel/accept now share scoped controls and current-input readings. Plain-file acceptance refuses changed owners and reopens the merge for changed disk text before adopting its baseline. Hidden plain-file editor controls retire during merge. At merge-only commit `e479ec8`, generated-output resolution still used the queued watch-loop API without an expected-source guard. The subsequent resolution integration below replaces that boundary.

Held-output resolution now sends both exact observed merge sides and awaits an applied/refused loop reply. The loop checks its current hold/produced bytes, tracked ownership, confined canonical target and disk bytes before writing; refusals retain the hold. Merge receipts retain the owning callback's actual reply, separating file application from later document propagation and verification. Focused filesystem safety checks (2), actual HTTP acknowledgement/refusal check (1), merge/restore checks (3), strict affected TypeScript, file-length and whitespace checks passed. The HTTP test required permission to bind its loopback listener. External filesystem writers can still race the comparison/rename interval; no atomic cross-process compare-and-swap or rollback is claimed.

The merge-driver notice now exposes scoped configuration/error reading, actual awaited initialization results and dismissal. Controls exist only for the displayed notice, with pending-operation protection.

Divergence keep/take/merge handlers and scoped state/error reads are shared. Promise-returning actions retain actual replies; void overwrite callbacks claim only request dispatch. Live buffer revisions refuse stale handles, and late reloads preserve newer edits. At divergence-only commit `0bb59df`, regeneration still awaited its expected-source/applied-result integration; the following entry completes that host contract.

Regeneration now requires the actual loaded disk side plus observed produced bytes and uses the same guarded, applied/refused loop acknowledgement as merge resolution. It restores generated-output permissions and no longer exposes the former unguarded queued host command. Focused evidence: three filesystem safety checks, the HTTP resolution/regeneration acknowledgement check and strict affected TypeScript passed. The HTTP test uses an explicitly allowed local loopback listener. Application still does not establish document propagation or focused verification, and unrelated external writers retain the previously documented comparison/rename race.

Plain-file action menus now register literate-view/review/adoption commands only after disclosure; closing retires those commands. Reads identify the owning path and current save/pending state, while private live buffer/hash guards expire stale handles. Adoption retains the actual host reply and new ownership even when subsequent UI navigation throws. Plain-file status reading remains available outside the menu and labels its hash as last-loaded metadata, not a current edit baseline. Automatic menu dismissal waits for the local human requests started by its click, so delayed intent persistence cannot retire a command before execution; a new disclosure epoch is protected from an older completion. Seven focused checks passed: menu intent-retention/reopened-epoch checks (2), scoped file action/navigation/adoption (1), existing file ownership display (3), and adopted-but-navigation-failed handling (1); strict affected TypeScript passed. No real adoption filesystem mutation is claimed by the mocked-host component checks.

Document/file/output/comparison source editors now opt into shared current-state range replacement and actual undo/redo history. Actual filters/listeners remain authoritative; replies distinguish requested text application from source change, persistence, approval and verification. Reading-only output instances now reject human/agent input and still reconcile source refreshes. Six focused editor/owner/history/filter/save/callback checks passed. At editor-input commit `04802aa`, multi-range selection, completion interactions and clipboard/gesture edges remained separate work; the following entry covers source/selection/fold paging.

Editor source reading now has explicit UTF-8 byte pages (up to 8192 raw bytes), independent of UTF-16 editing coordinates. Line reads refuse oversized content with a byte-reader recovery path; large fold sets use pages of at most 64 entries. Selection pages are revision-bound and bounded to 64 entries. Actual multiple-selection capability is disclosed, and typed selection/primary-range controls use normalized CodeMirror selections. Replacing current selections uses the native replacement operation and limits total inserted UTF-8 bytes to 1 MiB. Expand/collapse-all uses the actual library fold commands without source edits. Three focused Unicode/source-budget/fold/selection/input checks and strict affected TypeScript passed. Raw byte limits do not represent serialized JSON size or provider token cost; action-result accounting remains separate.

Shared UI registration/poll/reply/receipts/close now use versioned routes. The bridge validates its returned scope before attaching, with explicit update-app-and-engine recovery for an unsupported protocol. Guarded output resolution/regeneration use versioned paths as well, preventing an old engine from ignoring observed-side fields on its older unsafe endpoint. Nine focused checks passed: six frontend handshake/lifetime cases, actor/reply matching, actual HTTP old-route refusal/versioned registration/guarded-output application, and real confined Python UI transport/context retention with a scripted provider. These checks establish no real paid-provider or native released-flow witness.

The registry now snapshots disclosed state and completed results into immutable JSON before retaining/delivering them. Mutable controller state cannot rewrite an earlier receipt/retry, and changed disclosed values expire a known handle even without a caller's manual revision bump. Captured invocation parameters and actor are frozen. A non-serializable result after execution reports uncertain side effects and remains deduplicated. Action transport reports actual serialized `result_bytes` separately from raw content limits, transport/accounting envelopes and unmeasured provider token cost. Focused alias/tamper/retry/unknown-result/UTF-8-accounting, existing dedupe/state recheck, and menu journal ownership checks passed with strict affected TypeScript.

Shared editor replacements now validate ordered current ranges, complete UTF-16 character boundaries, well-formed resulting Unicode and a 1 MiB total inserted UTF-8 budget before dispatch. Single-range and current-selection edits share this preparation. A focused surrogate/malformed-input/no-change/whole-character case, the existing history case and multi-selection budget/input case passed with strict affected TypeScript. Editor filters remain authoritative and may transform/refuse a prepared transaction; the actual-result fields still distinguish that outcome from persistence or verification.

### Literate-view Python conversations (October 20)

Literate views now accept Hickory Python as well as ACP. Python uses the captured originating-window UI broker and a virtual editor context; it does not treat the synthetic conversation subject as a file. Sessions and hydration use workspace-owned literate-agent storage. The editor answers fresh completion capture requests with its current buffer identity and bytes. The prompt asks agents to discover available tools; ACP's named literate tools are not advertised as Python tools. Shared representation toolbar/menu/version controls remain a separate outstanding integration.

The literate-view main toolbar and disclosed More actions menu now share commands for source save/refresh, local discard, rebuild/navigation, local keep and opening the agent conversation. Menu commands retire while closed; live source revisions invalidate stale handles. Save receipts report actual acknowledged revision/file hashes and whether later edits remain dirty; keep and navigation replies make no source-save or verification claim. Version-selection and per-file code tools remain separate outstanding controls.

Literate revision selection is now an explicit disclosure. Base/target setters, comparison requests, return to loaded current code and unchanged-region folding share their host callbacks. Comparison receipts return resolved revisions and editability, not an execution claim. Per-file representation code tools and the view library still need their own control integration.

The view library now shares source-path selection, real create/open results, catalogue refresh, existing-view navigation and local arrangement discard. A bounded UTF-8 catalogue reading omits source/file contents. Host failures stay visible and failed discards retain their rows. The declaration inventory remains a review aid, not complete parity proof; per-file code-intelligence and remaining workspace families still require audit.

Editor language extensions now supply their own controller facet. The shared editor adapter follows late provider attachment/removal and mapped-source providers instead of registering unavailable fake capabilities. Definition/reference commands reuse human navigation handlers; hover inspection reuses the human hover query and creates a private immutable byte snapshot. Its explicit snapshot reader pages at most 8192 UTF-8 bytes without repeating runtime reads. Source changes or newer hover captures invalidate old handles/snapshots. Rename, code actions, completion, signature/inlay reading and remaining intelligence controls still need integration.

Installed language providers now expose shared formatting and typed symbol rename. Rename uses the same preparation/mapping/edit routine as the human prompt, with a typed model name instead of a blocking prompt. Formatting, rename and chosen human code actions validate the whole mapped batch before one native transaction: read-only, invalid/overlapping ranges, split surrogate pairs, malformed text and the shared total insertion budget refuse the batch. Source changes and provider removal cancel pending mutation authority before application. Replies report actual buffer changes, unsaved/unverified state and other documents reported by the server; they do not apply those other documents silently. Completion and code-action selection remain outstanding.

Code-action inspection now retains immutable provider choices bound to the exact source and selection. A paged title/choice catalogue and typed opaque choice submission use the same controller as the human key binding and popup callback. A fabricated title/ID cannot supply an edit. Unsupported server commands/resource operations are refused by both adapters without applying an edit prefix; implementing those operations remains a #75 code gap, not an OS limitation. Completion and other language annotations still require integration.

Native editor completion popups now share open/status/read/select/accept/close operations. The actual CodeMirror popup owns provider queries and application ranges; there is no second model-only completion pipeline. Opaque snapshot identities bind source, selection, native option identity and popup state; explicit byte pages retain project-vs-language origin information and actual insertion text. Accept receipts distinguish a requested native action, actual source change and saved/verified state. The native completion filter refuses malformed Unicode and oversized/invalid batches for both humans and agents.

Language annotations have an explicit bounded reader over the actual CodeMirror decoration/signature state: semantic marks, highlighted occurrences, inlay labels and active signature. Signature rendering preserves the human bottom-pinned surface and shares a source-bound field instead of a DOM-only value. Late token/hint/signature/highlight/folding replies refuse changed source or removed provider lifetime, and stale fold caches clear on editing. Decoration positions can remain mapped until fresh provider results; a rendered annotation is not independent type or execution verification.

Project-transition review now shares capture refresh, A-to-B finalization, authored revision creation/discard/application and incorporation/status commands. Ambiguous-write attribution checkboxes remain human-only with an addressable agent input request. Application rechecks the captured context against the exact draft and owning conversation before dispatch, records separate disk/editor outcomes, stops later editor writes after failure and exposes recovery bytes only through explicit disclosure. Source-equality verification remains separate from runtime verification and explanatory accuracy.

Provider evidence now shares pin/context selection, expansion, exact discussion/source navigation and nested diff/lineage disclosure. Expanded or pinned review context uses an explicitly labelled visible-comment projection; hidden original diff and raw lineage bytes require their own explicit read. Evidence identity/origin remain intact and selection is distinct from request delivery or operation authority. Browser link receipts report a requested opening and actual returned handle availability, not a page-load witness.

GitHub review import now shares typed repository/PR/document selection and read-only refresh, with explicit capture-status reading. Replies retain source identity, incomplete pagination and unavailable reviewed revisions. Changing selection or editor source during capture refuses late attachment; changing selection afterward marks the retained capture historical. No focused editor is not itself proof of source drift. These adapters never post replies, resolve threads or publish.

The history story lens now shares earlier-card disclosure, per-commit diff/detail navigation, draft reword/move/drop, eligible recipe replay, typed working-tree/recipe inputs, stage-all/commit and recipe-run/commit. Git-changing operations serialize across visible story cards. Receipts retain actual Git replies; failed commits preserve their message draft. Folded cards and closed details do not register their content controls. These are existing Git operations with host floor/working-tree checks; the browser snapshot is not a global filesystem/Git transaction or a runtime-verification witness.

Picture panels now expose explicit displayed-pixel capture through their own loaded image element. The bounded PNG is optional/downscaled at the requested maximum dimension, digest-bound and source/window-lifetime checked. Status metadata omits inline image bytes and URL query credentials. Python retains immutable artifacts and selects them as whole user-material request parts; Rust resolves only host-retained descriptors, never worker paths or remote URLs. Anthropic and OpenAI-compatible adapters emit native image content; ACP receives MCP image blocks. Text-only/unsupported models can still reject image requests explicitly. Browser canvas/CORS/secure-context failures return an addressable input request and visible retry guidance; oversized captures require a smaller explicit dimension. This covers picture panels, not every graph/canvas/media renderer.

Focused evidence for images: real Chrome pixel/digest capture, retained artifact reload/tamper/forged descriptor refusal, worker artifact assembly, real confined Python and HTTP broker with a scripted model/UI endpoint, MCP image validation and quoted-prose refusal, two provider wire-format checks, and the existing text-only worker transport case. No paid-model interpretation, native package or released/deployed recording is asserted.

### Graph controller checkpoint (2026-10-20)

Graph scenes now expose explicit byte-paged controller reading (nodes, edges,
layout, selection, reported assertions and focus ownership), shared toolbar
add/layout/fit commands, and per-node rename/move/resize handles. Native drag,
resize and rename use the same mutation callbacks. Derived topology remains
read-only while layout stays editable. Receipts report a requested document
commit, never a confirmed disk save. Commands bind the current authored and
resolved inputs; changing only the resolved topology reloads the scene.
Focused tests cover both actor paths, snap/round semantics, stale handle refusal,
derived topology protection, teardown and resolved-input replacement.

This checkpoint does not certify graph selection/inspector styling, connector
reconnection, inline label drafts, viewport navigation or rendered pixel capture.
Those remain part of the issue's incomplete capability audit.

The next graph checkpoint adds native selection synchronization, per-object
finite palette/style/label controls, node/edge deletion and node connection/
edge reconnection through shared semantic handlers. Connector names are read
from the actual side/slot model and validated before edits; parallel edges keep
separate identities. Focused tests exercise an ACP-labelled actor followed by a
native inspector edit, invalid inputs without writes, connection/reconnection,
and incident-edge removal. Inline draft and viewport/pixel coverage remains open.

Graph inline node and edge label drafts now have open/read/change/commit/cancel
controls shared with native typing. A changed committed label requires explicit
recovery and retains the draft; cancellation/reopening starts from the current
label. The inspector's label input is bound to its edge identity rather than a
later selection. Viewport reading and bounded pan/zoom setting use React Flow's
own controller. Focused tests cover mixed native/agent drafting, stale node and
edge label refusal, viewport arguments and existing native inspector behavior.
This still does not establish rendered graph/diagram pixel capture or exhaustive
browser gesture parity.

### Files buffer reconciliation checkpoint (2026-10-20)

`FilesystemTreeEditor` now opts into the shared editor reading/editing/history/
selection/fold controllers and registers its Dry Run, Apply, deletion confirmation
and cancellation handlers. Paged status includes the actual operation receipts
and required deletion request. Draft changes invalidate deletion confirmation;
pending writes exclude duplicate Apply and native document edits. Cancellation,
owner replacement and failures stop subsequent batch operations while reporting
completed responses and uncertain failed effects without claiming rollback.
Focused tests use real CodeMirror with mocked filesystem/provider replies.
Filesystem activation and context-menu mapping still need their own audit.

Files row activation and context menus now use shared line-target resolution,
including presentation-only reorders and associated-object activation callbacks.
The model's menu request supplies a controller-owned anchor point; native mouse
events use that same semantic menu boundary. No synthetic DOM event is dispatched.

### GitHub work-item draft checkpoint (2026-10-20)

Expanded GitHub issue/PR bodies and comments now have explicit paged reading,
separate draft setters and shared submit controls. A provider body that changes
while locally edited preserves that draft and requires an explicit reset/review;
there is no implicit overwrite. Pending submission excludes another submission,
failed comments retain their drafts, and actual provider replies are returned
without claiming independent verification or cross-client compare-and-swap.
Focused tests use mocked provider replies; no GitHub content was posted. The
collapsed work-item rows, provider status, lazy checks and outbound links still
need disclosure/read/action registration.

GitHub workspace rows now register explicit disclosure/status controls and
separate collapsed summaries. Expanding an issue/PR or check performs its native
lazy fetch; body, comments, reviews, conflicts and actual bounded logs are readable
only in the expanded subtree. Closing it retires child handles. Refresh and
pending-read discard have actual controller state; discarding a reply does not
claim to cancel the host request. A changed row source refuses late replies and
labels prior captures while refreshing, preserving body drafts and disabling
submission until the current capture arrives. Notification read and HTTPS GitHub
navigation use shared handlers and report actual replies/navigation requests.
Focused tests cover disclosure/retirement, lazy network boundaries, stale responses,
draft retention, notification replies, native/agent link dispatch and unsafe-link
refusal. Host/provider replies remain mocked; no live check logs were monitored.

### New Project checkpoint (2026-10-20)

Measured templates/options and current form/preview have explicit reading and
typed shared field/selection commands. Create returns the actual scaffold terminal
as running, without asserting its eventual commit. Pending start excludes repeat
submission, edits and close; template/language/toolchain and preview responses bind
their originating inputs. Native folder picking uses the same handler and actual
response; builds without it expose typed location entry and a disabled reason.
Focused tests use mocked toolchain/chooser replies, not native/provider witnesses.

### Assumption audit checkpoint (2026-10-20)

Audit search/filtering, visible-record and nested raw reading, source navigation,
session byte export/download requests and verification standing now have shared
controls. The annotation form's draft and required-human-declaration/result are
addressable. Its existing endpoint records human declarations; agent submission
is refused with a concrete request instead of impersonating human testimony.
Existing model audit tools retain model attribution. Focused tests prove no POST
for an agent and the actual mocked reply for a human using the prepared draft.

### Scratchpad checkpoint (2026-10-20)

Scratchpad reading, draft replacement and save now share native/agent handlers.
Agent-assisted drafts and agent-triggered saves retain explicit authorship on a
versioned API, with no silent downgrade to the human-only route. Pending saves
exclude another save and native edits; refusals preserve text. Focused testing
also exposed and fixed legacy `.hick` output and a concurrent filename race:
new notes are `.md` and use exclusive creation. Tests include real engine
parse/weave, a real loopback HTTP route, and mocked-host mixed UI actor cases.

Shared `ActionNotice` recovery callbacks and visible literal feedback now have
registered reading/action controls, bound to actual callback identity. A recovery
request does not report a successful save. `ChangePreview` uses shared disclosure
and bounded reading; collapsed children stay unmounted. Custom child components
are counted as unresolved in literal-text projection and keep their own reading
responsibility. Focused tests cover native/agent retry, stale callback refusal,
preview mounting and child-reader retirement.

### Fleet controller checkpoint (2026-10-20)

Fleet roster/status, pairing, disclosed invitations, grants/removal and actual
replies now use shared controllers. Hidden invitation controls retire on close;
raw replies require their own disclosure. Pending foreground operations exclude
another mutation and native input; stale roster-key handles are refused.
Clipboard denial reports a required-input request instead of false success.
Validation uses mocked local fleet APIs, without real pairing or grant changes.
Host pairing cancellation and stronger cross-process target identity checks
remain explicit code gaps; this checkpoint is not full fleet certification.

Fleet grant/removal now use versioned observed-key preconditions, checked under
the host roster's stable write lock. Updated CLI/server writers serialize and
atomically replace complete rosters; unreadable rosters cannot become empty
writes. Focused tests include four real processes writing 40 private-fixture
entries, stale key refusal without changes, current phone authority, required-key
HTTP refusal and no client downgrade. This resolves the cooperating-writer
identity gap noted above; arbitrary noncooperating file editors and legacy
unscoped callers are outside that precondition contract. Pairing cancellation
still needs its actual host request/result/cancel implementation.

Window-owned versioned fleet pairing jobs now expose start/list/status/cancel/
forget with live-window checks, exact-argument idempotency, bounded retention,
actual cancellation outcomes and close cleanup. Peer transport refuses these
local authority routes. Four focused fixture-worker tests pass; this backend
checkpoint does not claim UI adoption or a native/provider witness.

The Fleet pane now adopts those window-owned jobs through the shared UI scope.
Native/model start, metadata reading, cancel, refresh and forgetting share real
host identities; full replies require disclosure. Unknown starts reuse exact
request IDs; early cancellation forwards only after identity recovery and never
claims confirmed stopping. Closed controllers refuse late adoption. Focused
mocked-host/native-control/agent-action and versioned-API tests pass; the earlier
UI host-cancellation code gap is resolved for this request surface. This does
not establish universal native/provider/release parity.

Shared ContextMenu now owns revision-bound item/dismiss actions and paged
metadata while open. Same-label callback replacement retires old authority;
async picks exclude duplicates and report actual callback failure. Dismissal
never claims operation cancellation. Two focused callback-fixture tests pass;
caller-specific write/result contracts remain owned by their controllers.

FenceConvert and CSV dataset promotion now expose shared drafts, paged preview/
metadata and guarded conversion/cancel controls. Real CodeMirror tests witness
stale-preview refusal, source-span tracking through repeated CSV size changes,
and exact promotion with surrounding bytes preserved. Grid writes retain agent
attribution. Conversion is explicitly a source edit, not saving, execution or
output writing; these focused tests do not certify the release/provider flow.

Output-baseline controls now bind async status/pin/drop requests to the actual
document lifetime, serialize writes and refuse stale status adoption. Native
menu callbacks await the same controller. The visible badge has paged verdict/
tooltip reading and shared refresh/stop commands; failed drops remain visible.
Three focused fixture-API tests pass, without claiming a new live comparison,
provider or release witness.

Terminal controller actions now consume actual host-request success/refusal
instead of reporting completed after swallowed errors. Polling cannot erase an
operation error or adopt an older supervision reply, closed controllers refuse
new writes, and automatic-answer state comes from the host. Seven focused
fixture/controller tests pass. The existing prompt-human-authority policy is
unchanged and is not claimed as an OS exception or full prompt-answer parity.

Recording-strip and monitor-dock status/actions now share explicit controllers.
Visible metadata is paged; current target/callback replacement retires handles,
pending callbacks exclude duplicates, and actual host refusal is reported.
Navigation callbacks do not certify process state. Two focused mocked-callback
cases and six recording-strip presentation cases pass. No concurrent host-anchor
CAS, real PTY/provider or native-release claim is added by this checkpoint.

AttentionCard now has paged visible reading, shared queue/navigation/interrupt
controls and explicitly labeled agent-prepared reply drafts. Existing human
prompt authority yields a question-bound decision request; actual human
callbacks report host success/refusal with that request identity and no semantic
approval attestation. Changed questions retire old drafts/handles. One new mixed
callback-fixture case and five presentation cases pass. This preserves an
application-policy boundary, not an OS exception or full prompt-answer parity.

Source-observation visibility/pinning now shares native state and the existing
selected-context projection. Root reading excludes annotation payloads; detailed
rendered-line/evidence reading appears only while expanded and retires on close.
Source/stop changes refuse old handles; pinning folded evidence remains explicit.
Five existing context cases and one new mixed fixture case pass, without adding
a live debugger/provider/release witness or claiming provider delivery.

Runtime-log relationships now expose shared disclosure, visible stream/source/
style selectors, paged contributor metadata, location navigation and review
requests. Missing-source captures remain readable. Review hashing binds the
actual record/event source and refuses replaced/closed captures; queued source
jumps also bind their source record. Two focused fixture UI cases with real
read-only editors and a mocked review opener pass. Ribbons outside this owned
runtime-log path and actual provider/release/deployed witnesses remain separate.

A focused registry-level review found and fixed operation-lock interference:
menu dismissal and pending fleet-start cancellation now have separate subjects
from the callbacks they control. Two focused tests hold the original registered
action pending while invoking the escape command, then settle the actual result.
This corrects the earlier direct-hook coverage gap without claiming rollback.

WorktreeRegionEditor now shares source-bound route/reset/apply controls and
paged state. Changed parent snapshot/region/input/writable membership retains a
read-only stale draft until explicit reset. Missing snapshots refuse writes;
pending apply excludes duplicates and exposes actual per-target callback replies.
Two real-CodeMirror/mocked-write cases pass. The containing MergedView's source
loading, target selection, undo and late-result lifetimes remain outstanding;
this child checkpoint does not certify the complete worktree view.


MergedView now shares membership, write-target permission, region disclosure,
complete-project inspection, refresh, paged displayed reading and guarded undo.
File/membership lifetimes refuse stale adoption/retargeting. Unknown write effects
require refreshing before further region edits; earlier replies are labeled
separately from a newer failed request. This resolves the containing-view gaps
named in the preceding region-child checkpoint for these tested paths.

Five focused fixture-controller cases, one integrated native/agent/CodeMirror
case and six presentation cases pass. Writes/undo in web tests are mocked. The
versioned read route separately has a real temporary-worktree HTTP witness for
all/empty/literal-comma membership, exact spaced filenames and malformed-input
refusal; two parser, one peer-policy and one no-downgrade client case also pass.
No native/provider/released/deployed witness or exhaustive coverage is claimed.

Lineage replay now shares stop selection, working-tree recovery and paged
selection/status reading. Unknown revisions are explicitly unavailable, including
when no replay stops were loaded. Project replacement retires the old component
and its pending replies. Read failures retain replay recovery and a shared retry.
Historical replay excludes current working-tree structural navigation; replay
weaves source with today's binary and does not execute cells.

Eight focused slider cases and three mocked-API containing-view cases pass,
covering mixed native/model selection, stale project handles, empty-history
recovery, failed-read retry, late previous-project refusal and historical
structure exclusion. Targeted TypeScript checking passes. This does not certify
lineage column/fold controls or provide a native/provider/release witness.

Lineage columns now share toolbar search/link filters, per-file search/open,
visible-block selection, gutter folding, folded-range edge/all reveal and
other-stage disclosure/navigation. Paged column reading contains displayed lines
and rendered metadata; folded source requires reveal. Model replacement retires
column state/handles. Gutter gesture state is column-owned and cancelled when its
capture, file, ranges or mounted lifetime changes. An entirely folded file renders
one recovery hole instead of duplicating the same hole.

Twelve focused column cases pass, including real native gutter gestures, mixed
model fold/reveal and native/model file switching, stale capture handles and
unfinished-gesture cleanup. Targeted TypeScript checking passes. These are
in-process model/DOM fixtures, not actual native/provider or release evidence.

LaunchProfiles now shares the currently open picker's Run/Debug selection and
close handlers. Its paged reading includes displayed profile names, origins,
program/command, eligibility, context and errors; hidden environment/configuration
payloads are not copied into the reading. Replacing or closing the picker retires
selection handles, and command-only targets retain their actual Debug refusal.
A returned selection request does not claim process startup/debugger success.

Seven focused mocked-launch cases pass, including a mixed ACP-registry selection,
disabled Debug refusal and same-label picker replacement. Targeted TypeScript
checking passes; these cases do not start real processes or providers.

EditorRuler now shares native keyboard/drag and model wrapping-measure changes,
with bounds and owner/view/callback-bound handles. Paged reading reports the
shown proportional-font measure or rendered table-column bands. A hidden prose
marker has no model measure command. The document supplies its path owner;
queued measurements from a replaced view are ignored and pointer cancellation
discards the draft. A measure request does not claim source edits or persistence.

Ten existing ruler presentation cases and two new mixed-registry/stale-owner
and cancelled-pointer cases pass. Real CodeMirror is used with fixture geometry; targeted TypeScript
checking includes the containing document editor. This is not a native witness.

Recorded session cards now provide paged displayed-message/summary metadata and
shared reasoning/tool-result/input/context/observation/action disclosure. Bodies
remain behind the existing evidence disclosure; ACP cards retain their adapter
precision labels. Source chips share historical/current navigation and relationship
review requests with the native click/keyboard handlers. Changed captured links
retire old handles; missing exact historical review bytes require source capture.
Opening or challenging a source is not verification or a completed review.

SessionLens is owned by its path, rejects late previous-session adoption, exposes
read status and shared failure recovery, and cancels delayed scroll requests when
its capture retires. Five focused source-chip/disclosure cases, two existing lens
presentation/ribbon cases and two new lifecycle/retry cases pass. The ACP ribbon
test now targets the actual adapter-report caption. Targeted TypeScript checks
pass; these are recorded fixtures and mocked navigation/API boundaries, not live
provider or native-release evidence.

ProfileEvidence now shares disclosure, next-context pinning and displayed source
navigation. Aggregate rows are paged only while expanded. Source navigation opens
the actual recorded event; editor selection changes only when that event's source
bytes match the current editor. Stale displayed source disables navigation, and
run/source replacement resets disclosure/pinning and retires handles. Pin requests
do not claim provider context delivery; function totals remain separate from
per-line costs and include profiling overhead.

Two focused recorded-profile cases pass with a real CodeMirror fixture: mixed
native/model disclosure/pinning, exact aggregate reading, old-run handle refusal,
context-selection cleanup and unrelated-buffer navigation safety. Targeted
TypeScript checking passes. No real profiling execution/provider is asserted.

PatchEditor now shares file/side selectors only inside the existing Patch options
disclosure, plus fallback source opening, status reading and error reload. Owner
replacement retires inspection state; retired async saves do not adopt into the
replacement editor. Captured-project identity changes freeze comparison edits
until reinspection and refuse an old draft save. ComparisonEditor forwards its
agent input marker through patch regeneration into editable document transactions;
this is a local transaction marker, not a durable authorship certification.

Four existing patch behavior/safety cases and two new focused mixed-registry and
retired-inspection cases pass. The mixed case opens actual options, selects the
Old side, switches natively to New, edits the real CodeMirror draft through the
agent registry and checks the regenerated source/agent marker. Targeted TypeScript
checking passes. These cases do not certify cross-process project-apply races,
external captured-project persistence, provider execution or native packaging.
