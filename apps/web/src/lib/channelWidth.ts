// How wide the inter-pane channel is — the grid track between panes where
// ribbons, braces, and their connecting lines live. A presentation
// preference, so it persists per browser rather than per document —
// localStorage, with a hard default and a clamped range so a bad stored
// value can never squeeze the channel shut or blow the layout apart.
// Mirrors lib/ribbonStyle.ts.

export const CHANNEL_WIDTH_KEY = "hickory.channelWidth";

/** Default channel width, px. Wide enough for braces AND their links. */
export const CHANNEL_WIDTH_DEFAULT = 48;

export const CHANNEL_WIDTH_MIN = 12;
export const CHANNEL_WIDTH_MAX = 96;

/** The choices the toolbar offers. One control, three honest presets. */
export const CHANNEL_WIDTH_PRESETS = [
  { label: "Narrow", px: 24 },
  { label: "Normal", px: CHANNEL_WIDTH_DEFAULT },
  { label: "Wide", px: 72 },
] as const;

/**
 * A candidate width, made safe: non-numbers fall back to the default,
 * numbers clamp into [MIN, MAX] and round to whole pixels (a grid track
 * wants integers; nobody can see half a pixel of channel).
 */
export function clampChannelWidth(value: unknown): number {
  const n = typeof value === "string" ? Number(value) : value;
  if (typeof n !== "number" || !Number.isFinite(n)) return CHANNEL_WIDTH_DEFAULT;
  return Math.round(Math.min(CHANNEL_WIDTH_MAX, Math.max(CHANNEL_WIDTH_MIN, n)));
}

/** The persisted width, defaulting on anything unset or invalid. */
export function loadChannelWidth(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): number {
  const stored = storage?.getItem(CHANNEL_WIDTH_KEY);
  if (stored === null || stored === undefined || stored === "") return CHANNEL_WIDTH_DEFAULT;
  return clampChannelWidth(stored);
}

export function saveChannelWidth(
  width: number,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): void {
  storage?.setItem(CHANNEL_WIDTH_KEY, String(clampChannelWidth(width)));
}
