// @vitest-environment jsdom
import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { api } from "../api/client";
import type { Doc } from "../api/types";
import { useDocumentPersistence } from "./useDocumentPersistence";

const doc = (source: string) => ({ id: "d", path: "note.md", source }) as Doc;
beforeEach(() => vi.useFakeTimers());
afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); });

// Guarantee: docs/guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file.md
describe("acknowledged document persistence", () => {
  it("keeps changes unsaved until disk acknowledges the exact text, independently of rendering", async () => {
    const read = vi.spyOn(api, "doc").mockResolvedValueOnce(doc("old")).mockResolvedValueOnce(doc("new"));
    const render = vi.spyOn(api, "render");
    const confirmed = vi.fn();
    const { result } = renderHook(() => useDocumentPersistence("d", doc("old"), "new", confirmed));
    expect(result.current.savedSource).toBe("old");
    await act(() => vi.advanceTimersByTimeAsync(1200));
    expect(result.current.syncState).toBe("editing");
    expect(result.current.savedSource).toBe("old");
    await act(() => vi.advanceTimersByTimeAsync(300));
    expect(result.current.syncState).toBe("saved");
    expect(result.current.savedSource).toBe("new");
    expect(read).toHaveBeenCalledTimes(2);
    expect(confirmed).toHaveBeenCalledWith(doc("new"));
    expect(render).not.toHaveBeenCalled();
  });

  it("ignores a delayed acknowledgment after the user types newer text", async () => {
    let resolve!: (doc: Doc) => void;
    vi.spyOn(api, "doc").mockImplementationOnce(() => new Promise(done => { resolve = done; })).mockResolvedValue(doc("latest"));
    const confirmed = vi.fn();
    const { result, rerender } = renderHook(({ source }) => useDocumentPersistence("d", doc("old"), source, confirmed), { initialProps: { source: "first" } });
    await act(() => vi.advanceTimersByTimeAsync(1200));
    rerender({ source: "latest" });
    await act(async () => resolve(doc("first")));
    expect(result.current.savedSource).toBe("old");
    expect(confirmed).not.toHaveBeenCalled();
    await act(() => vi.advanceTimersByTimeAsync(1200));
    expect(result.current.savedSource).toBe("latest");
  });

  it("reports an unconfirmed save without accepting the buffer as saved", async () => {
    vi.spyOn(api, "doc").mockRejectedValue(new Error("offline"));
    const { result } = renderHook(() => useDocumentPersistence("d", doc("old"), "new", vi.fn()));
    await act(() => vi.advanceTimersByTimeAsync(1200));
    expect(result.current.syncState).toBe("failed");
    expect(result.current.savedSource).toBe("old");
  });

  it("stops waiting when automatic persistence never acknowledges the buffer", async () => {
    vi.spyOn(api, "doc").mockResolvedValue(doc("old"));
    const { result } = renderHook(() => useDocumentPersistence("d", doc("old"), "new", vi.fn()));
    await act(() => vi.advanceTimersByTimeAsync(10000));
    expect(result.current.syncState).toBe("failed");
    expect(result.current.savedSource).toBe("old");
    expect(vi.getTimerCount()).toBe(0);
  });
});
