<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/useUnsavedLifecycle.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Unsent responses survive restart](../../../../../../guarantees/agent/unsent-responses-survive-restart/contract.md)
- [A Folder Remembers Its Window](../../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md)
- [New Document Creates Another Unnamed Buffer](../../../../../../guarantees/authoring/new-document-is-an-act/contract.md)
- [Saved means the displayed bytes reached the file](../../../../../../guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file/contract.md)
- [Unsaved Work Survives Closing The App](../../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md)

Captured SHA-256: `0b26ebb5dcc34cf213f36704912adc8ba28a8758558135fc2059ca68805c33e1`.

<book:entry path="apps/web/src/views/useUnsavedLifecycle.ts">
import { flushResponseRecovery } from "../lib/responseRecovery";
import { viewBuffers, useViewBuffersVersion } from "../lib/viewBuffers";
// Close decisions use acknowledged file persistence, not a separate Save
// checkpoint. Recovery drafts never count as a successful file save.

import { useCallback, useEffect, useId, useMemo, useRef } from "react";
import type { Dispatch, MutableRefObject, SetStateAction } from "react";

import { discardRecoveryDraft } from "../lib/drafts";
import { untitledDraftKey } from "../lib/newDoc";
import { WORKSPACE_CLOSE, type WorkspaceCloseRequest } from "../lib/windowClose";
import { bufferEditor } from "../lib/editorBufferTargets";
import { plainSearchEditor } from "../lib/workspaceSearch";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiOutcome } from "../lib/uiActions";
import { api } from "../api/client";
import type { usePrompt } from "../components/PromptPanel";
import { activate, closeTab, panes, type Layout } from "../shell/layout";
import type { SessionRegistry } from "./documentSession";

type Prompt = ReturnType<typeof usePrompt>;
type PlainActions = {
  save: () => Promise<boolean>;
  discard: () => void;
  retain: () => Promise<void>;
  revision?: () => unknown;
  isDirty?: () => boolean;
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
  untitledSources,
  untitledBaselines = {},
  untitledSourcesRef,
  forgetUntitled,
  saveUntitled,
  prompt,
  plainDirtyTabs,
  plainUnsavedActions,
  onError,
  windowGuard,
}: {
  onError?: (message: string) => void;
  windowGuard?: () => unknown;
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

  const closing = useRef(new Set<string>());
  const tabLifetimes = useRef(new WeakMap<object, string>());
  const tabLifetime = (tab: object) => { let id = tabLifetimes.current.get(tab); if (!id) { id = crypto.randomUUID(); tabLifetimes.current.set(tab, id); } return id; };
  const tabSource = (paneId: string, tabId: string) => {
      const current = panes(layoutRef.current.root).find(pane => pane.id === paneId)?.tabs.find(tab => tab.id === tabId); if (!current) return null;
      const session = current.docId ? registry.get(current.docId) : undefined;
      return JSON.stringify([tabLifetime(current), current.id, current.kind, current.target, current.docId,
        viewBuffers.get(tabId)?.revision?.(), plainUnsavedActions.current.get(tabId)?.revision?.(),
        plainSearchEditor(current.target)?.state.doc.toString(), bufferEditor(tabId)?.state.doc.toString(),
        session?.docEditor?.state.doc.toString() ?? session?.liveSource,
        untitledSourcesRef.current[tabId] ?? untitledBaselines[tabId]]);
    };
  const tabDirty = (paneId: string, tabId: string) => {
      const current = panes(layoutRef.current.root).find(pane => pane.id === paneId)?.tabs.find(tab => tab.id === tabId); if (!current) return false;
      const buffer = viewBuffers.get(tabId); if (buffer) return buffer.isDirty?.() ?? buffer.dirty;
      if (current.kind === "file") return plainUnsavedActions.current.get(tabId)?.isDirty?.() ?? dirtyTabIds.has(tabId);
      if (current.kind === "untitled") return (untitledSourcesRef.current[tabId] ?? untitledBaselines[tabId] ?? "") !== (untitledBaselines[tabId] ?? "");
      const session = current.docId ? registry.get(current.docId) : undefined;
      return session ? (session.docEditor?.state.doc.toString() ?? session.liveSource) !== (session.savedSource ?? session.doc?.source ?? "") : dirtyTabIds.has(tabId);
    };
  const requestCloseTab = useCallback(async (paneId: string, tabId: string, signal?: AbortSignal, preparation?: { choice: "save" | "close"; baseline: string | null; after?: string | null }): Promise<UiOutcome> => {
    const find = () => panes(layoutRef.current.root).find(pane => pane.id === paneId)?.tabs.find(tab => tab.id === tabId);
    const tab = find();
    if (signal?.aborted) return { status: "completed", value: { tab: tabId, cancelled: true, tab_close_requested: false } };
    if (!tab) return { status: "unavailable", reason: "The originating tab is no longer open" };
    if (closing.current.has(tabId)) return { status: "unavailable", reason: "This tab already has a pending close decision" };
    const source = () => tabSource(paneId, tabId);
    const liveDirty = () => tabDirty(paneId, tabId);
    const baseline = source();
    closing.current.add(tabId);
    let effectsStarted = false;
    try {
      let decision = "clean";
      if (preparation && baseline !== preparation.baseline) throw Error("The captured tab changed before window-close preparation; no operation was submitted for it");
      if (liveDirty()) {
        const viewBuffer = viewBuffers.get(tabId);
        const name = tab.title ?? tab.target.split("/").pop() ?? "this file";
        const answer = preparation?.choice ?? await prompt.askChoice(`Save changes to ${name}? Closing without saving discards these changes.`, [
          { label: "Save", value: "save" as const }, { label: "Discard", value: "close" as const }, { label: "Cancel", value: "cancel" as const },
        ], { agent: true, currentRevision: source, signal });
        if (!answer || answer === "cancel" || signal?.aborted) return { status: "completed", value: { tab: tabId, cancelled: true, tab_close_requested: false } };
        if (source() !== baseline) throw Error("The originating tab changed while its close decision was open. No save or discard was submitted; inspect it and retry.");
        decision = answer; effectsStarted = true;
        if (answer === "save") {
          let saved = false;
          if (viewBuffer) saved = await viewBuffer.save();
          else if (tab.kind === "untitled") {
            const pane = panes(layoutRef.current.root).find(pane => pane.id === paneId)!;
            const activated = activate(layoutRef.current, paneId, pane.tabs.findIndex(entry => entry.id === tabId));
            layoutRef.current = activated; setLayout(activated);
            saved = await saveUntitled(false);
          } else if (tab.kind === "file") saved = (await plainUnsavedActions.current.get(tabId)?.save()) === true;
          else if (tab.docId) saved = (await registry.get(tab.docId)?.menuSave()) === true;
          if (!saved) return { status: "unavailable", reason: "Save was cancelled, failed or left newer edits; the originating tab remains open. Inspect the actual save/error state; a false acknowledgement does not imply rollback." };
        } else if (viewBuffer) await viewBuffer.discard();
        else if (tab.kind === "untitled") { await discardUntitled(tabId); if (source() !== baseline) throw Error("The unnamed tab changed during recovery cleanup; its newer draft was retained"); if (!preparation) forgetUntitled(tabId); }
        else if (tab.kind === "file") { plainUnsavedActions.current.get(tabId)?.discard(); await discardRecoveryDraft(tab.target); }
        else if (tab.docId) { await registry.get(tab.docId)?.discardUnsaved(); await discardRecoveryDraft(tab.target); }
        if (liveDirty() && source() !== baseline) throw Error("The originating tab has newer edits after its save/discard callback. The callback may have changed saved/recovery state; the tab remains open.");
      } else if (tab.kind === "untitled" && !preparation) forgetUntitled(tabId);
      if (signal?.aborted) return { status: "unavailable", reason: "Cancellation was requested after a save/discard may have started. The tab remains open; inspect the actual buffer state, not rollback." };
      const finalTab = find();
      if (!finalTab || (finalTab !== tab && !(decision === "save" && tab.kind === "untitled" && finalTab.kind === "document"))) return { status: "unavailable", reason: "The originating tab retired or was replaced during closing; no replacement tab was closed" };
      if (preparation) { preparation.after = source(); return { status: "completed", value: { tab: tabId, tab_prepared: true, decision } }; }
      setLayout(current => panes(current.root).find(pane => pane.id === paneId)?.tabs.find(tab => tab.id === tabId) === finalTab ? closeTab(current, paneId, tabId) : current);
      return { status: "completed", value: { tab: tabId, tab_close_requested: true, decision, scope: "Layout update requested after the actual close decision/save/discard callback; no window closure or verification is implied" } };
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      const message = effectsStarted ? `Tab close did not finish: ${detail}. Earlier save/discard or recovery effects may already have occurred; inspect the retained tab before retrying.` : detail;
      onError?.(message);
      return { status: "unavailable", reason: message };
    } finally { closing.current.delete(tabId); }
  }, [dirtyTabIds, registry, saveUntitled, prompt, layoutRef, setLayout, plainUnsavedActions, untitledSourcesRef, untitledBaselines, forgetUntitled, onError]);

  useEffect(() => {
    const beforeUnload = (event: BeforeUnloadEvent) => {
      if (dirtyTabIds.size === 0) return;
      event.preventDefault();
      event.returnValue = "";
    };
    window.addEventListener("beforeunload", beforeUnload);
    return () => window.removeEventListener("beforeunload", beforeUnload);
  }, [dirtyTabIds]);

  const pendingWindow = useRef<Promise<UiOutcome> | null>(null);
  const lastWindow = useRef<{ request: string; outcome: UiOutcome } | null>(null);
  const currentWindowRequest = useRef<string | null>(null);
  const windowGuardRef = useRef(windowGuard); windowGuardRef.current = windowGuard;
  const alive = useRef(true), activeWindowAbort = useRef<AbortController | null>(null);
  useEffect(() => { alive.current = true; return () => { alive.current = false; activeWindowAbort.current?.abort(); }; }, []);
  const windowTabs = () => panes(layoutRef.current.root).flatMap(pane => pane.tabs.map(tab => ({ pane: pane.id, id: tab.id, source: tabSource(pane.id, tab.id) })));
  const windowSnapshot = () => JSON.stringify([windowTabs(), windowGuardRef.current?.() ?? null]);
  const requestCloseWindow = (externalSignal?: AbortSignal): Promise<UiOutcome> => {
    if (pendingWindow.current) return pendingWindow.current;
    const request = crypto.randomUUID(); currentWindowRequest.current = request;
    const controller = new AbortController(), signal = controller.signal;
    const forward = () => controller.abort(); externalSignal?.addEventListener("abort", forward, { once: true });
    if (externalSignal?.aborted) controller.abort(); activeWindowAbort.current = controller;
    const task = (async (): Promise<UiOutcome> => {
      let preparing = false, submitted = false;
      try {
        if (signal.aborted || !alive.current) return { status: "completed", value: { window_close_request: request, cancelled: true, host_close_submitted: false } };
        const captured = windowTabs(), guard = JSON.parse(JSON.stringify(windowGuardRef.current?.() ?? null)) as unknown, baseline = JSON.stringify([captured, guard]);
        let choice: "save" | "close" = "save";
        if (captured.some(tab => tabDirty(tab.pane, tab.id))) {
          const answer = await prompt.askChoice("Save changes before closing Hickory Docs? Closing without saving discards all unsaved changes.", [
            { label: "Save all", value: "save" as const }, { label: "Close without saving", value: "close" as const }, { label: "Cancel", value: "cancel" as const },
          ], { agent: true, currentRevision: windowSnapshot, signal });
          if (!answer || answer === "cancel" || signal.aborted) return { status: "completed", value: { window_close_request: request, cancelled: true, host_close_submitted: false } };
          choice = answer;
        }
        if (windowSnapshot() !== baseline) throw Error("Workspace buffers or lifecycle state changed before window-close preparation");
        for (const tab of captured) {
          if (!tabDirty(tab.pane, tab.id)) continue;
          preparing = true;
          const preparation = { choice, baseline: tab.source, after: tab.source };
          const outcome = await requestCloseTab(tab.pane, tab.id, signal, preparation);
          if (outcome.status === "unavailable" || outcome.status === "requires_input") throw Error(outcome.reason);
          if (outcome.status !== "completed" || signal.aborted) throw Error("Window-close preparation did not complete; inspect each retained buffer and its actual save/discard result");
          tab.source = preparation.after;
        }
        const expected = JSON.stringify([captured, guard]);
        const current = () => alive.current && !signal.aborted && windowSnapshot() === expected && !windowTabs().some(tab => tabDirty(tab.pane, tab.id));
        // A successful discard can leave its old bytes visible until shutdown;
        // their exact captured revision, rather than a stale dirty flag, owns it.
        const preparedCurrent = () => alive.current && !signal.aborted && windowSnapshot() === expected;
        const safe = choice === "close" ? preparedCurrent : current;
        if (!safe()) throw Error("Workspace changed during close preparation; the host close request was not submitted");
        await flushUi?.(); await flushResponseRecovery();
        if (!safe()) throw Error("Workspace changed while close state was being flushed; the host close request was not submitted");
        submitted = true; const reply = await api.closeWindow();
        if (reply?.ok !== true) throw Error("The host returned an invalid window-close acknowledgement");
        return { status: "completed", value: { window_close_request: request, host_close_request_acknowledged: true, window_destruction_observed: false, owner_cancellation_requested: signal.aborted,
          scope: "Host callback acknowledgement only; closing the originating window can prevent delivery. A lost response is not proof of closure or rollback." } };
      } catch (error) {
        const detail = error instanceof Error ? error.message : String(error);
        const message = `${detail}${submitted ? ". Host close success is unacknowledged; the window may already be closing." : ""}${preparing ? ". Earlier save/discard or recovery effects may have occurred; inspect the retained buffers before retrying." : ""}`;
        if (alive.current) onError?.(message); return { status: "unavailable", reason: message, request };
      }
    })();
    pendingWindow.current = task;
    const settle = (outcome: UiOutcome) => { lastWindow.current = { request, outcome }; if (pendingWindow.current === task) pendingWindow.current = null;
      if (activeWindowAbort.current === controller) activeWindowAbort.current = null; externalSignal?.removeEventListener("abort", forward); };
    void task.then(settle, error => settle({ status: "unavailable", reason: String(error), request }));
    return task;
  };
  const windowCloseRef = useRef(requestCloseWindow); windowCloseRef.current = requestCloseWindow;
  useEffect(() => {
    const close = (event: Event) => {
      const request = (event as CustomEvent<WorkspaceCloseRequest>).detail;
      request?.accept?.();
      const task = windowCloseRef.current(request?.signal);
      if (request?.respond) void task.then(request.respond, error => request.respond({ status: "unavailable", reason: String(error) }));
    };
    window.addEventListener(WORKSPACE_CLOSE, close); return () => window.removeEventListener(WORKSPACE_CLOSE, close);
  }, []);
  const windowParent = `workspace-window:${useId()}`;
  const windowRevision = useUiRevision(() => [windowSnapshot(), !!pendingWindow.current, currentWindowRequest.current, lastWindow.current]);
  useUiCommands([
    { key: `${windowParent}:close`, command: { label: "Close workspace window", parent: windowParent, subject: `${windowParent}:close`, parameters: {},
      prerequisites: ["Source-bound Save/Discard/Cancel decisions precede the existing host close callback. The callback cannot witness physical window destruction, and its response may be lost on shutdown."],
      read: () => ({ revision: windowRevision(), enabled: alive.current && !pendingWindow.current, reason: pendingWindow.current ? "A window close flow is pending; inspect its prompt/status or cancel the owning UI request" : undefined }), execute: async (_args, _actor, signal) => requestCloseWindow(signal) } },
    { key: `${windowParent}:status`, command: { label: "Read workspace close status bytes", parent: windowParent, subject: `${windowParent}:status`, parameters: { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, prerequisites: [],
      read: () => ({ revision: windowRevision(), enabled: true }), execute: async args => { const bytes = new TextEncoder().encode(JSON.stringify({ pending: !!pendingWindow.current, current_request: currentWindowRequest.current, last: lastWindow.current, last_is_current: lastWindow.current?.request === currentWindowRequest.current }));
        const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
        if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside current close status" };
        return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } }; } } },
  ]);

  return { dirtyTabIds, dirtyPaths, requestCloseTab, requestCloseWindow };
}
</book:entry>

</book:doc>
