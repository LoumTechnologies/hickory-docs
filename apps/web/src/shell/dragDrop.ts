// Dragging a tab: where it may land, and what landing does to the tree.
//
// Pure, like layout.ts, and for the same reason: "drop a tab in the right
// third of a pane and a pane appears to its right" is geometry plus tree
// surgery, and both are answerable at a desk. The pointer events that feed
// this live in ShellView and stay dull.
//
// See docs/specs/freeform/shell-layouts.md.

import {
  closeTab,
  emptyPane,
  expandPane,
  paneById,
  type Layout,
  type Node,
  type Pane,
  type Tab,
} from "./layout";

/**
 * Where over a pane a drop would land.
 *
 * The edge thirds split; the middle joins. Width is judged before height so
 * the four corners belong to left/right — a row split is the common case,
 * and giving the wider strips to it makes the common thing the easy aim.
 */
export type DropZone = "left" | "right" | "top" | "bottom" | "center";

/** The slice of a DOMRect the zone math needs; a plain object so tests
 * never touch the DOM. */
export interface DropRect {
  left: number;
  top: number;
  width: number;
  height: number;
}

export function zoneAt(rect: DropRect, x: number, y: number): DropZone {
  if (rect.width <= 0 || rect.height <= 0) return "center";
  const fx = (x - rect.left) / rect.width;
  const fy = (y - rect.top) / rect.height;
  if (fx < 1 / 3) return "left";
  if (fx > 2 / 3) return "right";
  if (fy < 1 / 3) return "top";
  if (fy > 2 / 3) return "bottom";
  return "center";
}

// ---------------------------------------------------------------------------
// Tree surgery
// ---------------------------------------------------------------------------

let dropCounter = 0;
/** Split ids born from a drop. A distinct prefix keeps them out of
 * layout.ts's `split-N` sequence without sharing its private counter. */
function nextDropId(): string {
  dropCounter += 1;
  return `split-drop-${dropCounter}`;
}

/** Whether a pane is the folder tree's. The tree is a place files open FROM,
 * not a place tabs land: joining it would bury the tree under a document. */
function holdsTree(pane: Pane): boolean {
  return pane.tabs.some((tab) => tab.kind === "tree");
}

/** Rebuild the tree with one pane swapped for whatever `make` returns.
 * Never removes nodes — removal is closeTab's job, reused below. */
function swapPane(node: Node, id: string, make: (pane: Pane) => Node): Node {
  if (node.type === "pane") return node.id === id ? make(node) : node;
  return { ...node, children: node.children.map((child) => swapPane(child, id, make)) };
}

/**
 * Drop a tab on a pane's body.
 *
 * `center` joins the target pane, in front. An edge splits the target in
 * that direction and the dragged tab is the new pane, alone. Moving the last
 * tab out of a pane closes that pane — closeTab already knows every rule
 * about emptied panes, so the removal IS closeTab.
 *
 * Two drops are refused as meaningless rather than performed as damage:
 * a tab on its own pane's center (it is already there), and a pane's only
 * tab on that same pane's edge (close the pane, then split what is gone).
 */
export function moveTab(
  layout: Layout,
  fromPaneId: string,
  tabId: string,
  toPaneId: string,
  zone: DropZone,
): Layout {
  const from = paneById(layout, fromPaneId);
  const to = paneById(layout, toPaneId);
  const moving = from?.tabs.find((tab) => tab.id === tabId);
  if (!from || !to || !moving) return layout;
  if (fromPaneId === toPaneId && (zone === "center" || from.tabs.length === 1)) return layout;
  // Dropping INTO the tree pane's center is refused, not performed: the tree
  // is chrome, and a tab joined to it would hide it. The edges still split —
  // tabs can be arranged around the tree, never onto it.
  if (holdsTree(to) && zone === "center") return layout;

  const removed = closeTab(layout, fromPaneId, tabId);

  if (zone === "center") {
    const root = swapPane(removed.root, toPaneId, (pane) => ({
      ...pane,
      tabs: [...pane.tabs, moving],
      active: pane.tabs.length,
    }));
    return { root, focus: toPaneId };
  }

  const fresh: Pane = { ...emptyPane(to.region), tabs: [moving], active: 0 };
  const direction = zone === "left" || zone === "right" ? "row" : "column";
  const first = zone === "left" || zone === "top";
  const root = swapPane(removed.root, toPaneId, (pane) => ({
    type: "split",
    id: nextDropId(),
    direction,
    children: first ? [fresh, pane] : [pane, fresh],
    sizes: [0.5, 0.5],
  }));
  return { root, focus: fresh.id };
}

/**
 * Drop a tab on a tab bar: insert at that index, active.
 *
 * `index` is a caret position — before the tab currently at that index, or
 * after everything when it equals the tab count. Same-pane drops reorder;
 * dropping a tab back into its own slot changes nothing but which tab is
 * active, because a drag that went nowhere should not pretend otherwise.
 */
export function moveTabToIndex(
  layout: Layout,
  fromPaneId: string,
  tabId: string,
  toPaneId: string,
  index: number,
): Layout {
  const from = paneById(layout, fromPaneId);
  const to = paneById(layout, toPaneId);
  const moving = from?.tabs.find((tab) => tab.id === tabId);
  if (!from || !to || !moving) return layout;
  // The tree pane's tab bar takes no arrivals, for the same reason its
  // center takes no drops. Reordering within it stays allowed (harmless).
  if (holdsTree(to) && fromPaneId !== toPaneId) return layout;

  if (fromPaneId === toPaneId) {
    const current = from.tabs.findIndex((tab) => tab.id === tabId);
    const at = clamp(index, from.tabs.length);
    // The caret counts slots WITH the dragged tab still in place; taking it
    // out shifts everything after it left by one.
    const settled = current < at ? at - 1 : at;
    const tabs = from.tabs.filter((tab) => tab.id !== tabId);
    tabs.splice(settled, 0, moving);
    const root = swapPane(layout.root, fromPaneId, (pane) => ({ ...pane, tabs, active: settled }));
    return { root, focus: fromPaneId };
  }

  const removed = closeTab(layout, fromPaneId, tabId);
  const at = clamp(index, to.tabs.length);
  const root = swapPane(removed.root, toPaneId, (pane) => {
    const tabs: Tab[] = [...pane.tabs];
    tabs.splice(at, 0, moving);
    return { ...pane, tabs, active: at };
  });
  return { root, focus: toPaneId };
}

function clamp(index: number, count: number): number {
  return Math.max(0, Math.min(Math.floor(index), count));
}

/**
 * Drop a tab onto a collapsed pane's icon strip: expand the strip, then join
 * its center.
 *
 * The zone the pointer happened to be in is deliberately not consulted — a
 * 36px strip has no thirds worth aiming at, so every drop on it means "put
 * this in there", never "split a sliver off it". The tree pane's refusal
 * still holds: expanding it just to bury it would be the same damage with an
 * extra step, so the drop leaves the layout — collapse included — alone.
 */
export function dropOnCollapsed(
  layout: Layout,
  fromPaneId: string,
  tabId: string,
  toPaneId: string,
): Layout {
  const to = paneById(layout, toPaneId);
  if (!to || !to.collapsed) return layout;
  const expanded = expandPane(layout, toPaneId);
  const moved = moveTab(expanded, fromPaneId, tabId, toPaneId, "center");
  // moveTab hands back its input untouched when it refuses; a refused drop
  // must not leave the strip expanded as a side effect.
  return moved === expanded ? layout : moved;
}
