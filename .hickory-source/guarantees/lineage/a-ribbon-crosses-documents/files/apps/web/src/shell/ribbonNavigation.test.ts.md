<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/ribbonNavigation.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Ribbon Crosses Documents, Because Lineage Does](../../../../../contract.md)

Captured SHA-256: `c16d830d7620151426010be74ebd7ad1a80c7b216840dbb1ec069925cf496ce5`.

<book:entry path="apps/web/src/shell/ribbonNavigation.test.ts">
// @vitest-environment jsdom
// Guarantee: docs/guarantees/lineage/a-ribbon-crosses-documents.md
import { expect, it } from "vitest";
import { directionBetween, prepareRibbonOpen, type Direction, type PaneBox } from "./ribbonNavigation";
import { panes, type Layout, type Pane } from "./layout";
import { openDocTab, openFileTab, openGeneratedTab } from "../views/workspaceState";
const pane = (id: string, kind: "document" | "chat" | "tree"): Pane => ({ type: "pane", id, active: 0, tabs: [{ id, kind, target: id }] });
function fixture(order: string[]) {
  const children = order.map(id => pane(id, id === "editor" ? "document" : id === "agent" ? "chat" : "tree"));
  const layout: Layout = { root: { type: "split", id: "root", direction: "row", children, sizes: children.map(() => 1 / children.length) }, focus: "agent" };
  const boxes: PaneBox[] = order.map((id, index) => ({ id, left: index * 200, right: index * 200 + 180, top: 0, bottom: 300 }));
  return { layout, boxes };
}
it.each([["editor", "tree", "agent"], ["tree", "editor", "agent"], ["agent", "tree", "editor"]])("routes a tree connection into its directional editor: %s %s %s", (...order) => {
  const { layout, boxes } = fixture(order);
  const direction = order.indexOf("tree") < order.indexOf("agent") ? "left" : "right";
  const routed = prepareRibbonOpen(layout, boxes, { from: "agent", direction });
  expect(routed.focus).toBe("editor");
  for (const opened of [openFileTab(routed, "new.py", routed.focus), openDocTab(routed, "note", "note.md", routed.focus), openGeneratedTab(routed, "owner", "out.py", [], routed.focus)]) {
    expect(panes(opened.root).find(p => p.tabs.some(t => ["new.py", "note.md", "out.py"].includes(t.target)))?.id).toBe("editor");
    expect(panes(opened.root).find(p => p.id === "agent")?.tabs).toEqual(panes(layout.root).find(p => p.id === "agent")?.tabs);
  }
});
it("uses the pane the connection points to when the target is already open", () => {
  const { layout, boxes } = fixture(["editor", "tree", "agent"]);
  expect(prepareRibbonOpen(layout, boxes, { from: "agent", into: "editor", direction: "left" }).focus).toBe("editor");
});
it.each(["left", "right", "up", "down"] as Direction[])("creates an editor toward %s when no editor exists", direction => {
  const { layout, boxes } = fixture(["tree", "agent"]);
  const routed = prepareRibbonOpen(layout, boxes, { from: "agent", direction });
  expect(routed.focus).not.toBe("agent");
  const split = panes(routed.root).map(p => p.id);
  expect(split.indexOf(routed.focus) < split.indexOf("agent")).toBe(direction === "left" || direction === "up");
  expect(panes(routed.root).find(p => p.id === "agent")?.tabs[0].kind).toBe("chat");
});
it("follows vertical placement too", () => {
  const { layout, boxes } = fixture(["editor", "tree", "agent"]);
  const vertical = boxes.map((box, index) => ({ ...box, left: 0, right: 300, top: index * 200, bottom: index * 200 + 180 }));
  expect(prepareRibbonOpen(layout, vertical, { from: "agent", direction: "up" }).focus).toBe("editor");
  expect(directionBetween(new DOMRect(0, 400, 300, 180), new DOMRect(0, 200, 300, 180))).toBe("up");
});
it("routes ordinary editor provenance too, independent of which pane has focus", () => {
  const { layout, boxes } = fixture(["agent", "tree", "editor"]);
  const other = pane("other", "document");
  if (layout.root.type !== "split") throw new Error("expected split");
  layout.root.children.unshift(other); layout.root.sizes = [0.25, 0.25, 0.25, 0.25];
  boxes.unshift({ id: "other", left: -200, right: -20, top: 0, bottom: 300 });
  const routed = prepareRibbonOpen(layout, boxes, { from: "editor", direction: "left" });
  expect(routed.focus).toBe("other");
});
</book:entry>

</book:doc>
