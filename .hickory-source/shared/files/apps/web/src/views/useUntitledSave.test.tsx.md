<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/useUntitledSave.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Folder Remembers Its Window](../../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md)
- [New Document Creates Another Unnamed Buffer](../../../../../../guarantees/authoring/new-document-is-an-act/contract.md)

Captured SHA-256: `0f504b759ce662aae3dad6a069b2b438ec7889a65b35dee72f772784630c469a`.

<book:entry path="apps/web/src/views/useUntitledSave.test.tsx">
import { useRef, useState } from "react";
import { useWorkspaceUi } from "./useWorkspaceUi";
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { panes, type Layout } from "../shell/layout";
import { initialWorkspace, openUntitledTab } from "./workspaceState";
import { useUntitledSave } from "./useUntitledSave";

afterEach(() => { cleanup(); vi.restoreAllMocks(); });

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
// Guarantee: docs/guarantees/authoring/new-document-is-an-act.md
it("uses an explicit destination without opening a native chooser", async () => {
  const fixture = saveFixture();
  await act(async () => { expect(await fixture.result.current(true, fixture.id, "chosen.md")).toBe(true); });
  expect(fixture.picker).not.toHaveBeenCalled();
  expect(fixture.create).toHaveBeenCalledExactlyOnceWith("p", "chosen.md", "# Captured\n");
  expect(fixture.setLayout).toHaveBeenCalledTimes(1);
  fixture.unmount();
});
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
</book:entry>

</book:doc>
