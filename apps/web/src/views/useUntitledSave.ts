// Saving the one buffer that has no project path yet.
//
// An Untitled tab is a workspace draft until this hook creates its document.
// Keeping the transition here prevents WorkspaceView from becoming the owner
// of another editor-specific state machine.

import { useCallback } from "react";
import type { Dispatch, MutableRefObject, SetStateAction } from "react";

import { api } from "../api/client";
import { untitledDraftKey, untitledSaveName, wrapUntitled } from "../lib/newDoc";
import { focusedEditor, untitledEditor } from "../editor/activeEditor";
import { panes, type Layout } from "../shell/layout";
import { redirect } from "../router";
import { adoptUntitledTab } from "./workspaceState";

export function useUntitledSave({
  layoutRef,
  setLayout,
  onError,
  sourceFor,
  onSaved,
}: {
  layoutRef: MutableRefObject<Layout>;
  setLayout: Dispatch<SetStateAction<Layout>>;
  onError: (message: string) => void;
  sourceFor?: (tabId: string) => string | undefined;
  onSaved?: (tabId: string) => void;
}): (saveAs: boolean) => Promise<boolean> {
  return useCallback(
    async (_saveAs: boolean) => {
      const pane = panes(layoutRef.current.root).find(
        (candidate) => candidate.id === layoutRef.current.focus,
      );
      const tab = pane?.tabs[pane.active];
      if (!tab || tab.kind !== "untitled") return false;
      const editor = untitledEditor() ?? focusedEditor();
      const untitled = {
        tabId: tab.id,
        source: sourceFor?.(tab.id) ?? editor?.state.doc.toString() ?? "",
      };
      if (untitled.source.length === 0) return true;
      try {
        const project = (await api.projects())[0];
        if (!project) throw new Error("no folder is open");
        const docs = await api.projectDocs(project.id).catch(() => []);
        const suggested = untitledSaveName(untitled.source, docs.map((doc) => doc.path));
        // An untitled document has no path to reuse. The desktop shell owns
        // the platform Save dialog, so it can choose any location while
        // beginning in the folder already open here.
        const picked = await api.saveFileDialog(suggested);
        const path = picked.path;
        if (!path) return false;
        const created = await api.createDoc(project.id, path, wrapUntitled(untitled.source));
        const latest = untitledEditor()?.state.doc.toString() ?? untitled.source;
        if (latest !== untitled.source) await api.saveDoc(created.id, wrapUntitled(latest));
        // Tell the mounted keeper first: its unmount cleanup must not write
        // this now-real file back into the draft store after we discard it.
        window.dispatchEvent(new CustomEvent("hickory-untitled-saved", { detail: untitled.tabId }));
        onSaved?.(untitled.tabId);
        void api.discardDraft(untitledDraftKey(untitled.tabId));
        setLayout((current) =>
          adoptUntitledTab(current, untitled.tabId, created.id, created.path),
        );
        redirect(`/docs/${created.id}`);
        return true;
      } catch (e) {
        onError(e instanceof Error ? e.message : String(e));
        return false;
      }
    },
    [layoutRef, onError, onSaved, setLayout, sourceFor],
  );
}
