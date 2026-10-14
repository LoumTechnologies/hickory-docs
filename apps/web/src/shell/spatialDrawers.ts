import { samePath } from "../lib/paths";
import type { SpatialSides } from "../lib/spatialSides";
import { spatialRegions, type SpatialState, type Rect, type Side } from "./spatialLayout";
import type { SpatialRelation } from "./spatialRelations";
import type { SpatialItem } from "./SpatialController";

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
  relations: readonly SpatialRelation[], views: SpatialState["views"], pending: ReadonlySet<string>) {
  return items.filter(item => item.key !== center?.key && (views[item.key]?.pin || views[item.key]?.revealed || pending.has(item.key)
    || relations.some(relation => center && (samePath(relation.fromPath, center.path) && samePath(relation.toPath, item.path)
      || samePath(relation.toPath, center.path) && samePath(relation.fromPath, item.path)))));
}

/** Collapsed drawers consume no geometry. Overlays use the center's rectangle;
 * only explicit drawer pins reduce its width. */
export function workspaceRegions(width: number, height: number, state: SpatialState) {
  const drawer = state.drawers;
  if (!drawer) return spatialRegions(width, height, state.columns, state.sides);
  const wanted = (side: Side) => Math.min(520, Math.max(280, width * (state.columns[side === "left" ? 0 : 1] + .13)));
  const pins = Number(drawer.visible.left && drawer.pinned.left) + Number(drawer.visible.right && drawer.pinned.right);
  const reserved = (side: Side) => drawer.visible[side] && drawer.pinned[side] ? Math.min(wanted(side), width * (pins === 2 ? .32 : .45)) : 0;
  const left = reserved("left"), right = reserved("right");
  const center: Rect = { x: left, y: 0, w: width - left - right, h: height };
  const bounds = (side: Side): Rect => {
    const w = drawer.visible[side] ? reserved(side) || Math.min(wanted(side), center.w * .9) : 0;
    const y = drawer.pinned[side] ? 0 : 36;
    return { x: side === "left" ? reserved(side) ? 0 : center.x : reserved(side) ? width - right : center.x + center.w - w, y, w, h: Math.max(0, height - y) };
  };
  const leftBounds = bounds("left"), rightBounds = bounds("right");
  const rail = (side: Side, box: Rect): Rect => {
    const w = drawer.visible[side] && drawer.files[side] ? drawer.contexts[side] ? Math.min(160, box.w * .4) : box.w : 0;
    return { x: side === "left" ? box.x : box.x + box.w - w, y: box.y + 36, w, h: Math.max(0, box.h - 36) };
  };
  const leftRail = rail("left", leftBounds), rightRail = rail("right", rightBounds);
  return { small: width < 650, center, leftRail, rightRail, gap: 8,
    left: { x: leftBounds.x + leftRail.w, y: leftBounds.y + 36, w: leftBounds.w - leftRail.w, h: Math.max(0, leftBounds.h - 36) },
    right: { x: rightBounds.x, y: rightBounds.y + 36, w: rightBounds.w - rightRail.w, h: Math.max(0, rightBounds.h - 36) },
    drawers: { left: leftBounds, right: rightBounds } };
}
