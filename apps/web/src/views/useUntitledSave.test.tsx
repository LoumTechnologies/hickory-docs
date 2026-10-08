import { act, renderHook } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { panes } from "../shell/layout";
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
