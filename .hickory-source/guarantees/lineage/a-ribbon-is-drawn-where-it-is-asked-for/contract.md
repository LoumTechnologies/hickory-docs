<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Show the requested connections without changing their meaning

For: Authors editing generated text and reviewers tracing a result to its source.

Focus a source block, then inspect its connections in a folded or grouped view. Grouping must not invent new provenance.

## What must happen

Connections are drawn according to the current requested source/caret/fold visibility policy. Grouping joins eligible neighboring presentations without changing source identity, destination, attribution family or byte evidence. Intervening prose and distinct destinations or provenance families remain separate.

## Limits of the evidence

A grouped drawing is presentation, not a new claim of semantic support.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lineage/a-ribbon-is-drawn-where-it-is-asked-for.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the request-driven visibility rule and include the later coalescing requirement. Keep browser-appearance acceptance separate from grouping tests.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-f0a28dc54293"></a>
## `apps/web/src/lib/ribbonVisibility.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Show the requested connections without changing their meaning](contract.md).

<a id="f-f0a28dc54293-expression_statement"></a>
### `expression_statement`

Protects docs/guarantees/lineage/a-ribbon-is-drawn-where-it-is-asked-for.md

<storycode:copy id="f-f0a28dc54293-expression_statement">
// Protects docs/guarantees/lineage/a-ribbon-is-drawn-where-it-is-asked-for.md
import { describe, expect, it } from "vitest";

import {
  caretTouches,
  loadRibbonVisibility,
  RIBBON_VISIBILITY_KEY,
  saveRibbonVisibility,
} from "./ribbonVisibility";

describe("ribbon visibility persistence", () => {
  it("defaults to the caret with nothing stored", () => {
    localStorage.removeItem(RIBBON_VISIBILITY_KEY);
    expect(loadRibbonVisibility()).toBe("caret");
  });

  it("round-trips a saved choice through localStorage", () => {
    saveRibbonVisibility("always");
    expect(localStorage.getItem(RIBBON_VISIBILITY_KEY)).toBe("always");
    expect(loadRibbonVisibility()).toBe("always");
    saveRibbonVisibility("caret");
    expect(loadRibbonVisibility()).toBe("caret");
  });

  it("falls back to the caret on a value nothing ever wrote", () => {
    localStorage.setItem(RIBBON_VISIBILITY_KEY, "sometimes");
    expect(loadRibbonVisibility()).toBe("caret");
  });

  it("works without any storage at all", () => {
    expect(loadRibbonVisibility(null)).toBe("caret");
    expect(() => saveRibbonVisibility("always", null)).not.toThrow();
  });
});

</storycode:copy>
<a id="f-f0a28dc54293-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-f0a28dc54293-expression_statement-2">
describe("whether the caret is in an involved block", () => {
  const block = { from: 10, to: 20 };

  it("is in when the caret sits inside", () => {
    expect(caretTouches(block, { from: 15, to: 15 })).toBe(true);
  });

  it("counts both edges — a caret at the block's end is still in it", () => {
    expect(caretTouches(block, { from: 10, to: 10 })).toBe(true);
    expect(caretTouches(block, { from: 20, to: 20 })).toBe(true);
  });

  it("is out on either side", () => {
    expect(caretTouches(block, { from: 9, to: 9 })).toBe(false);
    expect(caretTouches(block, { from: 21, to: 21 })).toBe(false);
  });

  it("counts a selection that merely overlaps, in either direction", () => {
    expect(caretTouches(block, { from: 0, to: 12 })).toBe(true);
    expect(caretTouches(block, { from: 18, to: 40 })).toBe(true);
    // A backwards selection is the same selection.
    expect(caretTouches(block, { from: 40, to: 18 })).toBe(true);
    expect(caretTouches(block, { from: 30, to: 40 })).toBe(false);
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/ribbonVisibility.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/ribbonVisibility.test.ts"><storycode:paste select="#f-f0a28dc54293-expression_statement"/><storycode:paste select="#f-f0a28dc54293-expression_statement-2"/></storycode:file>

<a id="output-4b7840bd8111"></a>
## `apps/web/src/lib/ribbonVisibility.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Show the requested connections without changing their meaning](contract.md).

<a id="f-4b7840bd8111-RibbonVisibility"></a>
### `RibbonVisibility`

When the lineage connections are drawn at all.
A document with a dozen copies into a generated file draws a dozen braces,
all the time, over the text they are about. That is the right picture while
you are asking "where did this come from" and noise the rest of the time —
so the default answers the question only where it is being asked: the
connections whose blocks hold the CARET. Everything else stays measured,
keeps its hover reveal, and simply is not painted.
The old behaviour — every connection, always — is a setting rather than a
removal: reading a whole document's provenance at once is a real way to
read it, and it is the only way to see a relationship you did not already
suspect. A presentation preference, so it persists per browser rather than
per document. Mirrors lib/ribbonStyle.ts.

<storycode:copy id="f-4b7840bd8111-RibbonVisibility">
// When the lineage connections are drawn at all.
//
// A document with a dozen copies into a generated file draws a dozen braces,
// all the time, over the text they are about. That is the right picture while
// you are asking "where did this come from" and noise the rest of the time —
// so the default answers the question only where it is being asked: the
// connections whose blocks hold the CARET. Everything else stays measured,
// keeps its hover reveal, and simply is not painted.
//
// The old behaviour — every connection, always — is a setting rather than a
// removal: reading a whole document's provenance at once is a real way to
// read it, and it is the only way to see a relationship you did not already
// suspect. A presentation preference, so it persists per browser rather than
// per document. Mirrors lib/ribbonStyle.ts.

export type RibbonVisibility = "caret" | "always";

</storycode:copy>
<a id="f-4b7840bd8111-RIBBON_VISIBILITY_KEY"></a>
### `RIBBON_VISIBILITY_KEY`

<storycode:copy id="f-4b7840bd8111-RIBBON_VISIBILITY_KEY">
export const RIBBON_VISIBILITY_KEY = "hickory.ribbonVisibility";

</storycode:copy>
<a id="f-4b7840bd8111-isVisibility"></a>
### `isVisibility`

<storycode:copy id="f-4b7840bd8111-isVisibility">
const isVisibility = (value: unknown): value is RibbonVisibility =>
  value === "caret" || value === "always";

</storycode:copy>
<a id="f-4b7840bd8111-loadRibbonVisibility"></a>
### `loadRibbonVisibility`

The persisted choice, defaulting to "caret" on anything unset or invalid.

<storycode:copy id="f-4b7840bd8111-loadRibbonVisibility">
/** The persisted choice, defaulting to "caret" on anything unset or invalid. */
export function loadRibbonVisibility(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): RibbonVisibility {
  const stored = storage?.getItem(RIBBON_VISIBILITY_KEY);
  return isVisibility(stored) ? stored : "caret";
}

</storycode:copy>
<a id="f-4b7840bd8111-saveRibbonVisibility"></a>
### `saveRibbonVisibility`

<storycode:copy id="f-4b7840bd8111-saveRibbonVisibility">
export function saveRibbonVisibility(
  visibility: RibbonVisibility,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): void {
  storage?.setItem(RIBBON_VISIBILITY_KEY, visibility);
}

</storycode:copy>
<a id="f-4b7840bd8111-Involved"></a>
### `Involved`

A character range on one side of a connection.

<storycode:copy id="f-4b7840bd8111-Involved">
/** A character range on one side of a connection. */
export interface Involved {
  from: number;
  to: number;
}

</storycode:copy>
<a id="f-4b7840bd8111-caretTouches"></a>
### `caretTouches`

Whether a selection puts the caret inside an involved range.
OVERLAP, not containment: a selection that crosses the block is as much
"you are working here" as a bare caret in the middle of it, and an empty
selection is the caret. The range's own end counts — a caret at the last
character of a copy block is still in that block, and the block's `to` is
the position after it.

<storycode:copy id="f-4b7840bd8111-caretTouches">
/**
 * Whether a selection puts the caret inside an involved range.
 *
 * OVERLAP, not containment: a selection that crosses the block is as much
 * "you are working here" as a bare caret in the middle of it, and an empty
 * selection is the caret. The range's own end counts — a caret at the last
 * character of a copy block is still in that block, and the block's `to` is
 * the position after it.
 */
export function caretTouches(range: Involved, selection: Involved): boolean {
  const from = Math.min(selection.from, selection.to);
  const to = Math.max(selection.from, selection.to);
  return from <= range.to && to >= range.from;
}
</storycode:copy>

### Assemble `apps/web/src/lib/ribbonVisibility.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/ribbonVisibility.ts"><storycode:paste select="#f-4b7840bd8111-RibbonVisibility"/><storycode:paste select="#f-4b7840bd8111-RIBBON_VISIBILITY_KEY"/><storycode:paste select="#f-4b7840bd8111-isVisibility"/><storycode:paste select="#f-4b7840bd8111-loadRibbonVisibility"/><storycode:paste select="#f-4b7840bd8111-saveRibbonVisibility"/><storycode:paste select="#f-4b7840bd8111-Involved"/><storycode:paste select="#f-4b7840bd8111-caretTouches"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/lib/coalesceRibbons.ts](../../../mechanisms/apps/web/src/lib/coalesceRibbons.ts.md#output-8e259ab2001c)
- [apps/web/src/lib/ribbonVisibility.ts](contract.md#output-4b7840bd8111)
- [apps/web/src/shell/Ribbons.tsx](../../../mechanisms/apps/web/src/shell/Ribbons.tsx.md#output-b916e85d3b0b)
- [apps/web/src/shell/coalesceLinks.ts](../../../mechanisms/apps/web/src/shell/coalesceLinks.ts.md#output-f87ad5795296)
- [apps/web/src/shell/linkShapes.ts](../../../mechanisms/apps/web/src/shell/linkShapes.ts.md#output-5b6df5b99815)
- [apps/web/src/views/SettingsView.tsx](../../../mechanisms/apps/web/src/views/SettingsView.tsx.md#output-38894484eb8f)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)

### Tests and fixtures

- [apps/web/src/lib/ribbonVisibility.test.ts](contract.md#output-f0a28dc54293)
- [apps/web/src/views/SettingsView.test.tsx](../../../mechanisms/apps/web/src/views/SettingsView.tsx.md#output-5bfac230ae5d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
