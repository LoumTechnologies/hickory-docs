<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/useUnsavedLifecycle.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0a84f897aa4c"></a>
## `apps/web/src/views/useUnsavedLifecycle.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Unsent responses survive restart](../../../../../guarantees/agent/unsent-responses-survive-restart/contract.md), [A Folder Remembers Its Window](../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md), [New Document Creates Another Unnamed Buffer](../../../../../guarantees/authoring/new-document-is-an-act/contract.md), [Saved means the displayed bytes reached the file](../../../../../guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file/contract.md), [Close or recover a document without losing the wrong draft](../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md).

<a id="f-0a84f897aa4c-Prompt"></a>
### `Prompt`

<storycode:copy id="f-0a84f897aa4c-Prompt">
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
</storycode:copy>
<a id="f-0a84f897aa4c-PlainActions"></a>
### `PlainActions`

<storycode:copy id="f-0a84f897aa4c-PlainActions">
type PlainActions = {
  save: () => Promise<boolean>;
  discard: () => void;
  retain: () => Promise<void>;
  revision?: () => unknown;
  isDirty?: () => boolean;
};

</storycode:copy>
<a id="f-0a84f897aa4c-discardUntitled"></a>
### `discardUntitled`

<storycode:copy id="f-0a84f897aa4c-discardUntitled">
async function discardUntitled(id: string) {
  const pending: Promise<unknown>[] = [];
  window.dispatchEvent(new CustomEvent("hickory-untitled-discarded", {
    detail: { id, waitUntil: (operation: Promise<unknown>) => pending.push(operation) },
  }));
  // Recovery waits for any in-flight write before deleting the draft.
  if (pending.length) await Promise.all(pending);
  else await api.discardDraft(untitledDraftKey(id));
}

</storycode:copy>
<a id="f-0a84f897aa4c-useUnsavedLifecycle"></a>
### `useUnsavedLifecycle`

<storycode:copy id="f-0a84f897aa4c-useUnsavedLifecycle">
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
</storycode:copy>

### Assemble `apps/web/src/views/useUnsavedLifecycle.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useUnsavedLifecycle.ts"><storycode:paste select="#f-0a84f897aa4c-Prompt"/><storycode:paste select="#f-0a84f897aa4c-PlainActions"/><storycode:paste select="#f-0a84f897aa4c-discardUntitled"/><storycode:paste select="#f-0a84f897aa4c-useUnsavedLifecycle"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-18ff98d49ede"></a>
## `apps/web/src/views/useUnsavedLifecycle.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Folder Remembers Its Window](../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md), [New Document Creates Another Unnamed Buffer](../../../../../guarantees/authoring/new-document-is-an-act/contract.md), [Saved means the displayed bytes reached the file](../../../../../guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file/contract.md), [Close or recover a document without losing the wrong draft](../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md).

<a id="f-18ff98d49ede-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement">
import { viewBuffers } from "../lib/viewBuffers";
// @vitest-environment jsdom

import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { api } from "../api/client";
import { untitledDraftKey } from "../lib/newDoc";
import { freeform, panes, open, tab } from "../shell/layout";
import { SessionRegistry } from "./documentSession";
import { openUntitledTab } from "./workspaceState";
import { useUnsavedLifecycle } from "./useUnsavedLifecycle";

afterEach(() => { cleanup(); viewBuffers.ids().forEach(id => viewBuffers.forget(id)); vi.restoreAllMocks(); });

</storycode:copy>
<a id="f-18ff98d49ede-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement-2">
describe("closing unsaved work", () => {
  // Guarantees:
  // - docs/guarantees/authoring/new-document-is-an-act.md
  // - docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
  it("discards an unnamed draft when its tab closes", async () => {
    const saveDraft = vi.spyOn(api, "saveDraft").mockResolvedValue({ ok: true });
    const discardDraft = vi.spyOn(api, "discardDraft").mockResolvedValue({ ok: true });
    const layout = openUntitledTab(freeform());
    const pane = panes(layout.root).find((candidate) =>
      candidate.tabs.some((tab) => tab.kind === "untitled"),
    )!;
    const tab = pane.tabs.find((candidate) => candidate.kind === "untitled")!;
    const askChoice = vi.fn().mockResolvedValue("close");
    const setLayout = vi.fn();
    const forgetUntitled = vi.fn();

    const { result } = renderHook(() =>
      useUnsavedLifecycle({
        layout,
        layoutRef: { current: layout },
        setLayout,
        registry: new SessionRegistry(),
        retainSavedDrafts: false,
        untitledSources: { [tab.id]: "remember this" },
        untitledSourcesRef: { current: { [tab.id]: "remember this" } },
        forgetUntitled,
        saveUntitled: vi.fn(),
        prompt: { askChoice } as never,
        plainDirtyTabs: new Set(),
        plainUnsavedActions: { current: new Map() },
      }),
    );

    await act(async () => { await result.current.requestCloseTab(pane.id, tab.id); });

    await waitFor(() => expect(setLayout).toHaveBeenCalledTimes(1));
    expect(askChoice.mock.calls[0][0]).toContain(
      "discards these changes",
    );
    expect(askChoice.mock.calls[0][1].map((option: { label: string }) => option.label)).toEqual([
      "Save",
      "Discard",
      "Cancel",
    ]);
    expect(saveDraft).not.toHaveBeenCalled();
    expect(discardDraft).toHaveBeenCalledWith(untitledDraftKey(tab.id));
    expect(forgetUntitled).toHaveBeenCalledWith(tab.id);
  });
  it.each([true, false])("discards a named file even when recovery is %s", async (retainSavedDrafts) => {
    const layout = open(freeform(), tab("file", "note.txt"));
    const pane = panes(layout.root)[0];
    const file = pane.tabs[0];
    const discard = vi.fn();
    const retain = vi.fn();
    const discardDraft = vi.spyOn(api, "discardDraft").mockResolvedValue({ ok: true });
    const setLayout = vi.fn();
    const { result } = renderHook(() => useUnsavedLifecycle({
      layout, layoutRef: { current: layout }, setLayout, registry: new SessionRegistry(),
      retainSavedDrafts, untitledSources: {}, untitledSourcesRef: { current: {} },
      forgetUntitled: vi.fn(), saveUntitled: vi.fn(), prompt: { askChoice: vi.fn().mockResolvedValue("close") } as never,
      plainDirtyTabs: new Set([file.id]), plainUnsavedActions: { current: new Map([[file.id, { save: vi.fn(), discard, retain }]]) },
    }));
    await act(async () => { await result.current.requestCloseTab(pane.id, file.id); });
    await waitFor(() => expect(setLayout).toHaveBeenCalledTimes(1));
    expect(discard).toHaveBeenCalledOnce();
    expect(discardDraft).toHaveBeenCalledWith("note.txt");
    expect(retain).not.toHaveBeenCalled();
  });

});

</storycode:copy>
<a id="f-18ff98d49ede-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement-3">
// Guarantee: docs/guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file.md
it("keeps an inactive view open when its attempted Save fails", async () => {
  const layout = open(freeform(), tab("tool", "literate:id=v", "Literate view"));
  const pane = panes(layout.root)[0];
  const workingTab = pane.tabs[0];
  const save = vi.fn().mockResolvedValue(false);
  viewBuffers.set(workingTab.id, { dirty: true, save, discard: vi.fn() });
  const askChoice = vi.fn().mockResolvedValue("save");
  const setLayout = vi.fn();
  const { result, unmount } = renderHook(() => useUnsavedLifecycle({ layout, layoutRef: { current: layout }, setLayout,
    registry: new SessionRegistry(), retainSavedDrafts: true, untitledSources: {}, untitledSourcesRef: { current: {} },
    forgetUntitled: vi.fn(), saveUntitled: vi.fn(), prompt: { askChoice } as never,
    plainDirtyTabs: new Set(), plainUnsavedActions: { current: new Map() },
  }));
  await act(async () => { await result.current.requestCloseTab(pane.id, workingTab.id); });
  await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
  expect(askChoice.mock.calls[0][0]).toContain("discards these changes");
  expect(setLayout).not.toHaveBeenCalled();
  unmount(); viewBuffers.forget(workingTab.id);
});

</storycode:copy>
<a id="f-18ff98d49ede-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement-4">
// Guarantee: docs/guarantees/authoring/a-folder-remembers-its-window.md
it("waits for the latest layout write before approving a clean window close", async () => {
  let finish!: () => void;
  const flushUi = vi.fn(() => new Promise<void>(resolve => { finish = resolve; }));
  const close = vi.spyOn(api, "closeWindow").mockResolvedValue({ ok: true });
  const layout = freeform();
  const hook = renderHook(() => useUnsavedLifecycle({
    layout, layoutRef: { current: layout }, setLayout: vi.fn(), flushUi,
    registry: new SessionRegistry(), retainSavedDrafts: false,
    untitledSources: {}, untitledSourcesRef: { current: {} }, forgetUntitled: vi.fn(),
    saveUntitled: vi.fn(), prompt: { askChoice: vi.fn() } as never,
    plainDirtyTabs: new Set(), plainUnsavedActions: { current: new Map() },
  }));
  act(() => window.dispatchEvent(new Event("hickory-workspace-close-request")));
  expect(flushUi).toHaveBeenCalledTimes(1);
  expect(close).not.toHaveBeenCalled();
  await act(async () => finish());
  expect(close).toHaveBeenCalledTimes(1);
  hook.unmount();
});
</storycode:copy>
<a id="f-18ff98d49ede-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement-5">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("refuses changed unnamed bytes and duplicate close requests while a decision is pending", async () => {
  const layout = openUntitledTab(freeform()), pane = panes(layout.root)[0], tab = pane.tabs[0];
  const sources = { current: { [tab.id]: "captured" } };
  let choose!: (value: string) => void;
  const askChoice = vi.fn((_title: string, _options: unknown[], _policy?: unknown) => new Promise<string>(resolve => { choose = resolve; }));
  const setLayout = vi.fn(), forget = vi.fn(), error = vi.fn(), discard = vi.spyOn(api, "discardDraft").mockResolvedValue({ ok: true });
  const { result } = renderHook(() => useUnsavedLifecycle({ layout, layoutRef: { current: layout }, setLayout, registry: new SessionRegistry(), retainSavedDrafts: false,
    untitledSources: sources.current, untitledSourcesRef: sources, forgetUntitled: forget, saveUntitled: vi.fn(), prompt: { askChoice } as never,
    plainDirtyTabs: new Set(), plainUnsavedActions: { current: new Map() }, onError: error }));
  let pending!: ReturnType<typeof result.current.requestCloseTab>;
  await act(async () => { pending = result.current.requestCloseTab(pane.id, tab.id); });
  const policy = askChoice.mock.calls[0][2] as unknown as { agent: boolean; currentRevision: () => unknown };
  expect(policy.agent).toBe(true); const observed = policy.currentRevision();
  await act(async () => { expect(await result.current.requestCloseTab(pane.id, tab.id)).toMatchObject({ status: "unavailable" }); });
  sources.current[tab.id] = "new unseen bytes";
  expect(policy.currentRevision()).not.toBe(observed);
  await act(async () => { choose("close"); expect(await pending).toMatchObject({ status: "unavailable", reason: expect.stringContaining("No save or discard was submitted") }); });
  expect(setLayout).not.toHaveBeenCalled(); expect(discard).not.toHaveBeenCalled(); expect(forget).not.toHaveBeenCalled(); expect(error).toHaveBeenCalled();
});
</storycode:copy>
<a id="f-18ff98d49ede-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement-6">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("keeps newer plain-file edits open after an acknowledged discard and delayed recovery cleanup", async () => {
  const layout = open(freeform(), tab("file", "note.txt")), pane = panes(layout.root)[0], file = pane.tabs[0];
  let source = "original dirty", dirty = true, finish!: (value: { ok: true }) => void;
  const discard = vi.fn(() => { source = "saved base"; dirty = false; });
  vi.spyOn(api, "discardDraft").mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  const setLayout = vi.fn(), actions = { save: vi.fn(), discard, retain: vi.fn(), revision: () => source, isDirty: () => dirty };
  const { result } = renderHook(() => useUnsavedLifecycle({ layout, layoutRef: { current: layout }, setLayout, registry: new SessionRegistry(), retainSavedDrafts: false,
    untitledSources: {}, untitledSourcesRef: { current: {} }, forgetUntitled: vi.fn(), saveUntitled: vi.fn(), prompt: { askChoice: vi.fn().mockResolvedValue("close") } as never,
    plainDirtyTabs: new Set([file.id]), plainUnsavedActions: { current: new Map([[file.id, actions]]) } }));
  let pending!: ReturnType<typeof result.current.requestCloseTab>;
  await act(async () => { pending = result.current.requestCloseTab(pane.id, file.id); });
  expect(discard).toHaveBeenCalledTimes(1); source = "new edits during cleanup"; dirty = true;
  await act(async () => { finish({ ok: true }); expect(await pending).toMatchObject({ status: "unavailable", reason: expect.stringContaining("newer edits") }); });
  expect(setLayout).not.toHaveBeenCalled(); expect(source).toBe("new edits during cleanup");
});
</storycode:copy>
<a id="f-18ff98d49ede-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement-7">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("refuses a same-id same-source tab replacement while its decision is open", async () => {
  const layout = openUntitledTab(freeform()), pane = panes(layout.root)[0], entry = pane.tabs[0], layoutRef = { current: layout };
  let choose!: (value: string) => void;
  const askChoice = vi.fn(() => new Promise<string>(resolve => { choose = resolve; }));
  const setLayout = vi.fn(), discard = vi.spyOn(api, "discardDraft").mockResolvedValue({ ok: true });
  const { result } = renderHook(() => useUnsavedLifecycle({ layout, layoutRef, setLayout, registry: new SessionRegistry(), retainSavedDrafts: false,
    untitledSources: { [entry.id]: "same source" }, untitledSourcesRef: { current: { [entry.id]: "same source" } }, forgetUntitled: vi.fn(), saveUntitled: vi.fn(), prompt: { askChoice } as never,
    plainDirtyTabs: new Set(), plainUnsavedActions: { current: new Map() } }));
  let pending!: ReturnType<typeof result.current.requestCloseTab>;
  await act(async () => { pending = result.current.requestCloseTab(pane.id, entry.id); });
  layoutRef.current = { ...layout, root: { ...pane, tabs: pane.tabs.map(tab => ({ ...tab })) } };
  await act(async () => { choose("close"); expect(await pending).toMatchObject({ status: "unavailable" }); });
  expect(discard).not.toHaveBeenCalled(); expect(setLayout).not.toHaveBeenCalled();
});
</storycode:copy>
<a id="f-18ff98d49ede-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement-8">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("keeps a tab open when cancellation arrives after its save callback started", async () => {
  const layout = open(freeform(), tab("generated", "out.py")), pane = panes(layout.root)[0], entry = pane.tabs[0];
  let finish!: (value: boolean) => void;
  const save = vi.fn(() => new Promise<boolean>(resolve => { finish = resolve; }));
  viewBuffers.set(entry.id, { dirty: true, revision: () => "captured", save, discard: vi.fn() });
  const setLayout = vi.fn(), controller = new AbortController();
  const { result } = renderHook(() => useUnsavedLifecycle({ layout, layoutRef: { current: layout }, setLayout, registry: new SessionRegistry(), retainSavedDrafts: false,
    untitledSources: {}, untitledSourcesRef: { current: {} }, forgetUntitled: vi.fn(), saveUntitled: vi.fn(), prompt: { askChoice: vi.fn().mockResolvedValue("save") } as never,
    plainDirtyTabs: new Set(), plainUnsavedActions: { current: new Map() } }));
  let pending!: ReturnType<typeof result.current.requestCloseTab>;
  await act(async () => { pending = result.current.requestCloseTab(pane.id, entry.id, controller.signal); });
  expect(save).toHaveBeenCalledTimes(1);
  controller.abort();
  await act(async () => { finish(true); expect(await pending).toMatchObject({ status: "unavailable", reason: expect.stringContaining("not rollback") }); });
  expect(setLayout).not.toHaveBeenCalled();
});
</storycode:copy>
<a id="f-18ff98d49ede-windowHook"></a>
### `windowHook`

<storycode:copy id="f-18ff98d49ede-windowHook">
function windowHook(overrides: Partial<Parameters<typeof useUnsavedLifecycle>[0]> = {}, registry?: UiActions) {
  const layout = overrides.layout ?? freeform();
  const props: Parameters<typeof useUnsavedLifecycle>[0] = { layout, layoutRef: { current: layout }, setLayout: vi.fn(), registry: new SessionRegistry(), retainSavedDrafts: false,
    untitledSources: {}, untitledSourcesRef: { current: {} }, forgetUntitled: vi.fn(), saveUntitled: vi.fn(), prompt: { askChoice: vi.fn() } as never,
    plainDirtyTabs: new Set(), plainUnsavedActions: { current: new Map() }, ...overrides };
  return { ...renderHook(() => useUnsavedLifecycle(props), registry ? { wrapper: ({ children }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider> } : {}), props };
}
</storycode:copy>
<a id="f-18ff98d49ede-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement-9">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("acknowledges the actual host close callback without claiming window destruction", async () => {
  const close = vi.spyOn(api, "closeWindow").mockResolvedValue({ ok: true });
  const app = windowHook();
  await act(async () => { expect(await app.result.current.requestCloseWindow()).toMatchObject({ status: "completed", value: { host_close_request_acknowledged: true, window_destruction_observed: false } }); });
  expect(close).toHaveBeenCalledTimes(1);
});
</storycode:copy>
<a id="f-18ff98d49ede-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement-10">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it.each(["new input", "new lifecycle state"])("refuses window closure when %s arrives during UI flushing", async change => {
  const layout = openUntitledTab(freeform()), id = panes(layout.root)[0].tabs[0].id;
  const sources = { current: { [id]: "base" } }, lifecycle = ["terminal-a"];
  let finish!: () => void;
  const flushUi = vi.fn(() => new Promise<void>(resolve => { finish = resolve; }));
  const close = vi.spyOn(api, "closeWindow").mockResolvedValue({ ok: true });
  const app = windowHook({ layout, untitledSources: sources.current, untitledSourcesRef: sources, untitledBaselines: { [id]: "base" }, flushUi, windowGuard: () => lifecycle });
  let pending!: ReturnType<typeof app.result.current.requestCloseWindow>;
  await act(async () => { pending = app.result.current.requestCloseWindow(); });
  if (change === "new input") sources.current[id] = "new retained bytes"; else lifecycle.push("terminal-b");
  await act(async () => { finish(); expect(await pending).toMatchObject({ status: "unavailable", reason: expect.stringContaining("not submitted") }); });
  expect(close).not.toHaveBeenCalled(); expect(app.props.forgetUntitled).not.toHaveBeenCalled();
});
</storycode:copy>
<a id="f-18ff98d49ede-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement-11">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("prepares an observed window discard without closing tabs or forgetting their cached source", async () => {
  const layout = openUntitledTab(freeform()), id = panes(layout.root)[0].tabs[0].id, sources = { current: { [id]: "observed dirty bytes" } };
  const close = vi.spyOn(api, "closeWindow").mockResolvedValue({ ok: true });
  const discard = vi.spyOn(api, "discardDraft").mockResolvedValue({ ok: true });
  const app = windowHook({ layout, untitledSources: sources.current, untitledSourcesRef: sources, prompt: { askChoice: vi.fn().mockResolvedValue("close") } as never });
  await act(async () => { expect(await app.result.current.requestCloseWindow()).toMatchObject({ status: "completed", value: { host_close_request_acknowledged: true } }); });
  expect(discard).toHaveBeenCalledWith(untitledDraftKey(id)); expect(close).toHaveBeenCalledTimes(1);
  expect(app.props.setLayout).not.toHaveBeenCalled(); expect(app.props.forgetUntitled).not.toHaveBeenCalled(); expect(sources.current[id]).toBe("observed dirty bytes");
});
</storycode:copy>
<a id="f-18ff98d49ede-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement-12">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("retains a real host close refusal instead of swallowing it", async () => {
  const close = vi.spyOn(api, "closeWindow").mockRejectedValue(Error("this engine has no native window")), error = vi.fn();
  const app = windowHook({ onError: error });
  await act(async () => { expect(await app.result.current.requestCloseWindow()).toMatchObject({ status: "unavailable", reason: expect.stringContaining("no native window") }); });
  expect(close).toHaveBeenCalledTimes(1); expect(error).toHaveBeenCalledWith(expect.stringContaining("unacknowledged"));
});
</storycode:copy>
<a id="f-18ff98d49ede-expression_statement-13"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement-13">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("keeps close status readable while pending and distinguishes an earlier refusal from the latest request", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let hold = false, finish!: () => void;
  const flushUi = vi.fn(() => hold ? new Promise<void>(resolve => { finish = resolve; }) : Promise.resolve());
  const close = vi.spyOn(api, "closeWindow").mockRejectedValueOnce(Error("first refusal")).mockResolvedValue({ ok: true });
  const app = windowHook({ flushUi }, registry);
  await act(async () => { expect(await app.result.current.requestCloseWindow()).toMatchObject({ status: "unavailable" }); });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const action = (label: string) => actions().find(action => action.label === label)!;
  hold = true; let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, { handle: action("Close workspace window").handle, arguments: {}, request_id: "close-second" }); });
  // A native duplicate joins this exact in-flight close flow.
  act(() => window.dispatchEvent(new Event("hickory-workspace-close-request")));
  let state: { pending: boolean; last_is_current: boolean; current_request: string; last: { request: string } } | undefined;
  await act(async () => {
    const read = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, { handle: action("Read workspace close status bytes").handle, arguments: { offset: 0, limit: 8192 }, request_id: "read-close" }) as unknown as { outcome: { value: { bytes: number[] } } };
    state = JSON.parse(new TextDecoder().decode(new Uint8Array(read.outcome.value.bytes)));
  });
  expect(state).toMatchObject({ pending: true, last_is_current: false }); expect(state!.last.request).not.toBe(state!.current_request);
  expect(close).toHaveBeenCalledTimes(1);
  await act(async () => { finish(); expect(await pending).toMatchObject({ status: "result", outcome: { value: { host_close_request_acknowledged: true, window_destruction_observed: false } } }); });
  expect(close).toHaveBeenCalledTimes(2);
});

</storycode:copy>
<a id="f-18ff98d49ede-expression_statement-14"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement-14">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("retains all tabs after a later save refuses and reports earlier save effects", async () => {
  const layout = open(open(freeform(), tab("generated", "a.py")), tab("generated", "b.py"));
  const entries = panes(layout.root).flatMap(pane => pane.tabs), close = vi.spyOn(api, "closeWindow").mockResolvedValue({ ok: true });
  let dirty = true;
  const firstSave = vi.fn(async () => { dirty = false; return true; }), secondSave = vi.fn(async () => false);
  viewBuffers.set(entries[0].id, { dirty: true, isDirty: () => dirty, revision: () => dirty ? "pending" : "saved", save: firstSave, discard: vi.fn() });
  viewBuffers.set(entries[1].id, { dirty: true, revision: () => "pending-b", save: secondSave, discard: vi.fn() });
  const app = windowHook({ layout, prompt: { askChoice: vi.fn().mockResolvedValue("save") } as never });
  await act(async () => { expect(await app.result.current.requestCloseWindow()).toMatchObject({ status: "unavailable", reason: expect.stringContaining("Earlier save/discard") }); });
  expect(firstSave).toHaveBeenCalledTimes(1); expect(secondSave).toHaveBeenCalledTimes(1);
  expect(close).not.toHaveBeenCalled(); expect(app.props.setLayout).not.toHaveBeenCalled();
  expect(dirty).toBe(false);
});
</storycode:copy>
<a id="f-18ff98d49ede-expression_statement-15"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement-15">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("saves all captured dirty buffers before acknowledging host closure", async () => {
  const layout = open(freeform(), tab("generated", "out.py")), entry = panes(layout.root)[0].tabs[0];
  let dirty = true;
  const save = vi.fn(async () => { dirty = false; return true; });
  viewBuffers.set(entry.id, { dirty: true, isDirty: () => dirty, revision: () => dirty ? "pending" : "saved", save, discard: vi.fn() });
  const flush = vi.fn(async () => { expect(dirty).toBe(false); });
  const close = vi.spyOn(api, "closeWindow").mockImplementation(async () => { expect(flush).toHaveBeenCalledTimes(1); return { ok: true }; });
  const app = windowHook({ layout, flushUi: flush, prompt: { askChoice: vi.fn().mockResolvedValue("save") } as never });
  await act(async () => { expect(await app.result.current.requestCloseWindow()).toMatchObject({ status: "completed", value: { host_close_request_acknowledged: true } }); });
  expect(save).toHaveBeenCalledTimes(1); expect(close).toHaveBeenCalledTimes(1); expect(app.props.setLayout).not.toHaveBeenCalled();
});
</storycode:copy>
<a id="f-18ff98d49ede-expression_statement-16"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement-16">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("refuses stale window discard decisions before touching recovery", async () => {
  const layout = openUntitledTab(freeform()), id = panes(layout.root)[0].tabs[0].id, sources = { current: { [id]: "captured" } };
  let decide!: (value: string) => void;
  const askChoice = vi.fn(() => new Promise<string>(resolve => { decide = resolve; }));
  const close = vi.spyOn(api, "closeWindow").mockResolvedValue({ ok: true }), discard = vi.spyOn(api, "discardDraft").mockResolvedValue({ ok: true });
  const app = windowHook({ layout, untitledSources: sources.current, untitledSourcesRef: sources, prompt: { askChoice } as never });
  let pending!: ReturnType<typeof app.result.current.requestCloseWindow>;
  await act(async () => { pending = app.result.current.requestCloseWindow(); });
  sources.current[id] = "newer retained input";
  await act(async () => { decide("close"); expect(await pending).toMatchObject({ status: "unavailable", reason: expect.stringContaining("before window-close preparation") }); });
  expect(close).not.toHaveBeenCalled(); expect(discard).not.toHaveBeenCalled(); expect(app.props.forgetUntitled).not.toHaveBeenCalled();
});
</storycode:copy>
<a id="f-18ff98d49ede-expression_statement-17"></a>
### `expression_statement`

<storycode:copy id="f-18ff98d49ede-expression_statement-17">
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("never submits host closure after owner cancellation during a save", async () => {
  const layout = open(freeform(), tab("generated", "out.py")), entry = panes(layout.root)[0].tabs[0];
  let finish!: (value: boolean) => void;
  const save = vi.fn(() => new Promise<boolean>(resolve => { finish = resolve; }));
  viewBuffers.set(entry.id, { dirty: true, revision: () => "captured", save, discard: vi.fn() });
  const controller = new AbortController(), close = vi.spyOn(api, "closeWindow").mockResolvedValue({ ok: true });
  const app = windowHook({ layout, prompt: { askChoice: vi.fn().mockResolvedValue("save") } as never });
  let pending!: ReturnType<typeof app.result.current.requestCloseWindow>;
  await act(async () => { pending = app.result.current.requestCloseWindow(controller.signal); });
  expect(save).toHaveBeenCalledTimes(1); controller.abort();
  await act(async () => { finish(true); expect(await pending).toMatchObject({ status: "unavailable", reason: expect.stringContaining("not rollback") }); });
  expect(close).not.toHaveBeenCalled(); expect(app.props.setLayout).not.toHaveBeenCalled();
});
</storycode:copy>

### Assemble `apps/web/src/views/useUnsavedLifecycle.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useUnsavedLifecycle.test.tsx"><storycode:paste select="#f-18ff98d49ede-expression_statement"/><storycode:paste select="#f-18ff98d49ede-expression_statement-2"/><storycode:paste select="#f-18ff98d49ede-expression_statement-3"/><storycode:paste select="#f-18ff98d49ede-expression_statement-4"/><storycode:paste select="#f-18ff98d49ede-expression_statement-5"/><storycode:paste select="#f-18ff98d49ede-expression_statement-6"/><storycode:paste select="#f-18ff98d49ede-expression_statement-7"/><storycode:paste select="#f-18ff98d49ede-expression_statement-8"/><storycode:paste select="#f-18ff98d49ede-windowHook"/><storycode:paste select="#f-18ff98d49ede-expression_statement-9"/><storycode:paste select="#f-18ff98d49ede-expression_statement-10"/><storycode:paste select="#f-18ff98d49ede-expression_statement-11"/><storycode:paste select="#f-18ff98d49ede-expression_statement-12"/><storycode:paste select="#f-18ff98d49ede-expression_statement-13"/><storycode:paste select="#f-18ff98d49ede-expression_statement-14"/><storycode:paste select="#f-18ff98d49ede-expression_statement-15"/><storycode:paste select="#f-18ff98d49ede-expression_statement-16"/><storycode:paste select="#f-18ff98d49ede-expression_statement-17"/></storycode:file>


</storycode:doc>
