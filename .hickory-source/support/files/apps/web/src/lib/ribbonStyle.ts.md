<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/ribbonStyle.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Word Navigation Is The Person's Choice](../../../../../../guarantees/authoring/word-navigation-is-the-persons-choice/contract.md)

Captured SHA-256: `087cfed8981f4960993ec7a0dc2f95b65266adbbd1a182c989dec876dec02460`.

<book:entry path="apps/web/src/lib/ribbonStyle.ts">
// Which way the lineage connections are drawn: filled Sankey bands, or a
// pair of curly braces joined by a thin line. A presentation preference, so
// it persists per browser rather than per document — localStorage, with a
// hard default of "braces" for anything unset or unrecognisable. A stored
// "bands" remains valid: only the default flipped.

export type RibbonStyle = "bands" | "braces";

export const RIBBON_STYLE_KEY = "hickory.ribbonStyle";

const isStyle = (value: unknown): value is RibbonStyle =>
  value === "bands" || value === "braces";

/** The persisted style, defaulting to "braces" on anything unset or invalid. */
export function loadRibbonStyle(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): RibbonStyle {
  const stored = storage?.getItem(RIBBON_STYLE_KEY);
  return isStyle(stored) ? stored : "braces";
}

export function saveRibbonStyle(
  style: RibbonStyle,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): void {
  storage?.setItem(RIBBON_STYLE_KEY, style);
}
</book:entry>

</book:doc>
