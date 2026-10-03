import { renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { freeform, open, tab } from "../shell/layout";
import { useRecentFiles } from "./useRecentFiles";

afterEach(() => vi.unstubAllGlobals());

describe("recent files", () => {
  it("reports document, generated and plain file navigation, without recording rerenders", async () => {
    const fetch = vi.fn().mockResolvedValue(new Response(null, { status: 204 }));
    vi.stubGlobal("fetch", fetch);
    let layout = open(freeform(), tab("document", "notes/today.md"));
    const { result, rerender } = renderHook(({ layout }) => useRecentFiles(layout), {
      initialProps: { layout },
    });
    await waitFor(() => expect(fetch).toHaveBeenCalledTimes(1));
    expect(result.current).toBe("notes/today.md");
    expect(JSON.parse(fetch.mock.calls[0][1].body)).toEqual({ path: "notes/today.md" });
    rerender({ layout });
    expect(fetch).toHaveBeenCalledTimes(1);
    for (const kind of ["generated", "file"] as const) {
      layout = open(layout, tab(kind, `${kind}.txt`));
      rerender({ layout });
    }
    expect(fetch).toHaveBeenCalledTimes(3);
  });

  it("ignores non-file tabs and tolerates a browser host without recent-file support", async () => {
    const fetch = vi.fn().mockRejectedValue(new Error("no desktop host"));
    vi.stubGlobal("fetch", fetch);
    let layout = freeform();
    const { rerender } = renderHook(({ layout }) => useRecentFiles(layout), {
      initialProps: { layout },
    });
    for (const kind of ["tool", "tree", "chat", "terminal", "untitled", "scratchpad"] as const) {
      layout = open(layout, tab(kind, kind));
      rerender({ layout });
    }
    expect(fetch).not.toHaveBeenCalled();
    layout = open(layout, tab("file", "readme.md"));
    rerender({ layout });
    await waitFor(() => expect(fetch).toHaveBeenCalledTimes(1));
  });
});
