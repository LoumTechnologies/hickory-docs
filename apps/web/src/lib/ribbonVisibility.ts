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

export const RIBBON_VISIBILITY_KEY = "hickory.ribbonVisibility";

const isVisibility = (value: unknown): value is RibbonVisibility =>
  value === "caret" || value === "always";

/** The persisted choice, defaulting to "caret" on anything unset or invalid. */
export function loadRibbonVisibility(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): RibbonVisibility {
  const stored = storage?.getItem(RIBBON_VISIBILITY_KEY);
  return isVisibility(stored) ? stored : "caret";
}

export function saveRibbonVisibility(
  visibility: RibbonVisibility,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): void {
  storage?.setItem(RIBBON_VISIBILITY_KEY, visibility);
}

/** A character range on one side of a connection. */
export interface Involved {
  from: number;
  to: number;
}

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
