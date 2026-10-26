<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/ribbonStyle.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5fe1be43bb6f"></a>
## `apps/web/src/lib/ribbonStyle.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Word Navigation Is The Person's Choice](../../../../../guarantees/authoring/word-navigation-is-the-persons-choice/contract.md).

<a id="f-5fe1be43bb6f-RibbonStyle"></a>
### `RibbonStyle`

Which way the lineage connections are drawn: filled Sankey bands, or a
pair of curly braces joined by a thin line. A presentation preference, so
it persists per browser rather than per document — localStorage, with a
hard default of "braces" for anything unset or unrecognisable. A stored
"bands" remains valid: only the default flipped.

<storycode:copy id="f-5fe1be43bb6f-RibbonStyle">
// Which way the lineage connections are drawn: filled Sankey bands, or a
// pair of curly braces joined by a thin line. A presentation preference, so
// it persists per browser rather than per document — localStorage, with a
// hard default of "braces" for anything unset or unrecognisable. A stored
// "bands" remains valid: only the default flipped.

export type RibbonStyle = "bands" | "braces";

</storycode:copy>
<a id="f-5fe1be43bb6f-RIBBON_STYLE_KEY"></a>
### `RIBBON_STYLE_KEY`

<storycode:copy id="f-5fe1be43bb6f-RIBBON_STYLE_KEY">
export const RIBBON_STYLE_KEY = "hickory.ribbonStyle";

</storycode:copy>
<a id="f-5fe1be43bb6f-isStyle"></a>
### `isStyle`

<storycode:copy id="f-5fe1be43bb6f-isStyle">
const isStyle = (value: unknown): value is RibbonStyle =>
  value === "bands" || value === "braces";

</storycode:copy>
<a id="f-5fe1be43bb6f-loadRibbonStyle"></a>
### `loadRibbonStyle`

The persisted style, defaulting to "braces" on anything unset or invalid.

<storycode:copy id="f-5fe1be43bb6f-loadRibbonStyle">
/** The persisted style, defaulting to "braces" on anything unset or invalid. */
export function loadRibbonStyle(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): RibbonStyle {
  const stored = storage?.getItem(RIBBON_STYLE_KEY);
  return isStyle(stored) ? stored : "braces";
}

</storycode:copy>
<a id="f-5fe1be43bb6f-saveRibbonStyle"></a>
### `saveRibbonStyle`

<storycode:copy id="f-5fe1be43bb6f-saveRibbonStyle">
export function saveRibbonStyle(
  style: RibbonStyle,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): void {
  storage?.setItem(RIBBON_STYLE_KEY, style);
}
</storycode:copy>

### Assemble `apps/web/src/lib/ribbonStyle.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/ribbonStyle.ts"><storycode:paste select="#f-5fe1be43bb6f-RibbonStyle"/><storycode:paste select="#f-5fe1be43bb6f-RIBBON_STYLE_KEY"/><storycode:paste select="#f-5fe1be43bb6f-isStyle"/><storycode:paste select="#f-5fe1be43bb6f-loadRibbonStyle"/><storycode:paste select="#f-5fe1be43bb6f-saveRibbonStyle"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-986b3a14aa0f"></a>
## `apps/web/src/lib/ribbonStyle.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-986b3a14aa0f-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-986b3a14aa0f-expression_statement">
import { describe, expect, it } from "vitest";

import { loadRibbonStyle, RIBBON_STYLE_KEY, saveRibbonStyle } from "./ribbonStyle";

describe("ribbon style persistence", () => {
  it("defaults to braces with nothing stored", () => {
    localStorage.removeItem(RIBBON_STYLE_KEY);
    expect(loadRibbonStyle()).toBe("braces");
  });

  it("round-trips a saved choice through localStorage", () => {
    saveRibbonStyle("braces");
    expect(localStorage.getItem(RIBBON_STYLE_KEY)).toBe("braces");
    expect(loadRibbonStyle()).toBe("braces");
    // "bands" stays a valid persisted value — only the default changed.
    saveRibbonStyle("bands");
    expect(loadRibbonStyle()).toBe("bands");
  });

  it("falls back to braces on a value nothing ever wrote", () => {
    localStorage.setItem(RIBBON_STYLE_KEY, "sparkles");
    expect(loadRibbonStyle()).toBe("braces");
  });

  it("works without any storage at all", () => {
    expect(loadRibbonStyle(null)).toBe("braces");
    expect(() => saveRibbonStyle("braces", null)).not.toThrow();
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/ribbonStyle.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/ribbonStyle.test.ts"><storycode:paste select="#f-986b3a14aa0f-expression_statement"/></storycode:file>


</storycode:doc>
