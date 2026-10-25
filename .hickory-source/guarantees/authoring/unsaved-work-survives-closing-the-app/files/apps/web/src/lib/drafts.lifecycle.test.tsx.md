<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/drafts.lifecycle.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Unsaved Work Survives Closing The App](../../../../../contract.md)

Captured SHA-256: `2830aaea6f15ffaa816d97e47886d0f51d5a4c94af3d3503b20b283b9c12fa84`.

<book:entry path="apps/web/src/lib/drafts.lifecycle.test.tsx">
// @vitest-environment jsdom
// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { discardRecoveryDraft, useDraftKeeper } from "./drafts";

afterEach(() => { cleanup(); vi.restoreAllMocks(); });

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

it("queues new named recovery after an explicit discard cleanup instead of suppressing it forever", async () => {
  let contents = "old draft", finish!: () => void;
  const order: string[] = [];
  const save = vi.spyOn(api, "saveDraft").mockImplementation(async draft => { order.push(`write:${draft.contents}`); return { ok: true }; });
  const discard = vi.spyOn(api, "discardDraft").mockImplementation(async () => { await new Promise<void>(resolve => { finish = resolve; }); order.push("delete"); return { ok: true }; });
  const { unmount } = renderHook(() => useDraftKeeper({ path: "note.md", read: () => ({ contents, base: "base" }) }));
  act(() => window.dispatchEvent(new Event("pagehide"))); await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
  const deleting = discardRecoveryDraft("note.md"); await waitFor(() => expect(discard).toHaveBeenCalledTimes(1));
  contents = "new retained bytes"; act(() => window.dispatchEvent(new Event("pagehide")));
  expect(save).toHaveBeenCalledTimes(1);
  await act(async () => { finish(); await deleting; });
  await waitFor(() => expect(save).toHaveBeenCalledTimes(2));
  expect(order).toEqual(["write:old draft", "delete", "write:new retained bytes"]); unmount();
});
it("orders undo-to-base cleanup between an earlier recovery write and later input", async () => {
  let contents = "old draft", finish!: () => void;
  const order: string[] = []; let writes = 0;
  const save = vi.spyOn(api, "saveDraft").mockImplementation(async draft => { if (++writes === 1) await new Promise<void>(resolve => { finish = resolve; }); order.push(`write:${draft.contents}`); return { ok: true }; });
  const discard = vi.spyOn(api, "discardDraft").mockImplementation(async () => { order.push("delete"); return { ok: true }; });
  const { unmount } = renderHook(() => useDraftKeeper({ path: "note.md", read: () => ({ contents, base: "base" }) }));
  act(() => window.dispatchEvent(new Event("pagehide"))); await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
  contents = "base"; act(() => window.dispatchEvent(new Event("pagehide")));
  contents = "new input"; act(() => window.dispatchEvent(new Event("pagehide")));
  expect(discard).not.toHaveBeenCalled();
  await act(async () => finish()); await waitFor(() => expect(save).toHaveBeenCalledTimes(2));
  expect(order).toEqual(["write:old draft", "delete", "write:new input"]); unmount();
});
it("does not let a retired path's save cleanup suppress the replacement path's recovery", async () => {
  const save = vi.spyOn(api, "saveDraft").mockResolvedValue({ ok: true });
  const discard = vi.spyOn(api, "discardDraft").mockResolvedValue({ ok: true });
  const sources: Record<string, string> = { "first.md": "old source", "second.md": "new source" };
  const { result, rerender, unmount } = renderHook(({ path }) => useDraftKeeper({ path, read: () => ({ contents: sources[path], base: "base" }) }), { initialProps: { path: "first.md" } });
  const oldCleanup = result.current;
  rerender({ path: "second.md" });
  act(() => oldCleanup());
  await waitFor(() => expect(discard).toHaveBeenCalledWith("first.md"));
  act(() => window.dispatchEvent(new Event("pagehide")));
  await waitFor(() => expect(save).toHaveBeenCalledWith(expect.objectContaining({ path: "second.md", contents: "new source" })));
  unmount();
});
</book:entry>

</book:doc>
