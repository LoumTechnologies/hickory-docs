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

/** Actual size. */
export const ZOOM_DEFAULT = 1;

/** Which scope a zoom command acts on. */
export type ZoomScope = "ui" | "tab";

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

/** One step up the ladder, stopping at the top rather than wrapping. */
export function zoomIn(value: number): number {
  const i = (ZOOM_STEPS as readonly number[]).indexOf(clampZoom(value));
  return ZOOM_STEPS[Math.min(i + 1, ZOOM_STEPS.length - 1)];
}

/** One step down, stopping at the bottom. */
export function zoomOut(value: number): number {
  const i = (ZOOM_STEPS as readonly number[]).indexOf(clampZoom(value));
  return ZOOM_STEPS[Math.max(i - 1, 0)];
}

/** How a zoom level is shown to a person. */
export function zoomLabel(value: number): string {
  return `${Math.round(clampZoom(value) * 100)}%`;
}

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

/** Apply a step to a level. */
export function applyZoom(value: number, command: "in" | "out" | "reset"): number {
  if (command === "reset") return ZOOM_DEFAULT;
  return command === "in" ? zoomIn(value) : zoomOut(value);
}

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

/** The custom property the whole-UI level is applied through. */
export const UI_ZOOM_VAR = "--ui-zoom";
/** The custom property one pane's level is applied through. */
export const TAB_ZOOM_VAR = "--tab-zoom";

export function loadUiZoom(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): number {
  const stored = storage?.getItem(UI_ZOOM_KEY);
  if (stored === null || stored === undefined || stored === "") return ZOOM_DEFAULT;
  return clampZoom(Number(stored));
}

export function saveUiZoom(
  value: number,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): void {
  storage?.setItem(UI_ZOOM_KEY, String(clampZoom(value)));
}

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
