import { flushSync } from "react-dom";
// Saving the one buffer that has no project path yet.
//
// An Untitled tab is a workspace draft until this hook creates its document.
// Keeping the transition here prevents WorkspaceView from becoming the owner
// of another editor-specific state machine.

import { useCallback, useEffect, useRef } from "react";
import type { Dispatch, MutableRefObject, SetStateAction } from "react";

import { api } from "../api/client";
import { untitledDraftKey, untitledSaveName, wrapUntitled } from "../lib/newDoc";
import { bufferEditor } from "../lib/editorBufferTargets";
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
}): (saveAs: boolean, tabId?: string, destination?: string) => Promise<boolean> {
  const pending = useRef(new Set<string>());
  const mounted = useRef(true);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  return useCallback(
    async (_saveAs: boolean, tabId?: string, destination?: string) => {
      const pane = panes(layoutRef.current.root).find(
        (candidate) => candidate.id === layoutRef.current.focus,
      );
      const tab = tabId ? panes(layoutRef.current.root).flatMap(pane => pane.tabs).find(tab => tab.id === tabId) : pane?.tabs[pane.active];
      if (!tab || tab.kind !== "untitled") return false;
      const editor = bufferEditor(tab.id);
      const untitled = {
        tabId: tab.id,
        source: sourceFor?.(tab.id) ?? editor?.state.doc.toString() ?? "",
      };
      if (untitled.source.length === 0) return true;
      if (pending.current.has(tab.id)) { onError("This unnamed buffer already has a pending save."); return false; }
      pending.current.add(tab.id);
      const unchanged = () => mounted.current && panes(layoutRef.current.root).some(pane => pane.tabs.some(entry => entry.id === tab.id && entry.kind === "untitled")) &&
        (sourceFor?.(tab.id) ?? bufferEditor(tab.id)?.state.doc.toString() ?? untitled.source) === untitled.source;
      let submittedPath: string | undefined;
      let acknowledged = false;
      try {
        const project = (await api.projects())[0];
        if (!project) throw new Error("no folder is open");
        const docs = await api.projectDocs(project.id).catch(() => []);
        const suggested = untitledSaveName(untitled.source, docs.map((doc) => doc.path));
        // An untitled document has no path to reuse. The desktop shell owns
        // the platform Save dialog, so it can choose any location while
        // beginning in the folder already open here.
        const path = destination ?? (await api.saveFileDialog(suggested)).path;
        if (!path) return false;
        if (!unchanged()) throw Error("The originating unnamed buffer changed or closed before saving. No file was created; inspect it and retry.");
        submittedPath = path;
        const created = await api.createDoc(project.id, path, wrapUntitled(untitled.source));
        acknowledged = true;
        if (!unchanged()) throw Error(`Saved the captured source to ${created.path}, but the originating buffer changed or closed during writing. Its newer draft was not overwritten or adopted. Inspect the created file before retrying.`);
        // Tell the mounted keeper first: its unmount cleanup must not write
        // this now-real file back into the draft store after we discard it.
        window.dispatchEvent(new CustomEvent("hickory-untitled-saved", { detail: untitled.tabId }));
        onSaved?.(untitled.tabId);
        void api.discardDraft(untitledDraftKey(untitled.tabId)).catch(error => onError(`Named file saved, but recovery draft cleanup failed: ${String(error)}`));
        // A window close may immediately persist its layout after Save all.
        // Publish the named tab before reporting that this save succeeded.
        flushSync(() => setLayout(current =>
          adoptUntitledTab(current, untitled.tabId, created.id, created.path),
        ));
        redirect(`/docs/${created.id}`);
        return true;
      } catch (e) {
        const message = e instanceof Error ? e.message : String(e);
        onError(submittedPath && !acknowledged ? `Creation of ${submittedPath} was not acknowledged: ${message}. Inspect the destination before retrying.` : message);
        return false;
      } finally { pending.current.delete(tab.id); }
    },
    [layoutRef, onError, onSaved, setLayout, sourceFor],
  );
}
