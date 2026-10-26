<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/useDocumentRender.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4c3020d7353c"></a>
## `apps/web/src/views/useDocumentRender.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-4c3020d7353c-useDocumentRender"></a>
### `useDocumentRender`

<storycode:copy id="f-4c3020d7353c-useDocumentRender">
import { useCallback, useEffect, useRef } from "react";
import { request } from "../api/client";
import type { Block, RenderResponse } from "../api/types";

/** A source-backed projection. Save acknowledgement and render validity are
 * independent: invalid syntax can remain saved without hiding its editor. */
export function useDocumentRender(document: string, currentSource: () => string | null,
  accept: (blocks: Block[]) => void, failed: (error: string | null) => void) {
  const latest = useRef({ currentSource, accept, failed }); latest.current = { currentSource, accept, failed };
  const sequence = useRef(0);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const refreshRender = useCallback(async (expectedSource?: string) => {
    if (timer.current !== null) { clearTimeout(timer.current); timer.current = null; }
    const revision = ++sequence.current;
    const requested = expectedSource ?? latest.current.currentSource();
    try {
      // A save must not join a pre-save api.render already in flight.
      const render = await request<RenderResponse & { document: string; source_hash: string }>("GET", `/api/docs/${document}/render`);
      const source = requested ?? latest.current.currentSource();
      if (render.document !== document || typeof render.source_hash !== "string") throw new Error("The document render has no current source receipt. Refresh the document.");
      if (source !== null) {
        const hash = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(source));
        const expected = [...new Uint8Array(hash)].map(byte => byte.toString(16).padStart(2, "0")).join("");
        if (render.source_hash.replace(/^sha256:/, "") !== expected || latest.current.currentSource() !== source) return;
      }
      if (revision !== sequence.current) return;
      latest.current.accept(render.blocks); latest.current.failed(null);
    } catch (error) {
      if (revision === sequence.current && (requested === null || latest.current.currentSource() === requested)) {
        latest.current.failed(error instanceof Error ? error.message : String(error));
      }
    }
  }, [document]);
  const scheduleRender = useCallback(() => {
    if (timer.current === null) timer.current = setTimeout(() => { timer.current = null; void refreshRender(); }, 150);
  }, [refreshRender]);
  useEffect(() => () => { ++sequence.current; if (timer.current !== null) clearTimeout(timer.current); }, [document]);
  return { refreshRender, scheduleRender };
}
</storycode:copy>

### Assemble `apps/web/src/views/useDocumentRender.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useDocumentRender.ts"><storycode:paste select="#f-4c3020d7353c-useDocumentRender"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-944cca0004b7"></a>
## `apps/web/src/views/useDocumentRender.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-944cca0004b7-receipt"></a>
### `receipt`

<storycode:copy id="f-944cca0004b7-receipt">
import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
// @ts-expect-error Node test runtime only.
import { webcrypto } from "node:crypto";
import { installMockHandler } from "../api/client";
import { useDocumentRender } from "./useDocumentRender";
async function receipt(source: string, id: string) {
  const hash = await webcrypto.subtle.digest("SHA-256", new TextEncoder().encode(source));
  return { document: "d", source_hash: [...new Uint8Array(hash)].map(byte => byte.toString(16).padStart(2, "0")).join(""), blocks: [{ kind: "prose", id }] };
}
</storycode:copy>
<a id="f-944cca0004b7-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-944cca0004b7-expression_statement">
afterEach(() => { cleanup(); installMockHandler(null as never); vi.unstubAllGlobals(); });
</storycode:copy>
<a id="f-944cca0004b7-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-944cca0004b7-expression_statement-2">
describe("source-backed render refresh", () => {
  it("accepts the newest acknowledged projection and refuses late older source responses", async () => {
    vi.stubGlobal("crypto", webcrypto);
    const replies: ((value: unknown) => void)[] = [];
    installMockHandler(() => new Promise(resolve => { replies.push(resolve); }));
    let source = "before";
    const accept = vi.fn(), failed = vi.fn();
    const hook = renderHook(() => useDocumentRender("d", () => source, accept, failed));
    const older = hook.result.current.refreshRender(source);
    source = "after adding a cell";
    const newer = hook.result.current.refreshRender(source);
    const current = await receipt(source, "new-cell");
    await act(async () => { replies[1](current); await newer; });
    await act(async () => { replies[0](await receipt("before", "old-cell")); await older; });
    expect(accept).toHaveBeenCalledTimes(1);
    expect(accept).toHaveBeenCalledWith(current.blocks);
  });
  it("does not publish a projection for another document or superseded editor text", async () => {
    vi.stubGlobal("crypto", webcrypto);
    let source = "saved";
    const accept = vi.fn();
    let answer = { ...await receipt(source, "cell"), document: "other" };
    installMockHandler(async () => answer);
    const hook = renderHook(() => useDocumentRender("d", () => source, accept, vi.fn()));
    await hook.result.current.refreshRender(source);
    expect(accept).not.toHaveBeenCalled();
    answer = await receipt("saved", "old-cell");
    installMockHandler(async () => { source = "new unsaved source"; return answer; });
    await hook.result.current.refreshRender("saved");
    expect(accept).not.toHaveBeenCalled();
  });
  it("does not publish an in-flight refresh after its document session unmounts", async () => {
    vi.stubGlobal("crypto", webcrypto);
    let reply!: (value: unknown) => void;
    installMockHandler(() => new Promise(resolve => { reply = resolve; }));
    const accept = vi.fn();
    const hook = renderHook(() => useDocumentRender("d", () => "saved", accept, vi.fn()));
    const waiting = hook.result.current.refreshRender("saved");
    hook.unmount();
    reply(await receipt("saved", "cell"));
    await waiting;
    expect(accept).not.toHaveBeenCalled();
  });
});
</storycode:copy>

### Assemble `apps/web/src/views/useDocumentRender.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useDocumentRender.test.tsx"><storycode:paste select="#f-944cca0004b7-receipt"/><storycode:paste select="#f-944cca0004b7-expression_statement"/><storycode:paste select="#f-944cca0004b7-expression_statement-2"/></storycode:file>


</storycode:doc>
