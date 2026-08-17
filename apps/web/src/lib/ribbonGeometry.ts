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
  /** The band's real top was above the pane and got pulled down to the edge:
   * the range continues past the top of what you see. Brace mode draws that
   * end OPEN — no horn curl — so the spine visibly runs off the edge. */
  clampedTop: boolean;
  /** Same for the bottom edge. */
  clampedBottom: boolean;
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
  /** Stagger for bands clamped to the same edge: nth off-screen anchor sits
   * `n * step` px along the edge, so several of them fan out in document
   * order instead of stacking into one invisible line. */
  stagger = 0,
): Band {
  const step = 7;
  const spread = Math.min(stagger * step, 56);
  if (yBot <= viewTop) {
    // Fully above: the whole range continues past the TOP edge, so that is
    // the open end — the sliver merely marks where to look.
    const top = viewTop + spread;
    return { yTop: top, yBot: top + sliver, clamped: true, clampedTop: true, clampedBottom: false };
  }
  if (yTop >= viewBot) {
    const bot = viewBot - spread;
    return { yTop: bot - sliver, yBot: bot, clamped: true, clampedTop: false, clampedBottom: true };
  }
  let clampedTop = false;
  let clampedBottom = false;
  if (yTop < viewTop) {
    yTop = viewTop;
    clampedTop = true;
  }
  if (yBot > viewBot) {
    yBot = viewBot;
    clampedBottom = true;
  }
  return { yTop, yBot, clamped: clampedTop || clampedBottom, clampedTop, clampedBottom };
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

/**
 * Give a band a minimum visible thickness, growing it around its midpoint.
 * A block whose source span is a single short line still needs to be grabbable.
 */
export function atLeast(yTop: number, yBot: number, minPx = 4): { yTop: number; yBot: number } {
  const h = yBot - yTop;
  if (h >= minPx) return { yTop, yBot };
  return bandAround(yTop, yBot, minPx);
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

/**
 * A ribbon routed THROUGH a waypoint band — the file node in the middle
 * column. Two Sankey segments (source→node, node→output) joined across the
 * node's own width, so the eye follows content into the file it lands in and
 * out again into the generated text.
 */
export function ribbonPathVia(
  x0: number,
  y0Top: number,
  y0Bot: number,
  nodeLeft: number,
  nodeRight: number,
  nTop: number,
  nBot: number,
  x1: number,
  y1Top: number,
  y1Bot: number,
): string {
  const m0 = (x0 + nodeLeft) / 2;
  const m1 = (nodeRight + x1) / 2;
  return [
    `M ${fmt(x0)} ${fmt(y0Top)}`,
    `C ${fmt(m0)} ${fmt(y0Top)} ${fmt(m0)} ${fmt(nTop)} ${fmt(nodeLeft)} ${fmt(nTop)}`,
    `L ${fmt(nodeRight)} ${fmt(nTop)}`,
    `C ${fmt(m1)} ${fmt(nTop)} ${fmt(m1)} ${fmt(y1Top)} ${fmt(x1)} ${fmt(y1Top)}`,
    `L ${fmt(x1)} ${fmt(y1Bot)}`,
    `C ${fmt(m1)} ${fmt(y1Bot)} ${fmt(m1)} ${fmt(nBot)} ${fmt(nodeRight)} ${fmt(nBot)}`,
    `L ${fmt(nodeLeft)} ${fmt(nBot)}`,
    `C ${fmt(m0)} ${fmt(nBot)} ${fmt(m0)} ${fmt(y0Bot)} ${fmt(x0)} ${fmt(y0Bot)}`,
    "Z",
  ].join(" ");
}

/**
 * Which horizontal edge of a terminal (a tab connector or a divider port
 * button) a ribbon should attach to: a source band whose midpoint sits above
 * the terminal's centre lands on the TOP edge, one below lands on the BOTTOM.
 *
 * Terminals live in chrome — a tab strip or a thin vertical divider — so a
 * ribbon always arrives on a horizontal edge: attaching to the side of a
 * button inside a 4px divider would give the ribbon no width to land on.
 */
export function terminalEdge(
  yTop: number,
  yBot: number,
  terminalTop: number,
  terminalBot: number,
): "top" | "bottom" {
  return (yTop + yBot) / 2 <= (terminalTop + terminalBot) / 2 ? "top" : "bottom";
}

/**
 * A closed ribbon from a vertical band to a HORIZONTAL edge — the underside
 * of a tab, or the top/bottom of a divider port button. Two elbow beziers:
 * horizontal tangent at the source, vertical tangent at the terminal, so the
 * band turns 90° and lands flat on the edge instead of ending in mid-air.
 *
 * Works from either side: the edge's corner nearer the source takes the
 * band's first boundary, the farther corner takes the second, so the path
 * never crosses itself whichever pane the source is in.
 */
export function ribbonTerminalPath(
  x0: number,
  y0Top: number,
  y0Bot: number,
  edgeLeft: number,
  edgeRight: number,
  edgeY: number,
): string {
  const fromLeft = x0 <= (edgeLeft + edgeRight) / 2;
  const nearX = fromLeft ? edgeLeft : edgeRight;
  const farX = fromLeft ? edgeRight : edgeLeft;
  return [
    `M ${fmt(x0)} ${fmt(y0Top)}`,
    `C ${fmt((x0 + nearX) / 2)} ${fmt(y0Top)} ${fmt(nearX)} ${fmt((y0Top + edgeY) / 2)} ${fmt(nearX)} ${fmt(edgeY)}`,
    `L ${fmt(farX)} ${fmt(edgeY)}`,
    `C ${fmt(farX)} ${fmt((y0Bot + edgeY) / 2)} ${fmt((x0 + farX) / 2)} ${fmt(y0Bot)} ${fmt(x0)} ${fmt(y0Bot)}`,
    "Z",
  ].join(" ");
}

/**
 * The two x anchors of a pane-to-pane connection, on the OUTER edges of the
 * line-number rails facing the channel between the panes.
 *
 * Each pane hands in its full horizontal extent INCLUDING both rails: `left`
 * is the outer edge of its left (CodeMirror) gutter, `right` the outer edge
 * of its right rail. Whichever pane sits left contributes its right edge,
 * the other its left edge — decided by midpoints, so two overlapping rects
 * mid-drag still pick a consistent pair instead of crossing over.
 */
export interface PaneEdges {
  left: number;
  right: number;
}

export function anchorEdges(
  a: PaneEdges,
  b: PaneEdges,
): { forward: boolean; x0: number; x1: number } {
  const forward = (a.left + a.right) / 2 <= (b.left + b.right) / 2;
  return forward ? { forward, x0: a.right, x1: b.left } : { forward, x0: a.left, x1: b.right };
}

/**
 * How far a brace bulges for a span this tall: the nominal depth, shrunk so
 * a one-line brace never folds over itself (the horn and nub curves each
 * need a quarter of the height), and never below 1px.
 */
export function braceDepth(yTop: number, yBot: number, depth = 6): number {
  return Math.max(1, Math.min(depth, (yBot - yTop) / 4));
}

/**
 * Where a brace's centre nub tip lands: two depths out from the anchor
 * column, on the midline of the span. The thin connecting line starts here.
 */
export function braceNub(
  x: number,
  yTop: number,
  yBot: number,
  dir: 1 | -1,
  depth = 6,
): { x: number; y: number } {
  const d = braceDepth(yTop, yBot, depth);
  return { x: x + dir * 2 * d, y: (yTop + yBot) / 2 };
}

/** Which ends of a brace are OPEN — clamped to the viewport edge, so the
 * spine runs straight off the edge instead of curling a horn inward. */
export interface BraceOpenEnds {
  top?: boolean;
  bottom?: boolean;
}

/**
 * A math-annotation curly brace spanning EXACTLY [yTop, yBot] in the column
 * at `x`: two horns whose tips touch the anchor column at the first and last
 * pixel rows, a straight spine one depth out, and a centre nub two depths
 * out. `dir` is which side the spine and nub bulge toward (+1 right, -1
 * left) — toward the channel, away from the line numbers the horns touch.
 *
 * An end marked `open` draws NO horn: the range really continues past the
 * viewport edge there (the band was clamped), so the spine runs straight to
 * that y and visually off the edge — a horn would claim the range ends where
 * it merely stops being visible. The nub stays centred on the VISIBLE
 * extent, matching braceNub.
 *
 * `horn` is how far the horn arms reach INWARD from the anchor column —
 * across the line-number rail, in the -dir direction — so the tips land on
 * the rail's inner (text-side) edge and the brace visibly wraps the line
 * numbers of the included range. 0 keeps the tips on the anchor column. An
 * open end ignores it: no horn, nothing reaches into the gutter.
 *
 * An open (stroked, unfilled) path on purpose: a brace is drawn, not filled.
 */
export function bracePath(
  x: number,
  yTop: number,
  yBot: number,
  dir: 1 | -1,
  depth = 6,
  open: BraceOpenEnds = {},
  horn = 0,
): string {
  const d = braceDepth(yTop, yBot, depth);
  const spine = x + dir * d;
  const nub = x + dir * 2 * d;
  const mid = (yTop + yBot) / 2;
  const tip = x - dir * Math.max(0, horn);
  const top = open.top
    ? [`M ${fmt(spine)} ${fmt(yTop)}`]
    : [
        `M ${fmt(tip)} ${fmt(yTop)}`,
        ...(tip === x ? [] : [`L ${fmt(x)} ${fmt(yTop)}`]),
        `Q ${fmt(spine)} ${fmt(yTop)} ${fmt(spine)} ${fmt(yTop + d)}`,
      ];
  const bottom = open.bottom
    ? [`L ${fmt(spine)} ${fmt(yBot)}`]
    : [
        `L ${fmt(spine)} ${fmt(yBot - d)}`,
        `Q ${fmt(spine)} ${fmt(yBot)} ${fmt(x)} ${fmt(yBot)}`,
        ...(tip === x ? [] : [`L ${fmt(tip)} ${fmt(yBot)}`]),
      ];
  return [
    ...top,
    `L ${fmt(spine)} ${fmt(mid - d)}`,
    `Q ${fmt(spine)} ${fmt(mid)} ${fmt(nub)} ${fmt(mid)}`,
    `Q ${fmt(spine)} ${fmt(mid)} ${fmt(spine)} ${fmt(mid + d)}`,
    ...bottom,
  ].join(" ");
}

/**
 * Control-arm length for a brace link: proportional to the distance between
 * the endpoints (40%), floored so two nearby endpoints still get enough arm
 * to curve rather than kink. The arm is what makes the departure tangent
 * REAL — a control point glued to its endpoint is a corner in disguise.
 */
export function linkArm(x0: number, y0: number, x1: number, y1: number, minArm = 12): number {
  return Math.max(minArm, 0.4 * Math.hypot(x1 - x0, y1 - y0));
}

/**
 * The thin line joining two brace nubs: a single cubic that leaves EACH nub
 * along the nub's own outward direction (`dir`: +1 rightward, -1 leftward —
 * horizontal, away from that side's text), with C1 continuity at the tip.
 * The eye follows the line back to the point it originates from, and a curve
 * with no sudden direction change is easy to distinguish from another one
 * crossing it. Control arms are distance-proportional (see linkArm), so
 * nearby endpoints do not kink either.
 */
export function braceLinkPath(
  x0: number,
  y0: number,
  dir0: 1 | -1,
  x1: number,
  y1: number,
  dir1: 1 | -1,
): string {
  const arm = linkArm(x0, y0, x1, y1);
  return `M ${fmt(x0)} ${fmt(y0)} C ${fmt(x0 + dir0 * arm)} ${fmt(y0)} ${fmt(x1 + dir1 * arm)} ${fmt(y1)} ${fmt(x1)} ${fmt(y1)}`;
}

/**
 * The thin line from a brace nub to a HORIZONTAL terminal edge — the
 * underside connector of a tab, or the facing edge of a divider port. One
 * continuous cubic: it leaves the nub along the nub's outward direction
 * (`dir0`, horizontal, away from the text) and arrives vertically on the
 * edge, approaching from the source's side — smooth the whole way, never a
 * corner at either end. Arms are distance-proportional (see linkArm).
 */
export function braceLinkToEdgePath(
  x0: number,
  y0: number,
  dir0: 1 | -1,
  x1: number,
  edgeY: number,
): string {
  const arm = linkArm(x0, y0, x1, edgeY);
  // The arrival control sits between the source's y and the edge, so the
  // curve lands on the terminal travelling toward it, not past it.
  const approach = edgeY >= y0 ? edgeY - arm : edgeY + arm;
  return `M ${fmt(x0)} ${fmt(y0)} C ${fmt(x0 + dir0 * arm)} ${fmt(y0)} ${fmt(x1)} ${fmt(approach)} ${fmt(x1)} ${fmt(edgeY)}`;
}

/**
 * A ribbon that ENDS at the waypoint band: content flowing into a file that
 * is not the one currently open on the right. It shows the document feeding
 * every generated file, not just the visible one.
 */
export function ribbonStubPath(
  x0: number,
  y0Top: number,
  y0Bot: number,
  nodeLeft: number,
  nTop: number,
  nBot: number,
): string {
  const m0 = (x0 + nodeLeft) / 2;
  return [
    `M ${fmt(x0)} ${fmt(y0Top)}`,
    `C ${fmt(m0)} ${fmt(y0Top)} ${fmt(m0)} ${fmt(nTop)} ${fmt(nodeLeft)} ${fmt(nTop)}`,
    `L ${fmt(nodeLeft)} ${fmt(nBot)}`,
    `C ${fmt(m0)} ${fmt(nBot)} ${fmt(m0)} ${fmt(y0Bot)} ${fmt(x0)} ${fmt(y0Bot)}`,
    "Z",
  ].join(" ");
}
