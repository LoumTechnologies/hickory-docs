<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/dragDrop.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8a2f882c194c"></a>
## `apps/web/src/shell/dragDrop.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-8a2f882c194c-DropZone"></a>
### `DropZone`

Dragging a tab: where it may land, and what landing does to the tree.
Pure, like layout.ts, and for the same reason: "drop a tab in the right
third of a pane and a pane appears to its right" is geometry plus tree
surgery, and both are answerable at a desk. The pointer events that feed
this live in ShellView and stay dull.
See docs/specs/freeform/shell-layouts.md.

<storycode:copy id="f-8a2f882c194c-DropZone">
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

</storycode:copy>
<a id="f-8a2f882c194c-DropRect"></a>
### `DropRect`

The slice of a DOMRect the zone math needs; a plain object so tests
never touch the DOM.

<storycode:copy id="f-8a2f882c194c-DropRect">
/** The slice of a DOMRect the zone math needs; a plain object so tests
 * never touch the DOM. */
export interface DropRect {
  left: number;
  top: number;
  width: number;
  height: number;
}

</storycode:copy>
<a id="f-8a2f882c194c-zoneAt"></a>
### `zoneAt`

<storycode:copy id="f-8a2f882c194c-zoneAt">
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

</storycode:copy>
<a id="f-8a2f882c194c-dropCounter"></a>
### `dropCounter`

<storycode:copy id="f-8a2f882c194c-dropCounter">
// ---------------------------------------------------------------------------
// Tree surgery
// ---------------------------------------------------------------------------

let dropCounter = 0;
</storycode:copy>
<a id="f-8a2f882c194c-nextDropId"></a>
### `nextDropId`

Split ids born from a drop. A distinct prefix keeps them out of
layout.ts's `split-N` sequence without sharing its private counter.

<storycode:copy id="f-8a2f882c194c-nextDropId">
/** Split ids born from a drop. A distinct prefix keeps them out of
 * layout.ts's `split-N` sequence without sharing its private counter. */
function nextDropId(): string {
  dropCounter += 1;
  return `split-drop-${dropCounter}`;
}

</storycode:copy>
<a id="f-8a2f882c194c-holdsTree"></a>
### `holdsTree`

Whether a pane is the folder tree's. The tree is a place files open FROM,
not a place tabs land: joining it would bury the tree under a document.

<storycode:copy id="f-8a2f882c194c-holdsTree">
/** Whether a pane is the folder tree's. The tree is a place files open FROM,
 * not a place tabs land: joining it would bury the tree under a document. */
function holdsTree(pane: Pane): boolean {
  return pane.tabs.some((tab) => tab.kind === "tree");
}

</storycode:copy>
<a id="f-8a2f882c194c-swapPane"></a>
### `swapPane`

Rebuild the tree with one pane swapped for whatever `make` returns.
Never removes nodes — removal is closeTab's job, reused below.

<storycode:copy id="f-8a2f882c194c-swapPane">
/** Rebuild the tree with one pane swapped for whatever `make` returns.
 * Never removes nodes — removal is closeTab's job, reused below. */
function swapPane(node: Node, id: string, make: (pane: Pane) => Node): Node {
  if (node.type === "pane") return node.id === id ? make(node) : node;
  return { ...node, children: node.children.map((child) => swapPane(child, id, make)) };
}

</storycode:copy>
<a id="f-8a2f882c194c-moveTab"></a>
### `moveTab`

Drop a tab on a pane's body.
`center` joins the target pane, in front. An edge splits the target in
that direction and the dragged tab is the new pane, alone. Moving the last
tab out of a pane closes that pane — closeTab already knows every rule
about emptied panes, so the removal IS closeTab.
Two drops are refused as meaningless rather than performed as damage:
a tab on its own pane's center (it is already there), and a pane's only
tab on that same pane's edge (close the pane, then split what is gone).

<storycode:copy id="f-8a2f882c194c-moveTab">
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

</storycode:copy>
<a id="f-8a2f882c194c-moveTabToIndex"></a>
### `moveTabToIndex`

Drop a tab on a tab bar: insert at that index, active.
`index` is a caret position — before the tab currently at that index, or
after everything when it equals the tab count. Same-pane drops reorder;
dropping a tab back into its own slot changes nothing but which tab is
active, because a drag that went nowhere should not pretend otherwise.

<storycode:copy id="f-8a2f882c194c-moveTabToIndex">
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

</storycode:copy>
<a id="f-8a2f882c194c-clamp"></a>
### `clamp`

<storycode:copy id="f-8a2f882c194c-clamp">
function clamp(index: number, count: number): number {
  return Math.max(0, Math.min(Math.floor(index), count));
}

</storycode:copy>
<a id="f-8a2f882c194c-dropOnCollapsed"></a>
### `dropOnCollapsed`

Drop a tab onto a collapsed pane's icon strip: expand the strip, then join
its center.
The zone the pointer happened to be in is deliberately not consulted — a
36px strip has no thirds worth aiming at, so every drop on it means "put
this in there", never "split a sliver off it". The tree pane's refusal
still holds: expanding it just to bury it would be the same damage with an
extra step, so the drop leaves the layout — collapse included — alone.

<storycode:copy id="f-8a2f882c194c-dropOnCollapsed">
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
</storycode:copy>

### Assemble `apps/web/src/shell/dragDrop.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/dragDrop.ts"><storycode:paste select="#f-8a2f882c194c-DropZone"/><storycode:paste select="#f-8a2f882c194c-DropRect"/><storycode:paste select="#f-8a2f882c194c-zoneAt"/><storycode:paste select="#f-8a2f882c194c-dropCounter"/><storycode:paste select="#f-8a2f882c194c-nextDropId"/><storycode:paste select="#f-8a2f882c194c-holdsTree"/><storycode:paste select="#f-8a2f882c194c-swapPane"/><storycode:paste select="#f-8a2f882c194c-moveTab"/><storycode:paste select="#f-8a2f882c194c-moveTabToIndex"/><storycode:paste select="#f-8a2f882c194c-clamp"/><storycode:paste select="#f-8a2f882c194c-dropOnCollapsed"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-98444bdaa6cc"></a>
## `apps/web/src/shell/dragDrop.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-98444bdaa6cc-doc"></a>
### `doc`

<storycode:copy id="f-98444bdaa6cc-doc">
import { describe, expect, it } from "vitest";
import { dropOnCollapsed, moveTab, moveTabToIndex, zoneAt, type DropRect } from "./dragDrop";
import { collapsePane, freeform, open, panes, split, tab, type Layout, type Pane } from "./layout";

const doc = (path: string) => tab("document", path);

</storycode:copy>
<a id="f-98444bdaa6cc-openAll"></a>
### `openAll`

<storycode:copy id="f-98444bdaa6cc-openAll">
function openAll(layout: Layout, ...paths: string[]): Layout {
  return paths.reduce((current, path) => open(current, doc(path)), layout);
}

</storycode:copy>
<a id="f-98444bdaa6cc-twoPanes"></a>
### `twoPanes`

Two side-by-side panes: a.md + b.md on the left, c.md on the right.

<storycode:copy id="f-98444bdaa6cc-twoPanes">
/** Two side-by-side panes: a.md + b.md on the left, c.md on the right. */
function twoPanes(): { layout: Layout; left: Pane; right: Pane } {
  let layout = openAll(freeform(), "a.md", "b.md");
  const leftId = layout.focus;
  layout = split(layout, leftId, "row");
  layout = open(layout, doc("c.md"));
  const all = panes(layout.root);
  return {
    layout,
    left: all.find((pane) => pane.id === leftId)!,
    right: all.find((pane) => pane.id !== leftId)!,
  };
}

</storycode:copy>
<a id="f-98444bdaa6cc-targets"></a>
### `targets`

<storycode:copy id="f-98444bdaa6cc-targets">
function targets(pane: Pane): string[] {
  return pane.tabs.map((t) => t.target);
}

</storycode:copy>
<a id="f-98444bdaa6cc-paneById"></a>
### `paneById`

<storycode:copy id="f-98444bdaa6cc-paneById">
function paneById(layout: Layout, id: string): Pane | null {
  return panes(layout.root).find((pane) => pane.id === id) ?? null;
}

</storycode:copy>
<a id="f-98444bdaa6cc-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-98444bdaa6cc-expression_statement">
describe("zones: edge thirds split, the middle joins", () => {
  const rect: DropRect = { left: 100, top: 50, width: 300, height: 150 };

  it("gives the left and right thirds of the width to left and right", () => {
    expect(zoneAt(rect, 101, 125)).toBe("left");
    expect(zoneAt(rect, 199, 51)).toBe("left"); // corners belong to the sides
    expect(zoneAt(rect, 301, 125)).toBe("right");
    expect(zoneAt(rect, 399, 199)).toBe("right");
  });

  it("gives the top and bottom thirds of the height to top and bottom, in the middle band", () => {
    expect(zoneAt(rect, 250, 60)).toBe("top");
    expect(zoneAt(rect, 250, 190)).toBe("bottom");
  });

  it("calls the central third center", () => {
    expect(zoneAt(rect, 250, 125)).toBe("center");
  });

  it("shrugs at a degenerate rect instead of dividing by zero", () => {
    expect(zoneAt({ left: 0, top: 0, width: 0, height: 0 }, 5, 5)).toBe("center");
  });
});

</storycode:copy>
<a id="f-98444bdaa6cc-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-98444bdaa6cc-expression_statement-2">
describe("moveTab: center joins the target pane", () => {
  it("appends the tab and makes it active, focusing the target", () => {
    const { layout, left, right } = twoPanes();
    const after = moveTab(layout, left.id, left.tabs[0].id, right.id, "center");
    const target = paneById(after, right.id)!;
    expect(targets(target)).toEqual(["c.md", "a.md"]);
    expect(target.tabs[target.active].target).toBe("a.md");
    expect(after.focus).toBe(right.id);
    expect(targets(paneById(after, left.id)!)).toEqual(["b.md"]);
  });

  it("closes the source pane when its last tab leaves", () => {
    const { layout, left, right } = twoPanes();
    // Empty the right pane's neighbourhood: drag c.md (the right pane's
    // only tab) into the left pane.
    const after = moveTab(layout, right.id, right.tabs[0].id, left.id, "center");
    expect(panes(after.root)).toHaveLength(1);
    expect(after.root.type).toBe("pane"); // a split with one child is not a split
    expect(targets(paneById(after, left.id)!)).toEqual(["a.md", "b.md", "c.md"]);
  });

  it("does nothing when a tab is dropped on its own pane's center", () => {
    const { layout, left } = twoPanes();
    expect(moveTab(layout, left.id, left.tabs[0].id, left.id, "center")).toBe(layout);
  });
});

</storycode:copy>
<a id="f-98444bdaa6cc-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-98444bdaa6cc-expression_statement-3">
describe("moveTab: an edge splits the target", () => {
  it("puts the dragged tab in a new pane to the right", () => {
    const { layout, left, right } = twoPanes();
    const after = moveTab(layout, left.id, left.tabs[0].id, right.id, "right");
    expect(panes(after.root)).toHaveLength(3);
    const fresh = paneById(after, after.focus)!;
    expect(targets(fresh)).toEqual(["a.md"]);
    // The new pane sits AFTER the target in a row split.
    const parent = findParent(after, fresh.id)!;
    expect(parent.direction).toBe("row");
    expect(parent.children.map((c) => c.id)).toEqual([right.id, fresh.id]);
    expect(parent.sizes).toEqual([0.5, 0.5]);
  });

  it("puts the dragged tab before the target for left, and stacks for top/bottom", () => {
    for (const [zone, direction, freshFirst] of [
      ["left", "row", true],
      ["top", "column", true],
      ["bottom", "column", false],
    ] as const) {
      const { layout, left, right } = twoPanes();
      const after = moveTab(layout, left.id, left.tabs[0].id, right.id, zone);
      const parent = findParent(after, after.focus)!;
      expect(parent.direction).toBe(direction);
      const at = parent.children.findIndex((c) => c.id === after.focus);
      expect(at).toBe(freshFirst ? 0 : 1);
    }
  });

  it("splits a pane with its own tab, when it has more than one", () => {
    const { layout, left } = twoPanes();
    const after = moveTab(layout, left.id, left.tabs[0].id, left.id, "bottom");
    expect(panes(after.root)).toHaveLength(3);
    expect(targets(paneById(after, left.id)!)).toEqual(["b.md"]);
    expect(targets(paneById(after, after.focus)!)).toEqual(["a.md"]);
  });

  it("refuses a pane's only tab dropped on that same pane's edge", () => {
    // Performing it would close the pane and then split what is gone.
    const { layout, right } = twoPanes();
    expect(moveTab(layout, right.id, right.tabs[0].id, right.id, "left")).toBe(layout);
  });

  it("closes the source pane when its last tab leaves for an edge", () => {
    const { layout, left, right } = twoPanes();
    const after = moveTab(layout, right.id, right.tabs[0].id, left.id, "top");
    expect(panes(after.root)).toHaveLength(2);
    expect(targets(paneById(after, after.focus)!)).toEqual(["c.md"]);
    expect(targets(paneById(after, left.id)!)).toEqual(["a.md", "b.md"]);
  });

  it("keeps the target's region on the new pane", () => {
    // A pane split off a declared region still belongs to it, matching what
    // the split button does.
    let layout = freeform();
    layout = open(layout, doc("a.md"));
    const first = layout.focus;
    layout = split(layout, first, "row");
    layout = open(layout, doc("b.md"));
    const second = layout.focus;
    const withRegion: Layout = {
      ...layout,
      root: mapPanes(layout.root, (pane) =>
        pane.id === second ? { ...pane, region: "outputs" } : pane,
      ),
    };
    const source = paneById(withRegion, first)!;
    const after = moveTab(withRegion, first, source.tabs[0].id, second, "right");
    expect(paneById(after, after.focus)!.region).toBe("outputs");
  });
});

</storycode:copy>
<a id="f-98444bdaa6cc-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-98444bdaa6cc-expression_statement-4">
describe("moveTab: junk in, layout out", () => {
  it("ignores unknown panes and unknown tabs", () => {
    const { layout, left, right } = twoPanes();
    expect(moveTab(layout, "pane-nope", left.tabs[0].id, right.id, "center")).toBe(layout);
    expect(moveTab(layout, left.id, "tab-nope", right.id, "center")).toBe(layout);
    expect(moveTab(layout, left.id, left.tabs[0].id, "pane-nope", "center")).toBe(layout);
  });
});

</storycode:copy>
<a id="f-98444bdaa6cc-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-98444bdaa6cc-expression_statement-5">
describe("moveTabToIndex: dropping on a tab bar", () => {
  it("inserts into another pane at the caret, active", () => {
    const { layout, left, right } = twoPanes();
    const after = moveTabToIndex(layout, left.id, left.tabs[1].id, right.id, 0);
    const target = paneById(after, right.id)!;
    expect(targets(target)).toEqual(["b.md", "c.md"]);
    expect(target.active).toBe(0);
    expect(after.focus).toBe(right.id);
    expect(targets(paneById(after, left.id)!)).toEqual(["a.md"]);
  });

  it("closes the source pane when its last tab moves to another tab bar", () => {
    const { layout, left, right } = twoPanes();
    const after = moveTabToIndex(layout, right.id, right.tabs[0].id, left.id, 1);
    expect(panes(after.root)).toHaveLength(1);
    expect(targets(paneById(after, left.id)!)).toEqual(["a.md", "c.md", "b.md"]);
  });

  it("reorders within a pane, counting the caret with the tab still in place", () => {
    let layout = openAll(freeform(), "a.md", "b.md", "c.md");
    const pane = panes(layout.root)[0];
    // Drag a.md past b.md: caret index 2 with a still present lands after b.
    layout = moveTabToIndex(layout, pane.id, pane.tabs[0].id, pane.id, 2);
    const after = panes(layout.root)[0];
    expect(targets(after)).toEqual(["b.md", "a.md", "c.md"]);
    expect(after.active).toBe(1);
  });

  it("leaves the order alone when a tab is dropped back into its own slot", () => {
    let layout = openAll(freeform(), "a.md", "b.md", "c.md");
    const pane = panes(layout.root)[0];
    const before = targets(pane);
    // Both carets around the tab mean "where it already is".
    for (const index of [1, 2]) {
      const after = moveTabToIndex(layout, pane.id, pane.tabs[1].id, pane.id, index);
      expect(targets(panes(after.root)[0])).toEqual(before);
      expect(panes(after.root)[0].active).toBe(1);
    }
  });

  it("clamps a caret past the end", () => {
    const { layout, left, right } = twoPanes();
    const after = moveTabToIndex(layout, left.id, left.tabs[0].id, right.id, 99);
    expect(targets(paneById(after, right.id)!)).toEqual(["c.md", "a.md"]);
  });

  it("ignores unknown panes and tabs", () => {
    const { layout, left, right } = twoPanes();
    expect(moveTabToIndex(layout, "pane-nope", left.tabs[0].id, right.id, 0)).toBe(layout);
    expect(moveTabToIndex(layout, left.id, "tab-nope", right.id, 0)).toBe(layout);
    expect(moveTabToIndex(layout, left.id, left.tabs[0].id, "pane-nope", 0)).toBe(layout);
  });
});

</storycode:copy>
<a id="f-98444bdaa6cc-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-98444bdaa6cc-expression_statement-6">
describe("the folder tree pane takes no arrivals", () => {
  // The document shell keeps the tree at the left as a normal pane; joining
  // a tab INTO it would bury the tree, so center drops and tab-bar drops are
  // refused. The edges still split, so tabs can be arranged around it.
  function withTreePane(): { layout: Layout; tree: Pane; docs: Pane } {
    let layout = openAll(freeform(), "a.md", "b.md");
    const docsId = layout.focus;
    layout = split(layout, docsId, "row");
    layout = open(layout, tab("tree", "folder", "Files"));
    const all = panes(layout.root);
    return {
      layout,
      tree: all.find((pane) => pane.id !== docsId)!,
      docs: all.find((pane) => pane.id === docsId)!,
    };
  }

  it("refuses a center drop onto the tree pane", () => {
    const { layout, tree, docs } = withTreePane();
    expect(moveTab(layout, docs.id, docs.tabs[0].id, tree.id, "center")).toBe(layout);
  });

  it("refuses a tab-bar drop into the tree pane", () => {
    const { layout, tree, docs } = withTreePane();
    expect(moveTabToIndex(layout, docs.id, docs.tabs[0].id, tree.id, 0)).toBe(layout);
  });

  it("still splits on the tree pane's edges, so tabs can land beside it", () => {
    const { layout, tree, docs } = withTreePane();
    const after = moveTab(layout, docs.id, docs.tabs[0].id, tree.id, "right");
    expect(panes(after.root)).toHaveLength(3);
    expect(targets(paneById(after, after.focus)!)).toEqual(["a.md"]);
    expect(targets(paneById(after, tree.id)!)).toEqual(["folder"]);
  });
});

</storycode:copy>
<a id="f-98444bdaa6cc-SplitNode"></a>
### `SplitNode`

Local helpers: walking for a split parent, mapping panes in place.

<storycode:copy id="f-98444bdaa6cc-SplitNode">
// Local helpers: walking for a split parent, mapping panes in place.
type SplitNode = Extract<Layout["root"], { type: "split" }>;

</storycode:copy>
<a id="f-98444bdaa6cc-findParent"></a>
### `findParent`

<storycode:copy id="f-98444bdaa6cc-findParent">
function findParent(layout: Layout, paneId: string): SplitNode | null {
  const walk = (node: Layout["root"]): SplitNode | null => {
    if (node.type === "pane") return null;
    if (node.children.some((child) => child.id === paneId)) return node;
    for (const child of node.children) {
      const found = walk(child);
      if (found) return found;
    }
    return null;
  };
  return walk(layout.root);
}

</storycode:copy>
<a id="f-98444bdaa6cc-mapPanes"></a>
### `mapPanes`

<storycode:copy id="f-98444bdaa6cc-mapPanes">
function mapPanes(node: Layout["root"], make: (pane: Pane) => Pane): Layout["root"] {
  if (node.type === "pane") return make(node);
  return { ...node, children: node.children.map((child) => mapPanes(child, make)) };
}

</storycode:copy>
<a id="f-98444bdaa6cc-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-98444bdaa6cc-expression_statement-7">
describe("dropOnCollapsed: a strip expands and joins, never splits", () => {
  function withCollapsedRight(): { layout: Layout; left: Pane; right: Pane } {
    const { layout, left, right } = twoPanes();
    const collapsed = collapsePane(layout, right.id);
    return {
      layout: collapsed,
      left: paneById(collapsed, left.id)!,
      right: paneById(collapsed, right.id)!,
    };
  }

  it("expands the strip and drops the tab in its center", () => {
    const { layout, left, right } = withCollapsedRight();
    const after = dropOnCollapsed(layout, left.id, left.tabs[0].id, right.id);
    const target = paneById(after, right.id)!;
    expect(target.collapsed).toBe(false);
    expect(targets(target)).toEqual(["c.md", "a.md"]);
    expect(target.tabs[target.active].target).toBe("a.md");
    expect(after.focus).toBe(right.id);
  });

  it("guards the edge zones: there is no way to split a 36px strip", () => {
    // The API takes no zone at all — the guard is that edges cannot even be
    // expressed, so a drop near a strip's edge joins instead of splitting.
    const { layout, left, right } = withCollapsedRight();
    const after = dropOnCollapsed(layout, left.id, left.tabs[0].id, right.id);
    expect(panes(after.root)).toHaveLength(2);
  });

  it("is a no-op on a pane that is not collapsed", () => {
    const { layout, left, right } = twoPanes();
    expect(dropOnCollapsed(layout, left.id, left.tabs[0].id, right.id)).toBe(layout);
  });

  it("leaves a collapsed tree pane collapsed: the refusal has no side effects", () => {
    let layout = open(freeform(), tab("tree", "/repo"));
    const treeId = layout.focus;
    layout = split(layout, treeId, "row");
    layout = open(layout, tab("document", "a.md"));
    layout = open(layout, tab("document", "b.md"));
    const docPane = layout.focus;
    const collapsed = collapsePane(layout, treeId);
    const moving = paneById(collapsed, docPane)!.tabs[0];
    const after = dropOnCollapsed(collapsed, docPane, moving.id, treeId);
    expect(after).toBe(collapsed);
    expect(paneById(after, treeId)?.collapsed).toBe(true);
  });
});
</storycode:copy>

### Assemble `apps/web/src/shell/dragDrop.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/dragDrop.test.ts"><storycode:paste select="#f-98444bdaa6cc-doc"/><storycode:paste select="#f-98444bdaa6cc-openAll"/><storycode:paste select="#f-98444bdaa6cc-twoPanes"/><storycode:paste select="#f-98444bdaa6cc-targets"/><storycode:paste select="#f-98444bdaa6cc-paneById"/><storycode:paste select="#f-98444bdaa6cc-expression_statement"/><storycode:paste select="#f-98444bdaa6cc-expression_statement-2"/><storycode:paste select="#f-98444bdaa6cc-expression_statement-3"/><storycode:paste select="#f-98444bdaa6cc-expression_statement-4"/><storycode:paste select="#f-98444bdaa6cc-expression_statement-5"/><storycode:paste select="#f-98444bdaa6cc-expression_statement-6"/><storycode:paste select="#f-98444bdaa6cc-SplitNode"/><storycode:paste select="#f-98444bdaa6cc-findParent"/><storycode:paste select="#f-98444bdaa6cc-mapPanes"/><storycode:paste select="#f-98444bdaa6cc-expression_statement-7"/></storycode:file>


</storycode:doc>
