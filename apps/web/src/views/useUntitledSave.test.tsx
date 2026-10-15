import { useRef, useState } from "react";
import { useWorkspaceUi } from "./useWorkspaceUi";
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { panes, type Layout } from "../shell/layout";
import { initialWorkspace, openUntitledTab } from "./workspaceState";
import { useUntitledSave } from "./useUntitledSave";

afterEach(() => vi.restoreAllMocks());

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
