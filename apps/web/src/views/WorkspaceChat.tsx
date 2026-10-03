import { useMemo, useRef, type MutableRefObject } from "react";
import { ChatDock } from "../components/ChatDock";
import { api } from "../api/client";
import { getWorkspaceRealtime, LocalRealtime } from "../api/realtime";
import type { AgentEditorContext } from "../api/agentTypes";
import { panes, type Layout, type Tab } from "../shell/layout";
import type { SessionRegistry } from "./documentSession";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { openPlainSearchFiles } from "../lib/workspaceSearch";

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
    return { name: tab.title ?? tab.target, path: tab.kind === "untitled" ? null : tab.target,
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
  return <ChatDock docId="workspace" realtime={realtime} getContext={getContext}
    contextLabel={[folder, ...tabs.map(tab => `${tab.title ?? tab.target}${tab.kind === "untitled" ? " (unsaved)" : ""}`)].filter(Boolean).join(" · ") || "No open editors or folder"}
    onOpenSession={onOpenSession}
    onAgentFinished={() => {
      registry.all().forEach(session => session.refresh());
      window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
    }} />;
}
