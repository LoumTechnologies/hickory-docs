// Pure anchor math for the Split-view lineage ribbons. Everything here takes
// numbers and returns numbers/strings — DOM measurement (line rects, scroller
// bounds) happens in SplitView.tsx and feeds these functions, so the geometry
// is unit-testable without a browser.

export interface Band {
  yTop: number;
  yBot: number;
  /** True when the real anchor was (partly) outside the visible pane and the
   * band was clamped to the pane edge — rendered as a faded tail. */
  clamped: boolean;
}

/**
 * Clamp a vertical band [yTop, yBot] to the visible pane [viewTop, viewBot].
 * A band fully outside collapses to a thin sliver at the nearer edge so the
 * ribbon still points the right way.
 */
export function clampBand(
  yTop: number,
  yBot: number,
  viewTop: number,
  viewBot: number,
  sliver = 2,
): Band {
  if (yBot <= viewTop) return { yTop: viewTop, yBot: viewTop + sliver, clamped: true };
  if (yTop >= viewBot) return { yTop: viewBot - sliver, yBot: viewBot, clamped: true };
  let clamped = false;
  if (yTop < viewTop) {
    yTop = viewTop;
    clamped = true;
  }
  if (yBot > viewBot) {
    yBot = viewBot;
    clamped = true;
  }
  return { yTop, yBot, clamped };
}

/**
 * Ribbon thickness (px) for a fragment carrying `bytes` of the file's
 * `totalBytes`: proportional share of `maxPx`, floored at `minPx` so tiny
 * fragments stay visible.
 */
export function thicknessFor(
  bytes: number,
  totalBytes: number,
  maxPx = 26,
  minPx = 3,
): number {
  if (totalBytes <= 0 || bytes <= 0) return minPx;
  return Math.max(minPx, Math.min(maxPx, (bytes / totalBytes) * maxPx));
}

/** Center a band of thickness `t` on the middle of [yTop, yBot]. */
export function bandAround(yTop: number, yBot: number, t: number): { yTop: number; yBot: number } {
  const mid = (yTop + yBot) / 2;
  return { yTop: mid - t / 2, yBot: mid + t / 2 };
}

const fmt = (n: number) => (Math.round(n * 10) / 10).toString();

/**
 * A closed Sankey-style ribbon between two vertical bands: two cubic beziers
 * with horizontal tangents (left edge x0, right edge x1), flowing left→right.
 */
export function ribbonPath(
  x0: number,
  y0Top: number,
  y0Bot: number,
  x1: number,
  y1Top: number,
  y1Bot: number,
): string {
  const mx = (x0 + x1) / 2;
  return [
    `M ${fmt(x0)} ${fmt(y0Top)}`,
    `C ${fmt(mx)} ${fmt(y0Top)} ${fmt(mx)} ${fmt(y1Top)} ${fmt(x1)} ${fmt(y1Top)}`,
    `L ${fmt(x1)} ${fmt(y1Bot)}`,
    `C ${fmt(mx)} ${fmt(y1Bot)} ${fmt(mx)} ${fmt(y0Bot)} ${fmt(x0)} ${fmt(y0Bot)}`,
    "Z",
  ].join(" ");
}
