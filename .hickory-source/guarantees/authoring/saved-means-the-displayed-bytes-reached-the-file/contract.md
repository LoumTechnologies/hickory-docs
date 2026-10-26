<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Saved means the displayed bytes reached the file

For: Authors editing executable documents and the files those documents produce.

Given an edited named document, ordinary file, generated-file buffer, or
literate view, when its save succeeds, then only the acknowledged text becomes
clean. An earlier save response cannot mark later typing saved. Rendering
failure does not invalidate acknowledged document persistence. A failed or
conflicted write stays dirty and closing waits for a successful Save or an
explicit discard/retention choice.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-64b5e8cd78b2"></a>
## `apps/web/src/views/useDocumentPersistence.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Saved means the displayed bytes reached the file](contract.md).

<a id="f-64b5e8cd78b2-doc"></a>
### `doc`

<storycode:copy id="f-64b5e8cd78b2-doc">
// @vitest-environment jsdom
import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { api } from "../api/client";
import type { Doc } from "../api/types";
import { useDocumentPersistence } from "./useDocumentPersistence";

const doc = (source: string) => ({ id: "d", path: "note.md", source }) as Doc;
</storycode:copy>
<a id="f-64b5e8cd78b2-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-64b5e8cd78b2-expression_statement">
beforeEach(() => vi.useFakeTimers());
</storycode:copy>
<a id="f-64b5e8cd78b2-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-64b5e8cd78b2-expression_statement-2">
afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); });

</storycode:copy>
<a id="f-64b5e8cd78b2-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-64b5e8cd78b2-expression_statement-3">
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
</storycode:copy>

### Assemble `apps/web/src/views/useDocumentPersistence.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useDocumentPersistence.test.tsx"><storycode:paste select="#f-64b5e8cd78b2-doc"/><storycode:paste select="#f-64b5e8cd78b2-expression_statement"/><storycode:paste select="#f-64b5e8cd78b2-expression_statement-2"/><storycode:paste select="#f-64b5e8cd78b2-expression_statement-3"/></storycode:file>

<a id="output-ef518e4c83d6"></a>
## `apps/web/src/views/useDocumentPersistence.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Saved means the displayed bytes reached the file](contract.md).

<a id="f-ef518e4c83d6-SyncState"></a>
### `SyncState`

<storycode:copy id="f-ef518e4c83d6-SyncState">
import { useEffect, useRef, useState } from "react";
import { api } from "../api/client";
import type { Doc } from "../api/types";

export type SyncState = "idle" | "editing" | "saved" | "failed";

</storycode:copy>
<a id="f-ef518e4c83d6-useDocumentPersistence"></a>
### `useDocumentPersistence`

Saved means the current text was acknowledged on disk. Rendering is a
separate operation: a syntax error must never turn a successful save into
an unsaved buffer. Ignore replies for text superseded while waiting.

<storycode:copy id="f-ef518e4c83d6-useDocumentPersistence">
/** Saved means the current text was acknowledged on disk. Rendering is a
 * separate operation: a syntax error must never turn a successful save into
 * an unsaved buffer. Ignore replies for text superseded while waiting. */
export function useDocumentPersistence(docId: string, doc: Doc | null, source: string | null, confirmed: (doc: Doc) => void) {
  const [savedSource, setSavedSource] = useState<string | null>(null);
  const [syncState, setSyncState] = useState<SyncState>("idle");
  const confirmedRef = useRef(confirmed); confirmedRef.current = confirmed;
  useEffect(() => {
    if (doc && savedSource === null) setSavedSource(doc.source);
  }, [doc, savedSource]);
  useEffect(() => {
    if (source === null || !doc) return;
    if (source === doc.source) {
      setSavedSource(source);
      setSyncState(current => current === "idle" ? "idle" : "saved");
      return;
    }
    setSyncState("editing");
    let live = true;
    let attempts = 0;
    let timer: ReturnType<typeof setTimeout>;
    const check = async () => {
      try {
        const stored = await api.doc(docId);
        if (!live) return;
        if (stored.source !== source) {
          if (++attempts >= 30) setSyncState("failed");
          else timer = setTimeout(check, 300);
          return;
        }
        setSavedSource(source); setSyncState("saved"); confirmedRef.current(stored);
      } catch {
        if (live) setSyncState("failed");
      }
    };
    timer = setTimeout(check, 1200);
    return () => { live = false; clearTimeout(timer); };
  }, [docId, source, doc?.source]); // eslint-disable-line react-hooks/exhaustive-deps
  return { savedSource, setSavedSource, syncState, setSyncState };
}
</storycode:copy>

### Assemble `apps/web/src/views/useDocumentPersistence.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useDocumentPersistence.ts"><storycode:paste select="#f-ef518e4c83d6-SyncState"/><storycode:paste select="#f-ef518e4c83d6-useDocumentPersistence"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/lib/outputSave.ts](../../../mechanisms/apps/web/src/lib/outputSave.ts.md#output-ff2a9a5b4231)
- [apps/web/src/lib/plainFileSave.ts](../../../mechanisms/apps/web/src/lib/plainFileSave.ts.md#output-07b4144df7a0)
- [apps/web/src/lib/viewBuffers.ts](../../../mechanisms/apps/web/src/lib/viewBuffers.ts.md#output-7b9e9d92a497)
- [apps/web/src/shell/views.tsx](../../../mechanisms/apps/web/src/shell/views.tsx.md#output-68a8fba10658)
- [apps/web/src/views/RepresentationPane.tsx](../../../mechanisms/apps/web/src/views/RepresentationPane.tsx.md#output-cd66c1c2250e)
- [apps/web/src/views/documentSession.tsx](../../../mechanisms/apps/web/src/views/documentSession.tsx.md#output-bfb59577f48b)
- [apps/web/src/views/useDocumentPersistence.ts](contract.md#output-ef518e4c83d6)
- [apps/web/src/views/useUnsavedLifecycle.ts](../../../mechanisms/apps/web/src/views/useUnsavedLifecycle.ts.md#output-0a84f897aa4c)

### Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../mechanisms/apps/web/e2e/literate-editor.spec.ts.md#output-648693644a28)
- [apps/web/src/lib/outputSave.test.ts](../../../mechanisms/apps/web/src/lib/outputSave.ts.md#output-5dd719c6fb43)
- [apps/web/src/lib/plainFileSave.test.ts](../../../mechanisms/apps/web/src/lib/plainFileSave.ts.md#output-94afcad10b27)
- [apps/web/src/shell/views.test.tsx](../../../mechanisms/apps/web/src/shell/views.tsx.md#output-003396968e5c)
- [apps/web/src/views/RepresentationPane.test.tsx](../../../mechanisms/apps/web/src/views/RepresentationPane.tsx.md#output-17926fbc73d7)
- [apps/web/src/views/useDocumentPersistence.test.tsx](contract.md#output-64b5e8cd78b2)
- [apps/web/src/views/useUnsavedLifecycle.test.tsx](../../../mechanisms/apps/web/src/views/useUnsavedLifecycle.ts.md#output-18ff98d49ede)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-5370130a675d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
