<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Trace a ribbon to the visible part of its source

For: Authors editing generated text and reviewers tracing a result to its source.

Narrow or horizontally scroll the Files pane while a ribbon points into it. The endpoint must remain on the visible row or fall back to the divider.

## What must happen

Cross-document lineage, context and declared links retain their recorded source identity and use the appropriate visible editor, Files row or divider endpoint. Files-row endpoints are clipped on both axes by every scrolling or clipping ancestor, excluding borders and scrollbar space. Fully clipped rows use the existing divider fallback. Tree scrolling schedules overlay updates. This presentation rule changes neither provenance semantics nor citation support.

## Limits of the evidence

The recorded geometry review assumes the shell's ordinary untransformed layout. Rotated/scaled custom hosts and semantic truth of citations are outside that witness.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/a-ribbon-crosses-documents.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold the October 8 visible-row correction into the current endpoint contract. Retire the rejected Input-marker presentation proposal as history.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-6d8acc68818f"></a>
## `apps/web/src/lib/visibleElementRect.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Trace a ribbon to the visible part of its source](contract.md).

<a id="f-6d8acc68818f-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-6d8acc68818f-expression_statement">
// @vitest-environment jsdom
// Guarantee: docs/guarantees/lineage/a-ribbon-crosses-documents.md
import { afterEach, expect, it, vi } from "vitest";
import { findTerminal, attachTerminal } from "../shell/Ribbons";

afterEach(() => { document.body.replaceChildren(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-6d8acc68818f-fixture"></a>
### `fixture`

<storycode:copy id="f-6d8acc68818f-fixture">
function fixture() {
  const container = document.createElement("div");
  container.innerHTML = '<div class="shell-pane__body"><div class="cm-scroller"><div data-tree-path="long-file.md"></div></div></div><button data-ribbon-port="file:long-file.md"></button>';
  document.body.append(container);
  const pane = container.querySelector<HTMLElement>(".shell-pane__body")!;
  const scroller = container.querySelector<HTMLElement>(".cm-scroller")!;
  const row = container.querySelector<HTMLElement>("[data-tree-path]")!;
  pane.style.overflow = scroller.style.overflow = "auto";
  vi.spyOn(pane, "getBoundingClientRect").mockReturnValue(new DOMRect(20, 10, 180, 300));
  vi.spyOn(scroller, "getBoundingClientRect").mockReturnValue(new DOMRect(30, 40, 160, 200));
  for (const [element, width, height] of [[pane, 180, 300], [scroller, 145, 185]] as const) {
    Object.defineProperties(element, { clientWidth: { value: width }, clientHeight: { value: height } });
  }
  const rect = vi.spyOn(row, "getBoundingClientRect").mockReturnValue(new DOMRect(40, 60, 600, 25));
  return { container, rect };
}
</storycode:copy>
<a id="f-6d8acc68818f-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-6d8acc68818f-expression_statement-2">
it.each([true, false])("lands on the visible row edge, excluding its scrollbar, braces=%s", braces => {
  const { container, rect } = fixture();
  const terminal = findTerminal(container, "file", "long-file.md")!;
  expect(terminal.ends).toBe("tree");
  expect(terminal.rect.right).toBe(175); // 30 + 145; the row's unclipped end is 640.
  const attach = attachTerminal(terminal, new DOMRect(), 800, { yTop: 50, yBot: 100 }, braces, { x: 790, y: 75 }, -1);
  expect(attach.connector).toMatchObject({ x: 172, width: 3 });
  expect(braces ? attach.link : attach.band).toContain("175");
  // Horizontal scrolling moves the row, but the visible right edge stays put.
  rect.mockReturnValue(new DOMRect(-200, 60, 600, 25));
  expect(findTerminal(container, "file", "long-file.md")!.rect.left).toBe(30);
  expect(findTerminal(container, "file", "long-file.md")!.rect.right).toBe(175);
});
</storycode:copy>
<a id="f-6d8acc68818f-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-6d8acc68818f-expression_statement-3">
it.each([new DOMRect(250, 60, 100, 25), new DOMRect(40, 230, 600, 25)])("uses the port when the row is fully clipped", hidden => {
  const { container, rect } = fixture(); rect.mockReturnValue(hidden);
  expect(findTerminal(container, "file", "long-file.md")!.ends).toBe("port");
});
</storycode:copy>

### Assemble `apps/web/src/lib/visibleElementRect.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/visibleElementRect.test.ts"><storycode:paste select="#f-6d8acc68818f-expression_statement"/><storycode:paste select="#f-6d8acc68818f-fixture"/><storycode:paste select="#f-6d8acc68818f-expression_statement-2"/><storycode:paste select="#f-6d8acc68818f-expression_statement-3"/></storycode:file>

</storycode:doc>
