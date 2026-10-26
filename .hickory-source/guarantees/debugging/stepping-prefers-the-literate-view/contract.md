<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Stepping prefers the launching literate view

For: Authors debugging generated code while navigating its document source.

Given a debugging session launched from a literate view, when it pauses at a
line represented by that view, then navigation reveals the line in the view
instead of opening its ordinary source file. This also applies to a callee in
another represented file. The paused-line highlight follows the mapped frame.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/stepping-prefers-the-literate-view.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-4f4a7f326234"></a>
## `apps/web/src/debug/literateDebugNavigation.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Stepping prefers the launching literate view](contract.md).

<a id="f-4f4a7f326234-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-4f4a7f326234-expression_statement">
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { retainCapturedContext } from "../lib/capturedContexts";
import { IDLE_SESSION } from "./useDebugger";
import { ensurePlainDebugHost, PlainDebugHosts, resetPlainDebugHosts } from "./plainDebugHosts";
import { clearPreferredDebugSurface, preferDebugSurface, registerDebugSurface } from "./preferredDebugLocation";
import { OPEN_LOCATION_EVENT } from "../lib/revealLine";

vi.mock("../lib/capturedContexts", () => ({retainCapturedContext:vi.fn(() => vi.fn())}));

</storycode:copy>
<a id="f-4f4a7f326234-state"></a>
### `state`

<storycode:copy id="f-4f4a7f326234-state">
const state = vi.hoisted(() => ({ debug: {} }));
</storycode:copy>
<a id="f-4f4a7f326234-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-4f4a7f326234-expression_statement-2">
vi.mock("./useDebugger", async importOriginal => ({
  ...await importOriginal<typeof import("./useDebugger")>(),
  useWorkspaceDebugger: () => state.debug,
}));
</storycode:copy>
<a id="f-4f4a7f326234-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-4f4a7f326234-expression_statement-3">
afterEach(() => { cleanup(); resetPlainDebugHosts(); clearPreferredDebugSurface("main.py"); });

</storycode:copy>
<a id="f-4f4a7f326234-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-4f4a7f326234-expression_statement-4">
// Guarantee: docs/guarantees/debugging/stepping-prefers-the-literate-view.md
it("lets the launching literate surface consume step navigation before opening a plain file", () => {
  const frame = { id: 1, name: "helper", source: "helper.py", source_line: 2, line: null, in_document: false };
  state.debug = { ...IDLE_SESSION, status: "paused", frames: [frame] };
  const reveal = vi.fn(() => true), open = vi.fn();
  const unregister = registerDebugSurface("view", reveal);
  preferDebugSurface("main.py", "view");
  window.addEventListener(OPEN_LOCATION_EVENT, open);
  ensurePlainDebugHost("main.py");
  const mounted = render(<PlainDebugHosts />);
  expect(reveal).toHaveBeenCalledWith(frame);
  expect(open).not.toHaveBeenCalled();
  reveal.mockReturnValue(false);
  act(() => { state.debug = { ...state.debug, frames: [{ ...frame, source_line: 3 }] }; });
  mounted.rerender(<PlainDebugHosts />);
  expect(open).toHaveBeenCalledOnce();
  window.removeEventListener(OPEN_LOCATION_EVENT, open);
  unregister();
});

</storycode:copy>
<a id="f-4f4a7f326234-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-4f4a7f326234-expression_statement-5">
it("retains a captured project continuously across running and paused states", () => {
  const release = vi.fn();
  vi.mocked(retainCapturedContext).mockClear().mockReturnValue(release);
  state.debug = {...IDLE_SESSION, status:"starting"};
  ensurePlainDebugHost("hick-project:///test/app.py");
  const mounted = render(<PlainDebugHosts/>);
  for (const status of ["running","paused","running","paused"]) {
    act(() => {state.debug = {...IDLE_SESSION,status};});
    mounted.rerender(<PlainDebugHosts/>);
    expect(release).not.toHaveBeenCalled();
  }
  expect(retainCapturedContext).toHaveBeenCalledOnce();
  act(() => {state.debug = {...IDLE_SESSION,status:"finished"};});
  mounted.rerender(<PlainDebugHosts/>);
  expect(release).toHaveBeenCalledOnce();
});
</storycode:copy>

### Assemble `apps/web/src/debug/literateDebugNavigation.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/literateDebugNavigation.test.tsx"><storycode:paste select="#f-4f4a7f326234-expression_statement"/><storycode:paste select="#f-4f4a7f326234-state"/><storycode:paste select="#f-4f4a7f326234-expression_statement-2"/><storycode:paste select="#f-4f4a7f326234-expression_statement-3"/><storycode:paste select="#f-4f4a7f326234-expression_statement-4"/><storycode:paste select="#f-4f4a7f326234-expression_statement-5"/></storycode:file>

<a id="output-dd01d28ebce2"></a>
## `apps/web/src/debug/preferredDebugLocation.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Stepping prefers the launching literate view](contract.md).

<a id="f-dd01d28ebce2-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-dd01d28ebce2-expression_statement">
import { expect, it, vi } from "vitest";
import { clearPreferredDebugSurface, preferDebugSurface, registerDebugSurface, revealPreferredDebugLocation, representedFrameOffset } from "./preferredDebugLocation";
import { type RepresentedFile } from "../lsp/representationMapping";
import type { Frame } from "./client";

// Guarantee: docs/guarantees/debugging/stepping-prefers-the-literate-view.md
it("keeps a callee represented in the launching view and falls back for absent lines", () => {
  const source = "# Helper\nreturn 2\n";
  const file: RepresentedFile = { path: "helper.py", hash: "", content: "return 2\n", provenance: [
    { start: 0, end: 9, origin: { kind: "literal", doc_path: "view", span: [9, 18] } },
  ] };
  const reveal = vi.fn();
  const unregister = registerDebugSurface("view", frame => {
    const offset = representedFrameOffset(source, [file], frame);
    if (offset === null) return false;
    reveal(offset);
    return true;
  });
  const frame: Frame = { id: 1, name: "helper", source: "helper.py", source_line: 0, line: null, in_document: false };
  preferDebugSurface("main.py", "view");
  expect(revealPreferredDebugLocation("main.py", frame)).toBe(true);
  expect(reveal).toHaveBeenCalledWith(9);
  expect(revealPreferredDebugLocation("main.py", { ...frame, source: "library.py" })).toBe(false);
  expect(revealPreferredDebugLocation("main.py", { ...frame, source_line: 10 })).toBe(false);
  expect(revealPreferredDebugLocation("other.py", frame)).toBe(false);
  clearPreferredDebugSurface("main.py");
  expect(revealPreferredDebugLocation("main.py", frame)).toBe(false);
  preferDebugSurface("main.py", "view");
  unregister();
  expect(revealPreferredDebugLocation("main.py", frame)).toBe(false);
  clearPreferredDebugSurface("main.py");
});
</storycode:copy>

### Assemble `apps/web/src/debug/preferredDebugLocation.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/preferredDebugLocation.test.ts"><storycode:paste select="#f-dd01d28ebce2-expression_statement"/></storycode:file>

## Implementation review

### Tests and fixtures

- [apps/web/src/debug/literateDebugNavigation.test.tsx](contract.md#output-4f4a7f326234)
- [apps/web/src/debug/preferredDebugLocation.test.ts](contract.md#output-dd01d28ebce2)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
