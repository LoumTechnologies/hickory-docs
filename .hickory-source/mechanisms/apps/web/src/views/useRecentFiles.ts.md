<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/useRecentFiles.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-040bc6a6d6b3"></a>
## `apps/web/src/views/useRecentFiles.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-040bc6a6d6b3-useRecentFiles"></a>
### `useRecentFiles`

<storycode:copy id="f-040bc6a6d6b3-useRecentFiles">
import { useEffect } from "react";
import { paneById, type Layout } from "../shell/layout";

/** Report file navigation to the desktop's machine-wide recent list.
 * Browser hosts have no such route; remembering never interrupts opening. */
export function useRecentFiles(layout: Layout): string | null {
  const pane = paneById(layout, layout.focus);
  const tab = pane?.tabs[pane.active];
  const file = tab && ["document", "generated", "file"].includes(tab.kind)
    ? tab.target
    : null;
  useEffect(() => {
    if (!file) return;
    void fetch("/api/recent-files", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ path: file }),
    }).catch(() => {});
  }, [file]);
  return file;
}
</storycode:copy>

### Assemble `apps/web/src/views/useRecentFiles.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useRecentFiles.ts"><storycode:paste select="#f-040bc6a6d6b3-useRecentFiles"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-d8fbfc8aad0c"></a>
## `apps/web/src/views/useRecentFiles.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-d8fbfc8aad0c-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-d8fbfc8aad0c-expression_statement">
import { renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { freeform, open, tab } from "../shell/layout";
import { useRecentFiles } from "./useRecentFiles";

afterEach(() => vi.unstubAllGlobals());

</storycode:copy>
<a id="f-d8fbfc8aad0c-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-d8fbfc8aad0c-expression_statement-2">
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
    const { result, rerender } = renderHook(({ layout }) => useRecentFiles(layout), {
      initialProps: { layout },
    });
    for (const kind of ["tool", "tree", "chat", "terminal", "untitled", "scratchpad"] as const) {
      layout = open(layout, tab(kind, kind));
      rerender({ layout });
      expect(result.current).toBeNull();
    }
    expect(fetch).not.toHaveBeenCalled();
    layout = open(layout, tab("file", "readme.md"));
    rerender({ layout });
    await waitFor(() => expect(fetch).toHaveBeenCalledTimes(1));
  });
});
</storycode:copy>

### Assemble `apps/web/src/views/useRecentFiles.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useRecentFiles.test.tsx"><storycode:paste select="#f-d8fbfc8aad0c-expression_statement"/><storycode:paste select="#f-d8fbfc8aad0c-expression_statement-2"/></storycode:file>


</storycode:doc>
