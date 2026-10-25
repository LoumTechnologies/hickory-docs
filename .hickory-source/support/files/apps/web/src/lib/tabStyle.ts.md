<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/tabStyle.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Word Navigation Is The Person's Choice](../../../../../../guarantees/authoring/word-navigation-is-the-persons-choice/contract.md)

Captured SHA-256: `ee4a69506a0d03c141fa35c0a295eeefec1c655f57b8b6fe75c6b5619bb6ae93`.

<book:entry path="apps/web/src/lib/tabStyle.ts">
// Where a pane's tabs live: across the top (default), or down a left
// sidebar, grouped by folder. A presentation preference, so it persists per
// browser rather than per document — localStorage, with a hard default of
// "top" for anything unset or unrecognisable. Mirrors lib/ribbonStyle.ts.

export type TabStyle = "top" | "side";

export const TAB_STYLE_KEY = "hickory.tabStyle";
export const TAB_STYLE_CHANGED = "hickory-tab-style-changed";

const isStyle = (value: unknown): value is TabStyle => value === "top" || value === "side";

/** The persisted style, defaulting to "top" on anything unset or invalid. */
export function loadTabStyle(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): TabStyle {
  const stored = storage?.getItem(TAB_STYLE_KEY);
  return isStyle(stored) ? stored : "top";
}

export function saveTabStyle(
  style: TabStyle,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): void {
  storage?.setItem(TAB_STYLE_KEY, style);
  if (typeof window !== "undefined") window.dispatchEvent(new Event(TAB_STYLE_CHANGED));
}
</book:entry>

</book:doc>
