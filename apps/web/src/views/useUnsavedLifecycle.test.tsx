import { viewBuffers } from "../lib/viewBuffers";
// @vitest-environment jsdom

import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { api } from "../api/client";
import { untitledDraftKey } from "../lib/newDoc";
import { freeform, panes, open, tab } from "../shell/layout";
import { SessionRegistry } from "./documentSession";
import { openUntitledTab } from "./workspaceState";
import { useUnsavedLifecycle } from "./useUnsavedLifecycle";

afterEach(() => vi.restoreAllMocks());

describe("closing unsaved work", () => {
  // Guarantees:
  // - docs/guarantees/authoring/new-document-is-an-act.md
  // - docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
  it("retains an unnamed draft when its tab closes", async () => {
    const saveDraft = vi.spyOn(api, "saveDraft").mockResolvedValue({ ok: true });
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

    act(() => result.current.requestCloseTab(pane.id, tab.id));

    await waitFor(() => expect(setLayout).toHaveBeenCalledTimes(1));
    expect(askChoice.mock.calls[0][0]).toContain(
      "restored next time you open Hickory Docs",
    );
    expect(askChoice.mock.calls[0][1].map((option: { label: string }) => option.label)).toEqual([
      "Save",
      "Close and retain",
      "Cancel",
    ]);
    expect(saveDraft).toHaveBeenCalledWith(expect.objectContaining({ path: untitledDraftKey(tab.id), contents: "remember this" }));
    expect(forgetUntitled).not.toHaveBeenCalled();
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
  act(() => result.current.requestCloseTab(pane.id, workingTab.id));
  await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
  expect(askChoice.mock.calls[0][0]).toContain("no recovery draft");
  expect(setLayout).not.toHaveBeenCalled();
  unmount(); viewBuffers.forget(workingTab.id);
});
