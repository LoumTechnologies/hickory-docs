import type { Rect } from "./spatialLayout";

/** The primary tab is a slot, so switching tabs does not discard its neighbors. */
export const PRIMARY = "@center";
export type Tile = string | { axis: "row" | "column"; first: Tile; second: Tile };
export type DockEdge = "left" | "right" | "top" | "bottom" | "center";
export function pruneTiles(tile: Tile, keep: (key: string) => boolean): Tile | undefined {
  if (typeof tile === "string") return keep(tile) ? tile : undefined;
  const first = pruneTiles(tile.first, keep), second = pruneTiles(tile.second, keep);
  return first && second ? { ...tile, first, second } : first ?? second;
}
export function dockTile(tree: Tile, key: string, target: string, edge: Exclude<DockEdge, "center">): Tile {
  if (key === target) return tree;
  const clean = pruneTiles(tree, item => item !== key) ?? PRIMARY;
  const visit = (tile: Tile): Tile => {
    if (typeof tile !== "string") return { ...tile, first: visit(tile.first), second: visit(tile.second) };
    if (tile !== target) return tile;
    const before = edge === "left" || edge === "top";
    return { axis: edge === "left" || edge === "right" ? "row" : "column", first: before ? key : tile, second: before ? tile : key };
  };
  return visit(clean);
}
export function tileRects(tree: Tile, rect: Rect): Map<string, Rect> {
  const result = new Map<string, Rect>();
  const visit = (tile: Tile, box: Rect) => {
    if (typeof tile === "string") { result.set(tile, box); return; }
    const gap = 6, horizontal = tile.axis === "row", size = Math.max(0, ((horizontal ? box.w : box.h) - gap) / 2);
    visit(tile.first, { ...box, ...(horizontal ? { w: size } : { h: size }) });
    visit(tile.second, horizontal ? { ...box, x: box.x + size + gap, w: size } : { ...box, y: box.y + size + gap, h: size });
  };
  visit(tree, rect); return result;
}
export function dockHit(rect: Rect, x: number, y: number): DockEdge {
  const distances = [(x - rect.x) / rect.w, (rect.x + rect.w - x) / rect.w, (y - rect.y) / rect.h, (rect.y + rect.h - y) / rect.h];
  const nearest = Math.min(...distances);
  return nearest < .23 ? (["left", "right", "top", "bottom"] as const)[distances.indexOf(nearest)] : "center";
}
export function readTiles(value: unknown, depth = 0): Tile | undefined {
  if (typeof value === "string") return value;
  if (!value || typeof value !== "object" || depth > 12) return;
  const node = value as Record<string, unknown>, first = readTiles(node.first, depth + 1), second = readTiles(node.second, depth + 1);
  if ((node.axis === "row" || node.axis === "column") && first && second) return { axis: node.axis, first, second };
}
