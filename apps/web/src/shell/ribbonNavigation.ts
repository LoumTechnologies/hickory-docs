import { focus, paneById, panes, split, type Layout } from "./layout";
import type { RibbonTarget } from "./Ribbons";
import { samePath } from "../lib/paths";
import { visibleElementRect } from "../lib/visibleElementRect";

export type Direction = "left" | "right" | "up" | "down";
export interface RibbonOpenIntent { from: string; into?: string; direction: Direction }
export interface PaneBox { id: string; left: number; right: number; top: number; bottom: number }
export function directionBetween(a: Pick<DOMRect, "left" | "right" | "top" | "bottom">, b: Pick<DOMRect, "left" | "right" | "top" | "bottom">): Direction {
  const dx = b.left + b.right - a.left - a.right, dy = b.top + b.bottom - a.top - a.bottom;
  return Math.abs(dx) >= Math.abs(dy) ? dx < 0 ? "left" : "right" : dy < 0 ? "up" : "down";
}

/** Capture the drawn destination before opening changes focus or pane geometry. */
export function ribbonOpenIntent(container: HTMLElement | null, source: HTMLElement | undefined, target: RibbonTarget): RibbonOpenIntent | undefined {
  const origin = source?.closest<HTMLElement>("[data-shell-pane-id]");
  if (!container || !origin) return;
  const tab = [...container.querySelectorAll<HTMLElement>("[data-shell-tab-target]")].find(el =>
    samePath(el.dataset.shellTabSource ?? "", target.path) || ["document", "generated", "file", "untitled"].includes(el.dataset.shellTabKind ?? "") &&
    (samePath(el.dataset.shellTabTarget ?? "", target.path) || el.dataset.shellTabBuffer === target.path));
  const destination = tab?.closest<HTMLElement>("[data-shell-pane-id]");
  const tree = [...container.querySelectorAll<HTMLElement>("[data-tree-path]")].find(el => samePath(el.dataset.treePath ?? "", target.path));
  const port = [...container.querySelectorAll<HTMLElement>("[data-ribbon-port]")].find(el => el.dataset.ribbonPort?.endsWith(`:${target.path}`));
  const rect = destination?.getBoundingClientRect() ?? (tree ? visibleElementRect(tree) : null) ?? port?.getBoundingClientRect();
  if (!rect) return;
  return { from: origin.dataset.shellPaneId!, into: destination?.dataset.shellPaneId,
    direction: directionBetween(origin.getBoundingClientRect(), rect) };
}

export function paneBoxes(container: HTMLElement | null): PaneBox[] {
  return [...container?.querySelectorAll<HTMLElement>("[data-shell-pane-id]") ?? []].map(el => {
    const { left, right, top, bottom } = el.getBoundingClientRect();
    return { id: el.dataset.shellPaneId!, left, right, top, bottom };
  });
}

/** Existing targets win; otherwise open in the nearest editor in the connection's direction. */
export function prepareRibbonOpen(layout: Layout, boxes: readonly PaneBox[], intent?: RibbonOpenIntent): Layout {
  if (!intent) return layout;
  if (intent.into && paneById(layout, intent.into)) return focus(layout, intent.into);
  const origin = boxes.find(box => box.id === intent.from);
  if (!origin || !paneById(layout, intent.from)) return layout;
  const horizontal = intent.direction === "left" || intent.direction === "right";
  const sign = intent.direction === "left" || intent.direction === "up" ? -1 : 1;
  const x = (origin.left + origin.right) / 2, y = (origin.top + origin.bottom) / 2;
  const candidates = panes(layout.root).filter(pane => !pane.collapsed && pane.id !== intent.from &&
    (pane.tabs.length === 0 || ["document", "generated", "file", "untitled"].includes(pane.tabs[pane.active]?.kind)));
  const ranked = candidates.flatMap(pane => {
    const box = boxes.find(box => box.id === pane.id);
    if (!box) return [];
    const dx = (box.left + box.right) / 2 - x, dy = (box.top + box.bottom) / 2 - y;
    const distance = sign * (horizontal ? dx : dy);
    if (distance <= 0) return [];
    // Prefer alignment with the arrow before a more distant diagonal editor.
    const cross = horizontal ? Math.max(0, box.top - origin.bottom, origin.top - box.bottom) : Math.max(0, box.left - origin.right, origin.left - box.right);
    return [{ id: pane.id, score: distance + cross * 2 }];
  }).sort((a, b) => a.score - b.score);
  if (ranked[0]) return focus(layout, ranked[0].id);
  return split(layout, intent.from, horizontal ? "row" : "column", sign < 0);
}
