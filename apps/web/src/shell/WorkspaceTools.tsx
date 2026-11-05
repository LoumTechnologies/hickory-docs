import { lazy, useEffect, useRef } from "react";
import type { EditorView } from "@codemirror/view";
import { LineageSettings } from "../components/LineageSettings";
import { ProvenanceToggles } from "./ProvenanceToggles";
import type { ComponentProps } from "react";
import { useUiCommands } from "../lib/uiActionBridge";
import "./quietWorkspace.css";

const CoverageTools = lazy(() => import("../coverage/CoverageTools").then(module => ({ default: module.CoverageTools })));
const NativeRuntimeHistory = lazy(() => import("../debug/RuntimeRuns").then(module => ({ default: module.NativeRuntimeHistory })));
const ReviewPanel = lazy(() => import("../components/ReviewPanel").then(module => ({ default: module.ReviewPanel })));
const MeetingImport = lazy(() => import("../components/MeetingImport").then(module => ({ default: module.MeetingImport })));
const SnapshotExport = lazy(() => import("../components/SnapshotExport").then(module => ({ default: module.SnapshotExport })));
const WorkflowTour = lazy(() => import("../components/WorkflowTour").then(module => ({ default: module.WorkflowTour })));

export type WorkspaceTool = "connections" | "execution" | "review" | "import" | "export" | "help";
export const TOOL_LABELS: Record<WorkspaceTool, string> = {
  connections: "Connection display", execution: "Tests and execution", review: "Review this file",
  import: "Import meeting", export: "Export document snapshot", help: "Workflow help",
};

export function WorkspaceTools({ tool, path, document, source, view, provenance, projectId, onGraph, onOpen, onOpenNote, close }: {
  tool: WorkspaceTool; path: string | null; document?: string; source?: string; view?: EditorView | null;
  provenance: ComponentProps<typeof ProvenanceToggles>; projectId: string | null;
  onGraph: () => void; onOpen: (path: string, line?: number) => void; onOpenNote: (id: string) => void; close: () => void;
}) {
  const panel = useRef<HTMLElement>(null);
  useEffect(() => {
    const previous = window.document.activeElement as HTMLElement | null;
    panel.current?.querySelector<HTMLButtonElement>("button")?.focus();
    return () => { if (previous?.isConnected) previous.focus(); };
  }, []);
  const invoke = useUiCommands([{ key: "workspace-tool:graph", command: { label: "Show document connections graph", parent: "workspace-tools", subject: "workspace-tool", parameters: {}, prerequisites: [], read: () => ({ revision: projectId ?? "", enabled: !!projectId && tool === "connections" }), execute: async () => { onGraph(); return { status: "completed", value: { graph_requested: true } }; } } }, { key: "workspace-tool:close", command: { label: "Close workspace tool", subject: "workspace-tool", parent: "workspace-tools", parameters: {}, prerequisites: [], read: () => ({ revision: tool, enabled: true }), execute: async () => { close(); return { status: "completed", value: null }; } } }]);
  return <aside ref={panel} className="workspace-tool-panel" role="region" aria-label={TOOL_LABELS[tool]} onKeyDown={event => {
    if (event.key === "Escape" && !(event.target as HTMLElement).closest("input,textarea,[contenteditable=true]")) { event.stopPropagation(); close(); }
  }}>
    <header><strong>{TOOL_LABELS[tool]}</strong>{path && <small title={path}>{path}</small>}<button type="button" aria-label="Close workspace tool" onClick={close}>Close</button></header>
    {tool === "connections" && <><ProvenanceToggles {...provenance} /><LineageSettings />{projectId && <button type="button" onClick={() => { void invoke("workspace-tool:graph"); }}>Show document connections graph</button>}</>}
    {tool === "execution" && <><CoverageTools path={path} /><NativeRuntimeHistory path={path ?? undefined} source={source} view={view} /></>}
    {tool === "review" && <ReviewPanel document={path ?? undefined} source={source} view={view} initialOpen />}
    {tool === "import" && <MeetingImport onOpen={onOpen} onOpenNote={onOpenNote} />}
    {tool === "export" && <SnapshotExport docId={document} />}
    {tool === "help" && <WorkflowTour surface="app" onOpen={onOpen} focused={{ path: path ?? undefined, source, view }} />}
  </aside>;
}
