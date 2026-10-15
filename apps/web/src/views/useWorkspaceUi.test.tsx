import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { useState } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { freeform, open, panes, split, tab, type Layout } from "../shell/layout";
import { useWorkspaceUi, UI_SAVE_DEBOUNCE_MS } from "./useWorkspaceUi";

afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); });

// Guarantee: docs/guarantees/authoring/a-folder-remembers-its-window.md
it("loads each folder before saving, restoring its tabs and split proportions", async () => {
  let stored = open(freeform(), tab("file", "a.txt"));
  stored = split(stored, stored.focus, "column");
  stored = open(stored, tab("file", "b.txt"));
  const load = vi.spyOn(api, "workspaceUi").mockResolvedValue({ state: { version: 1, layout: stored } });
  const save = vi.spyOn(api, "saveWorkspaceUi").mockResolvedValue({ ok: true });
  const { result, rerender } = renderHook(({ scope }: { scope: string | null }) => {
    const [layout, setLayout] = useState<Layout>(freeform);
    return { layout, ...useWorkspaceUi(layout, setLayout, scope) };
  }, { initialProps: { scope: null as string | null } });
  expect(load).not.toHaveBeenCalled();
  rerender({ scope: "/notes/a" });
  await waitFor(() => expect(result.current.hydrated).toBe(true));
  expect(result.current.layout.root.type).toBe("split");
  expect(panes(result.current.layout.root).flatMap(pane => pane.tabs.map(tab => tab.target))).toEqual(["a.txt", "b.txt"]);
  expect(save).not.toHaveBeenCalled();
  const other = open(freeform(), tab("file", "other.txt"));
  load.mockResolvedValue({ state: { version: 1, layout: other } });
  rerender({ scope: "/notes/b" });
  await waitFor(() => expect(panes(result.current.layout.root)[0].tabs[0].target).toBe("other.txt"));
  await act(() => result.current.flush());
  expect(save).toHaveBeenLastCalledWith(expect.objectContaining({ layout: result.current.layout }), false);
});

it("flushes the latest layout immediately on close or pagehide, including empty splits", async () => {
  vi.spyOn(api, "workspaceUi").mockResolvedValue({ state: null });
  const save = vi.spyOn(api, "saveWorkspaceUi").mockResolvedValue({ ok: true });
  const { result } = renderHook(() => {
    const [layout, setLayout] = useState<Layout>(freeform);
    return { setLayout, layout, ...useWorkspaceUi(layout, setLayout) };
  });
  await waitFor(() => expect(result.current.hydrated).toBe(true));
  vi.useFakeTimers();
  act(() => result.current.setLayout(current => split(current, current.focus, "row")));
  expect(save).not.toHaveBeenCalled();
  await act(() => result.current.flush());
  expect(save).toHaveBeenLastCalledWith(expect.objectContaining({ layout: result.current.layout }), false);
  await act(async () => { window.dispatchEvent(new Event("pagehide")); });
  expect(save).toHaveBeenLastCalledWith(expect.objectContaining({ layout: result.current.layout }), true);
  await act(async () => { vi.advanceTimersByTime(UI_SAVE_DEBOUNCE_MS); });
});

it("ignores a late response from the previous folder", async () => {
  let answer!: (value: { state: null }) => void;
  const load = vi.spyOn(api, "workspaceUi").mockReturnValueOnce(new Promise(resolve => { answer = resolve; }));
  load.mockResolvedValue({ state: { version: 1, wrap: { "b.txt": 60 } } });
  vi.spyOn(api, "saveWorkspaceUi").mockResolvedValue({ ok: true });
  const apply = vi.fn();
  const { result, rerender } = renderHook(({ scope }) => useWorkspaceUi(freeform(), apply, scope), { initialProps: { scope: "a" } });
  rerender({ scope: "b" });
  await waitFor(() => expect(result.current.hydrated).toBe(true));
  await act(async () => answer({ state: null }));
  expect(result.current.wrapFor("b.txt")).toBe(60);
});
