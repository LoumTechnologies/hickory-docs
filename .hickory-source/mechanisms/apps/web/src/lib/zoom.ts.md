<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/zoom.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f4db122cd56f"></a>
## `apps/web/src/lib/zoom.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-f4db122cd56f-ZOOM_STEPS"></a>
### `ZOOM_STEPS`

Zoom, at two scopes, because the two are genuinely different requests.
- **The whole UI.** "This laptop's screen is too small / I am presenting /
my eyes are tired." Everything grows: tabs, the tree, the toolbars, the
text. This is what a browser's ⌘+ does and what people mean by "zoom"
nine times in ten.
- **One tab.** "This one file is dense" — a table, a wide diagram, a log —
"and I do not want the rest of the window to move." Only that pane's
content changes size; the furniture around it stays put, which is the
whole point, because moving the furniture is what makes UI zoom
disorienting for a single file.
The steps are a fixed ladder rather than a percentage the caller adds to.
Two reasons: repeated multiplication by 1.1 lands on 1.0000000002 and the
"actual size" check stops working, and a ladder makes every zoom level
reachable by the same number of presses on every machine.
Pure. What holds a zoom level — the root element, a pane, an xterm's font
option — is the caller's problem; this file only answers "what is the next
one up from here?".
The ladder, smallest first. 1 is actual size and must be on it.

<storycode:copy id="f-f4db122cd56f-ZOOM_STEPS">
// Zoom, at two scopes, because the two are genuinely different requests.
//
//  - **The whole UI.** "This laptop's screen is too small / I am presenting /
//    my eyes are tired." Everything grows: tabs, the tree, the toolbars, the
//    text. This is what a browser's ⌘+ does and what people mean by "zoom"
//    nine times in ten.
//  - **One tab.** "This one file is dense" — a table, a wide diagram, a log —
//    "and I do not want the rest of the window to move." Only that pane's
//    content changes size; the furniture around it stays put, which is the
//    whole point, because moving the furniture is what makes UI zoom
//    disorienting for a single file.
//
// The steps are a fixed ladder rather than a percentage the caller adds to.
// Two reasons: repeated multiplication by 1.1 lands on 1.0000000002 and the
// "actual size" check stops working, and a ladder makes every zoom level
// reachable by the same number of presses on every machine.
//
// Pure. What holds a zoom level — the root element, a pane, an xterm's font
// option — is the caller's problem; this file only answers "what is the next
// one up from here?".

/** The ladder, smallest first. 1 is actual size and must be on it. */
export const ZOOM_STEPS = [
  0.5, 0.67, 0.8, 0.9, 1, 1.1, 1.25, 1.5, 1.75, 2, 2.5, 3,
] as const;

</storycode:copy>
<a id="f-f4db122cd56f-ZOOM_DEFAULT"></a>
### `ZOOM_DEFAULT`

<storycode:copy id="f-f4db122cd56f-ZOOM_DEFAULT">
/** Actual size. */
export const ZOOM_DEFAULT = 1;

</storycode:copy>
<a id="f-f4db122cd56f-ZoomScope"></a>
### `ZoomScope`

<storycode:copy id="f-f4db122cd56f-ZoomScope">
/** Which scope a zoom command acts on. */
export type ZoomScope = "ui" | "tab";

</storycode:copy>
<a id="f-f4db122cd56f-clampZoom"></a>
### `clampZoom`

The nearest level on the ladder — what a stored or hand-edited value
becomes before anything lays out with it.

<storycode:copy id="f-f4db122cd56f-clampZoom">
/** The nearest level on the ladder — what a stored or hand-edited value
 * becomes before anything lays out with it. */
export function clampZoom(value: unknown): number {
  const n = typeof value === "number" ? value : Number.NaN;
  if (!Number.isFinite(n)) return ZOOM_DEFAULT;
  let best: number = ZOOM_STEPS[0];
  for (const step of ZOOM_STEPS) {
    if (Math.abs(step - n) < Math.abs(best - n)) best = step;
  }
  return best;
}

</storycode:copy>
<a id="f-f4db122cd56f-zoomIn"></a>
### `zoomIn`

One step up the ladder, stopping at the top rather than wrapping.

<storycode:copy id="f-f4db122cd56f-zoomIn">
/** One step up the ladder, stopping at the top rather than wrapping. */
export function zoomIn(value: number): number {
  const i = (ZOOM_STEPS as readonly number[]).indexOf(clampZoom(value));
  return ZOOM_STEPS[Math.min(i + 1, ZOOM_STEPS.length - 1)];
}

</storycode:copy>
<a id="f-f4db122cd56f-zoomOut"></a>
### `zoomOut`

<storycode:copy id="f-f4db122cd56f-zoomOut">
/** One step down, stopping at the bottom. */
export function zoomOut(value: number): number {
  const i = (ZOOM_STEPS as readonly number[]).indexOf(clampZoom(value));
  return ZOOM_STEPS[Math.max(i - 1, 0)];
}

</storycode:copy>
<a id="f-f4db122cd56f-zoomLabel"></a>
### `zoomLabel`

<storycode:copy id="f-f4db122cd56f-zoomLabel">
/** How a zoom level is shown to a person. */
export function zoomLabel(value: number): string {
  return `${Math.round(clampZoom(value) * 100)}%`;
}

</storycode:copy>
<a id="f-f4db122cd56f-zoomCommandFor"></a>
### `zoomCommandFor`

The keyboard's answer, or null when the event is not a zoom command.
The key names are the mess they are because a keyboard's `+` is `=` with
Shift on a US layout and its own key elsewhere, and because engines
disagree about which one they report for the numpad. Matching all of them
is not defensive coding, it is the only way ⌘+ works on a German keyboard.

<storycode:copy id="f-f4db122cd56f-zoomCommandFor">
/**
 * The keyboard's answer, or null when the event is not a zoom command.
 *
 * The key names are the mess they are because a keyboard's `+` is `=` with
 * Shift on a US layout and its own key elsewhere, and because engines
 * disagree about which one they report for the numpad. Matching all of them
 * is not defensive coding, it is the only way ⌘+ works on a German keyboard.
 */
export function zoomCommandFor(event: {
  key: string;
  metaKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
}): "in" | "out" | "reset" | null {
  if (!(event.metaKey || event.ctrlKey)) return null;
  // Alt is the modifier that switches SCOPE (see useZoom); it must not stop
  // the command being recognised.
  switch (event.key) {
    case "=":
    case "+":
      return "in";
    case "-":
    case "_":
      return "out";
    case "0":
      return "reset";
    default:
      return null;
  }
}

</storycode:copy>
<a id="f-f4db122cd56f-applyZoom"></a>
### `applyZoom`

<storycode:copy id="f-f4db122cd56f-applyZoom">
/** Apply a step to a level. */
export function applyZoom(value: number, command: "in" | "out" | "reset"): number {
  if (command === "reset") return ZOOM_DEFAULT;
  return command === "in" ? zoomIn(value) : zoomOut(value);
}

</storycode:copy>
<a id="f-f4db122cd56f-UI_ZOOM_KEY"></a>
### `UI_ZOOM_KEY`

Where each scope's level is kept
The two scopes persist in two places, and the split is the point rather
than an accident:
- **UI zoom is a display preference of this MACHINE**, like the theme and
the tab style. It follows the screen and the eyes in front of it, not
the project, so it lives in localStorage beside the other appearance
preferences (lib/channelWidth.ts is the pattern).
- **Tab zoom belongs to the TAB**, so it is workspace state and travels
with the arrangement it was set in — see lib/uiState.ts.

<storycode:copy id="f-f4db122cd56f-UI_ZOOM_KEY">
// ---------------------------------------------------------------------------
// Where each scope's level is kept
// ---------------------------------------------------------------------------
//
// The two scopes persist in two places, and the split is the point rather
// than an accident:
//
//  - **UI zoom is a display preference of this MACHINE**, like the theme and
//    the tab style. It follows the screen and the eyes in front of it, not
//    the project, so it lives in localStorage beside the other appearance
//    preferences (lib/channelWidth.ts is the pattern).
//  - **Tab zoom belongs to the TAB**, so it is workspace state and travels
//    with the arrangement it was set in — see lib/uiState.ts.

export const UI_ZOOM_KEY = "hickory.uiZoom";

</storycode:copy>
<a id="f-f4db122cd56f-UI_ZOOM_VAR"></a>
### `UI_ZOOM_VAR`

The custom property the whole-UI level is applied through.

<storycode:copy id="f-f4db122cd56f-UI_ZOOM_VAR">
/** The custom property the whole-UI level is applied through. */
export const UI_ZOOM_VAR = "--ui-zoom";
</storycode:copy>
<a id="f-f4db122cd56f-TAB_ZOOM_VAR"></a>
### `TAB_ZOOM_VAR`

The custom property one pane's level is applied through.

<storycode:copy id="f-f4db122cd56f-TAB_ZOOM_VAR">
/** The custom property one pane's level is applied through. */
export const TAB_ZOOM_VAR = "--tab-zoom";

</storycode:copy>
<a id="f-f4db122cd56f-loadUiZoom"></a>
### `loadUiZoom`

<storycode:copy id="f-f4db122cd56f-loadUiZoom">
export function loadUiZoom(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): number {
  const stored = storage?.getItem(UI_ZOOM_KEY);
  if (stored === null || stored === undefined || stored === "") return ZOOM_DEFAULT;
  return clampZoom(Number(stored));
}

</storycode:copy>
<a id="f-f4db122cd56f-saveUiZoom"></a>
### `saveUiZoom`

<storycode:copy id="f-f4db122cd56f-saveUiZoom">
export function saveUiZoom(
  value: number,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): void {
  storage?.setItem(UI_ZOOM_KEY, String(clampZoom(value)));
}

</storycode:copy>
<a id="f-f4db122cd56f-applyUiZoom"></a>
### `applyUiZoom`

Put the whole-UI level on the document.
Through the root's FONT SIZE, not a transform. A CSS transform would scale
the rendered pixels — blurry text, hit targets that no longer line up with
what is drawn, and a scrollbar that belongs to the untransformed box. This
app sizes in `rem` almost throughout, so moving the root's font size
re-lays-out at the new size, which is what zoom is supposed to mean.

<storycode:copy id="f-f4db122cd56f-applyUiZoom">
/**
 * Put the whole-UI level on the document.
 *
 * Through the root's FONT SIZE, not a transform. A CSS transform would scale
 * the rendered pixels — blurry text, hit targets that no longer line up with
 * what is drawn, and a scrollbar that belongs to the untransformed box. This
 * app sizes in `rem` almost throughout, so moving the root's font size
 * re-lays-out at the new size, which is what zoom is supposed to mean.
 */
export function applyUiZoom(value: number, root: HTMLElement | null = document.documentElement): void {
  const level = clampZoom(value);
  root?.style.setProperty(UI_ZOOM_VAR, String(level));
}
</storycode:copy>

### Assemble `apps/web/src/lib/zoom.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/zoom.ts"><storycode:paste select="#f-f4db122cd56f-ZOOM_STEPS"/><storycode:paste select="#f-f4db122cd56f-ZOOM_DEFAULT"/><storycode:paste select="#f-f4db122cd56f-ZoomScope"/><storycode:paste select="#f-f4db122cd56f-clampZoom"/><storycode:paste select="#f-f4db122cd56f-zoomIn"/><storycode:paste select="#f-f4db122cd56f-zoomOut"/><storycode:paste select="#f-f4db122cd56f-zoomLabel"/><storycode:paste select="#f-f4db122cd56f-zoomCommandFor"/><storycode:paste select="#f-f4db122cd56f-applyZoom"/><storycode:paste select="#f-f4db122cd56f-UI_ZOOM_KEY"/><storycode:paste select="#f-f4db122cd56f-UI_ZOOM_VAR"/><storycode:paste select="#f-f4db122cd56f-TAB_ZOOM_VAR"/><storycode:paste select="#f-f4db122cd56f-loadUiZoom"/><storycode:paste select="#f-f4db122cd56f-saveUiZoom"/><storycode:paste select="#f-f4db122cd56f-applyUiZoom"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-74a00f76a18a"></a>
## `apps/web/src/lib/zoom.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-74a00f76a18a-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-74a00f76a18a-expression_statement">
import { describe, expect, it } from "vitest";
import {
  ZOOM_DEFAULT,
  ZOOM_STEPS,
  applyZoom,
  clampZoom,
  zoomCommandFor,
  zoomIn,
  zoomLabel,
  zoomOut,
} from "./zoom";

describe("the zoom ladder", () => {
  it("has actual size on it", () => {
    // Otherwise "reset" lands between two steps and every later step is off
    // by that amount.
    expect(ZOOM_STEPS).toContain(ZOOM_DEFAULT);
  });

  it("steps up and down without drifting", () => {
    // The reason this is a ladder rather than repeated ×1.1: multiplication
    // lands on 1.0000000002 and the actual-size check quietly stops working.
    let level: number = ZOOM_DEFAULT;
    for (let i = 0; i < 4; i++) level = zoomIn(level);
    for (let i = 0; i < 4; i++) level = zoomOut(level);
    expect(level).toBe(ZOOM_DEFAULT);
  });

  it("stops at the ends rather than wrapping round", () => {
    expect(zoomOut(ZOOM_STEPS[0])).toBe(ZOOM_STEPS[0]);
    expect(zoomIn(ZOOM_STEPS[ZOOM_STEPS.length - 1])).toBe(ZOOM_STEPS[ZOOM_STEPS.length - 1]);
  });

  it("snaps a stored or hand-edited value onto the ladder", () => {
    expect(clampZoom(1.12)).toBe(1.1);
    expect(clampZoom(99)).toBe(3);
    expect(clampZoom(0)).toBe(0.5);
    expect(clampZoom("big")).toBe(ZOOM_DEFAULT);
    expect(clampZoom(undefined)).toBe(ZOOM_DEFAULT);
  });

  it("says a level the way a person reads one", () => {
    expect(zoomLabel(1)).toBe("100%");
    expect(zoomLabel(1.25)).toBe("125%");
    expect(zoomLabel(0.67)).toBe("67%");
  });
});

</storycode:copy>
<a id="f-74a00f76a18a-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-74a00f76a18a-expression_statement-2">
describe("the zoom keys", () => {
  const press = (key: string, mods: Partial<{ metaKey: boolean; ctrlKey: boolean; altKey: boolean }> = {}) =>
    zoomCommandFor({ key, metaKey: false, ctrlKey: false, altKey: false, ...mods });

  it("needs a command modifier", () => {
    expect(press("=")).toBeNull();
    expect(press("=", { metaKey: true })).toBe("in");
    expect(press("=", { ctrlKey: true })).toBe("in");
  });

  it("accepts every spelling of plus and minus a keyboard produces", () => {
    // `+` is `=` with Shift on a US layout and its own key elsewhere, and
    // engines disagree about the numpad. This is not defensive coding — it is
    // the only way ⌘+ works on a German keyboard.
    for (const key of ["=", "+"]) expect(press(key, { metaKey: true })).toBe("in");
    for (const key of ["-", "_"]) expect(press(key, { metaKey: true })).toBe("out");
  });

  it("reads zero as actual size", () => {
    expect(press("0", { metaKey: true })).toBe("reset");
  });

  it("still recognises the command with Alt held, which only picks the scope", () => {
    expect(press("=", { metaKey: true, altKey: true })).toBe("in");
  });

  it("ignores everything else", () => {
    expect(press("k", { metaKey: true })).toBeNull();
  });
});

</storycode:copy>
<a id="f-74a00f76a18a-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-74a00f76a18a-expression_statement-3">
describe("applying a command", () => {
  it("resets to actual size from anywhere", () => {
    expect(applyZoom(2.5, "reset")).toBe(ZOOM_DEFAULT);
    expect(applyZoom(0.5, "reset")).toBe(ZOOM_DEFAULT);
  });

  it("moves one rung", () => {
    expect(applyZoom(1, "in")).toBe(1.1);
    expect(applyZoom(1, "out")).toBe(0.9);
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/zoom.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/zoom.test.ts"><storycode:paste select="#f-74a00f76a18a-expression_statement"/><storycode:paste select="#f-74a00f76a18a-expression_statement-2"/><storycode:paste select="#f-74a00f76a18a-expression_statement-3"/></storycode:file>


</storycode:doc>
