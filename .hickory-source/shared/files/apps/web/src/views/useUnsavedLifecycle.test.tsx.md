<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/useUnsavedLifecycle.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Folder Remembers Its Window](../../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md)
- [New Document Creates Another Unnamed Buffer](../../../../../../guarantees/authoring/new-document-is-an-act/contract.md)
- [Saved means the displayed bytes reached the file](../../../../../../guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file/contract.md)
- [Unsaved Work Survives Closing The App](../../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md)

Captured SHA-256: `518608027b16d7d4254c8c7f3abbc843bf0130e16c3ebb7bc0fa1fd263c95cdb`.

<book:entry path="apps/web/src/views/useUnsavedLifecycle.test.tsx">
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
function windowHook(overrides: Partial<Parameters<typeof useUnsavedLifecycle>[0]> = {}, registry?: UiActions) {
  const layout = overrides.layout ?? freeform();
  const props: Parameters<typeof useUnsavedLifecycle>[0] = { layout, layoutRef: { current: layout }, setLayout: vi.fn(), registry: new SessionRegistry(), retainSavedDrafts: false,
    untitledSources: {}, untitledSourcesRef: { current: {} }, forgetUntitled: vi.fn(), saveUntitled: vi.fn(), prompt: { askChoice: vi.fn() } as never,
    plainDirtyTabs: new Set(), plainUnsavedActions: { current: new Map() }, ...overrides };
  return { ...renderHook(() => useUnsavedLifecycle(props), registry ? { wrapper: ({ children }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider> } : {}), props };
}
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("acknowledges the actual host close callback without claiming window destruction", async () => {
  const close = vi.spyOn(api, "closeWindow").mockResolvedValue({ ok: true });
  const app = windowHook();
  await act(async () => { expect(await app.result.current.requestCloseWindow()).toMatchObject({ status: "completed", value: { host_close_request_acknowledged: true, window_destruction_observed: false } }); });
  expect(close).toHaveBeenCalledTimes(1);
});
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
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
it("retains a real host close refusal instead of swallowing it", async () => {
  const close = vi.spyOn(api, "closeWindow").mockRejectedValue(Error("this engine has no native window")), error = vi.fn();
  const app = windowHook({ onError: error });
  await act(async () => { expect(await app.result.current.requestCloseWindow()).toMatchObject({ status: "unavailable", reason: expect.stringContaining("no native window") }); });
  expect(close).toHaveBeenCalledTimes(1); expect(error).toHaveBeenCalledWith(expect.stringContaining("unacknowledged"));
});
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
</book:entry>

</book:doc>
