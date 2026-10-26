<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/ribbonNavigation.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-03960f2f41be"></a>
## `apps/web/src/shell/ribbonNavigation.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-03960f2f41be-Direction"></a>
### `Direction`

<storycode:copy id="f-03960f2f41be-Direction">
import { focus, paneById, panes, split, type Layout } from "./layout";
import type { RibbonTarget } from "./Ribbons";
import { samePath } from "../lib/paths";
import { visibleElementRect } from "../lib/visibleElementRect";

export type Direction = "left" | "right" | "up" | "down";
</storycode:copy>
<a id="f-03960f2f41be-RibbonOpenIntent"></a>
### `RibbonOpenIntent`

<storycode:copy id="f-03960f2f41be-RibbonOpenIntent">
export interface RibbonOpenIntent { from: string; into?: string; direction: Direction }
</storycode:copy>
<a id="f-03960f2f41be-PaneBox"></a>
### `PaneBox`

<storycode:copy id="f-03960f2f41be-PaneBox">
export interface PaneBox { id: string; left: number; right: number; top: number; bottom: number }
</storycode:copy>
<a id="f-03960f2f41be-directionBetween"></a>
### `directionBetween`

<storycode:copy id="f-03960f2f41be-directionBetween">
export function directionBetween(a: Pick<DOMRect, "left" | "right" | "top" | "bottom">, b: Pick<DOMRect, "left" | "right" | "top" | "bottom">): Direction {
  const dx = b.left + b.right - a.left - a.right, dy = b.top + b.bottom - a.top - a.bottom;
  return Math.abs(dx) >= Math.abs(dy) ? dx < 0 ? "left" : "right" : dy < 0 ? "up" : "down";
}

</storycode:copy>
<a id="f-03960f2f41be-ribbonOpenIntent"></a>
### `ribbonOpenIntent`

Capture the drawn destination before opening changes focus or pane geometry.

<storycode:copy id="f-03960f2f41be-ribbonOpenIntent">
/** Capture the drawn destination before opening changes focus or pane geometry. */
export function ribbonOpenIntent(container: HTMLElement | null, source: HTMLElement | undefined, target: RibbonTarget): RibbonOpenIntent | undefined {
  const origin = source?.closest<HTMLElement>("[data-shell-pane-id]");
  if (!container || !origin) return;
  const tab = [...container.querySelectorAll<HTMLElement>("[data-shell-tab-target]")].find(el =>
    samePath(el.dataset.shellTabSource ?? "", target.path) || ["document", "generated", "file", "untitled"].includes(el.dataset.shellTabKind ?? "") &&
    (samePath(el.dataset.shellTabTarget ?? "", target.path) || el.dataset.shellTabBuffer === target.path));
  const spatial = [...container.querySelectorAll<HTMLElement>("[data-spatial-key]")].find(el => samePath(el.dataset.shellTarget ?? "", target.path));
  const destination = spatial ?? tab?.closest<HTMLElement>("[data-shell-pane-id]");
  const tree = [...container.querySelectorAll<HTMLElement>("[data-tree-path]")].find(el => samePath(el.dataset.treePath ?? "", target.path));
  const port = [...container.querySelectorAll<HTMLElement>("[data-ribbon-port]")].find(el => el.dataset.ribbonPort?.endsWith(`:${target.path}`));
  const rect = destination?.getBoundingClientRect() ?? (tree ? visibleElementRect(tree) : null) ?? port?.getBoundingClientRect();
  if (!rect) return;
  return { from: origin.dataset.shellPaneId!, into: destination?.dataset.shellPaneId,
    direction: directionBetween(origin.getBoundingClientRect(), rect) };
}

</storycode:copy>
<a id="f-03960f2f41be-paneBoxes"></a>
### `paneBoxes`

<storycode:copy id="f-03960f2f41be-paneBoxes">
export function paneBoxes(container: HTMLElement | null): PaneBox[] {
  return [...container?.querySelectorAll<HTMLElement>("[data-shell-pane-id]") ?? []].map(el => {
    const { left, right, top, bottom } = el.getBoundingClientRect();
    return { id: el.dataset.shellPaneId!, left, right, top, bottom };
  });
}

</storycode:copy>
<a id="f-03960f2f41be-prepareRibbonOpen"></a>
### `prepareRibbonOpen`

Existing targets win; otherwise open in the nearest editor in the connection's direction.

<storycode:copy id="f-03960f2f41be-prepareRibbonOpen">
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
</storycode:copy>

### Assemble `apps/web/src/shell/ribbonNavigation.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/ribbonNavigation.ts"><storycode:paste select="#f-03960f2f41be-Direction"/><storycode:paste select="#f-03960f2f41be-RibbonOpenIntent"/><storycode:paste select="#f-03960f2f41be-PaneBox"/><storycode:paste select="#f-03960f2f41be-directionBetween"/><storycode:paste select="#f-03960f2f41be-ribbonOpenIntent"/><storycode:paste select="#f-03960f2f41be-paneBoxes"/><storycode:paste select="#f-03960f2f41be-prepareRibbonOpen"/></storycode:file>

</storycode:doc>
