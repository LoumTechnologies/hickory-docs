import { describe, expect, it } from "vitest";

import {
  activate,
  collapsePane,
  panes,
  split,
  tab as makeTab,
  treePane,
  type Layout,
  type Pane,
  type Tab,
} from "../shell/layout";
import {
  activateDocTab,
  adoptUntitledTab,
  docIdsIn,
  findDocTab,
  focusedDocId,
  initialWorkspace,
  isWorkspaceEmpty,
  openDocTab,
  openGeneratedTab,
  openIntoDeclared,
  openTerminalTab,
  openUntitledTab,
} from "./workspaceState";

/** Every tab in the layout, flattened, for the nothing-closes invariant. */
function allTabs(layout: Layout): Tab[] {
  return panes(layout.root).flatMap((pane) => pane.tabs);
}

function editorPane(layout: Layout): Pane {
  const pane = panes(layout.root).find((candidate) => !candidate.tabs.some((t) => t.kind === "tree"));
  if (!pane) throw new Error("no editor pane");
  return pane;
}

describe("initialWorkspace", () => {
  it("starts empty: a tree pane and one empty editor pane", () => {
    const layout = initialWorkspace();
    expect(isWorkspaceEmpty(layout)).toBe(true);
    expect(treePane(layout)).not.toBeNull();
    expect(panes(layout.root)).toHaveLength(2);
  });
});

describe("openDocTab", () => {
  it("adds a document tab in the focused pane and focuses it", () => {
    const layout = openDocTab(initialWorkspace(), "d1", "notes/a.hick");
    const found = findDocTab(layout, "d1");
    expect(found).not.toBeNull();
    expect(found!.tab.kind).toBe("document");
    expect(found!.tab.target).toBe("notes/a.hick");
    expect(found!.tab.docId).toBe("d1");
    expect(layout.focus).toBe(found!.pane.id);
    expect(isWorkspaceEmpty(layout)).toBe(false);
  });

  it("opening a second document ADDS a tab — the first stays exactly where it was", () => {
    const one = openDocTab(initialWorkspace(), "d1", "a.hick");
    const two = openDocTab(one, "d2", "b.hick");
    expect(findDocTab(two, "d1")).not.toBeNull();
    expect(findDocTab(two, "d2")).not.toBeNull();
    // Same pane, both tabs, second active.
    const pane = findDocTab(two, "d2")!.pane;
    expect(pane.tabs.map((t) => t.docId)).toEqual(["d1", "d2"]);
    expect(pane.active).toBe(1);
  });

  it("activates (never duplicates) a document that is already open — wherever it is", () => {
    // d1 in one pane, d2 in a split beside it, focus on d2's pane.
    let layout = openDocTab(initialWorkspace(), "d1", "a.hick");
    const d1Pane = findDocTab(layout, "d1")!.pane.id;
    layout = split(layout, d1Pane, "row");
    layout = openDocTab(layout, "d2", "b.hick");
    const before = allTabs(layout);

    const reopened = openDocTab(layout, "d1", "a.hick");
    // Nothing added, nothing closed: the same tabs, to the id.
    expect(allTabs(reopened).map((t) => t.id).sort()).toEqual(before.map((t) => t.id).sort());
    // The focus moved to d1's pane.
    expect(reopened.focus).toBe(d1Pane);
  });

  it("never closes anything: every open tab survives every open", () => {
    let layout = openDocTab(initialWorkspace(), "d1", "a.hick");
    layout = openGeneratedTab(layout, "d1", "out/a.py", []);
    layout = openDocTab(layout, "d2", "b.hick");
    layout = openDocTab(layout, "d3", "c.hick");
    const kinds = allTabs(layout).map((t) => `${t.kind}:${t.target}`);
    expect(kinds).toContain("tree:folder");
    expect(kinds).toContain("document:a.hick");
    expect(kinds).toContain("generated:out/a.py");
    expect(kinds).toContain("document:b.hick");
    expect(kinds).toContain("document:c.hick");
  });

  it("refuses to bury the tree: a focused tree pane sends the open elsewhere", () => {
    const base = initialWorkspace();
    const tree = treePane(base)!;
    const focusedOnTree: Layout = { ...base, focus: tree.id };
    const layout = openDocTab(focusedOnTree, "d1", "a.hick");
    const found = findDocTab(layout, "d1")!;
    expect(found.pane.id).not.toBe(tree.id);
  });

  it("leaves collapsed panes collapsed — opening arranges nothing", () => {
    let layout = openDocTab(initialWorkspace(), "d1", "a.hick");
    const d1Pane = findDocTab(layout, "d1")!.pane.id;
    layout = split(layout, d1Pane, "row");
    layout = openDocTab(layout, "d2", "b.hick");
    layout = collapsePane(layout, d1Pane);
    const opened = openDocTab(layout, "d3", "c.hick");
    const collapsed = panes(opened.root).find((pane) => pane.id === d1Pane);
    expect(collapsed?.collapsed).toBe(true);
    expect(findDocTab(opened, "d1")).not.toBeNull();
  });
});

describe("activateDocTab (navigation re-activation)", () => {
  it("activates an open document's tab without touching the rest", () => {
    let layout = openDocTab(initialWorkspace(), "d1", "a.hick");
    layout = openDocTab(layout, "d2", "b.hick");
    // Back/forward to d1: same tabs, d1 frontmost.
    const back = activateDocTab(layout, "d1");
    expect(back).not.toBeNull();
    const pane = findDocTab(back!, "d1")!.pane;
    expect(pane.active).toBe(pane.tabs.findIndex((t) => t.docId === "d1"));
    expect(allTabs(back!).length).toBe(allTabs(layout).length);
  });

  it("answers identically (same object) when the tab is already frontmost", () => {
    const layout = openDocTab(initialWorkspace(), "d1", "a.hick");
    expect(activateDocTab(layout, "d1")).toBe(layout);
  });

  it("answers null for a document with no tab, so the caller knows to open", () => {
    expect(activateDocTab(initialWorkspace(), "nope")).toBeNull();
  });
});

describe("openGeneratedTab", () => {
  it("opens beside the owning document, carrying the owner's id", () => {
    const layout = openGeneratedTab(openDocTab(initialWorkspace(), "d1", "a.hick"), "d1", "out/a.py", []);
    const pane = panes(layout.root).find((candidate) =>
      candidate.tabs.some((t) => t.kind === "generated"),
    )!;
    const generated = pane.tabs.find((t) => t.kind === "generated")!;
    expect(generated.docId).toBe("d1");
    // Not on top of the document that produced it, and never burying the
    // tree pane under a file (dragDrop.ts refuses the same drop).
    expect(pane.tabs.some((t) => t.kind === "document" && t.docId === "d1")).toBe(false);
    expect(pane.tabs.some((t) => t.kind === "tree")).toBe(false);
  });
});

describe("declared layouts", () => {
  const regions = [
    { name: "prose", match: ["**/*.hick"] },
    { name: "code", match: ["src/**"] },
  ];

  it("openIntoDeclared builds the regions, keeps the tree, and files the document", () => {
    const layout = openIntoDeclared(regions, "d1", "spec.hick");
    expect(treePane(layout)).not.toBeNull();
    const found = findDocTab(layout, "d1")!;
    expect(found.pane.region).toBe("prose");
  });

  it("is only for an empty workspace: isWorkspaceEmpty is the gate the caller checks", () => {
    // The rule lives in the caller (WorkspaceView.ensureDocOpen): a declared
    // layout applies iff isWorkspaceEmpty; a busy workspace ADDS instead.
    // This pins the gate's two answers.
    expect(isWorkspaceEmpty(initialWorkspace())).toBe(true);
    expect(isWorkspaceEmpty(openDocTab(initialWorkspace(), "d1", "a.hick"))).toBe(false);
    expect(isWorkspaceEmpty(openUntitledTab(initialWorkspace()))).toBe(false);
  });
});

describe("untitled", () => {
  it("opens one untitled buffer and re-activates it rather than stacking more", () => {
    const one = openUntitledTab(initialWorkspace());
    const again = openUntitledTab(one);
    expect(allTabs(again).filter((t) => t.kind === "untitled")).toHaveLength(1);
  });

  it("adoptUntitledTab converts the tab in place — same pane, same position", () => {
    let layout = openDocTab(initialWorkspace(), "d1", "a.hick");
    layout = openUntitledTab(layout);
    const pane = editorPane(layout);
    const untitled = pane.tabs.find((t) => t.kind === "untitled")!;
    const position = pane.tabs.findIndex((t) => t.id === untitled.id);

    const adopted = adoptUntitledTab(layout, untitled.id, "d9", "untitled.hick");
    const after = panes(adopted.root).find((candidate) => candidate.id === pane.id)!;
    const tab = after.tabs[position];
    expect(tab.id).toBe(untitled.id);
    expect(tab.kind).toBe("document");
    expect(tab.docId).toBe("d9");
    expect(tab.target).toBe("untitled.hick");
    expect(findDocTab(adopted, "d1")).not.toBeNull();
  });
});

describe("docIdsIn / focusedDocId", () => {
  it("collects every involved document once — doc tabs and generated tabs' owners", () => {
    let layout = openDocTab(initialWorkspace(), "d1", "a.hick");
    layout = openGeneratedTab(layout, "d1", "out/a.py", []);
    layout = openDocTab(layout, "d2", "b.hick");
    expect(docIdsIn(layout)).toEqual(["d1", "d2"]);
  });

  it("keeps a document's session id alive through its generated files alone", () => {
    // The doc tab is gone; its generated tab still names the owner.
    let layout = openGeneratedTab(openDocTab(initialWorkspace(), "d1", "a.hick"), "d1", "out/a.py", []);
    const doc = findDocTab(layout, "d1")!;
    const pane = panes(layout.root).find((candidate) => candidate.id === doc.pane.id)!;
    const without: Layout = {
      ...layout,
      root: (function strip(node): typeof node {
        if (node.type === "pane") {
          return node.id === pane.id
            ? { ...node, tabs: node.tabs.filter((t) => t.docId !== "d1" || t.kind !== "document"), active: 0 }
            : node;
        }
        return { ...node, children: node.children.map(strip) };
      })(layout.root),
    };
    expect(docIdsIn(without)).toEqual(["d1"]);
  });

  it("focusedDocId reads the focused pane's active tab, and answers null for chrome", () => {
    let layout = openDocTab(initialWorkspace(), "d1", "a.hick");
    expect(focusedDocId(layout)).toBe("d1");
    const generated = openGeneratedTab(layout, "d1", "out/a.py", []);
    expect(focusedDocId(generated)).toBe("d1");
    const tree = treePane(layout)!;
    layout = activate(layout, tree.id, 0);
    expect(focusedDocId(layout)).toBeNull();
    const untitled = openUntitledTab(initialWorkspace());
    expect(focusedDocId(untitled)).toBeNull();
  });
});

describe("adding docId to tabs breaks nothing kind-agnostic", () => {
  it("makeTab carries the docId through", () => {
    const entry = makeTab("generated", "out/a.py", "a.py", "d1");
    expect(entry.docId).toBe("d1");
  });
});

describe("openTerminalTab — a session gets one tab, and keeps it", () => {
  it("adds a terminal tab beside the tree rather than over it", () => {
    const layout = openTerminalTab(initialWorkspace(), "term-1", "build");
    const terminals = panes(layout.root).flatMap((p) =>
      p.tabs.filter((t) => t.kind === "terminal"),
    );
    expect(terminals.map((t) => t.target)).toEqual(["term-1"]);
    // The Files tree is still open: opening ADDS.
    expect(treePane(layout)).not.toBeNull();
  });

  it("fronts the existing tab instead of opening a second one", () => {
    const once = openTerminalTab(initialWorkspace(), "term-1", "build");
    const withOther = openTerminalTab(once, "term-2", "tests");
    const again = openTerminalTab(withOther, "term-1", "build");

    const targets = panes(again.root)
      .flatMap((p) => p.tabs)
      .filter((t) => t.kind === "terminal")
      .map((t) => t.target)
      .sort();
    expect(targets).toEqual(["term-1", "term-2"]);

    const pane = panes(again.root).find((p) => p.tabs.some((t) => t.target === "term-1"))!;
    expect(pane.tabs[pane.active].target).toBe("term-1");
  });
});
