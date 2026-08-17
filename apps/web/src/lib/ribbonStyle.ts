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
