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

<a id="output-da337093a58f"></a>
## `apps/web/src/lib/ribbons.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Trace a ribbon to the visible part of its source](contract.md).

<a id="f-da337093a58f-DOC"></a>
### `DOC`

<storycode:copy id="f-da337093a58f-DOC">
import { describe, expect, it } from "vitest";
import { deriveRibbons, drawnRange, fragmentKey, groupBySourceBlock, RIBBON_PALETTE_SIZE } from "./ribbons";
import { weaveOutputs } from "./weave";
import { WEAVE_SOURCE } from "../mock/mockData";
import type { OutputFile, Provenance } from "../api/types";

const DOC = "docs/weave-demo.md";

</storycode:copy>
<a id="f-da337093a58f-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-da337093a58f-expression_statement">
describe("deriveRibbons — from the weave demo's real provenance", () => {
  const [file] = weaveOutputs(WEAVE_SOURCE, DOC);

  it("yields one ribbon per non-synthetic provenance entry", () => {
    const ribbons = deriveRibbons(file, DOC, WEAVE_SOURCE);
    const editable = file.provenance.filter((p) => p.origin.kind !== "synthetic");
    expect(ribbons).toHaveLength(editable.length);
    expect(ribbons.length).toBeGreaterThanOrEqual(4); // 2 pastes + literals
  });

  it("synthetic ranges get no ribbon", () => {
    const ribbons = deriveRibbons(file, DOC, WEAVE_SOURCE);
    expect(file.provenance.some((p) => p.origin.kind === "synthetic")).toBe(true);
    expect(ribbons.every((r) => (r.kind as string) !== "synthetic")).toBe(true);
  });

  it("output text of each ribbon equals the source text of its span", () => {
    for (const r of deriveRibbons(file, DOC, WEAVE_SOURCE)) {
      expect(file.content.slice(...r.outputRange)).toBe(
        WEAVE_SOURCE.slice(...r.sourceSpan),
      );
    }
  });

  it("thickness input is the output byte length", () => {
    for (const r of deriveRibbons(file, DOC, WEAVE_SOURCE)) {
      expect(r.bytes).toBe(r.outputRange[1] - r.outputRange[0]); // ASCII demo
      expect(r.bytes).toBeGreaterThan(0);
    }
  });

  it("distinct source fragments get distinct stable colors, in order", () => {
    const ribbons = deriveRibbons(file, DOC, WEAVE_SOURCE);
    const seen = new Map<string, number>();
    for (const r of ribbons) {
      const k = `${r.sourceByteSpan[0]}:${r.sourceByteSpan[1]}`;
      if (seen.has(k)) expect(r.color).toBe(seen.get(k));
      else seen.set(k, r.color);
      expect(r.color).toBeGreaterThanOrEqual(0);
      expect(r.color).toBeLessThan(RIBBON_PALETTE_SIZE);
    }
    // First few distinct fragments walk the palette in order.
    expect([...seen.values()].slice(0, RIBBON_PALETTE_SIZE)).toEqual(
      [...seen.values()].slice(0, RIBBON_PALETTE_SIZE).map((_, i) => i),
    );
  });
});

</storycode:copy>
<a id="f-da337093a58f-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-da337093a58f-expression_statement-2">
describe("deriveRibbons — filtering and repeats", () => {
  it("the same fragment pasted twice keeps one color across two ribbons", () => {
    const file: OutputFile = {
      path: "out.py",
      language: "python",
      content: "abcabc",
      provenance: [
        { start: 0, end: 3, origin: { kind: "paste", doc_path: "d.md", span: [10, 13] } },
        { start: 3, end: 6, origin: { kind: "paste", doc_path: "d.md", span: [10, 13] } },
      ],
    };
    const ribbons = deriveRibbons(file, "d.md", "0123456789abc.md source");
    expect(ribbons).toHaveLength(2);
    expect(ribbons[0].color).toBe(ribbons[1].color);
  });

  it("provenance from other documents is skipped (no local anchor)", () => {
    const file: OutputFile = {
      path: "out.py",
      language: "python",
      content: "xy",
      provenance: [
        { start: 0, end: 1, origin: { kind: "literal", doc_path: "other.md", span: [0, 1] } },
        { start: 1, end: 2, origin: { kind: "literal", doc_path: "mine.md", span: [0, 1] } },
      ],
    };
    const ribbons = deriveRibbons(file, "mine.md", "s");
    expect(ribbons).toHaveLength(1);
    expect(ribbons[0].key).toBe("out.py:1");
  });

  it("empty output ranges are skipped", () => {
    const file: OutputFile = {
      path: "out.py",
      language: "python",
      content: "x",
      provenance: [
        { start: 1, end: 1, origin: { kind: "literal", doc_path: "d", span: [0, 0] } },
      ],
    };
    expect(deriveRibbons(file, "d", "src")).toHaveLength(0);
  });

  it("converts byte offsets to char offsets for non-ASCII content", () => {
    // "é" is 2 UTF-8 bytes but 1 UTF-16 unit.
    const source = "é<x>";
    const file: OutputFile = {
      path: "o",
      language: "text",
      content: "é!",
      provenance: [{ start: 2, end: 3, origin: { kind: "literal", doc_path: "d", span: [2, 3] } }],
    };
    const [r] = deriveRibbons(file, "d", source);
    expect(r.outputRange).toEqual([1, 2]); // after the 2-byte é
    expect(r.sourceSpan).toEqual([1, 2]);
  });
});

</storycode:copy>
<a id="f-da337093a58f-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-da337093a58f-expression_statement-3">
describe("groupBySourceBlock — one terminal band per source block", () => {
  const [file] = weaveOutputs(WEAVE_SOURCE, DOC);
  const ribbons = deriveRibbons(file, DOC, WEAVE_SOURCE);

  it("collapses fragments of one block into one group keyed by its byte span", () => {
    const groups = groupBySourceBlock(ribbons);
    const spans = new Set(ribbons.map((r) => fragmentKey(r)));
    expect(groups.size).toBe(spans.size);
    // A file a block feeds twice is one statement, not two.
    expect(groups.size).toBeLessThanOrEqual(ribbons.length);
  });

  it("sums the bytes of every fragment in the group", () => {
    const groups = groupBySourceBlock(ribbons);
    let total = 0;
    for (const group of groups.values()) total += group.bytes;
    expect(total).toBe(ribbons.reduce((sum, r) => sum + r.bytes, 0));
  });

  it("keeps the FIRST ribbon of the group, so a click lands where the block's text starts", () => {
    const groups = groupBySourceBlock(ribbons);
    for (const [key, group] of groups) {
      const first = ribbons.find((r) => fragmentKey(r) === key);
      expect(group.ribbon).toBe(first);
    }
  });
});

</storycode:copy>
<a id="f-da337093a58f-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-da337093a58f-expression_statement-4">
describe("whitespaceOnly", () => {
  // A whitespace-only attribution is true but says nothing until someone is
  // working exactly there — the overlay draws these on hover only.
  const file = (content: string, provenance: Provenance[]): OutputFile => ({
    path: "out.txt",
    content,
    provenance,
    language: "text",
  });
  const literal = (start: number, end: number, span: [number, number]): Provenance => ({
    start,
    end,
    origin: { kind: "literal", doc_path: "doc.md", span },
  });

  it("flags a ribbon whose attributed bytes are blank lines and indentation", () => {
    const content = "code\n\n    \nmore";
    const ribbons = deriveRibbons(
      file(content, [literal(4, 11, [10, 17])]),
      "doc.md",
      "0123456789\n\n    \nabc",
    );
    expect(ribbons).toHaveLength(1);
    expect(ribbons[0].whitespaceOnly).toBe(true);
  });

  it("keeps a ribbon with any visible character always-on", () => {
    const content = "code\n\nx\nmore";
    const ribbons = deriveRibbons(
      file(content, [literal(4, 8, [10, 14])]),
      "doc.md",
      "0123456789\n\nx\nzzzzzz",
    );
    expect(ribbons).toHaveLength(1);
    expect(ribbons[0].whitespaceOnly).toBe(false);
  });
});

</storycode:copy>
<a id="f-da337093a58f-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-da337093a58f-expression_statement-5">
describe("drawnRange", () => {
  // A span's DRAWN extent covers only lines where it owns visible
  // characters — trailing newlines/indentation on someone else's line must
  // not inflate a line-granular brace into a false claim.
  it("drops a leading newline belonging to a foreign line", () => {
    //            0         1
    //            0123456789012345678
    const text = "</hick:copy>\ncode\n";
    // Span starts at the newline after the foreign tag.
    expect(drawnRange(text, 12, 18)).toEqual([13, 18]);
  });

  it("drops trailing indentation on a line whose text is foreign", () => {
    const text = "own line\n    foreign\n";
    // Span covers "own line\n" plus the next line's indentation only.
    expect(drawnRange(text, 0, 13)).toEqual([0, 9]);
  });

  it("keeps blank lines the span covers", () => {
    const text = "a\n\n\nb\n";
    expect(drawnRange(text, 0, 6)).toEqual([0, 6]);
  });

  it("returns null for a whitespace-only span on foreign lines", () => {
    const text = "foreign\nalso foreign\n";
    expect(drawnRange(text, 7, 8)).toBeNull();
  });

  it("keeps a span fully inside one owned line", () => {
    const text = "abc def\n";
    expect(drawnRange(text, 4, 7)).toEqual([4, 7]);
  });

  it("never widens beyond the original span", () => {
    const text = "abcdef\n";
    const trimmed = drawnRange(text, 2, 4);
    expect(trimmed).toEqual([2, 4]);
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/ribbons.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/ribbons.test.ts"><storycode:paste select="#f-da337093a58f-DOC"/><storycode:paste select="#f-da337093a58f-expression_statement"/><storycode:paste select="#f-da337093a58f-expression_statement-2"/><storycode:paste select="#f-da337093a58f-expression_statement-3"/><storycode:paste select="#f-da337093a58f-expression_statement-4"/><storycode:paste select="#f-da337093a58f-expression_statement-5"/></storycode:file>

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

<a id="output-e45bafe1c480"></a>
## `apps/web/src/shell/ribbonNavigation.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Trace a ribbon to the visible part of its source](contract.md).

<a id="f-e45bafe1c480-pane"></a>
### `pane`

<storycode:copy id="f-e45bafe1c480-pane">
// @vitest-environment jsdom
// Guarantee: docs/guarantees/lineage/a-ribbon-crosses-documents.md
import { expect, it } from "vitest";
import { directionBetween, prepareRibbonOpen, type Direction, type PaneBox } from "./ribbonNavigation";
import { panes, type Layout, type Pane } from "./layout";
import { openDocTab, openFileTab, openGeneratedTab } from "../views/workspaceState";
const pane = (id: string, kind: "document" | "chat" | "tree"): Pane => ({ type: "pane", id, active: 0, tabs: [{ id, kind, target: id }] });
</storycode:copy>
<a id="f-e45bafe1c480-fixture"></a>
### `fixture`

<storycode:copy id="f-e45bafe1c480-fixture">
function fixture(order: string[]) {
  const children = order.map(id => pane(id, id === "editor" ? "document" : id === "agent" ? "chat" : "tree"));
  const layout: Layout = { root: { type: "split", id: "root", direction: "row", children, sizes: children.map(() => 1 / children.length) }, focus: "agent" };
  const boxes: PaneBox[] = order.map((id, index) => ({ id, left: index * 200, right: index * 200 + 180, top: 0, bottom: 300 }));
  return { layout, boxes };
}
</storycode:copy>
<a id="f-e45bafe1c480-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-e45bafe1c480-expression_statement">
it.each([["editor", "tree", "agent"], ["tree", "editor", "agent"], ["agent", "tree", "editor"]])("routes a tree connection into its directional editor: %s %s %s", (...order) => {
  const { layout, boxes } = fixture(order);
  const direction = order.indexOf("tree") < order.indexOf("agent") ? "left" : "right";
  const routed = prepareRibbonOpen(layout, boxes, { from: "agent", direction });
  expect(routed.focus).toBe("editor");
  for (const opened of [openFileTab(routed, "new.py", routed.focus), openDocTab(routed, "note", "note.md", routed.focus), openGeneratedTab(routed, "owner", "out.py", [], routed.focus)]) {
    expect(panes(opened.root).find(p => p.tabs.some(t => ["new.py", "note.md", "out.py"].includes(t.target)))?.id).toBe("editor");
    expect(panes(opened.root).find(p => p.id === "agent")?.tabs).toEqual(panes(layout.root).find(p => p.id === "agent")?.tabs);
  }
});
</storycode:copy>
<a id="f-e45bafe1c480-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-e45bafe1c480-expression_statement-2">
it("uses the pane the connection points to when the target is already open", () => {
  const { layout, boxes } = fixture(["editor", "tree", "agent"]);
  expect(prepareRibbonOpen(layout, boxes, { from: "agent", into: "editor", direction: "left" }).focus).toBe("editor");
});
</storycode:copy>
<a id="f-e45bafe1c480-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-e45bafe1c480-expression_statement-3">
it.each(["left", "right", "up", "down"] as Direction[])("creates an editor toward %s when no editor exists", direction => {
  const { layout, boxes } = fixture(["tree", "agent"]);
  const routed = prepareRibbonOpen(layout, boxes, { from: "agent", direction });
  expect(routed.focus).not.toBe("agent");
  const split = panes(routed.root).map(p => p.id);
  expect(split.indexOf(routed.focus) < split.indexOf("agent")).toBe(direction === "left" || direction === "up");
  expect(panes(routed.root).find(p => p.id === "agent")?.tabs[0].kind).toBe("chat");
});
</storycode:copy>
<a id="f-e45bafe1c480-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-e45bafe1c480-expression_statement-4">
it("follows vertical placement too", () => {
  const { layout, boxes } = fixture(["editor", "tree", "agent"]);
  const vertical = boxes.map((box, index) => ({ ...box, left: 0, right: 300, top: index * 200, bottom: index * 200 + 180 }));
  expect(prepareRibbonOpen(layout, vertical, { from: "agent", direction: "up" }).focus).toBe("editor");
  expect(directionBetween(new DOMRect(0, 400, 300, 180), new DOMRect(0, 200, 300, 180))).toBe("up");
});
</storycode:copy>
<a id="f-e45bafe1c480-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-e45bafe1c480-expression_statement-5">
it("routes ordinary editor provenance too, independent of which pane has focus", () => {
  const { layout, boxes } = fixture(["agent", "tree", "editor"]);
  const other = pane("other", "document");
  if (layout.root.type !== "split") throw new Error("expected split");
  layout.root.children.unshift(other); layout.root.sizes = [0.25, 0.25, 0.25, 0.25];
  boxes.unshift({ id: "other", left: -200, right: -20, top: 0, bottom: 300 });
  const routed = prepareRibbonOpen(layout, boxes, { from: "editor", direction: "left" });
  expect(routed.focus).toBe("other");
});
</storycode:copy>

### Assemble `apps/web/src/shell/ribbonNavigation.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/ribbonNavigation.test.ts"><storycode:paste select="#f-e45bafe1c480-pane"/><storycode:paste select="#f-e45bafe1c480-fixture"/><storycode:paste select="#f-e45bafe1c480-expression_statement"/><storycode:paste select="#f-e45bafe1c480-expression_statement-2"/><storycode:paste select="#f-e45bafe1c480-expression_statement-3"/><storycode:paste select="#f-e45bafe1c480-expression_statement-4"/><storycode:paste select="#f-e45bafe1c480-expression_statement-5"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/shell/Ribbons.tsx](../../../mechanisms/apps/web/src/shell/Ribbons.tsx.md#output-b916e85d3b0b)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)

### Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../mechanisms/apps/web/e2e/literate-editor.spec.ts.md#output-648693644a28)
- [apps/web/src/lib/ribbons.test.ts](contract.md#output-da337093a58f)
- [apps/web/src/lib/uiState.test.ts](../../../mechanisms/apps/web/src/lib/uiState.ts.md#output-43efcbce9427)
- [apps/web/src/lib/visibleElementRect.test.ts](contract.md#output-6d8acc68818f)
- [apps/web/src/shell/ribbonNavigation.test.ts](contract.md#output-e45bafe1c480)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
