// @vitest-environment jsdom

import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { api } from "../api/client";
import { freeform, panes } from "../shell/layout";
import { SessionRegistry } from "./documentSession";
import { openUntitledTab } from "./workspaceState";
import { useUnsavedLifecycle } from "./useUnsavedLifecycle";

afterEach(() => vi.restoreAllMocks());

describe("closing unsaved work", () => {
  // Guarantees:
  // - docs/guarantees/authoring/new-document-is-an-act.md
  // - docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
  it("says an unnamed draft will survive, persists it, then closes the tab", async () => {
    const layout = openUntitledTab(freeform());
    const pane = panes(layout.root).find((candidate) =>
      candidate.tabs.some((tab) => tab.kind === "untitled"),
    )!;
    const tab = pane.tabs.find((candidate) => candidate.kind === "untitled")!;
    const saveDraft = vi.spyOn(api, "saveDraft").mockResolvedValue({ ok: true });
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
      "unsaved changes will be retained and restored",
    );
    expect(askChoice.mock.calls[0][1].map((option: { label: string }) => option.label)).toEqual([
      "Save",
      "Close and retain",
      "Cancel",
    ]);
    expect(saveDraft).toHaveBeenCalledWith(
      expect.objectContaining({ path: `untitled:${tab.id}`, contents: "remember this" }),
    );
    expect(forgetUntitled).toHaveBeenCalledWith(tab.id);
  });
});
