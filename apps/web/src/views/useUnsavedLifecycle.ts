// One explicit-Save fact, consumed by every surface that talks about it.
//
// Crash recovery and Save are deliberately separate. A CRDT room can make
// bytes durable without deciding that the person meant to save them; this
// hook owns that distinction at the workspace boundary: decoration, close
// prompts, retention and discard all read the same session fields.

import { useCallback, useEffect, useMemo } from "react";
import type { Dispatch, MutableRefObject, SetStateAction } from "react";

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

export function useUnsavedLifecycle({
  layout,
  layoutRef,
  setLayout,
  registry,
  retainSavedDrafts,
  untitledSources,
  untitledSourcesRef,
  forgetUntitled,
  saveUntitled,
  prompt,
  plainDirtyTabs,
  plainUnsavedActions,
}: {
  layout: Layout;
  layoutRef: MutableRefObject<Layout>;
  setLayout: Dispatch<SetStateAction<Layout>>;
  registry: SessionRegistry;
  retainSavedDrafts: boolean;
  untitledSources: Record<string, string>;
  untitledSourcesRef: MutableRefObject<Record<string, string>>;
  forgetUntitled: (tabId: string) => void;
  saveUntitled: (saveAs: boolean) => Promise<boolean>;
  prompt: Prompt;
  plainDirtyTabs: ReadonlySet<string>;
  plainUnsavedActions: MutableRefObject<Map<string, PlainActions>>;
}) {
  const dirtyTabIds = useMemo(() => {
    const dirty = new Set<string>();
    for (const pane of panes(layout.root)) {
      for (const tab of pane.tabs) {
        if (tab.kind === "untitled" && (untitledSources[tab.id]?.length ?? 0) > 0) {
          dirty.add(tab.id);
        }
        if (tab.kind === "document" && tab.docId && registry.get(tab.docId)?.dirty) {
          dirty.add(tab.id);
        }
        if (tab.kind === "file" && plainDirtyTabs.has(tab.id)) dirty.add(tab.id);
      }
    }
    return dirty;
    // registry.version is the external session store's change signal.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [layout, untitledSources, registry, registry.version, plainDirtyTabs]);

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
        const recoverable = retainSavedDrafts;
        const name = tab.title ?? tab.target.split("/").pop() ?? "this file";
        const answer = await prompt.askChoice(
          `Save changes to ${name}? ${
            recoverable
              ? "If you close without saving, the unsaved changes will be retained and restored when you reopen it."
              : "Recovery for previously saved files is off; closing without saving discards these changes."
          }`,
          [
            { label: "Save", value: "save" as const },
            { label: recoverable ? "Close and retain" : "Discard", value: "close" as const },
            { label: "Cancel", value: "cancel" as const },
          ],
        );
        if (!answer || answer === "cancel") return;
        if (answer === "save") {
          let saved = false;
          if (tab.kind === "untitled") {
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
        } else if (tab.kind === "untitled") {
          forgetUntitled(tab.id);
        } else if (tab.kind === "file") {
          const actions = plainUnsavedActions.current.get(tab.id);
          if (retainSavedDrafts) await actions?.retain();
          else actions?.discard();
        } else if (tab.docId) {
          const session = registry.get(tab.docId);
          if (session && retainSavedDrafts && session.savedSource !== null) {
            await api.saveDraft({
              path: tab.target,
              contents: session.liveSource,
              base: session.savedSource,
              saved_at: Date.now(),
            });
          } else if (session) {
            await session.discardUnsaved();
          }
        }
        setLayout((current) => closeTab(current, paneId, tabId));
      })();
    }, [dirtyTabIds, registry, retainSavedDrafts, saveUntitled, prompt, layoutRef, setLayout, plainUnsavedActions, untitledSourcesRef, forgetUntitled],
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
          await api.closeWindow().catch(() => {});
          return;
        }
        const dirtySaved = registry.all().filter((session) => session.dirty);
        const dirtyPlain = panes(layoutRef.current.root)
          .flatMap((pane) => pane.tabs)
          .filter((tab) => tab.kind === "file" && plainDirtyTabs.has(tab.id))
          .map((tab) => tab.id);
        const hasUntitled = Object.values(untitledSourcesRef.current).some(Boolean);
        const retention =
          (dirtySaved.length === 0 && dirtyPlain.length === 0) || retainSavedDrafts
            ? "All unsaved changes will still be here when you reopen their files."
            : "Changes to previously saved files will be discarded because recovery for them is off.";
        const answer = await prompt.askChoice(
          `Save changes before closing Hickory Docs? ${retention}`,
          [
            { label: "Save all", value: "save" as const },
            { label: "Close without saving", value: "close" as const },
            { label: "Cancel", value: "cancel" as const },
          ],
        );
        if (!answer || answer === "cancel") return;
        if (answer === "save") {
          for (const session of dirtySaved) if (!(await session.menuSave())) return;
          for (const tabId of dirtyPlain) {
            if (!(await plainUnsavedActions.current.get(tabId)?.save())) return;
          }
          if (hasUntitled) {
            const holder = panes(layoutRef.current.root).find((pane) =>
              pane.tabs.some((tab) => tab.kind === "untitled"),
            );
            const index = holder?.tabs.findIndex((tab) => tab.kind === "untitled") ?? -1;
            if (holder && index >= 0) {
              const activated = activate(layoutRef.current, holder.id, index);
              layoutRef.current = activated;
              setLayout(activated);
            }
            if (!(await saveUntitled(false))) return;
          }
        } else {
          for (const tabId of Object.keys(untitledSourcesRef.current)) forgetUntitled(tabId);
          for (const session of dirtySaved) {
            if (retainSavedDrafts && session.doc && session.savedSource !== null) {
              await api.saveDraft({
                path: session.doc.path,
                contents: session.liveSource,
                base: session.savedSource,
                saved_at: Date.now(),
              });
            } else {
              await session.discardUnsaved();
            }
          }
          for (const tabId of dirtyPlain) {
            const actions = plainUnsavedActions.current.get(tabId);
            if (retainSavedDrafts) await actions?.retain();
            else actions?.discard();
          }
        }
        await api.closeWindow().catch(() => {});
      })();
    };
    window.addEventListener("hickory-workspace-close-request", onWindowClose);
    return () => window.removeEventListener("hickory-workspace-close-request", onWindowClose);
  }, [dirtyTabIds, registry, retainSavedDrafts, saveUntitled, prompt, layoutRef, setLayout, untitledSourcesRef, plainDirtyTabs, plainUnsavedActions]);

  return { dirtyTabIds, dirtyPaths, requestCloseTab };
}
