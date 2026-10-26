<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Every Hover Hint Is Drawn In The App Theme

For: Authors editing executable documents and the files those documents produce.

Given any control, row, or gutter marker in the app that explains itself on
hover, when the pointer rests on it (or keyboard focus reaches it), then the
explanation is drawn by the app in the palette the user chose — the same
tokens as every other raised surface — and never by the operating system's
`title=` tooltip.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/every-hover-hint-is-drawn-in-the-app-theme.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-fb855cfd8048"></a>
## `apps/web/src/lib/tooltip.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Every Hover Hint Is Drawn In The App Theme](contract.md).

<a id="f-fb855cfd8048-viewport"></a>
### `viewport`

<storycode:copy id="f-fb855cfd8048-viewport">
import { describe, expect, it } from "vitest";
import { TIP_ATTR, placeTip, tipTargetOf } from "./tooltip";

const viewport = { width: 1000, height: 800 };
</storycode:copy>
<a id="f-fb855cfd8048-tip"></a>
### `tip`

<storycode:copy id="f-fb855cfd8048-tip">
const tip = { width: 200, height: 40 };

</storycode:copy>
<a id="f-fb855cfd8048-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-fb855cfd8048-expression_statement">
describe("placeTip", () => {
  it("prefers above the anchor, centred on it", () => {
    const p = placeTip({ left: 400, top: 300, width: 100, height: 20 }, tip, viewport);
    expect(p.side).toBe("above");
    // 300 - 8 gap - 40 tall.
    expect(p.top).toBe(252);
    // 400 + 50 (centre) - 100 (half the tip).
    expect(p.left).toBe(350);
  });

  it("flips below when there is no room above", () => {
    const p = placeTip({ left: 400, top: 10, width: 100, height: 20 }, tip, viewport);
    expect(p.side).toBe("below");
    expect(p.top).toBe(38);
  });

  it("clamps to the right edge rather than leaving the window", () => {
    // A tab strip's rightmost tab: centring would put half the card offscreen,
    // which is the failure the OS tooltip never had.
    const p = placeTip({ left: 960, top: 300, width: 40, height: 20 }, tip, viewport);
    expect(p.left).toBe(viewport.width - tip.width - 6);
  });

  it("clamps to the left edge too", () => {
    const p = placeTip({ left: 0, top: 300, width: 20, height: 20 }, tip, viewport);
    expect(p.left).toBe(6);
  });

  it("keeps a tooltip with no room either way inside the viewport", () => {
    const tall = { width: 200, height: 700 };
    const p = placeTip({ left: 400, top: 400, width: 100, height: 20 }, tall, viewport);
    expect(p.top).toBeGreaterThanOrEqual(6);
    expect(p.top + tall.height).toBeLessThanOrEqual(viewport.height);
  });
});

</storycode:copy>
<a id="f-fb855cfd8048-ambient_declaration"></a>
### `ambient_declaration`

Read the sources off disk, the way styles.test.ts reads the stylesheet:
this invariant is about code that does not exist yet, so no amount of
rendering the current components can protect it.

<storycode:copy id="f-fb855cfd8048-ambient_declaration">
// Read the sources off disk, the way styles.test.ts reads the stylesheet:
// this invariant is about code that does not exist yet, so no amount of
// rendering the current components can protect it.
declare const require: (id: string) => {
  readFileSync(p: string, e: string): string;
  readdirSync(p: string, o: { withFileTypes: true }): { name: string; isDirectory(): boolean }[];
};
</storycode:copy>
<a id="f-fb855cfd8048-ambient_declaration-2"></a>
### `ambient_declaration`

<storycode:copy id="f-fb855cfd8048-ambient_declaration-2">
declare const process: { cwd(): string };

</storycode:copy>
<a id="f-fb855cfd8048-sources"></a>
### `sources`

<storycode:copy id="f-fb855cfd8048-sources">
function sources(dir: string, found: string[] = []): string[] {
  for (const entry of require("node:fs").readdirSync(dir, { withFileTypes: true })) {
    const path = `${dir}/${entry.name}`;
    if (entry.isDirectory()) sources(path, found);
    else if (/\.tsx?$/.test(entry.name) && !/\.test\.tsx?$/.test(entry.name)) found.push(path);
  }
  return found;
}

</storycode:copy>
<a id="f-fb855cfd8048-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-fb855cfd8048-expression_statement-2">
describe("no native tooltips", () => {
  it("leaves every hover hint to the themed layer", () => {
    // The OS draws `title=` in its own palette, which is never one of the
    // three in styles.css. See
    // docs/guarantees/authoring/every-hover-hint-is-drawn-in-the-app-theme.md
    const offenders = sources(`${process.cwd()}/src`)
      .filter((path) => !path.endsWith("components/TooltipLayer.tsx"))
      .filter((path) => {
        const src = require("node:fs").readFileSync(path, "utf8");
        // `document.title` is the window title, a different thing entirely.
        return /\btitle=["'{]/.test(src) || /(?<!document)\.title\s*=[^=]/.test(src);
      });
    expect(offenders, "use data-tip / .dataset.tip instead of title").toEqual([]);
  });
});

</storycode:copy>
<a id="f-fb855cfd8048-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-fb855cfd8048-expression_statement-3">
describe("tipTargetOf", () => {
  it("finds the tooltip owner from a child the pointer is actually over", () => {
    const button = document.createElement("button");
    button.setAttribute(TIP_ATTR, "Split right");
    const glyph = document.createElement("span");
    button.appendChild(glyph);
    expect(tipTargetOf(glyph)?.text).toBe("Split right");
    expect(tipTargetOf(glyph)?.el).toBe(button);
  });

  it("ignores an empty tooltip, so an absent message shows no empty card", () => {
    const el = document.createElement("span");
    el.setAttribute(TIP_ATTR, "  ");
    expect(tipTargetOf(el)).toBeNull();
    expect(tipTargetOf(document.createElement("div"))).toBeNull();
    expect(tipTargetOf(null)).toBeNull();
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/tooltip.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/tooltip.test.ts"><storycode:paste select="#f-fb855cfd8048-viewport"/><storycode:paste select="#f-fb855cfd8048-tip"/><storycode:paste select="#f-fb855cfd8048-expression_statement"/><storycode:paste select="#f-fb855cfd8048-ambient_declaration"/><storycode:paste select="#f-fb855cfd8048-ambient_declaration-2"/><storycode:paste select="#f-fb855cfd8048-sources"/><storycode:paste select="#f-fb855cfd8048-expression_statement-2"/><storycode:paste select="#f-fb855cfd8048-expression_statement-3"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/App.tsx](../../../mechanisms/apps/web/src/App.tsx.md#output-25cc6f821b46)
- [apps/web/src/components/TooltipLayer.tsx](../../../mechanisms/apps/web/src/components/TooltipLayer.tsx.md#output-7e5de871748f)
- [apps/web/src/debug/cmDebug.ts](../../../mechanisms/apps/web/src/debug/cmDebug.ts.md#output-9df55a31763b)
- [apps/web/src/editor/folding.ts](../../../mechanisms/apps/web/src/editor/folding.ts.md#output-eee9d1963829)
- [apps/web/src/editor/mathRender.ts](../../../mechanisms/apps/web/src/editor/mathRender.ts.md#output-b432fcb15a24)
- [apps/web/src/lib/tooltip.ts](../../../mechanisms/apps/web/src/lib/tooltip.ts.md#output-d5b928a2f7eb)
- [apps/web/src/styles.css](../../../mechanisms/apps/web/src/styles.css.md#output-8695905e2358)
- [apps/web/src/views/LandingView.tsx](../../../mechanisms/apps/web/src/views/LandingView.tsx.md#output-e86ad48a0990)

### Tests and fixtures

- [apps/web/src/components/TooltipLayer.test.tsx](../../agent/ui-action-dispatch/implementation/apps/web/src/components/TooltipLayer.test.tsx.md#output-4920bdec172e)
- [apps/web/src/lib/tooltip.test.ts](contract.md#output-fb855cfd8048)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
