import { describe, expect, it } from "vitest";
import { dropOnCollapsed, moveTab, moveTabToIndex, zoneAt, type DropRect } from "./dragDrop";
import { collapsePane, freeform, open, panes, split, tab, type Layout, type Pane } from "./layout";

const doc = (path: string) => tab("document", path);

function openAll(layout: Layout, ...paths: string[]): Layout {
  return paths.reduce((current, path) => open(current, doc(path)), layout);
}

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

function targets(pane: Pane): string[] {
  return pane.tabs.map((t) => t.target);
}

function paneById(layout: Layout, id: string): Pane | null {
  return panes(layout.root).find((pane) => pane.id === id) ?? null;
}

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

describe("moveTab: junk in, layout out", () => {
  it("ignores unknown panes and unknown tabs", () => {
    const { layout, left, right } = twoPanes();
    expect(moveTab(layout, "pane-nope", left.tabs[0].id, right.id, "center")).toBe(layout);
    expect(moveTab(layout, left.id, "tab-nope", right.id, "center")).toBe(layout);
    expect(moveTab(layout, left.id, left.tabs[0].id, "pane-nope", "center")).toBe(layout);
  });
});

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
    // Drag a.md past b.hick: caret index 2 with a still present lands after b.
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

// Local helpers: walking for a split parent, mapping panes in place.
type SplitNode = Extract<Layout["root"], { type: "split" }>;

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

function mapPanes(node: Layout["root"], make: (pane: Pane) => Pane): Layout["root"] {
  if (node.type === "pane") return make(node);
  return { ...node, children: node.children.map((child) => mapPanes(child, make)) };
}

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
