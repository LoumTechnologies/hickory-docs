<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/channelWidth.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1bd88b546d8a"></a>
## `apps/web/src/lib/channelWidth.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Word Navigation Is The Person's Choice](../../../../../guarantees/authoring/word-navigation-is-the-persons-choice/contract.md).

<a id="f-1bd88b546d8a-CHANNEL_WIDTH_KEY"></a>
### `CHANNEL_WIDTH_KEY`

How wide the inter-pane channel is — the grid track between panes where
ribbons, braces, and their connecting lines live. A presentation
preference, so it persists per browser rather than per document —
localStorage, with a hard default and a clamped range so a bad stored
value can never squeeze the channel shut or blow the layout apart.
Mirrors lib/ribbonStyle.ts.

<storycode:copy id="f-1bd88b546d8a-CHANNEL_WIDTH_KEY">
// How wide the inter-pane channel is — the grid track between panes where
// ribbons, braces, and their connecting lines live. A presentation
// preference, so it persists per browser rather than per document —
// localStorage, with a hard default and a clamped range so a bad stored
// value can never squeeze the channel shut or blow the layout apart.
// Mirrors lib/ribbonStyle.ts.

export const CHANNEL_WIDTH_KEY = "hickory.channelWidth";
</storycode:copy>
<a id="f-1bd88b546d8a-CHANNEL_WIDTH_CHANGED"></a>
### `CHANNEL_WIDTH_CHANGED`

<storycode:copy id="f-1bd88b546d8a-CHANNEL_WIDTH_CHANGED">
export const CHANNEL_WIDTH_CHANGED = "hickory-channel-width-changed";

</storycode:copy>
<a id="f-1bd88b546d8a-CHANNEL_WIDTH_DEFAULT"></a>
### `CHANNEL_WIDTH_DEFAULT`

Default channel width, px. Wide enough for braces AND their links.

<storycode:copy id="f-1bd88b546d8a-CHANNEL_WIDTH_DEFAULT">
/** Default channel width, px. Wide enough for braces AND their links. */
export const CHANNEL_WIDTH_DEFAULT = 48;

</storycode:copy>
<a id="f-1bd88b546d8a-CHANNEL_WIDTH_MIN"></a>
### `CHANNEL_WIDTH_MIN`

<storycode:copy id="f-1bd88b546d8a-CHANNEL_WIDTH_MIN">
export const CHANNEL_WIDTH_MIN = 12;
</storycode:copy>
<a id="f-1bd88b546d8a-CHANNEL_WIDTH_MAX"></a>
### `CHANNEL_WIDTH_MAX`

<storycode:copy id="f-1bd88b546d8a-CHANNEL_WIDTH_MAX">
export const CHANNEL_WIDTH_MAX = 96;

</storycode:copy>
<a id="f-1bd88b546d8a-CHANNEL_WIDTH_PRESETS"></a>
### `CHANNEL_WIDTH_PRESETS`

The choices the toolbar offers. One control, three honest presets.

<storycode:copy id="f-1bd88b546d8a-CHANNEL_WIDTH_PRESETS">
/** The choices the toolbar offers. One control, three honest presets. */
export const CHANNEL_WIDTH_PRESETS = [
  { label: "Narrow", px: 24 },
  { label: "Normal", px: CHANNEL_WIDTH_DEFAULT },
  { label: "Wide", px: 72 },
] as const;

</storycode:copy>
<a id="f-1bd88b546d8a-clampChannelWidth"></a>
### `clampChannelWidth`

A candidate width, made safe: non-numbers fall back to the default,
numbers clamp into [MIN, MAX] and round to whole pixels (a grid track
wants integers; nobody can see half a pixel of channel).

<storycode:copy id="f-1bd88b546d8a-clampChannelWidth">
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

</storycode:copy>
<a id="f-1bd88b546d8a-loadChannelWidth"></a>
### `loadChannelWidth`

The persisted width, defaulting on anything unset or invalid.

<storycode:copy id="f-1bd88b546d8a-loadChannelWidth">
/** The persisted width, defaulting on anything unset or invalid. */
export function loadChannelWidth(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): number {
  const stored = storage?.getItem(CHANNEL_WIDTH_KEY);
  if (stored === null || stored === undefined || stored === "") return CHANNEL_WIDTH_DEFAULT;
  return clampChannelWidth(stored);
}

</storycode:copy>
<a id="f-1bd88b546d8a-saveChannelWidth"></a>
### `saveChannelWidth`

<storycode:copy id="f-1bd88b546d8a-saveChannelWidth">
export function saveChannelWidth(
  width: number,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): void {
  storage?.setItem(CHANNEL_WIDTH_KEY, String(clampChannelWidth(width)));
  if (typeof window !== "undefined") window.dispatchEvent(new Event(CHANNEL_WIDTH_CHANGED));
}
</storycode:copy>

### Assemble `apps/web/src/lib/channelWidth.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/channelWidth.ts"><storycode:paste select="#f-1bd88b546d8a-CHANNEL_WIDTH_KEY"/><storycode:paste select="#f-1bd88b546d8a-CHANNEL_WIDTH_CHANGED"/><storycode:paste select="#f-1bd88b546d8a-CHANNEL_WIDTH_DEFAULT"/><storycode:paste select="#f-1bd88b546d8a-CHANNEL_WIDTH_MIN"/><storycode:paste select="#f-1bd88b546d8a-CHANNEL_WIDTH_MAX"/><storycode:paste select="#f-1bd88b546d8a-CHANNEL_WIDTH_PRESETS"/><storycode:paste select="#f-1bd88b546d8a-clampChannelWidth"/><storycode:paste select="#f-1bd88b546d8a-loadChannelWidth"/><storycode:paste select="#f-1bd88b546d8a-saveChannelWidth"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-dcc0b16347a4"></a>
## `apps/web/src/lib/channelWidth.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-dcc0b16347a4-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-dcc0b16347a4-expression_statement">
import { describe, expect, it } from "vitest";

import {
  CHANNEL_WIDTH_DEFAULT,
  CHANNEL_WIDTH_KEY,
  CHANNEL_WIDTH_MAX,
  CHANNEL_WIDTH_MIN,
  CHANNEL_WIDTH_PRESETS,
  clampChannelWidth,
  loadChannelWidth,
  saveChannelWidth,
} from "./channelWidth";

describe("clampChannelWidth", () => {
  it("passes a sane width through, rounded to whole pixels", () => {
    expect(clampChannelWidth(48)).toBe(48);
    expect(clampChannelWidth(47.6)).toBe(48);
    expect(clampChannelWidth("72")).toBe(72);
  });

  it("clamps into the sane range", () => {
    expect(clampChannelWidth(0)).toBe(CHANNEL_WIDTH_MIN);
    expect(clampChannelWidth(-40)).toBe(CHANNEL_WIDTH_MIN);
    expect(clampChannelWidth(4000)).toBe(CHANNEL_WIDTH_MAX);
  });

  it("falls back to the default on anything that is not a number", () => {
    expect(clampChannelWidth("sparkles")).toBe(CHANNEL_WIDTH_DEFAULT);
    expect(clampChannelWidth(NaN)).toBe(CHANNEL_WIDTH_DEFAULT);
    expect(clampChannelWidth(Infinity)).toBe(CHANNEL_WIDTH_DEFAULT);
    expect(clampChannelWidth(undefined)).toBe(CHANNEL_WIDTH_DEFAULT);
    expect(clampChannelWidth(null)).toBe(CHANNEL_WIDTH_DEFAULT);
  });

  it("accepts every preset unchanged", () => {
    for (const preset of CHANNEL_WIDTH_PRESETS) {
      expect(clampChannelWidth(preset.px)).toBe(preset.px);
    }
  });
});

</storycode:copy>
<a id="f-dcc0b16347a4-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-dcc0b16347a4-expression_statement-2">
describe("channel width persistence", () => {
  it("defaults to 48px with nothing stored", () => {
    localStorage.removeItem(CHANNEL_WIDTH_KEY);
    expect(loadChannelWidth()).toBe(CHANNEL_WIDTH_DEFAULT);
    expect(CHANNEL_WIDTH_DEFAULT).toBe(48);
  });

  it("round-trips a saved width through localStorage", () => {
    saveChannelWidth(72);
    expect(localStorage.getItem(CHANNEL_WIDTH_KEY)).toBe("72");
    expect(loadChannelWidth()).toBe(72);
    saveChannelWidth(24);
    expect(loadChannelWidth()).toBe(24);
  });

  it("clamps on the way in AND on the way out", () => {
    saveChannelWidth(4000);
    expect(localStorage.getItem(CHANNEL_WIDTH_KEY)).toBe(String(CHANNEL_WIDTH_MAX));
    localStorage.setItem(CHANNEL_WIDTH_KEY, "3");
    expect(loadChannelWidth()).toBe(CHANNEL_WIDTH_MIN);
  });

  it("recovers from a value nothing ever wrote", () => {
    localStorage.setItem(CHANNEL_WIDTH_KEY, "wide");
    expect(loadChannelWidth()).toBe(CHANNEL_WIDTH_DEFAULT);
  });

  it("works without any storage at all", () => {
    expect(loadChannelWidth(null)).toBe(CHANNEL_WIDTH_DEFAULT);
    expect(() => saveChannelWidth(48, null)).not.toThrow();
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/channelWidth.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/channelWidth.test.ts"><storycode:paste select="#f-dcc0b16347a4-expression_statement"/><storycode:paste select="#f-dcc0b16347a4-expression_statement-2"/></storycode:file>


</storycode:doc>
