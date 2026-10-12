import { emptyPane, panes, tab, withRail, type Layout, type Node, type Pane } from "./layout";
import type { SpatialSides } from "../lib/spatialSides";

export type Side = "left" | "right";
export interface Rect { x: number; y: number; w: number; h: number }
export interface ViewPlacement { side: Side; pin?: Rect; expanded?: boolean; height?: number }
export interface SpatialState {
  /** Global appearance preferences; supplied by the shell, not folder storage. */
  sides?: SpatialSides;
  center: string | null;
  stack: string[];
  views: Record<string, ViewPlacement>;
  auto: boolean;
  columns: [number, number];
}
export const HEADER_HEIGHT = 36;
export const TILE_GAP = 8;
export const spatialKey = (entry: Pane["tabs"][number]) => `${entry.kind}:${entry.kind === "untitled" ? entry.id : entry.target}`;
export const emptySpatialState = (): SpatialState => ({ center: null, stack: [], views: {}, auto: false, columns: [.23, .23] });

/** A view keeps its tab identity when it gains a tile; no editor bytes live here. */
export function spatialize(layout: Layout): Layout {
  let changed = false;
  const visit = (node: Node): Node => {
    if (node.type === "split") return { ...node, children: node.children.map(visit) };
    if (node.tabs.length < 2) return node;
    changed = true;
    const children = node.tabs.map((entry, index): Pane => ({
      ...(index === node.active ? node : emptyPane(node.region)), tabs: [entry], active: 0,
    }));
    return { type: "split", id: `spatial-${children.map(child => child.id).join("-")}`, direction: "row", children, sizes: children.map(() => 1 / children.length) };
  };
  let next = { ...layout, root: visit(layout.root) };
  const trees = panes(next.root).filter(pane => pane.tabs.some(entry => entry.kind === "tree"));
  if (trees.length === 1) { next = withRail(next, tab("tree", "folder:right", "Files"), .14); changed = true; }
  return changed ? next : layout;
}

export function intersects(a: Rect, b: Rect, gap = 0): boolean {
  return a.x < b.x + b.w + gap && a.x + a.w + gap > b.x && a.y < b.y + b.h + gap && a.y + a.h + gap > b.y;
}

/** Nearest free rectangle. A full tile never borrows space from a pin. */
export function clearSpatialSlot(r: Rect, occupied: readonly Rect[], low = -5000, high = 5000, shrink = true): Rect | null {
  const blocks = occupied.filter(b => r.x < b.x + b.w + 6 && r.x + r.w + 6 > b.x).sort((a, b) => a.y - b.y);
  const gaps: { start: number; end: number }[] = [];
  let start = low;
  for (const block of blocks) {
    if (block.y - TILE_GAP > start) gaps.push({ start, end: Math.min(high, block.y - TILE_GAP) });
    start = Math.max(start, block.y + block.h + TILE_GAP);
  }
  if (start < high) gaps.push({ start, end: high });
  const choices = gaps.filter(gap => gap.end - gap.start >= (shrink ? HEADER_HEIGHT : r.h)).map(gap => {
    const h = Math.min(r.h, gap.end - gap.start), y = Math.max(gap.start, Math.min(gap.end - h, r.y));
    return { ...r, y, h, distance: Math.abs(y + h / 2 - r.y - r.h / 2) };
  }).sort((a, b) => a.distance - b.distance);
  return choices[0] ?? null;
}

export function spatialRegions(width: number, height: number, columns: [number, number], sides: SpatialSides = { left: true, right: true }) {
  const rail = width < 650 ? 38 : Math.max(96, Math.min(210, width * .13)), leftRail = sides.left ? rail : 0, rightRail = sides.right ? rail : 0, W = width - leftRail - rightRail, gap = width < 650 ? 12 : 30;
  if (width < 650) {
    const any = sides.left || sides.right, both = sides.left && sides.right, top = any ? height * .54 : height, half = both ? (W - 12) / 2 : W - 4;
    return { small: true, leftRail: { x: 0, y: 0, w: leftRail, h: height }, rightRail: { x: width - rightRail, y: 0, w: rightRail, h: height }, center: { x: leftRail + 4, y: 0, w: W - 8, h: any ? top - 12 : height }, left: { x: leftRail + 4, y: top, w: sides.left ? half - 4 : 0, h: height - top }, right: { x: both ? leftRail + half + 8 : leftRail + 4, y: top, w: sides.right ? half - 4 : 0, h: height - top }, gap };
  }
  const left = sides.left ? W * columns[0] : 0, right = sides.right ? W * columns[1] : 0, leftGap = sides.left ? gap : 0, rightGap = sides.right ? gap : 0;
  return { small: false, leftRail: { x: 0, y: 0, w: leftRail, h: height }, rightRail: { x: width - rightRail, y: 0, w: rightRail, h: height }, center: { x: leftRail + left + leftGap, y: 0, w: W - left - right - leftGap - rightGap, h: height }, left: { x: leftRail + leftGap, y: 0, w: sides.left ? left - leftGap : 0, h: height }, right: { x: width - rightRail - right, y: 0, w: sides.right ? right - rightGap : 0, h: height }, gap };
}

/** Central content aligns; parallax displacement grows toward either edge. */
export function parallaxAnchor(sourceY: number, center: number, viewportHeight: number, speed = .75): number {
  const delta = sourceY - center, band = viewportHeight * .22;
  return center + delta - (1 - speed) * Math.sign(delta) * Math.max(0, Math.abs(delta) - band);
}

/** Use gaps, then translate a neighbor, then shrink it against the next pin. */
export function resizeSpatialPin(views: SpatialState["views"], key: string, delta: number, edge: "top" | "bottom", lane: Rect): SpatialState["views"] {
  const next = Object.fromEntries(Object.entries(views).map(([id, view]) => [id, { ...view, ...(view.pin ? { pin: { ...view.pin } } : {}) }]));
  const own = next[key], p = own?.pin;
  if (!p) return views;
  const top = edge === "top", growth = top ? -delta : delta;
  if (growth < 0) { const change = Math.max(growth, HEADER_HEIGHT - p.h); p.h += change; if (top) p.y -= change; return next; }
  const neighbors = Object.entries(next).filter(([id, view]) => id !== key && view.side === own.side && view.pin && (top ? view.pin.y + view.pin.h <= p.y + .1 : view.pin.y >= p.y + p.h - .1)).sort((a, b) => top ? b[1].pin!.y - a[1].pin!.y : a[1].pin!.y - b[1].pin!.y);
  const neighbor = neighbors[0]?.[1].pin, blocker = neighbors[1]?.[1].pin;
  const room = Math.max(0, top ? p.y - (neighbor ? neighbor.y + neighbor.h + TILE_GAP : lane.y) : (neighbor ? neighbor.y - TILE_GAP : lane.y + lane.h) - p.y - p.h);
  const needed = Math.max(0, growth - room), free = neighbor ? Math.max(0, top ? neighbor.y - (blocker ? blocker.y + blocker.h + TILE_GAP : lane.y) : (blocker ? blocker.y - TILE_GAP : lane.y + lane.h) - neighbor.y - neighbor.h) : 0;
  const shift = Math.min(needed, free), shrink = neighbor ? Math.min(needed - shift, neighbor.h - HEADER_HEIGHT) : 0, change = Math.min(growth, room + shift + shrink);
  p.h += change; if (top) p.y -= change;
  if (neighbor) { neighbor.y += top ? -shift : shift + shrink; neighbor.h -= shrink; }
  return next;
}

export function readSpatialState(raw: string | null): SpatialState {
  const state = emptySpatialState();
  try {
    const value = JSON.parse(raw ?? "null");
    if (value?.version !== 1) return state;
    if (typeof value.center === "string") state.center = value.center;
    if (Array.isArray(value.stack)) state.stack = value.stack.filter((key: unknown): key is string => typeof key === "string").slice(-24);
    state.auto = value.auto === true;
    if (Array.isArray(value.columns) && value.columns.length === 2 && value.columns.every((n: unknown) => typeof n === "number" && Number.isFinite(n) && n >= .16 && n <= .36)) state.columns = value.columns;
    for (const [key, rawView] of Object.entries(value.views ?? {})) {
      const view = rawView as ViewPlacement;
      if (view.side !== "left" && view.side !== "right") continue;
      state.views[key] = { side: view.side, expanded: view.expanded === true };
      if (typeof view.height === "number" && Number.isFinite(view.height)) state.views[key].height = Math.max(HEADER_HEIGHT, Math.min(4000, view.height));
      if (view.pin && [view.pin.x, view.pin.y, view.pin.w, view.pin.h].every(n => typeof n === "number" && Number.isFinite(n)) && view.pin.w > 0 && view.pin.h >= HEADER_HEIGHT) state.views[key].pin = view.pin;
    }
  } catch { /* Presentation storage never prevents opening a buffer. */ }
  return state;
}
