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
command search and spatial layout also have registered controllers. Both Python and
ACP/MCP use the originating window broker. This is not universal UI parity.
No real native/provider recording, packaged-release witness or matching deployed
build witness has been supplied.

In the application, send a task from the workspace conversation, then call
`ui_observe({parent: "root"})`, observe a returned group and submit
`ui_act({handle, arguments: {}, request_id: "operation-1"})`. Python snippets use
`tool("ui_observe", {...})` and `tool("ui_act", {...})`. The tools use the
window captured at Send; their schemas cannot choose a different window or actor.
Standalone CLI agents have no attached UI and do not advertise these tools.

The adapters are `lib/uiActionBridge.tsx` and `serve/ui_actions.rs`. A host-minted
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
excluding its accounting/transport envelope; provider token cost remains explicitly
unmeasured. Opening a menu or expanding a view must invoke its real state transition before
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
records source hashes, JSX controls and callback expressions, event listeners,
imperative DOM creation/dispatch and keymap declarations. Regenerate with
`node scripts/inventory-ui-actions.mjs`; use `--check` to detect drift. It is a
review aid, not an assertion that arbitrary callbacks are safe semantic commands.
The generated index records current file and declaration counts. Declarations can render many instances; these are not capability counts.
Spreads, dynamically constructed controls, canvas gestures, native menus and
rendered values need the manual mapping below and further adapter review.

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
