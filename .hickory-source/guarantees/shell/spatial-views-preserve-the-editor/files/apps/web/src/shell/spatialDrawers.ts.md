<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/spatialDrawers.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Spatial Views Preserve The Editor](../../../../../contract.md)

Captured SHA-256: `220b132f642df7e981e9efa4d2bc45c375b821a2babb4c5c2b1394bea4be106a`.

<book:entry path="apps/web/src/shell/spatialDrawers.ts">
import { samePath } from "../lib/paths";
import type { SpatialSides } from "../lib/spatialSides";
import { spatialRegions, type SpatialState, type Rect, type Side } from "./spatialLayout";
import type { SpatialRelation } from "./spatialRelations";
import type { SpatialItem } from "./SpatialController";

/** Room for a filename column and a readable preview beside it. */
export const FILES_PREVIEW_BESIDE_MIN_WIDTH = 360;

export interface DrawerPresentation {
  available: SpatialSides;
  visible: SpatialSides;
  files: SpatialSides;
  contexts: SpatialSides;
  pinned: SpatialSides;
  keys: readonly string[];
}

/** Open tabs alone are navigation, not contextual content. */
export function drawerContent<Item extends SpatialItem>(items: readonly Item[], center: SpatialItem | undefined,
  relations: readonly SpatialRelation[], views: SpatialState["views"], pending: ReadonlySet<string>, sources: readonly SpatialItem[] = center ? [center] : []) {
  return items.filter(item => item.key !== center?.key && (views[item.key]?.pin || views[item.key]?.revealed || pending.has(item.key)
    || relations.some(relation => sources.some(source => samePath(relation.fromPath, source.path) && samePath(relation.toPath, item.path)
      || samePath(relation.toPath, source.path) && samePath(relation.fromPath, item.path)))));
}

/** Collapsed drawers consume no geometry. Overlays use the center's rectangle;
 * only explicit drawer pins reduce its width. */
export function workspaceRegions(width: number, height: number, state: SpatialState) {
  const drawer = state.drawers;
  if (!drawer) return { ...spatialRegions(width, height, state.columns, state.sides), filesBelow: { left: false, right: false } };
  const wanted = (side: Side) => Math.min(520, Math.max(280, width * (state.columns[side === "left" ? 0 : 1] + .13)));
  const pins = Number(drawer.visible.left && drawer.pinned.left) + Number(drawer.visible.right && drawer.pinned.right);
  const reserved = (side: Side) => drawer.visible[side] && drawer.pinned[side] ? Math.min(wanted(side), width * (pins === 2 ? .32 : .45)) : 0;
  const left = reserved("left"), right = reserved("right");
  const center: Rect = { x: left, y: 0, w: width - left - right, h: height };
  const bounds = (side: Side): Rect => {
    const w = drawer.visible[side] ? reserved(side) || Math.min(wanted(side), center.w * .9) : 0;
    const y = drawer.pinned[side] ? 0 : 36;
    const origin = side === "left" ? reserved(side) ? 0 : center.x : reserved(side) ? width - right : center.x + center.w - w;
    const offset = state.drawerOffsets?.[side];
    const x = Math.max(0, Math.min(width - w, origin + (offset?.x ?? 0)));
    const top = Math.max(0, Math.min(height - Math.min(160, height), y + (offset?.y ?? 0)));
    return { x, y: top, w, h: Math.max(0, height - top) };
  };
  const leftBounds = bounds("left"), rightBounds = bounds("right");
  const filesBelow = { left: drawer.files.left && drawer.contexts.left && leftBounds.w < FILES_PREVIEW_BESIDE_MIN_WIDTH,
    right: drawer.files.right && drawer.contexts.right && rightBounds.w < FILES_PREVIEW_BESIDE_MIN_WIDTH };
  const rail = (side: Side, box: Rect): Rect => {
    const w = drawer.visible[side] && drawer.files[side] ? drawer.contexts[side] && !filesBelow[side] ? Math.min(160, box.w * .4) : box.w : 0;
    return { x: box.x, y: box.y + 36, w, h: Math.max(0, box.h - 36) };
  };
  const leftRail = rail("left", leftBounds), rightRail = rail("right", rightBounds);
  const lane = (side: Side, box: Rect, rail: Rect): Rect => {
    const inset = filesBelow[side] ? 8 : 0;
    return { x: box.x + (filesBelow[side] ? inset : rail.w), y: box.y + 36,
      w: Math.max(0, filesBelow[side] ? box.w - 2 * inset : box.w - rail.w), h: Math.max(0, box.h - 36) };
  };
  return { small: width < 650, center, leftRail, rightRail, gap: 8, filesBelow,
    left: lane("left", leftBounds, leftRail), right: lane("right", rightBounds, rightRail),
    drawers: { left: leftBounds, right: rightBounds } };
}
</book:entry>

</book:doc>
