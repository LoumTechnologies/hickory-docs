import { flushResponseRecovery } from "../lib/responseRecovery";
import { viewBuffers, useViewBuffersVersion } from "../lib/viewBuffers";
// Close decisions use acknowledged file persistence, not a separate Save
// checkpoint. Recovery drafts never count as a successful file save.

import { useCallback, useEffect, useMemo } from "react";
import type { Dispatch, MutableRefObject, SetStateAction } from "react";

import { discardRecoveryDraft } from "../lib/drafts";
import { untitledDraftKey } from "../lib/newDoc";
import { api } from "../api/client";
import type { usePrompt } from "../components/PromptPanel";
import { activate, closeTab, panes, type Layout } from "../shell/layout";
import type { SessionRegistry } from "./documentSession";

type Prompt = ReturnType<typeof usePrompt>;
type PlainActions = {
  save: () => Promise<boolean>;
  discard: () => void;
  retain: () => Promise<void>;
};

async function discardUntitled(id: string) {
  const pending: Promise<unknown>[] = [];
  window.dispatchEvent(new CustomEvent("hickory-untitled-discarded", {
    detail: { id, waitUntil: (operation: Promise<unknown>) => pending.push(operation) },
  }));
  // Recovery waits for any in-flight write before deleting the draft.
  if (pending.length) await Promise.all(pending);
  else await api.discardDraft(untitledDraftKey(id));
}

export function useUnsavedLifecycle({
  layout,
  flushUi,
  layoutRef,
  setLayout,
  registry,
  retainSavedDrafts,
  untitledSources,
  untitledBaselines = {},
  untitledSourcesRef,
  forgetUntitled,
  saveUntitled,
  prompt,
  plainDirtyTabs,
  plainUnsavedActions,
}: {
  layout: Layout;
  flushUi?: () => Promise<void>;
  layoutRef: MutableRefObject<Layout>;
  setLayout: Dispatch<SetStateAction<Layout>>;
  registry: SessionRegistry;
  retainSavedDrafts: boolean;
  untitledSources: Record<string, string>;
  untitledBaselines?: Record<string, string>;
  untitledSourcesRef: MutableRefObject<Record<string, string>>;
  forgetUntitled: (tabId: string) => void;
  saveUntitled: (saveAs: boolean) => Promise<boolean>;
  prompt: Prompt;
  plainDirtyTabs: ReadonlySet<string>;
  plainUnsavedActions: MutableRefObject<Map<string, PlainActions>>;
}) {
  const buffersVersion = useViewBuffersVersion();
  useEffect(() => {
    const open = new Set(panes(layout.root).flatMap(pane => pane.tabs.map(tab => tab.id)));
    viewBuffers.ids().filter(id => !open.has(id)).forEach(id => viewBuffers.forget(id));
  }, [layout]);
  const dirtyTabIds = useMemo(() => {
    const dirty = new Set<string>();
    for (const pane of panes(layout.root)) {
      for (const tab of pane.tabs) {
        if (tab.kind === "untitled" && (untitledSources[tab.id] ?? untitledBaselines[tab.id] ?? "") !== (untitledBaselines[tab.id] ?? "")) {
          dirty.add(tab.id);
        }
        if (tab.kind === "document" && tab.docId && registry.get(tab.docId)?.dirty) {
          dirty.add(tab.id);
        }
        if (tab.kind === "file" && plainDirtyTabs.has(tab.id)) dirty.add(tab.id);
        if (viewBuffers.get(tab.id)?.dirty) dirty.add(tab.id);
      }
    }
    return dirty;
    // registry.version is the external session store's change signal.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [layout, untitledSources, untitledBaselines, registry, registry.version, plainDirtyTabs, buffersVersion]);

  const dirtyPaths = useMemo(
    () =>
      new Set([
        ...panes(layout.root)
          .flatMap((pane) => pane.tabs)
          .filter((tab) => plainDirtyTabs.has(tab.id))
          .map((tab) => tab.target),
        ...registry
          .all()
          .filter((session) => session.dirty && session.doc)
          .map((session) => session.doc!.path),
      ]),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [layout, plainDirtyTabs, registry, registry.version],
  );

  const requestCloseTab = useCallback(
    (paneId: string, tabId: string) => {
      const pane = panes(layoutRef.current.root).find((candidate) => candidate.id === paneId);
      const tab = pane?.tabs.find((candidate) => candidate.id === tabId);
      if (!pane || !tab) return;
      if (!dirtyTabIds.has(tabId)) {
        if (tab.kind === "untitled") forgetUntitled(tab.id);
        setLayout((current) => closeTab(current, paneId, tabId));
        return;
      }
      void (async () => {
        const viewBuffer = viewBuffers.get(tab.id);
        const name = tab.title ?? tab.target.split("/").pop() ?? "this file";
        const answer = await prompt.askChoice(
          `Save changes to ${name}? Closing without saving discards these changes.`,
          [
            { label: "Save", value: "save" as const },
            { label: "Discard", value: "close" as const },
            { label: "Cancel", value: "cancel" as const },
          ],
        );
        if (!answer || answer === "cancel") return;
        if (answer === "save") {
          let saved = false;
          if (viewBuffer) saved = await viewBuffer.save();
          else if (tab.kind === "untitled") {
            const activated = activate(layoutRef.current, paneId, pane.tabs.indexOf(tab));
            layoutRef.current = activated;
            setLayout(activated);
            saved = await saveUntitled(false);
          } else if (tab.kind === "file") {
            saved = (await plainUnsavedActions.current.get(tab.id)?.save()) === true;
          } else if (tab.docId) {
            saved = (await registry.get(tab.docId)?.menuSave()) === true;
          }
          if (!saved) return;
        } else if (viewBuffer) {
          await viewBuffer.discard();
        } else if (tab.kind === "untitled") {
          await discardUntitled(tab.id);
          forgetUntitled(tab.id);
        } else if (tab.kind === "file") {
          plainUnsavedActions.current.get(tab.id)?.discard();
          await discardRecoveryDraft(tab.target);
        } else if (tab.docId) {
          await registry.get(tab.docId)?.discardUnsaved();
          await discardRecoveryDraft(tab.target);
        }
        setLayout((current) => closeTab(current, paneId, tabId));
      })();
    }, [dirtyTabIds, registry, retainSavedDrafts, saveUntitled, prompt, layoutRef, setLayout, plainUnsavedActions, untitledSourcesRef, untitledBaselines, forgetUntitled],
  );

  useEffect(() => {
    const beforeUnload = (event: BeforeUnloadEvent) => {
      if (dirtyTabIds.size === 0) return;
      event.preventDefault();
      event.returnValue = "";
    };
    window.addEventListener("beforeunload", beforeUnload);
    return () => window.removeEventListener("beforeunload", beforeUnload);
  }, [dirtyTabIds]);

  useEffect(() => {
    const onWindowClose = () => {
      void (async () => {
        if (dirtyTabIds.size === 0) {
          await flushUi?.();
          await flushResponseRecovery();
          await api.closeWindow().catch(() => {});
          return;
        }
        const dirtySaved = registry.all().filter((session) => session.dirty);
        const dirtyPlain = panes(layoutRef.current.root)
          .flatMap((pane) => pane.tabs)
          .filter((tab) => tab.kind === "file" && plainDirtyTabs.has(tab.id))
          .map((tab) => tab.id);
        const dirtyUntitled = panes(layoutRef.current.root).flatMap((pane) => pane.tabs)
          .filter((tab) => tab.kind === "untitled" && dirtyTabIds.has(tab.id));
        const dirtyViews = panes(layoutRef.current.root).flatMap(pane => pane.tabs)
          .map(tab => viewBuffers.get(tab.id)).filter(buffer => buffer?.dirty);
        const answer = await prompt.askChoice(
          "Save changes before closing Hickory Docs? Closing without saving discards all unsaved changes.",
          [
            { label: "Save all", value: "save" as const },
            { label: "Close without saving", value: "close" as const },
            { label: "Cancel", value: "cancel" as const },
          ],
        );
        if (!answer || answer === "cancel") return;
        if (answer === "save") {
          for (const buffer of dirtyViews) if (buffer && !(await buffer.save())) return;
          for (const session of dirtySaved) if (!(await session.menuSave())) return;
          for (const tabId of dirtyPlain) {
            if (!(await plainUnsavedActions.current.get(tabId)?.save())) return;
          }
          for (const tab of dirtyUntitled) {
            const holder = panes(layoutRef.current.root).find(pane => pane.tabs.some(entry => entry.id === tab.id));
            if (!holder) continue;
            const activated = activate(layoutRef.current, holder.id, holder.tabs.findIndex(entry => entry.id === tab.id));
            layoutRef.current = activated;
            setLayout(activated);
            if (!(await saveUntitled(false))) return;
          }
        } else {
          for (const buffer of dirtyViews) await buffer?.discard();
          for (const tab of dirtyUntitled) {
            await discardUntitled(tab.id);
            forgetUntitled(tab.id);
          }
          for (const session of dirtySaved) {
            await session.discardUnsaved();
            if (session.doc) await discardRecoveryDraft(session.doc.path);
          }
          for (const tabId of dirtyPlain) {
            plainUnsavedActions.current.get(tabId)?.discard();
            const tab = panes(layoutRef.current.root).flatMap(pane => pane.tabs).find(tab => tab.id === tabId);
            if (tab) await discardRecoveryDraft(tab.target);
          }
        }
        await flushUi?.();
        await flushResponseRecovery();
        await api.closeWindow().catch(() => {});
      })();
    };
    window.addEventListener("hickory-workspace-close-request", onWindowClose);
    return () => window.removeEventListener("hickory-workspace-close-request", onWindowClose);
  }, [dirtyTabIds, registry, retainSavedDrafts, saveUntitled, prompt, layoutRef, setLayout, untitledSourcesRef, untitledBaselines, plainDirtyTabs, plainUnsavedActions, forgetUntitled, flushUi]);

  return { dirtyTabIds, dirtyPaths, requestCloseTab };
}
