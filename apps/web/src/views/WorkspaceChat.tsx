import { providerObservationContext } from "../debug/providerObservations";
import { useContextProjectionActions } from "./uiContextProjectionActions";
import { captureUiWindow } from "../lib/uiActionBridge";
import "./contextProjection.css";
import { projectDocument, projectionEstimate, retainedReadingState, type ReadingProjection } from "../lib/contextProjection";
import { observationContext } from "../debug/observations";
import { bufferTarget } from "../lib/editorBufferTargets";
import { useEffect, useMemo, useReducer, useRef, useState, type MutableRefObject } from "react";
import { ChatDock } from "../components/ChatDock";
import { api, request } from "../api/client";
import { getWorkspaceRealtime, LocalRealtime } from "../api/realtime";
import type { AgentEditorContext } from "../api/agentTypes";
import { panes, type Layout, type Tab } from "../shell/layout";
import type { SessionRegistry } from "./documentSession";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { bufferEditor } from "../lib/editorBufferTargets";
import { applyAgentEdit } from "../lib/agentEdit";
import type { AgentChange } from "../api/acp";
import { openPlainSearchFiles, plainSearchEditor } from "../lib/workspaceSearch";
import type { FilesResponse } from "../api/types";
import { requestRecordedPatch, type RecordedPatchSource, type ReviewRecordedPatch, type WorkspaceDraftStage } from "../simple/AgentProposalAdapter";
import { AgentProposalModel, proposalHash, type PreparedProposal, type ProposalScope, type VerifiedProposal } from "../simple/AgentProposals";
import { invertChangePlan, planWorkspaceEdit, type ChangePlan, type WorkspaceChangeHistory } from "../simple/ChangePlan";
import { RecordedPatchReview } from "../simple/AgentProposalView";
import { normalizedSource } from "../simple/SourceText";

export function editorTabs(layout: Layout): Tab[] {
  return panes(layout.root).flatMap(pane => pane.tabs).filter(tab =>
    ["document", "generated", "file", "untitled"].includes(tab.kind),
  );
}

export async function editorContext(
  tabs: Tab[], focused: string | null, registry: SessionRegistry,
  untitled: Record<string, string>, plain: Map<string, string>,
  selection?: Record<string, "expanded" | "pinned">,
): Promise<AgentEditorContext> {
  const originatingWindow = await captureUiWindow();
  const visiblePlain = new Map(openPlainSearchFiles().map(file => [file.path, file.content]));
  const buffers = await Promise.all(tabs.map(async tab => {
    const session = registry.get(tab.docId);
    let content: string;
    if (tab.kind === "untitled") content = untitled[tab.id] ?? "";
    else if (tab.kind === "document") {
      content = session?.docEditor?.state.doc.toString() ?? session?.liveSource
        ?? (await api.doc(tab.docId!)).source;
    } else {
      content = plain.get(tab.id) ?? session?.openOutputs.get(tab.target)?.state.doc.toString()
        ?? visiblePlain.get(tab.target)
        ?? session?.outputs.get(tab.target)?.content ?? (await api.file(tab.target)).content;
    }
    return { id: tab.id, kind: tab.kind, ...(tab.kind === "untitled" ? { provenance_uri: bufferTarget(tab.id) } : {}), ...(tab.kind === "generated" ? { document: session?.doc?.path ?? tabs.find(t => t.kind === "document" && t.docId === tab.docId)?.target } : {}), name: tab.title ?? tab.target, path: tab.kind === "untitled" ? null : tab.target,
      observations: [...observationContext(tab.target, content), ...providerObservationContext(tab.target, content)], content, focused: tab.id === focused };
  }));
  const documents: ReadingProjection["documents"] = [];
  for (const buffer of buffers) {
    if (!selection?.[buffer.id]) continue;
    const tab = tabs.find(t => t.id === buffer.id)!;
    const session = registry.get(tab.docId);
    const view = tab.kind === "untitled" ? bufferEditor(tab.id) : tab.kind === "document" ? session?.docEditor
      : tab.kind === "generated" ? session?.openOutputs.get(tab.target) : plainSearchEditor(tab.target);
    const key = tab.kind === "untitled" ? bufferTarget(tab.id) : tab.kind === "document" ? `doc:${tab.docId}`
      : tab.kind === "generated" ? `output:${tab.docId}:${tab.target}` : `file:${tab.target}`;
    documents.push(await projectDocument(buffer.id, buffer.content, view ?? retainedReadingState(key), selection[buffer.id] === "pinned"));
  }
  // Keep the capture's buffer identity: a provider URI can be an alias rather
  // than the editor path, and flattening observations otherwise loses that join.
  const revisions = new Map(documents.map(document => [document.id, document.revision]));
  const observations = [...new Map(buffers.flatMap(buffer => buffer.observations.map(record => ({
    ...record, source_buffer: { id: buffer.id, revision: revisions.get(buffer.id) ?? null },
  }))).map(record => [JSON.stringify(record), record])).values()];
  const projection: ReadingProjection = { version: 1, documents, observations, estimate: projectionEstimate({ documents, observations }) };
  return { live: true, buffers, ...(originatingWindow ? { ui_window: originatingWindow } : {}), ...(selection ? { projection } : {}) };
}

/** A conversation belongs to the window, independent of which editor is active. */
type UntitledPatchSource = Pick<RecordedPatchSource, "text" | "revision" | "current">;
const matchesScope = (left: ProposalScope | null, right: ProposalScope) =>
  left?.namespace === right.namespace && left.workspace === right.workspace;
const patchScope = (root: FilesResponse | undefined): ProposalScope | null =>
  root?.root_path && root.workspace_id && /^[a-f0-9]{32}$/.test(root.workspace_id)
    ? {namespace: root.root_path, workspace: root.workspace_id} : null;

export function captureWorkspacePatch(
  tabs: readonly Tab[], scope: ProposalScope,
  readNamed: (tab: Tab) => RecordedPatchSource | null | undefined,
  readUntitled: (id: string) => UntitledPatchSource | null,
  alive: () => boolean,
) {
  const named = new Map<string, RecordedPatchSource>(), ids = new Set<string>();
  const ownership_sources: Record<string, string> = Object.create(null);
  const unaddressed_sources: Record<string, string> = Object.create(null);
  const saved_guards: Record<string, string> = Object.create(null);
  const owners: {current: () => boolean}[] = [];
  const sources = [];
  for (const tab of tabs) {
    if (tab.kind === "generated") continue;
    if (ids.has(tab.id)) throw Error("The original authored tab identity is ambiguous.");
    ids.add(tab.id);
    if (tab.kind === "untitled") {
      const source = readUntitled(tab.id);
      if (!source || !Number.isSafeInteger(source.revision) || source.revision < 0) {
        throw Error("An unnamed draft is still loading. Open it before reviewing this patch.");
      }
      owners.push(source);
      unaddressed_sources[tab.id] = source.text;
      sources.push({id: tab.id, path: null, save_guard: null, source_revision: String(source.revision), text: source.text});
      continue;
    }
    const source = readNamed(tab);
    if (!source || source.path !== tab.target || !matchesScope(source.scope, scope) || !source.guard
      || !Number.isSafeInteger(source.revision) || source.revision < 0
      || source.path.includes("\\") || source.path.includes(":") || /[\u0000-\u001f\u007f]/.test(source.path)
      || source.path.split("/").some(part => !part || part === "." || part === "..")) {
      throw Error("Open the original guarded editor before reviewing this patch: " + tab.target);
    }
    if (named.has(source.path)) throw Error("More than one editor owns this source: " + source.path);
    let text = source.text;
    if (source.base.includes("\r") && source.base !== text) {
      if (normalizedSource(source.base) !== text) {
        throw Error("This edited source has no exact line ending capture: " + source.path);
      }
      text = source.base;
    }
    owners.push(source);
    named.set(source.path, {...source, text});
    ownership_sources[source.path] = text;
    saved_guards[source.path] = source.guard;
    sources.push({id: tab.id, path: source.path, save_guard: source.guard, source_revision: String(source.revision), text});
  }
  const current = () => alive() && owners.every(owner => owner.current());
  const retain = async () => {
    const drafts: Record<string, unknown> = Object.create(null);
    for (const [path, source] of named) {
      drafts[path] = {save_guard: source.guard, text: source.text, base_sha256: await proposalHash(source.base), version: null, writable: true};
    }
    return {ownership_sources, unaddressed_sources, saved_guards, drafts};
  };
  return {current, sources, retain, named};
}

type DraftOwner = (path: string) => RecordedPatchSource | null | undefined;
type CapturedPatch = ReturnType<typeof captureWorkspacePatch>;
type DraftBinding = { tabId: string; guard: string };
type WindowHistoryPlan = ChangePlan & { scope: ProposalScope; bindings: ReadonlyMap<string, DraftBinding> };
type PreparedWindowDraft = PreparedProposal & { plan: WindowHistoryPlan; stages: WorkspaceDraftStage[] };

function stageDrafts(plan: ChangePlan, readOwner: DraftOwner): WorkspaceDraftStage[] {
  return plan.patches.map(patch => {
    const owner = readOwner(patch.path);
    const stage = owner?.text === patch.before && owner.current() ? owner.prepareDraft?.(patch.after) : null;
    if (!stage?.current()) throw Error(`Open the current authored editor before applying this patch to a draft: ${patch.path}`);
    return stage;
  });
}

export function commitWorkspaceDrafts(stages: WorkspaceDraftStage[]) {
  if (!stages.every(stage => stage.current())) throw Error("An editor changed. Review the patch again.");
  const attempted: WorkspaceDraftStage[] = [];
  try {
    for (const stage of stages) {
      attempted.push(stage);
      stage.commit();
    }
  } catch (reason) {
    const errors = [reason];
    for (const stage of attempted.reverse()) {
      try { stage.rollback(); } catch (error) { errors.push(error); }
    }
    if (errors.length > 1) throw new AggregateError(errors, "The workspace could not restore the interrupted draft transaction.");
    throw reason;
  }
}

async function prepareWindowDraft(proposal: VerifiedProposal, capture: CapturedPatch, alive: () => boolean): Promise<PreparedWindowDraft> {
  const source = proposal.value;
  const plan = planWorkspaceEdit(source.intention.title, source.intention.capture.edit,
    new Map([...capture.named].map(([path, owner]) => [path, {text: owner.text, writable: !!owner.prepareDraft}])));
  if (plan.patches.length !== source.plan.patches.length || source.plan.patches.some(({path, before, after}) =>
    before.kind !== "file" || after.kind !== "file" || !plan.patches.some(patch =>
      patch.path === path && patch.before === before.text && patch.after === after.text))) {
    throw Error("The exact captured edit must describe every reviewed source transition.");
  }
  const after_hashes = Object.fromEntries(await Promise.all(plan.patches.map(async patch => [patch.path, await proposalHash(patch.after)])));
  const current = () => alive() && capture.current();
  if (!current()) throw Error("An original editor changed. Review the patch again.");
  const stages = stageDrafts(plan, path => capture.named.get(path));
  const bindings = new Map<string, DraftBinding>();
  for (const entry of capture.sources) {
    if (entry.path && entry.save_guard) bindings.set(entry.path, {tabId: entry.id, guard: entry.save_guard});
  }
  return {proposal, kind: "draft", current: () => current() && stages.every(stage => stage.current()),
    after_hashes, plan: {...plan, scope: {workspace: proposal.workspace, namespace: proposal.namespace}, bindings}, stages};
}

function invertWindowDraft(plan: WindowHistoryPlan): WindowHistoryPlan {
  return {...invertChangePlan(plan), scope: plan.scope, bindings: plan.bindings};
}

export function WorkspaceChat({ layout, registry, untitledSources, plainSources, plainPatchOwners, untitledRevisions, patchRoot, folder, focusedEditor, onOpenSession, draftHistory }: {
  layout: Layout;
  registry: SessionRegistry;
  untitledSources: MutableRefObject<Record<string, string>>;
  plainSources: MutableRefObject<Map<string, string>>;
  plainPatchOwners?: MutableRefObject<Map<string, () => RecordedPatchSource | null>>;
  untitledRevisions?: MutableRefObject<Map<string, number>>;
  patchRoot?: FilesResponse;
  folder: string | null;
  focusedEditor: string | null;
  onOpenSession: (path: string) => void;
  draftHistory?: WorkspaceChangeHistory;
}) {
  const [projectionOpen, setProjectionOpen] = useState(false);
  const projectionEpoch = useRef(0);
  const [selection, setSelection] = useState<Record<string, "expanded" | "pinned">>({});
  const [preview, setPreview] = useState<AgentEditorContext | null>(null);
  const [previewError, setPreviewError] = useState("");
  const tabs = editorTabs(layout);
  const focused = tabs.some(tab => tab.id === focusedEditor) ? focusedEditor : tabs[0]?.id ?? null;
  const realtime = useMemo(() => getWorkspaceRealtime() ?? new LocalRealtime(), []);
  const scope = patchScope(patchRoot);
  const latest = useRef({ tabs, focused, scope });
  latest.current = { tabs, focused, scope };
  const sourceRoster = JSON.stringify([scope, tabs.filter(tab => tab.kind !== "generated")
    .map(tab => [tab.id, tab.kind, tab.target, tab.docId])]);
  const rosterStamp = useRef({key: sourceRoster, revision: 0});
  if (sourceRoster !== rosterStamp.current.key) {
    rosterStamp.current = {key: sourceRoster, revision: rosterStamp.current.revision + 1};
  }
  const [patchReview, setPatchReview] = useState<{ review: VerifiedProposal; anchor: HTMLElement } | null>(null);
  const patchTicket = useRef(0);
  const captureForApply = useRef<{ review: VerifiedProposal; capture: CapturedPatch } | null>(null);
  const resolveOwner = (tab: Tab) => tab.kind === "document"
    ? registry.get(tab.docId)?.recordedPatchSource : plainPatchOwners?.current.get(tab.id);
  const [, paintReview] = useReducer((version: number) => version + 1, 0);
  const model = useMemo(() => new AgentProposalModel((method, path, body) => request(method, path, body), {
    scope: () => {
      const active = latest.current.scope;
      if (!active) throw Error("Open the original workspace folder to apply this patch.");
      return active;
    },
    prepare: (review, alive) => {
      const selected = captureForApply.current;
      if (!draftHistory || selected?.review !== review) throw Error("The original patch editors are unavailable.");
      return prepareWindowDraft(review, selected.capture, alive);
    },
    draft: prepared => {
      const batch = prepared as PreparedWindowDraft;
      commitWorkspaceDrafts(batch.stages);
      const plan = invertWindowDraft(batch.plan);
      draftHistory?.record({kind: "draft", plan, read: () => plan});
    },
    publish: async () => { throw Error("This review has no resource publication action."); },
  }, "Hickory Docs"), [draftHistory]);
  model.changed = paintReview;
  useEffect(() => () => { patchTicket.current++; model.changed = () => {}; model.close(); }, [model]);
  const changeDraftHistory = (direction: "undo" | "redo") => {
    const token = draftHistory?.preview(direction);
    if (!draftHistory || !token) return;
    try {
      const plan = draftHistory.readDraft(token) as WindowHistoryPlan;
      if (!matchesScope(latest.current.scope, plan.scope)) throw Error("Open the original workspace to use this draft history.");
      commitWorkspaceDrafts(stageDrafts(plan, path => {
        const binding = plan.bindings.get(path);
        const tab = latest.current.tabs.find(tab => tab.id === binding?.tabId && tab.target === path);
        const owner = tab && resolveOwner(tab)?.();
        return owner?.guard === binding?.guard ? owner : null;
      }));
      const inverse = invertWindowDraft(plan);
      draftHistory.commit(token, {kind: "draft", plan: inverse, read: () => inverse});
      model.notice = direction === "undo" ? "Workspace draft Apply undone." : "Workspace draft Apply redone.";
    } catch (error) { model.notice = String(error); }
    paintReview();
  };
  const closePatchReview = () => {
    patchTicket.current++;
    captureForApply.current = null;
    model.close();
    setPatchReview(null);
  };
  const reviewRecordedPatch: ReviewRecordedPatch = async (turn, fence, messageCurrent, anchor) => {
    const captured = latest.current;
    if (!captured.scope) throw Error("Open a workspace folder to read a guarded patch.");
    const ticket = ++patchTicket.current;
    model.close();
    setPatchReview(null);
    const originalRoster = rosterStamp.current, originalScope = captured.scope;
    const alive = () => patchTicket.current === ticket && messageCurrent() && rosterStamp.current === originalRoster;
    const capture = captureWorkspacePatch(captured.tabs, originalScope, tab => {
      const read = resolveOwner(tab), source = read?.();
      return source && {...source, current: () => resolveOwner(tab) === read && source.current()};
    }, id => {
      const view = bufferEditor(id);
      if (!view?.dom.isConnected || !untitledRevisions) return null;
      const revision = untitledRevisions.current.get(id) ?? 0, document = view.state.doc;
      return {text: document.toString(), revision, current: () => bufferEditor(id) === view
        && view.dom.isConnected && view.state.doc === document && (untitledRevisions.current.get(id) ?? 0) === revision};
    }, alive);
    const focus = capture.sources.find(source => source.id === captured.focused)?.path ?? null;
    const review = await requestRecordedPatch(originalScope, turn, fence, capture.current, capture.sources, capture.retain, focus);
    if (!capture.current()) throw Error("The original editor or message changed. Open the fence again.");
    setPatchReview({ review, anchor });
    if (draftHistory) {
      captureForApply.current = { review, capture };
      void model.open(review.token, review);
    }
  };
  const getContext = async () => editorContext(latest.current.tabs, latest.current.focused,
    registry, untitledSources.current, plainSources.current, selection);
  function chooseContext(id: string, mode: "off" | "expanded" | "pinned") {
    projectionEpoch.current++;
    setSelection(current => { const next = { ...current }; if (mode === "off") delete next[id]; else next[id] = mode; return next; });
    setPreview(null); setPreviewError("");
  }
  async function refreshProjection() {
    const epoch = ++projectionEpoch.current;
    const captureTabs = JSON.stringify(latest.current);
    setPreview(null); setPreviewError("");
    try {
      const captured = await getContext();
      if (projectionEpoch.current === epoch && captureTabs === JSON.stringify(latest.current)) setPreview(captured);
    } catch (error) { if (projectionEpoch.current === epoch) setPreviewError(String(error)); }
  }
  const projectionActions = useContextProjectionActions({ open: projectionOpen, setOpen: setProjectionOpen, tabs, selection, choose: chooseContext, preview: preview?.projection ?? null, error: previewError, refresh: refreshProjection });
  const applyAgentChange = (change: AgentChange) => {
    const target = latest.current.tabs.find(tab => tab.id === (change.buffer ?? change.name));
    if (!target) throw new Error("The target editor was closed. Reopen it and ask the agent again.");
    if ((target.kind === "untitled" ? null : target.target) !== change.path) throw new Error("The target editor was saved or renamed. Nothing was applied to that editor; preserve the revision and review its destination.");
    const session = registry.get(target.docId);
    const view = target.kind === "untitled" ? bufferEditor(target.id)
      : target.kind === "document" ? session?.docEditor
      : target.kind === "generated" ? session?.openOutputs.get(target.target)
      : plainSearchEditor(target.target);
    applyAgentEdit(view, change);
  };
  return <div className="workspace-chat"><ChatDock contextControls={<details className="chat-note" open={projectionOpen} onToggle={event => setProjectionOpen(event.currentTarget.open)}><summary onClick={event => { event.preventDefault(); void projectionActions.invoke("visibility", { open: !projectionOpen }); }}>Next context projection</summary>
    <p>Select documents explicitly. Expanded text excludes folds; pin includes the complete document. Unavailable editor fold state omits text until reopened or pinned. Collapsing changes future selection; previous exposure remains in history.</p>
    {tabs.map(tab => <div key={tab.id}><label><input type="checkbox" checked={!!selection[tab.id]} onChange={event => void projectionActions.invoke("choose", { id: tab.id, mode: event.target.checked ? "expanded" : "off" })} />{tab.title ?? tab.target}</label>
      <label><input type="checkbox" disabled={!selection[tab.id]} checked={selection[tab.id] === "pinned"} onChange={event => void projectionActions.invoke("choose", { id: tab.id, mode: event.target.checked ? "pinned" : "expanded" })} />Pin full text</label></div>)}
    <button onClick={() => void projectionActions.invoke("refresh")}>Refresh preview</button>
    {previewError && <p role="alert">{previewError}</p>}
    <p>Preview is a capture, not a delivery receipt. Send captures selection again. Request history, instructions and tools add context; actual requests are recorded in the session.</p>
    {preview?.projection && <pre aria-label="Context projection preview">{JSON.stringify(preview.projection, null, 2)}</pre>}
  </details>} key={folder ?? "window"} docId="workspace" recoveryWorkspace={folder ?? "window"} applyAgentChange={applyAgentChange}
    reviewRecordedPatch={scope ? reviewRecordedPatch : undefined} realtime={realtime} getContext={getContext}
    contextLabel={[folder, ...tabs.map(tab => `${tab.title ?? tab.target}${tab.kind === "untitled" ? " (unsaved)" : ""}`)].filter(Boolean).join(" · ") || "No open editors or folder"}
    onOpenSession={onOpenSession}
    onAgentFinished={() => {
      registry.all().forEach(session => session.refresh());
      window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
    }} />
    {patchReview && matchesScope(scope, patchReview.review) && <RecordedPatchReview proposal={patchReview.review}
      anchor={patchReview.anchor} close={closePatchReview} model={draftHistory && model}
      undo={draftHistory?.preview("undo") ? () => changeDraftHistory("undo") : undefined}
      redo={draftHistory?.preview("redo") ? () => changeDraftHistory("redo") : undefined} />}
  </div>;
}
