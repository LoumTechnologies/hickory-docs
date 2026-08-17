// Where a pane's tabs live: across the top (default), or down a left
// sidebar, grouped by folder. A presentation preference, so it persists per
// browser rather than per document — localStorage, with a hard default of
// "top" for anything unset or unrecognisable. Mirrors lib/ribbonStyle.ts.

export type TabStyle = "top" | "side";

export const TAB_STYLE_KEY = "hickory.tabStyle";

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
}
