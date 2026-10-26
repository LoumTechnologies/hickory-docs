<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/tabStyle.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0732de414ed2"></a>
## `apps/web/src/lib/tabStyle.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Word Navigation Is The Person's Choice](../../../../../guarantees/authoring/word-navigation-is-the-persons-choice/contract.md).

<a id="f-0732de414ed2-TabStyle"></a>
### `TabStyle`

Where a pane's tabs live: across the top (default), or down a left
sidebar, grouped by folder. A presentation preference, so it persists per
browser rather than per document — localStorage, with a hard default of
"top" for anything unset or unrecognisable. Mirrors lib/ribbonStyle.ts.

<storycode:copy id="f-0732de414ed2-TabStyle">
// Where a pane's tabs live: across the top (default), or down a left
// sidebar, grouped by folder. A presentation preference, so it persists per
// browser rather than per document — localStorage, with a hard default of
// "top" for anything unset or unrecognisable. Mirrors lib/ribbonStyle.ts.

export type TabStyle = "top" | "side";

</storycode:copy>
<a id="f-0732de414ed2-TAB_STYLE_KEY"></a>
### `TAB_STYLE_KEY`

<storycode:copy id="f-0732de414ed2-TAB_STYLE_KEY">
export const TAB_STYLE_KEY = "hickory.tabStyle";
</storycode:copy>
<a id="f-0732de414ed2-TAB_STYLE_CHANGED"></a>
### `TAB_STYLE_CHANGED`

<storycode:copy id="f-0732de414ed2-TAB_STYLE_CHANGED">
export const TAB_STYLE_CHANGED = "hickory-tab-style-changed";

</storycode:copy>
<a id="f-0732de414ed2-isStyle"></a>
### `isStyle`

<storycode:copy id="f-0732de414ed2-isStyle">
const isStyle = (value: unknown): value is TabStyle => value === "top" || value === "side";

</storycode:copy>
<a id="f-0732de414ed2-loadTabStyle"></a>
### `loadTabStyle`

The persisted style, defaulting to "top" on anything unset or invalid.

<storycode:copy id="f-0732de414ed2-loadTabStyle">
/** The persisted style, defaulting to "top" on anything unset or invalid. */
export function loadTabStyle(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): TabStyle {
  const stored = storage?.getItem(TAB_STYLE_KEY);
  return isStyle(stored) ? stored : "top";
}

</storycode:copy>
<a id="f-0732de414ed2-saveTabStyle"></a>
### `saveTabStyle`

<storycode:copy id="f-0732de414ed2-saveTabStyle">
export function saveTabStyle(
  style: TabStyle,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): void {
  storage?.setItem(TAB_STYLE_KEY, style);
  if (typeof window !== "undefined") window.dispatchEvent(new Event(TAB_STYLE_CHANGED));
}
</storycode:copy>

### Assemble `apps/web/src/lib/tabStyle.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/tabStyle.ts"><storycode:paste select="#f-0732de414ed2-TabStyle"/><storycode:paste select="#f-0732de414ed2-TAB_STYLE_KEY"/><storycode:paste select="#f-0732de414ed2-TAB_STYLE_CHANGED"/><storycode:paste select="#f-0732de414ed2-isStyle"/><storycode:paste select="#f-0732de414ed2-loadTabStyle"/><storycode:paste select="#f-0732de414ed2-saveTabStyle"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-f293732f35bd"></a>
## `apps/web/src/lib/tabStyle.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-f293732f35bd-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-f293732f35bd-expression_statement">
import { describe, expect, it } from "vitest";

import { loadTabStyle, saveTabStyle, TAB_STYLE_KEY } from "./tabStyle";

describe("tab style persistence", () => {
  it("defaults to top with nothing stored", () => {
    localStorage.removeItem(TAB_STYLE_KEY);
    expect(loadTabStyle()).toBe("top");
  });

  it("round-trips a saved choice through localStorage", () => {
    saveTabStyle("side");
    expect(localStorage.getItem(TAB_STYLE_KEY)).toBe("side");
    expect(loadTabStyle()).toBe("side");
    saveTabStyle("top");
    expect(loadTabStyle()).toBe("top");
  });

  it("falls back to top on a value nothing ever wrote", () => {
    localStorage.setItem(TAB_STYLE_KEY, "diagonal");
    expect(loadTabStyle()).toBe("top");
  });

  it("works without any storage at all", () => {
    expect(loadTabStyle(null)).toBe("top");
    expect(() => saveTabStyle("side", null)).not.toThrow();
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/tabStyle.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/tabStyle.test.ts"><storycode:paste select="#f-f293732f35bd-expression_statement"/></storycode:file>


</storycode:doc>
