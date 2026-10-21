// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { freeform, panes } from "../shell/layout";
import { openUntitledTab } from "./workspaceState";
import { useUntitledRecovery } from "./useUntitledRecovery";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
it.each(["changed bytes", "replaced tab"])("resumes unnamed recovery after discard for %s, ordered after cleanup", async change => {
  const layout = openUntitledTab(freeform()), pane = panes(layout.root)[0], entry = pane.tabs[0], layoutRef = { current: layout };
  const sourcesRef = { current: { [entry.id]: "captured draft" } };
  vi.spyOn(api, "drafts").mockResolvedValue({ drafts: [] });
  const order: string[] = [];
  const save = vi.spyOn(api, "saveDraft").mockImplementation(async draft => { order.push(`write:${draft.contents}`); return { ok: true }; });
  let finish!: () => void;
  const discard = vi.spyOn(api, "discardDraft").mockImplementation(async () => { await new Promise<void>(resolve => { finish = resolve; }); order.push("delete"); return { ok: true }; });
  const setLayout = vi.fn(), setSources = vi.fn(), initialBaselines = {};
  const { result, unmount } = renderHook(() => useUntitledRecovery({ hydrated: true, layoutRef, sourcesRef, setLayout, setSources, initialBaselines }));
  await waitFor(() => expect(result.current.ready).toBe(true));
  act(() => window.dispatchEvent(new Event("pagehide"))); await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
  const work: Promise<unknown>[] = [];
  act(() => window.dispatchEvent(new CustomEvent("hickory-untitled-discarded", { detail: { id: entry.id, waitUntil: (operation: Promise<unknown>) => work.push(operation) } })));
  await waitFor(() => expect(discard).toHaveBeenCalledTimes(1));
  if (change === "changed bytes") sourcesRef.current[entry.id] = "new draft";
  else layoutRef.current = { ...layout, root: { ...pane, tabs: pane.tabs.map(tab => ({ ...tab })) } };
  act(() => window.dispatchEvent(new Event("pagehide"))); expect(save).toHaveBeenCalledTimes(1);
  await act(async () => { finish(); await Promise.all(work); }); await waitFor(() => expect(save).toHaveBeenCalledTimes(2));
  expect(order).toEqual(["write:captured draft", "delete", `write:${change === "changed bytes" ? "new draft" : "captured draft"}`]); unmount();
});
