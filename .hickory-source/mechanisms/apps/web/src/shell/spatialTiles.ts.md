<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/spatialTiles.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-cdea607538fb"></a>
## `apps/web/src/shell/spatialTiles.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Spatial Views Preserve The Editor](../../../../../guarantees/shell/spatial-views-preserve-the-editor/contract.md).

<a id="f-cdea607538fb-PRIMARY"></a>
### `PRIMARY`

<storycode:copy id="f-cdea607538fb-PRIMARY">
import type { Rect } from "./spatialLayout";

/** The primary tab is a slot, so switching tabs does not discard its neighbors. */
export const PRIMARY = "@center";
</storycode:copy>
<a id="f-cdea607538fb-Tile"></a>
### `Tile`

<storycode:copy id="f-cdea607538fb-Tile">
export type Tile = string | { axis: "row" | "column"; first: Tile; second: Tile };
</storycode:copy>
<a id="f-cdea607538fb-DockEdge"></a>
### `DockEdge`

<storycode:copy id="f-cdea607538fb-DockEdge">
export type DockEdge = "left" | "right" | "top" | "bottom" | "center";
</storycode:copy>
<a id="f-cdea607538fb-pruneTiles"></a>
### `pruneTiles`

<storycode:copy id="f-cdea607538fb-pruneTiles">
export function pruneTiles(tile: Tile, keep: (key: string) => boolean): Tile | undefined {
  if (typeof tile === "string") return keep(tile) ? tile : undefined;
  const first = pruneTiles(tile.first, keep), second = pruneTiles(tile.second, keep);
  return first && second ? { ...tile, first, second } : first ?? second;
}
</storycode:copy>
<a id="f-cdea607538fb-dockTile"></a>
### `dockTile`

<storycode:copy id="f-cdea607538fb-dockTile">
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
</storycode:copy>
<a id="f-cdea607538fb-tileRects"></a>
### `tileRects`

<storycode:copy id="f-cdea607538fb-tileRects">
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
</storycode:copy>
<a id="f-cdea607538fb-dockHit"></a>
### `dockHit`

<storycode:copy id="f-cdea607538fb-dockHit">
export function dockHit(rect: Rect, x: number, y: number): DockEdge {
  const distances = [(x - rect.x) / rect.w, (rect.x + rect.w - x) / rect.w, (y - rect.y) / rect.h, (rect.y + rect.h - y) / rect.h];
  const nearest = Math.min(...distances);
  return nearest < .23 ? (["left", "right", "top", "bottom"] as const)[distances.indexOf(nearest)] : "center";
}
</storycode:copy>
<a id="f-cdea607538fb-readTiles"></a>
### `readTiles`

<storycode:copy id="f-cdea607538fb-readTiles">
export function readTiles(value: unknown, depth = 0): Tile | undefined {
  if (typeof value === "string") return value;
  if (!value || typeof value !== "object" || depth > 12) return;
  const node = value as Record<string, unknown>, first = readTiles(node.first, depth + 1), second = readTiles(node.second, depth + 1);
  if ((node.axis === "row" || node.axis === "column") && first && second) return { axis: node.axis, first, second };
}
</storycode:copy>

### Assemble `apps/web/src/shell/spatialTiles.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/spatialTiles.ts"><storycode:paste select="#f-cdea607538fb-PRIMARY"/><storycode:paste select="#f-cdea607538fb-Tile"/><storycode:paste select="#f-cdea607538fb-DockEdge"/><storycode:paste select="#f-cdea607538fb-pruneTiles"/><storycode:paste select="#f-cdea607538fb-dockTile"/><storycode:paste select="#f-cdea607538fb-tileRects"/><storycode:paste select="#f-cdea607538fb-dockHit"/><storycode:paste select="#f-cdea607538fb-readTiles"/></storycode:file>

</storycode:doc>
