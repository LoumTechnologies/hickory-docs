import { useMemo, useRef, type MutableRefObject } from "react";
import { ChatDock } from "../components/ChatDock";
import { api } from "../api/client";
import { getWorkspaceRealtime, LocalRealtime } from "../api/realtime";
import type { AgentEditorContext } from "../api/agentTypes";
import { panes, type Layout, type Tab } from "../shell/layout";
import type { SessionRegistry } from "./documentSession";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { untitledEditor } from "../editor/activeEditor";
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
): Promise<AgentEditorContext> {
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
    return { id: tab.id, kind: tab.kind, ...(tab.kind === "generated" ? { document: session?.doc?.path ?? tabs.find(t => t.kind === "document" && t.docId === tab.docId)?.target } : {}), name: tab.title ?? tab.target, path: tab.kind === "untitled" ? null : tab.target,
      content, focused: tab.id === focused };
  }));
  return { buffers };
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
  const tabs = editorTabs(layout);
  const focused = tabs.some(tab => tab.id === focusedEditor) ? focusedEditor : tabs[0]?.id ?? null;
  const realtime = useMemo(() => getWorkspaceRealtime() ?? new LocalRealtime(), []);
  const latest = useRef({ tabs, focused });
  latest.current = { tabs, focused };
  const getContext = async () => editorContext(latest.current.tabs, latest.current.focused,
    registry, untitledSources.current, plainSources.current);
  const applyAgentChange = (change: AgentChange) => {
    const target = latest.current.tabs.find(tab => tab.id === (change.buffer ?? change.name));
    if (!target) throw new Error("The target editor was closed. Reopen it and ask the agent again.");
    const session = registry.get(target.docId);
    const view = target.kind === "untitled" ? untitledEditor()
      : target.kind === "document" ? session?.docEditor
      : target.kind === "generated" ? session?.openOutputs.get(target.target)
      : plainSearchEditor(target.target);
    applyAgentEdit(view, change);
  };
  return <ChatDock docId="workspace" applyAgentChange={applyAgentChange} realtime={realtime} getContext={getContext}
    contextLabel={[folder, ...tabs.map(tab => `${tab.title ?? tab.target}${tab.kind === "untitled" ? " (unsaved)" : ""}`)].filter(Boolean).join(" · ") || "No open editors or folder"}
    onOpenSession={onOpenSession}
    onAgentFinished={() => {
      registry.all().forEach(session => session.refresh());
      window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
    }} />;
}
