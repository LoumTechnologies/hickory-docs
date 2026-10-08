// @vitest-environment jsdom
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { discardRecoveryDraft, useDraftKeeper } from "./drafts";

afterEach(() => vi.restoreAllMocks());

it("waits for an unfinished recovery write and never rewrites explicitly discarded work", async () => {
  let finishWrite!: () => void;
  const writing = new Promise<void>(resolve => { finishWrite = resolve; });
  const save = vi.spyOn(api, "saveDraft").mockImplementation(async () => { await writing; return { ok: true }; });
  const discard = vi.spyOn(api, "discardDraft").mockResolvedValue({ ok: true });
  const { unmount } = renderHook(() => useDraftKeeper({
    path: "note.md", read: () => ({ contents: "unsaved", base: "original" }),
  }));
  act(() => window.dispatchEvent(new Event("pagehide")));
  await waitFor(() => expect(save).toHaveBeenCalledOnce());
  const discarded = discardRecoveryDraft("note.md");
  expect(discard).not.toHaveBeenCalled();
  await act(async () => { finishWrite(); await discarded; });
  expect(discard).toHaveBeenCalledWith("note.md");
  act(() => window.dispatchEvent(new Event("pagehide")));
  unmount();
  expect(save).toHaveBeenCalledOnce();
});
