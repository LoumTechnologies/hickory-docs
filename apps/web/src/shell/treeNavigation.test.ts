import { describe, expect, it } from "vitest";

import { treeNavigationIntent, treeSelectionRange, type NavigableTreeRow } from "./treeNavigation";

const rows: NavigableTreeRow[] = [
  { key: "src", parent: null, expandable: true, expanded: true, selectable: true },
  { key: "src/a", parent: "src", selectable: true },
  { key: "term", parent: "src", selectable: false },
  { key: "readme", parent: null, selectable: true },
];

// Guarantee: docs/guarantees/authoring/the-workspace-tree-edits-like-an-editor.md
describe("workspace-tree navigation", () => {
  it("moves in visible order and to the ends", () => {
    expect(treeNavigationIntent(rows, "src/a", "ArrowDown")).toEqual({ kind: "focus", key: "term" });
    expect(treeNavigationIntent(rows, "src/a", "ArrowUp")).toEqual({ kind: "focus", key: "src" });
    expect(treeNavigationIntent(rows, "src/a", "Home")).toEqual({ kind: "focus", key: "src" });
    expect(treeNavigationIntent(rows, "src/a", "End")).toEqual({ kind: "focus", key: "readme" });
  });

  it("uses left and right structurally", () => {
    expect(treeNavigationIntent(rows, "src", "ArrowLeft")).toEqual({ kind: "toggle", key: "src" });
    expect(treeNavigationIntent(rows, "src", "ArrowRight")).toEqual({ kind: "focus", key: "src/a" });
    expect(treeNavigationIntent(rows, "src/a", "ArrowLeft")).toEqual({ kind: "focus", key: "src" });
    expect(treeNavigationIntent([{ ...rows[0], expanded: false }], "src", "ArrowRight")).toEqual({ kind: "toggle", key: "src" });
  });

  it("activates with Enter", () => {
    expect(treeNavigationIntent(rows, "readme", "Enter")).toEqual({ kind: "activate", key: "readme" });
  });

  it("selects compatible rows across a visible range", () => {
    expect(treeSelectionRange(rows, "src/a", "readme")).toEqual(["src/a", "readme"]);
  });
});
