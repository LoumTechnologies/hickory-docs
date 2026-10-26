<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/useUntitledSave.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7589eb95cb0f"></a>
## `apps/web/src/views/useUntitledSave.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Folder Remembers Its Window](../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md), [New Document Creates Another Unnamed Buffer](../../../../../guarantees/authoring/new-document-is-an-act/contract.md).

<a id="f-7589eb95cb0f-useUntitledSave"></a>
### `useUntitledSave`

<storycode:copy id="f-7589eb95cb0f-useUntitledSave">
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
</storycode:copy>

### Assemble `apps/web/src/views/useUntitledSave.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useUntitledSave.ts"><storycode:paste select="#f-7589eb95cb0f-useUntitledSave"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-1a7b240c6588"></a>
## `apps/web/src/views/useUntitledSave.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Folder Remembers Its Window](../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md), [New Document Creates Another Unnamed Buffer](../../../../../guarantees/authoring/new-document-is-an-act/contract.md).

<a id="f-1a7b240c6588-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-1a7b240c6588-expression_statement">
import { useRef, useState } from "react";
import { useWorkspaceUi } from "./useWorkspaceUi";
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { panes, type Layout } from "../shell/layout";
import { initialWorkspace, openUntitledTab } from "./workspaceState";
import { useUntitledSave } from "./useUntitledSave";

afterEach(() => { cleanup(); vi.restoreAllMocks(); });

</storycode:copy>
<a id="f-1a7b240c6588-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-1a7b240c6588-expression_statement-2">
it("saves the requested unnamed buffer even when another tab is active or edited during the dialog", async () => {
  const layoutRef = { current: openUntitledTab(openUntitledTab(initialWorkspace()), true) };
  const tabs = panes(layoutRef.current.root).flatMap(pane => pane.tabs);
  const sources = { [tabs[0].id]: "# First\nfirst text", [tabs[1].id]: "# Second\nsecond text" };
  vi.spyOn(api, "projects").mockResolvedValue([{ id: "p", name: "p", visibility: "private", created_at: "" }]);
  vi.spyOn(api, "projectDocs").mockResolvedValue([]);
  let choose!: (value: { path: string }) => void;
  const picker = vi.spyOn(api, "saveFileDialog").mockImplementation(() => new Promise(resolve => { choose = resolve; }));
  const created = { id: "saved", path: "/outside/First.md", source: "# First\nfirst text\n", updated_at: "" };
  const create = vi.spyOn(api, "createDoc").mockResolvedValue(created);
  const save = vi.spyOn(api, "saveDoc").mockResolvedValue(created);
  vi.spyOn(api, "discardDraft").mockResolvedValue({ ok: true });
  const error = vi.fn();
  const { result } = renderHook(() => useUntitledSave({ layoutRef, setLayout: vi.fn(), sourceFor: id => sources[id], onError: error }));
  let pending!: Promise<boolean>;
  await act(async () => { pending = result.current(false, tabs[0].id); });
  expect(picker).toHaveBeenCalledWith("First.md");
  sources[tabs[1].id] = "Second note changed during the dialog";
  await act(async () => { choose({ path: created.path }); expect(await pending).toBe(true); });
  expect(create).toHaveBeenCalledWith("p", created.path, "# First\nfirst text\n");
  expect(save).not.toHaveBeenCalled();
  expect(error).not.toHaveBeenCalled();
});

</storycode:copy>
<a id="f-1a7b240c6588-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-1a7b240c6588-expression_statement-3">
// Guarantee: docs/guarantees/authoring/a-folder-remembers-its-window.md
it("publishes a newly named tab before the closing window flushes its layout", async () => {
  vi.spyOn(api, "projects").mockResolvedValue([{ id: "p", name: "p", visibility: "private", created_at: "" }]);
  vi.spyOn(api, "projectDocs").mockResolvedValue([]);
  vi.spyOn(api, "saveFileDialog").mockResolvedValue({ path: "Saved.md" });
  vi.spyOn(api, "createDoc").mockResolvedValue({ id: "saved", path: "Saved.md", source: "# Saved", updated_at: "" });
  vi.spyOn(api, "discardDraft").mockResolvedValue({ ok: true });
  vi.spyOn(api, "workspaceUi").mockResolvedValue({ state: null });
  const store = vi.spyOn(api, "saveWorkspaceUi").mockResolvedValue({ ok: true });
  const { result, unmount } = renderHook(() => {
    const [layout, setLayout] = useState<Layout>(() => openUntitledTab(initialWorkspace()));
    const layoutRef = useRef(layout); layoutRef.current = layout;
    const ui = useWorkspaceUi(layout, setLayout);
    const save = useUntitledSave({ layoutRef, setLayout, sourceFor: () => "# Saved", onError: vi.fn() });
    return { ui, save };
  });
  await waitFor(() => expect(result.current.ui.hydrated).toBe(true));
  await act(async () => {
    expect(await result.current.save(false)).toBe(true);
    await result.current.ui.flush();
  });
  const state = store.mock.calls.at(-1)![0] as { layout: Layout };
  expect(panes(state.layout.root).flatMap(pane => pane.tabs)).toEqual([expect.objectContaining({ kind: "document", target: "Saved.md", docId: "saved" })]);
  unmount();
});
</storycode:copy>
<a id="f-1a7b240c6588-saveFixture"></a>
### `saveFixture`

<storycode:copy id="f-1a7b240c6588-saveFixture">
function saveFixture() {
  const layoutRef = { current: openUntitledTab(initialWorkspace()) };
  const id = panes(layoutRef.current.root).flatMap(pane => pane.tabs).find(tab => tab.kind === "untitled")!.id;
  const sources = { [id]: "# Captured" };
  vi.spyOn(api, "projects").mockResolvedValue([{ id: "p", name: "p", visibility: "private", created_at: "" }]);
  vi.spyOn(api, "projectDocs").mockResolvedValue([]);
  const picker = vi.spyOn(api, "saveFileDialog");
  const created = { id: "named", path: "chosen.md", source: "# Captured\n", updated_at: "" };
  const create = vi.spyOn(api, "createDoc").mockResolvedValue(created);
  const discard = vi.spyOn(api, "discardDraft").mockResolvedValue({ ok: true });
  const error = vi.fn(), setLayout = vi.fn(), onSaved = vi.fn();
  const hook = renderHook(() => useUntitledSave({ layoutRef, setLayout, onError: error, onSaved, sourceFor: id => sources[id] }));
  return { ...hook, id, sources, picker, create, created, error, setLayout, discard, onSaved };
}
</storycode:copy>
<a id="f-1a7b240c6588-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-1a7b240c6588-expression_statement-4">
// Guarantee: docs/guarantees/authoring/new-document-is-an-act.md
it("uses an explicit destination without opening a native chooser", async () => {
  const fixture = saveFixture();
  await act(async () => { expect(await fixture.result.current(true, fixture.id, "chosen.md")).toBe(true); });
  expect(fixture.picker).not.toHaveBeenCalled();
  expect(fixture.create).toHaveBeenCalledExactlyOnceWith("p", "chosen.md", "# Captured\n");
  expect(fixture.setLayout).toHaveBeenCalledTimes(1);
  fixture.unmount();
});
</storycode:copy>
<a id="f-1a7b240c6588-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-1a7b240c6588-expression_statement-5">
// Guarantee: docs/guarantees/authoring/new-document-is-an-act.md
it("refuses an originating buffer changed before destination creation", async () => {
  const fixture = saveFixture();
  let resolve!: (docs: never[]) => void;
  vi.mocked(api.projectDocs).mockImplementation(() => new Promise(done => { resolve = done; }));
  let pending!: Promise<boolean>;
  await act(async () => { pending = fixture.result.current(true, fixture.id, "chosen.md"); });
  fixture.sources[fixture.id] = "new unseen bytes";
  await act(async () => { resolve([]); expect(await pending).toBe(false); });
  expect(fixture.create).not.toHaveBeenCalled(); expect(fixture.setLayout).not.toHaveBeenCalled();
  expect(fixture.error).toHaveBeenCalledWith(expect.stringContaining("No file was created"));
  fixture.unmount();
});
</storycode:copy>
<a id="f-1a7b240c6588-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-1a7b240c6588-expression_statement-6">
// Guarantee: docs/guarantees/authoring/new-document-is-an-act.md
it.each(["edited", "unmounted"])("retains the actual created-file outcome when its buffer is %s during writing", async change => {
  const fixture = saveFixture();
  let resolve!: (doc: typeof fixture.created) => void;
  fixture.create.mockImplementation(() => new Promise(done => { resolve = done; }));
  let pending!: Promise<boolean>;
  await act(async () => { pending = fixture.result.current(true, fixture.id, "chosen.md"); });
  expect(fixture.create).toHaveBeenCalledTimes(1);
  // A repeated request cannot create a second destination for this buffer.
  await act(async () => { expect(await fixture.result.current(true, fixture.id, "second.md")).toBe(false); });
  if (change === "edited") fixture.sources[fixture.id] = "newer draft"; else fixture.unmount();
  await act(async () => { resolve(fixture.created); expect(await pending).toBe(false); });
  expect(fixture.setLayout).not.toHaveBeenCalled(); expect(fixture.discard).not.toHaveBeenCalled(); expect(fixture.onSaved).not.toHaveBeenCalled();
  expect(fixture.error).toHaveBeenLastCalledWith(expect.stringContaining("Saved the captured source to chosen.md"));
  expect(fixture.create).toHaveBeenCalledTimes(1);
  if (change === "edited") fixture.unmount();
});
</storycode:copy>
<a id="f-1a7b240c6588-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-1a7b240c6588-expression_statement-7">
// Guarantee: docs/guarantees/authoring/new-document-is-an-act.md
it("reports an unacknowledged create without adopting or retrying it", async () => {
  const fixture = saveFixture();
  fixture.create.mockRejectedValue(Error("reply lost"));
  await act(async () => { expect(await fixture.result.current(true, fixture.id, "chosen.md")).toBe(false); });
  expect(fixture.create).toHaveBeenCalledTimes(1);
  expect(fixture.error).toHaveBeenCalledWith(expect.stringContaining("Creation of chosen.md was not acknowledged"));
  expect(fixture.setLayout).not.toHaveBeenCalled(); expect(fixture.discard).not.toHaveBeenCalled();
  expect(fixture.picker).not.toHaveBeenCalled(); fixture.unmount();
});
</storycode:copy>

### Assemble `apps/web/src/views/useUntitledSave.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useUntitledSave.test.tsx"><storycode:paste select="#f-1a7b240c6588-expression_statement"/><storycode:paste select="#f-1a7b240c6588-expression_statement-2"/><storycode:paste select="#f-1a7b240c6588-expression_statement-3"/><storycode:paste select="#f-1a7b240c6588-saveFixture"/><storycode:paste select="#f-1a7b240c6588-expression_statement-4"/><storycode:paste select="#f-1a7b240c6588-expression_statement-5"/><storycode:paste select="#f-1a7b240c6588-expression_statement-6"/><storycode:paste select="#f-1a7b240c6588-expression_statement-7"/></storycode:file>


</storycode:doc>
