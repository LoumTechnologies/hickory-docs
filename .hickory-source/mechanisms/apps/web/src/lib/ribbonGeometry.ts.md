<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/ribbonGeometry.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c5b2f9480c3f"></a>
## `apps/web/src/lib/ribbonGeometry.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-c5b2f9480c3f-Band"></a>
### `Band`

Pure anchor math for the Split-view lineage ribbons. Everything here takes
numbers and returns numbers/strings — DOM measurement (line rects, scroller
bounds) happens in SplitView.tsx and feeds these functions, so the geometry
is unit-testable without a browser.

<storycode:copy id="f-c5b2f9480c3f-Band">
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

</storycode:copy>
<a id="f-c5b2f9480c3f-bandVisible"></a>
### `bandVisible`

Whether a vertical band [yTop, yBot] overlaps a viewport [viewTop, viewBot]
at all. A band fully above or below the viewport is invisible, and its
connection is only drawn when the anchored section is on screen.

<storycode:copy id="f-c5b2f9480c3f-bandVisible">
/**
 * Whether a vertical band [yTop, yBot] overlaps a viewport [viewTop, viewBot]
 * at all. A band fully above or below the viewport is invisible, and its
 * connection is only drawn when the anchored section is on screen.
 */
export function bandVisible(
  yTop: number,
  yBot: number,
  viewTop: number,
  viewBot: number,
): boolean {
  return yBot > viewTop && yTop < viewBot;
}

</storycode:copy>
<a id="f-c5b2f9480c3f-clampBand"></a>
### `clampBand`

Clamp a vertical band [yTop, yBot] to the visible pane [viewTop, viewBot].
A band fully outside collapses to a thin sliver at the nearer edge so the
ribbon still points the right way.

<storycode:copy id="f-c5b2f9480c3f-clampBand">
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

</storycode:copy>
<a id="f-c5b2f9480c3f-thicknessFor"></a>
### `thicknessFor`

Ribbon thickness (px) for a fragment carrying `bytes` of the file's
`totalBytes`: proportional share of `maxPx`, floored at `minPx` so tiny
fragments stay visible.

<storycode:copy id="f-c5b2f9480c3f-thicknessFor">
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

</storycode:copy>
<a id="f-c5b2f9480c3f-bandAround"></a>
### `bandAround`

Center a band of thickness `t` on the middle of [yTop, yBot].

<storycode:copy id="f-c5b2f9480c3f-bandAround">
/** Center a band of thickness `t` on the middle of [yTop, yBot]. */
export function bandAround(yTop: number, yBot: number, t: number): { yTop: number; yBot: number } {
  const mid = (yTop + yBot) / 2;
  return { yTop: mid - t / 2, yBot: mid + t / 2 };
}

</storycode:copy>
<a id="f-c5b2f9480c3f-atLeast"></a>
### `atLeast`

Give a band a minimum visible thickness, growing it around its midpoint.
A block whose source span is a single short line still needs to be grabbable.

<storycode:copy id="f-c5b2f9480c3f-atLeast">
/**
 * Give a band a minimum visible thickness, growing it around its midpoint.
 * A block whose source span is a single short line still needs to be grabbable.
 */
export function atLeast(yTop: number, yBot: number, minPx = 4): { yTop: number; yBot: number } {
  const h = yBot - yTop;
  if (h >= minPx) return { yTop, yBot };
  return bandAround(yTop, yBot, minPx);
}

</storycode:copy>
<a id="f-c5b2f9480c3f-fmt"></a>
### `fmt`

<storycode:copy id="f-c5b2f9480c3f-fmt">
const fmt = (n: number) => (Math.round(n * 10) / 10).toString();

</storycode:copy>
<a id="f-c5b2f9480c3f-ribbonPath"></a>
### `ribbonPath`

A closed Sankey-style ribbon between two vertical bands: two cubic beziers
with horizontal tangents (left edge x0, right edge x1), flowing left→right.

<storycode:copy id="f-c5b2f9480c3f-ribbonPath">
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

</storycode:copy>
<a id="f-c5b2f9480c3f-ribbonPathVia"></a>
### `ribbonPathVia`

A ribbon routed THROUGH a waypoint band — the file node in the middle
column. Two Sankey segments (source→node, node→output) joined across the
node's own width, so the eye follows content into the file it lands in and
out again into the generated text.

<storycode:copy id="f-c5b2f9480c3f-ribbonPathVia">
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

</storycode:copy>
<a id="f-c5b2f9480c3f-terminalEdge"></a>
### `terminalEdge`

Which horizontal edge of a terminal (a tab connector or a divider port
button) a ribbon should attach to: a source band whose midpoint sits above
the terminal's centre lands on the TOP edge, one below lands on the BOTTOM.
Terminals live in chrome — a tab strip or a thin vertical divider — so a
ribbon always arrives on a horizontal edge: attaching to the side of a
button inside a 4px divider would give the ribbon no width to land on.

<storycode:copy id="f-c5b2f9480c3f-terminalEdge">
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

</storycode:copy>
<a id="f-c5b2f9480c3f-ribbonTerminalPath"></a>
### `ribbonTerminalPath`

A closed ribbon from a vertical band to a HORIZONTAL edge — the underside
of a tab, or the top/bottom of a divider port button. Two elbow beziers:
horizontal tangent at the source, vertical tangent at the terminal, so the
band turns 90° and lands flat on the edge instead of ending in mid-air.
Works from either side: the edge's corner nearer the source takes the
band's first boundary, the farther corner takes the second, so the path
never crosses itself whichever pane the source is in.

<storycode:copy id="f-c5b2f9480c3f-ribbonTerminalPath">
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

</storycode:copy>
<a id="f-c5b2f9480c3f-PaneEdges"></a>
### `PaneEdges`

The two x anchors of a pane-to-pane connection, on the OUTER edges of the
line-number rails facing the channel between the panes.
Each pane hands in its full horizontal extent INCLUDING both rails: `left`
is the outer edge of its left (CodeMirror) gutter, `right` the outer edge
of its right rail. Whichever pane sits left contributes its right edge,
the other its left edge — decided by midpoints, so two overlapping rects
mid-drag still pick a consistent pair instead of crossing over.

<storycode:copy id="f-c5b2f9480c3f-PaneEdges">
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

</storycode:copy>
<a id="f-c5b2f9480c3f-anchorEdges"></a>
### `anchorEdges`

<storycode:copy id="f-c5b2f9480c3f-anchorEdges">
export function anchorEdges(
  a: PaneEdges,
  b: PaneEdges,
): { forward: boolean; x0: number; x1: number } {
  const forward = (a.left + a.right) / 2 <= (b.left + b.right) / 2;
  return forward ? { forward, x0: a.right, x1: b.left } : { forward, x0: a.left, x1: b.right };
}

</storycode:copy>
<a id="f-c5b2f9480c3f-braceDepth"></a>
### `braceDepth`

How far a brace bulges for a span this tall: the nominal depth, shrunk so
a one-line brace never folds over itself (the horn and nub curves each
need a quarter of the height), and never below 1px.

<storycode:copy id="f-c5b2f9480c3f-braceDepth">
/**
 * How far a brace bulges for a span this tall: the nominal depth, shrunk so
 * a one-line brace never folds over itself (the horn and nub curves each
 * need a quarter of the height), and never below 1px.
 */
export function braceDepth(yTop: number, yBot: number, depth = 6): number {
  return Math.max(1, Math.min(depth, (yBot - yTop) / 4));
}

</storycode:copy>
<a id="f-c5b2f9480c3f-braceNub"></a>
### `braceNub`

Where a brace's centre nub tip lands: two depths out from the anchor
column, on the midline of the span. The thin connecting line starts here.

<storycode:copy id="f-c5b2f9480c3f-braceNub">
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

</storycode:copy>
<a id="f-c5b2f9480c3f-BraceOpenEnds"></a>
### `BraceOpenEnds`

Which ends of a brace are OPEN — clamped to the viewport edge, so the
spine runs straight off the edge instead of curling a horn inward.

<storycode:copy id="f-c5b2f9480c3f-BraceOpenEnds">
/** Which ends of a brace are OPEN — clamped to the viewport edge, so the
 * spine runs straight off the edge instead of curling a horn inward. */
export interface BraceOpenEnds {
  top?: boolean;
  bottom?: boolean;
}

</storycode:copy>
<a id="f-c5b2f9480c3f-bracePath"></a>
### `bracePath`

A math-annotation curly brace spanning EXACTLY [yTop, yBot] in the column
at `x`: two horns whose tips touch the anchor column at the first and last
pixel rows, a straight spine one depth out, and a centre nub two depths
out. `dir` is which side the spine and nub bulge toward (+1 right, -1
left) — toward the channel, away from the line numbers the horns touch.
An end marked `open` draws NO horn: the range really continues past the
viewport edge there (the band was clamped), so the spine runs straight to
that y and visually off the edge — a horn would claim the range ends where
it merely stops being visible. The nub stays centred on the VISIBLE
extent, matching braceNub.
`horn` is how far the horn arms reach INWARD from the anchor column —
across the line-number rail, in the -dir direction — so the tips land on
the rail's inner (text-side) edge and the brace visibly wraps the line
numbers of the included range. 0 keeps the tips on the anchor column. An
open end ignores it: no horn, nothing reaches into the gutter.
An open (stroked, unfilled) path on purpose: a brace is drawn, not filled.

<storycode:copy id="f-c5b2f9480c3f-bracePath">
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

</storycode:copy>
<a id="f-c5b2f9480c3f-linkArm"></a>
### `linkArm`

Control-arm length for a brace link: proportional to the distance between
the endpoints (40%), floored so two nearby endpoints still get enough arm
to curve rather than kink. The arm is what makes the departure tangent
REAL — a control point glued to its endpoint is a corner in disguise.

<storycode:copy id="f-c5b2f9480c3f-linkArm">
/**
 * Control-arm length for a brace link: proportional to the distance between
 * the endpoints (40%), floored so two nearby endpoints still get enough arm
 * to curve rather than kink. The arm is what makes the departure tangent
 * REAL — a control point glued to its endpoint is a corner in disguise.
 */
export function linkArm(x0: number, y0: number, x1: number, y1: number, minArm = 12): number {
  return Math.max(minArm, 0.4 * Math.hypot(x1 - x0, y1 - y0));
}

</storycode:copy>
<a id="f-c5b2f9480c3f-braceLinkPath"></a>
### `braceLinkPath`

The thin line joining two brace nubs: a single cubic that leaves EACH nub
along the nub's own outward direction (`dir`: +1 rightward, -1 leftward —
horizontal, away from that side's text), with C1 continuity at the tip.
The eye follows the line back to the point it originates from, and a curve
with no sudden direction change is easy to distinguish from another one
crossing it. Control arms are distance-proportional (see linkArm), so
nearby endpoints do not kink either.

<storycode:copy id="f-c5b2f9480c3f-braceLinkPath">
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

</storycode:copy>
<a id="f-c5b2f9480c3f-braceLinkToEdgePath"></a>
### `braceLinkToEdgePath`

The thin line from a brace nub to a HORIZONTAL terminal edge — the
underside connector of a tab, or the facing edge of a divider port. One
continuous cubic: it leaves the nub along the nub's outward direction
(`dir0`, horizontal, away from the text) and arrives vertically on the
edge, approaching from the source's side — smooth the whole way, never a
corner at either end. Arms are distance-proportional (see linkArm).

<storycode:copy id="f-c5b2f9480c3f-braceLinkToEdgePath">
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

</storycode:copy>
<a id="f-c5b2f9480c3f-ribbonStubPath"></a>
### `ribbonStubPath`

A ribbon that ENDS at the waypoint band: content flowing into a file that
is not the one currently open on the right. It shows the document feeding
every generated file, not just the visible one.

<storycode:copy id="f-c5b2f9480c3f-ribbonStubPath">
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
</storycode:copy>

### Assemble `apps/web/src/lib/ribbonGeometry.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/ribbonGeometry.ts"><storycode:paste select="#f-c5b2f9480c3f-Band"/><storycode:paste select="#f-c5b2f9480c3f-bandVisible"/><storycode:paste select="#f-c5b2f9480c3f-clampBand"/><storycode:paste select="#f-c5b2f9480c3f-thicknessFor"/><storycode:paste select="#f-c5b2f9480c3f-bandAround"/><storycode:paste select="#f-c5b2f9480c3f-atLeast"/><storycode:paste select="#f-c5b2f9480c3f-fmt"/><storycode:paste select="#f-c5b2f9480c3f-ribbonPath"/><storycode:paste select="#f-c5b2f9480c3f-ribbonPathVia"/><storycode:paste select="#f-c5b2f9480c3f-terminalEdge"/><storycode:paste select="#f-c5b2f9480c3f-ribbonTerminalPath"/><storycode:paste select="#f-c5b2f9480c3f-PaneEdges"/><storycode:paste select="#f-c5b2f9480c3f-anchorEdges"/><storycode:paste select="#f-c5b2f9480c3f-braceDepth"/><storycode:paste select="#f-c5b2f9480c3f-braceNub"/><storycode:paste select="#f-c5b2f9480c3f-BraceOpenEnds"/><storycode:paste select="#f-c5b2f9480c3f-bracePath"/><storycode:paste select="#f-c5b2f9480c3f-linkArm"/><storycode:paste select="#f-c5b2f9480c3f-braceLinkPath"/><storycode:paste select="#f-c5b2f9480c3f-braceLinkToEdgePath"/><storycode:paste select="#f-c5b2f9480c3f-ribbonStubPath"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-4ca95a05eac1"></a>
## `apps/web/src/lib/ribbonGeometry.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-4ca95a05eac1-cubic"></a>
### `cubic`

<storycode:copy id="f-4ca95a05eac1-cubic">
import { describe, expect, it } from "vitest";
import { anchorEdges, atLeast, bandAround, bandVisible, braceDepth, braceLinkPath, braceLinkToEdgePath, braceNub, bracePath, clampBand, linkArm, ribbonPath, ribbonPathVia, ribbonStubPath, ribbonTerminalPath, terminalEdge, thicknessFor } from "./ribbonGeometry";

/** Parse a single-cubic path `M x y C c1x c1y c2x c2y x y` into its points. */
function cubic(d: string) {
  const m =
    /^M (-?[\d.]+) (-?[\d.]+) C (-?[\d.]+) (-?[\d.]+) (-?[\d.]+) (-?[\d.]+) (-?[\d.]+) (-?[\d.]+)$/.exec(
      d,
    );
  expect(m, `not one cubic: ${d}`).not.toBeNull();
  const [, ...n] = m!.map(Number);
  return {
    start: { x: n[0], y: n[1] },
    c1: { x: n[2], y: n[3] },
    c2: { x: n[4], y: n[5] },
    end: { x: n[6], y: n[7] },
  };
}

</storycode:copy>
<a id="f-4ca95a05eac1-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-4ca95a05eac1-expression_statement">
describe("bandVisible", () => {
  it("is true when the band overlaps the viewport", () => {
    expect(bandVisible(120, 140, 100, 500)).toBe(true);
    expect(bandVisible(90, 110, 100, 500)).toBe(true);
    expect(bandVisible(490, 510, 100, 500)).toBe(true);
  });

  it("is false when the band sits fully above or below the viewport", () => {
    expect(bandVisible(0, 99, 100, 500)).toBe(false);
    expect(bandVisible(0, 100, 100, 500)).toBe(false);
    expect(bandVisible(500, 700, 100, 500)).toBe(false);
  });
});

</storycode:copy>
<a id="f-4ca95a05eac1-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-4ca95a05eac1-expression_statement-2">
describe("clampBand", () => {
  it("passes a fully visible band through untouched", () => {
    expect(clampBand(120, 140, 100, 500)).toEqual({
      yTop: 120,
      yBot: 140,
      clamped: false,
      clampedTop: false,
      clampedBottom: false,
    });
  });

  it("clamps a band that pokes above the pane, and says WHICH end", () => {
    expect(clampBand(80, 140, 100, 500)).toEqual({
      yTop: 100,
      yBot: 140,
      clamped: true,
      clampedTop: true,
      clampedBottom: false,
    });
  });

  it("clamps a band that pokes below the pane, and says WHICH end", () => {
    expect(clampBand(480, 560, 100, 500)).toEqual({
      yTop: 480,
      yBot: 500,
      clamped: true,
      clampedTop: false,
      clampedBottom: true,
    });
  });

  it("reports both ends when the band overflows the pane on both sides", () => {
    expect(clampBand(50, 600, 100, 500)).toEqual({
      yTop: 100,
      yBot: 500,
      clamped: true,
      clampedTop: true,
      clampedBottom: true,
    });
  });

  it("collapses a band fully above to a sliver at the top edge, open at the top", () => {
    expect(clampBand(0, 50, 100, 500)).toEqual({
      yTop: 100,
      yBot: 102,
      clamped: true,
      clampedTop: true,
      clampedBottom: false,
    });
  });

  it("collapses a band fully below to a sliver at the bottom edge, open at the bottom", () => {
    expect(clampBand(600, 700, 100, 500)).toEqual({
      yTop: 498,
      yBot: 500,
      clamped: true,
      clampedTop: false,
      clampedBottom: true,
    });
  });
});

</storycode:copy>
<a id="f-4ca95a05eac1-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-4ca95a05eac1-expression_statement-3">
describe("thicknessFor", () => {
  it("is proportional to the byte share", () => {
    expect(thicknessFor(50, 100, 26)).toBeCloseTo(13);
  });

  it("caps at maxPx and floors at minPx", () => {
    expect(thicknessFor(100, 100, 26)).toBe(26);
    expect(thicknessFor(1, 1_000_000, 26, 3)).toBe(3);
  });

  it("degrades safely on zero totals", () => {
    expect(thicknessFor(10, 0)).toBe(3);
    expect(thicknessFor(0, 10)).toBe(3);
  });
});

</storycode:copy>
<a id="f-4ca95a05eac1-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-4ca95a05eac1-expression_statement-4">
describe("bandAround", () => {
  it("centers the thickness on the band midpoint", () => {
    expect(bandAround(100, 120, 10)).toEqual({ yTop: 105, yBot: 115 });
  });
});

</storycode:copy>
<a id="f-4ca95a05eac1-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-4ca95a05eac1-expression_statement-5">
describe("ribbonPath", () => {
  it("builds a closed two-bezier ribbon between the two bands", () => {
    const d = ribbonPath(10, 100, 110, 200, 300, 340);
    expect(d.startsWith("M 10 100")).toBe(true);
    expect(d.endsWith("Z")).toBe(true);
    // Horizontal tangents: control points at the midpoint x.
    expect(d).toContain("C 105 100 105 300 200 300");
    expect(d).toContain("L 200 340");
    expect(d).toContain("C 105 340 105 110 10 110");
  });

  it("rounds coordinates to a tenth to keep paths compact", () => {
    const d = ribbonPath(0.123, 1.056, 2.849, 3.301, 4.05, 5.999);
    expect(d).toContain("M 0.1 1.1");
    expect(d).toContain("6");
    expect(d).not.toMatch(/\d\.\d\d/);
  });
});

</storycode:copy>
<a id="f-4ca95a05eac1-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-4ca95a05eac1-expression_statement-6">
describe("ribbons routed through the file-tree waypoint", () => {
  it("threads source → node → output as one closed path", () => {
    const d = ribbonPathVia(100, 10, 30, 200, 260, 40, 50, 400, 80, 120);
    // Starts at the source edge, reaches the node's left and right edges, ends
    // closed so the ribbon fills rather than strokes.
    expect(d.startsWith("M 100 10")).toBe(true);
    expect(d).toContain("200 40");
    expect(d).toContain("L 260 40");
    expect(d).toContain("400 80");
    expect(d.endsWith("Z")).toBe(true);
  });

  it("stubs stop at the node and never reach the output edge", () => {
    const d = ribbonStubPath(100, 10, 30, 200, 40, 50);
    expect(d.startsWith("M 100 10")).toBe(true);
    expect(d).toContain("200 40");
    expect(d).toContain("L 200 50");
    expect(d.endsWith("Z")).toBe(true);
  });

  it("gives a zero-height band a grabbable minimum without moving its center", () => {
    const b = atLeast(100, 100, 4);
    expect(b.yBot - b.yTop).toBe(4);
    expect((b.yTop + b.yBot) / 2).toBe(100);
    // A band that is already tall enough is returned untouched.
    expect(atLeast(10, 40, 4)).toEqual({ yTop: 10, yBot: 40 });
  });
});

</storycode:copy>
<a id="f-4ca95a05eac1-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-4ca95a05eac1-expression_statement-7">
describe("ribbons terminating on chrome (tabs and divider ports)", () => {
  // Which edge takes the band: a terminal is a tab or a button in a thin
  // vertical divider, so attachment is always to a horizontal edge — the
  // side of a 4px-wide button gives a ribbon nothing to land on.
  it("attaches a band above the terminal's centre to its top edge", () => {
    expect(terminalEdge(100, 140, 300, 320)).toBe("top");
  });

  it("attaches a band below the terminal's centre to its bottom edge", () => {
    expect(terminalEdge(500, 540, 300, 320)).toBe("bottom");
  });

  it("splits ties by the midpoints, not the extremes", () => {
    // Band midpoint 310 equals the terminal's centre: top wins the tie.
    expect(terminalEdge(300, 320, 300, 320)).toBe("top");
    expect(terminalEdge(311, 311, 300, 320)).toBe("bottom");
  });

  it("lands a source LEFT of the terminal flat on the edge, near corner first", () => {
    const d = ribbonTerminalPath(100, 10, 30, 400, 460, 200);
    // Starts at the source band's top boundary…
    expect(d.startsWith("M 100 10")).toBe(true);
    // …elbows to the NEAR corner (left, 400) with a vertical arrival…
    expect(d).toContain("400 200");
    // …runs the full edge to the far corner…
    expect(d).toContain("L 460 200");
    // …and returns to the band's bottom boundary, closed for filling.
    expect(d).toContain("100 30");
    expect(d.endsWith("Z")).toBe(true);
  });

  it("mirrors when the source is RIGHT of the terminal", () => {
    const d = ribbonTerminalPath(800, 10, 30, 400, 460, 200);
    // The near corner is now the right one, so the edge runs right→left.
    expect(d).toContain("L 400 200");
    expect(d.endsWith("Z")).toBe(true);
  });

  it("has horizontal tangents at the source and vertical at the terminal", () => {
    const d = ribbonTerminalPath(100, 10, 30, 400, 460, 200);
    // First control point shares the source's y (horizontal start), second
    // shares the terminal corner's x (vertical landing).
    expect(d).toContain("C 250 10 400 105 400 200");
  });

  it("works upward too: a terminal above the band attaches from below", () => {
    const d = ribbonTerminalPath(100, 500, 540, 400, 460, 60);
    expect(d.startsWith("M 100 500")).toBe(true);
    expect(d).toContain("400 60");
    expect(d).toContain("L 460 60");
    expect(d.endsWith("Z")).toBe(true);
  });
});

</storycode:copy>
<a id="f-4ca95a05eac1-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-4ca95a05eac1-expression_statement-8">
describe("anchorEdges", () => {
  const doc = { left: 0, right: 400 };
  const out = { left: 500, right: 900 };

  it("anchors the left pane's right rail edge and the right pane's left gutter edge", () => {
    expect(anchorEdges(doc, out)).toEqual({ forward: true, x0: 400, x1: 500 });
  });

  it("swaps edges when the panes swap sides", () => {
    expect(anchorEdges(out, doc)).toEqual({ forward: false, x0: 500, x1: 400 });
  });

  it("decides by midpoints, so overlapping rects mid-drag still pick one pair", () => {
    // a's midpoint (200) is left of b's (450) even though the rects overlap.
    expect(anchorEdges({ left: 0, right: 400 }, { left: 300, right: 600 })).toEqual({
      forward: true,
      x0: 400,
      x1: 300,
    });
  });
});

</storycode:copy>
<a id="f-4ca95a05eac1-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-4ca95a05eac1-expression_statement-9">
describe("bracePath", () => {
  it("touches the anchor column at exactly the first and last pixel rows", () => {
    const path = bracePath(100, 40, 140, 1, 6);
    expect(path.startsWith("M 100 40")).toBe(true);
    expect(path.endsWith("100 140")).toBe(true);
  });

  it("bulges toward the channel: right for dir 1, left for dir -1", () => {
    // Spine at x+6, nub at x+12 for a span tall enough for full depth.
    expect(bracePath(100, 0, 100, 1, 6)).toContain("Q 106 50 112 50");
    expect(bracePath(100, 0, 100, -1, 6)).toContain("Q 94 50 88 50");
  });

  it("shrinks its depth on a short span instead of folding over itself", () => {
    // One 16px line: depth clamps to 4, so the nub sits 8 out, not 12.
    expect(braceDepth(0, 16, 6)).toBe(4);
    expect(braceNub(100, 0, 16, 1, 6)).toEqual({ x: 108, y: 8 });
  });

  it("never loses the brace entirely, even on a degenerate span", () => {
    expect(braceDepth(50, 50, 6)).toBe(1);
  });

  // Clamped ends: when the underlying range continues past the viewport edge,
  // the brace must NOT curl a horn inward at that edge — the spine runs
  // straight to the clamp y, visually off the edge.
  describe("open (clamped) ends", () => {
    it("open top: starts flat ON the spine, no curve toward the anchor column", () => {
      const d = bracePath(100, 40, 140, 1, 6, { top: true });
      // Begins at the spine x (106), at the clamp y — never touching x=100.
      expect(d.startsWith("M 106 40 L 106 84")).toBe(true);
      // No horn: the first command after M is a straight L, not a Q.
      expect(d.split(" ")[3]).toBe("L");
      // Bottom horn still closes onto the anchor column.
      expect(d.endsWith("Q 106 140 100 140")).toBe(true);
    });

    it("open bottom: the spine runs straight off the clamp, no bottom horn", () => {
      const d = bracePath(100, 40, 140, 1, 6, { bottom: true });
      // Top horn intact.
      expect(d.startsWith("M 100 40 Q 106 40 106 46")).toBe(true);
      // Ends with a straight line to the clamp y at the spine x — no Q, no
      // return to the anchor column.
      expect(d.endsWith("L 106 140")).toBe(true);
      expect(d).not.toContain("100 140");
    });

    it("both ends open: a straight spine with only the centre nub", () => {
      const d = bracePath(100, 40, 140, 1, 6, { top: true, bottom: true });
      expect(d).toBe(
        "M 106 40 L 106 84 Q 106 90 112 90 Q 106 90 106 96 L 106 140",
      );
    });

    it("keeps the nub centred on the VISIBLE extent, agreeing with braceNub", () => {
      const d = bracePath(100, 40, 140, 1, 6, { top: true });
      const nub = braceNub(100, 40, 140, 1, 6);
      expect(d).toContain(`Q 106 ${nub.y} ${nub.x} ${nub.y}`);
    });

    it("stays degenerate-safe on a one-line clamped sliver", () => {
      // A 2px sliver: depth clamps to 1; the open path must still be valid
      // and stay on its spine.
      const d = bracePath(100, 498, 500, 1, 6, { bottom: true });
      expect(d.endsWith("L 101 500")).toBe(true);
    });

    it("mirrors for dir -1", () => {
      const d = bracePath(100, 40, 140, -1, 6, { top: true });
      expect(d.startsWith("M 94 40 L 94 84")).toBe(true);
    });

    it("closed braces are unchanged by an empty open spec", () => {
      expect(bracePath(100, 40, 140, 1, 6, {})).toBe(bracePath(100, 40, 140, 1, 6));
    });
  });

  // Horn reach: the horn arms extend INWARD across the line-number rail —
  // opposite the bulge direction — so the tips land on the rail's inner
  // (text-side) edge and the brace visibly wraps the included line numbers.
  describe("horns reaching across the line-number rail", () => {
    it("puts the horn tips one rail-width in from the anchor column", () => {
      const d = bracePath(100, 40, 140, 1, 6, {}, 30);
      // Top tip at x - horn, straight across the gutter, then the curl.
      expect(d.startsWith("M 70 40 L 100 40 Q 106 40 106 46")).toBe(true);
      // Bottom mirrors: curl back to the column, then across to the tip.
      expect(d.endsWith("Q 106 140 100 140 L 70 140")).toBe(true);
    });

    it("mirrors for dir -1: the rail is on the brace's other side", () => {
      const d = bracePath(100, 40, 140, -1, 6, {}, 30);
      expect(d.startsWith("M 130 40 L 100 40 Q 94 40 94 46")).toBe(true);
      expect(d.endsWith("Q 94 140 100 140 L 130 140")).toBe(true);
    });

    it("an open end draws no horn at all — nothing reaches into the gutter", () => {
      const d = bracePath(100, 40, 140, 1, 6, { top: true }, 30);
      // Open top: straight down the spine from the clamp, no tip, no L
      // across the gutter at the top.
      expect(d.startsWith("M 106 40 L 106 84")).toBe(true);
      // The closed bottom still gets its full horn.
      expect(d.endsWith("Q 106 140 100 140 L 70 140")).toBe(true);
    });

    it("zero horn keeps the tips on the anchor column, byte-for-byte", () => {
      expect(bracePath(100, 40, 140, 1, 6, {}, 0)).toBe(bracePath(100, 40, 140, 1, 6));
    });

    it("stays valid on a short span: depth shrinks, the horn does not fold", () => {
      // One 16px line, depth clamps to 4; the horn is horizontal so its
      // length never competes with the vertical curl budget.
      const d = bracePath(100, 0, 16, 1, 6, {}, 40);
      expect(d.startsWith("M 60 0 L 100 0 Q 104 0 104 4")).toBe(true);
      expect(d.endsWith("Q 104 16 100 16 L 60 16")).toBe(true);
    });
  });
});

</storycode:copy>
<a id="f-4ca95a05eac1-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-4ca95a05eac1-expression_statement-10">
describe("braceNub", () => {
  it("sits two depths out on the midline, on the dir side", () => {
    expect(braceNub(100, 40, 140, 1, 6)).toEqual({ x: 112, y: 90 });
    expect(braceNub(100, 40, 140, -1, 6)).toEqual({ x: 88, y: 90 });
  });
});

</storycode:copy>
<a id="f-4ca95a05eac1-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-4ca95a05eac1-expression_statement-11">
describe("linkArm", () => {
  it("is proportional to the endpoint distance", () => {
    expect(linkArm(0, 0, 300, 400)).toBeCloseTo(200); // 0.4 · 500
  });

  it("never shrinks below the minimum, so nearby endpoints do not kink", () => {
    expect(linkArm(100, 100, 104, 103)).toBe(12);
  });
});

</storycode:copy>
<a id="f-4ca95a05eac1-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-4ca95a05eac1-expression_statement-12">
describe("braceLinkPath", () => {
  // The maintainer's complaint was a sudden direction change at the brace
  // tip: the link must LEAVE each nub with C1 continuity along the nub's own
  // outward direction — horizontal, away from the text — and curve smoothly
  // the whole way. The tangent at a cubic's endpoint runs through its
  // adjacent control point, so these tests read the control points.
  it("departs each nub horizontally, along that nub's outward direction", () => {
    const arm = Math.round(linkArm(112, 90, 488, 210) * 10) / 10;
    const p = cubic(braceLinkPath(112, 90, 1, 488, 210, -1));
    expect(p.start).toEqual({ x: 112, y: 90 });
    expect(p.end).toEqual({ x: 488, y: 210 });
    // Horizontal tangents: each control shares its endpoint's y…
    expect(p.c1.y).toBe(90);
    expect(p.c2.y).toBe(210);
    // …one arm out along the endpoint's own outward direction.
    expect(p.c1.x).toBeCloseTo(112 + arm, 1);
    expect(p.c2.x).toBeCloseTo(488 - arm, 1);
  });

  it("keeps the departure outward even when the target sits behind the nub", () => {
    // A nub bulging right whose target is LEFT of it: the old midpoint
    // control flipped the tangent back toward the text — a kink at the tip.
    const p = cubic(braceLinkPath(400, 100, 1, 200, 300, 1));
    expect(p.c1.x).toBeGreaterThan(400);
    expect(p.c2.x).toBeGreaterThan(200);
  });

  it("gives near-touching nubs a real arm instead of a corner", () => {
    const p = cubic(braceLinkPath(100, 100, 1, 106, 104, -1));
    expect(p.c1.x - 100).toBeGreaterThanOrEqual(12);
    expect(106 - p.c2.x).toBeGreaterThanOrEqual(12);
  });
});

</storycode:copy>
<a id="f-4ca95a05eac1-expression_statement-13"></a>
### `expression_statement`

<storycode:copy id="f-4ca95a05eac1-expression_statement-13">
describe("braceLinkToEdgePath", () => {
  it("is one continuous cubic: horizontal out of the nub, vertical onto the edge", () => {
    const arm = Math.round(linkArm(112, 90, 300, 30) * 10) / 10;
    const p = cubic(braceLinkToEdgePath(112, 90, 1, 300, 30));
    expect(p.start).toEqual({ x: 112, y: 90 });
    expect(p.end).toEqual({ x: 300, y: 30 });
    // Horizontal departure along the nub's outward direction…
    expect(p.c1.y).toBe(90);
    expect(p.c1.x).toBeCloseTo(112 + arm, 1);
    // …and a vertical arrival: the last control shares the terminal's x and
    // approaches the edge from the source's side, so there is no corner.
    expect(p.c2.x).toBe(300);
    expect(p.c2.y).toBeCloseTo(30 + arm, 1);
  });

  it("approaches from above when the terminal edge is below the nub", () => {
    const arm = Math.round(linkArm(112, 90, 300, 500) * 10) / 10;
    const p = cubic(braceLinkToEdgePath(112, 90, -1, 300, 500));
    // Outward is leftward here; the arrival control sits above the edge.
    expect(p.c1.x).toBeCloseTo(112 - arm, 1);
    expect(p.c2.x).toBe(300);
    expect(p.c2.y).toBeCloseTo(500 - arm, 1);
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/ribbonGeometry.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/ribbonGeometry.test.ts"><storycode:paste select="#f-4ca95a05eac1-cubic"/><storycode:paste select="#f-4ca95a05eac1-expression_statement"/><storycode:paste select="#f-4ca95a05eac1-expression_statement-2"/><storycode:paste select="#f-4ca95a05eac1-expression_statement-3"/><storycode:paste select="#f-4ca95a05eac1-expression_statement-4"/><storycode:paste select="#f-4ca95a05eac1-expression_statement-5"/><storycode:paste select="#f-4ca95a05eac1-expression_statement-6"/><storycode:paste select="#f-4ca95a05eac1-expression_statement-7"/><storycode:paste select="#f-4ca95a05eac1-expression_statement-8"/><storycode:paste select="#f-4ca95a05eac1-expression_statement-9"/><storycode:paste select="#f-4ca95a05eac1-expression_statement-10"/><storycode:paste select="#f-4ca95a05eac1-expression_statement-11"/><storycode:paste select="#f-4ca95a05eac1-expression_statement-12"/><storycode:paste select="#f-4ca95a05eac1-expression_statement-13"/></storycode:file>


</storycode:doc>
