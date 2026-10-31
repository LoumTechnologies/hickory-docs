import { providerObservationContext } from "../debug/providerObservations";
import { useContextProjectionActions } from "./uiContextProjectionActions";
import { captureUiWindow } from "../lib/uiActionBridge";
import "./contextProjection.css";
import { projectDocument, projectionEstimate, retainedReadingState, type ReadingProjection } from "../lib/contextProjection";
import { observationContext } from "../debug/observations";
import { bufferTarget } from "../lib/editorBufferTargets";
import { useMemo, useRef, useState, type MutableRefObject } from "react";
import { ChatDock } from "../components/ChatDock";
import { api } from "../api/client";
import { getWorkspaceRealtime, LocalRealtime } from "../api/realtime";
import type { AgentEditorContext } from "../api/agentTypes";
import { panes, type Layout, type Tab } from "../shell/layout";
import type { SessionRegistry } from "./documentSession";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { bufferEditor } from "../lib/editorBufferTargets";
import { applyAgentEdit } from "../lib/agentEdit";
import type { AgentChange } from "../api/acp";
import { openPlainSearchFiles, plainSearchEditor } from "../lib/workspaceSearch";

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
  const observations = [...new Map(buffers.flatMap(buffer => buffer.observations).map(record => [JSON.stringify(record), record])).values()];
  const projection: ReadingProjection = { version: 1, documents, observations, estimate: projectionEstimate({ documents, observations }) };
  return { live: true, buffers, ...(originatingWindow ? { ui_window: originatingWindow } : {}), ...(selection ? { projection } : {}) };
}

/** A conversation belongs to the window, independent of which editor is active. */
export function WorkspaceChat({ layout, registry, untitledSources, plainSources, folder, focusedEditor, onOpenSession }: {
  layout: Layout;
  registry: SessionRegistry;
  untitledSources: MutableRefObject<Record<string, string>>;
  plainSources: MutableRefObject<Map<string, string>>;
  folder: string | null;
  focusedEditor: string | null;
  onOpenSession: (path: string) => void;
}) {
  const [projectionOpen, setProjectionOpen] = useState(false);
  const projectionEpoch = useRef(0);
  const [selection, setSelection] = useState<Record<string, "expanded" | "pinned">>({});
  const [preview, setPreview] = useState<AgentEditorContext | null>(null);
  const [previewError, setPreviewError] = useState("");
  const tabs = editorTabs(layout);
  const focused = tabs.some(tab => tab.id === focusedEditor) ? focusedEditor : tabs[0]?.id ?? null;
  const realtime = useMemo(() => getWorkspaceRealtime() ?? new LocalRealtime(), []);
  const latest = useRef({ tabs, focused });
  latest.current = { tabs, focused };
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
  </details>} key={folder ?? "window"} docId="workspace" recoveryWorkspace={folder ?? "window"} applyAgentChange={applyAgentChange} realtime={realtime} getContext={getContext}
    contextLabel={[folder, ...tabs.map(tab => `${tab.title ?? tab.target}${tab.kind === "untitled" ? " (unsaved)" : ""}`)].filter(Boolean).join(" · ") || "No open editors or folder"}
    onOpenSession={onOpenSession}
    onAgentFinished={() => {
      registry.all().forEach(session => session.refresh());
      window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
    }} /></div>;
}
