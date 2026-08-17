import { describe, expect, it } from "vitest";

import { emptyPane, tab, type Node } from "./layout";
import {
  dividerLocked,
  gridTracks,
  groupTabsByFolder,
  monogram,
  STRIP_PX,
  tabIcon,
} from "./tabStrip";

describe("tabIcon: what a tab becomes when its pane is a strip", () => {
  it("gives the tree pane a folder and a document a doc glyph", () => {
    expect(tabIcon(tab("tree", "/repo"))).toEqual({ kind: "folder" });
    expect(tabIcon(tab("document", "guide.hick"))).toEqual({ kind: "doc" });
  });

  it("gives a generated file its extension as a monogram", () => {
    expect(tabIcon(tab("generated", "src/app.py"))).toEqual({ kind: "mono", text: "py" });
    expect(tabIcon(tab("generated", "README.md"))).toEqual({ kind: "mono", text: "md" });
  });
});

describe("monogram: the extension, or the name's first letters", () => {
  it("takes the extension of the LAST segment, lowercased", () => {
    expect(monogram("a/b/notes.MD")).toBe("md");
    expect(monogram("deep/dir.py/file.rs")).toBe("rs");
  });

  it("caps at three characters", () => {
    expect(monogram("page.html")).toBe("htm");
  });

  it("falls back to the first two letters of an extensionless name", () => {
    // A dotfile's leading dot is a convention, not an extension.
    expect(monogram("Makefile")).toBe("ma");
    expect(monogram(".gitignore")).toBe("gi");
    expect(monogram("chat")).toBe("ch");
  });

  it("never returns an empty badge", () => {
    expect(monogram("")).toBe("?");
  });
});

describe("groupTabsByFolder: side tabs as a folder tree", () => {
  it("groups tabs under directory headers, indenting children", () => {
    const rows = groupTabsByFolder([
      tab("document", "guide.hick"),
      tab("generated", "src/app.py"),
      tab("generated", "src/util/io.py"),
    ]);
    expect(rows).toEqual([
      expect.objectContaining({ kind: "tab", depth: 0 }),
      expect.objectContaining({ kind: "header", name: "src", depth: 0 }),
      expect.objectContaining({ kind: "tab", depth: 1 }),
      expect.objectContaining({ kind: "header", name: "util", depth: 1 }),
      expect.objectContaining({ kind: "tab", depth: 2 }),
    ]);
  });

  it("keeps the tree pane's tab ungrouped, at the top", () => {
    const tree = tab("tree", "/home/me/repo");
    const rows = groupTabsByFolder([tab("generated", "src/app.py"), tree]);
    expect(rows[0]).toEqual({ kind: "tab", tab: tree, index: 1, depth: 0 });
    // Its path never became directory headers.
    expect(rows.filter((row) => row.kind === "header").map((r) => r.name)).toEqual(["src"]);
  });

  it("keeps each tab row's index pointing into the PANE's tab array", () => {
    const a = tab("generated", "src/a.py");
    const b = tab("generated", "b.md");
    const rows = groupTabsByFolder([a, b]);
    const tabs = rows.filter((row) => row.kind === "tab");
    // Visual order regroups (root-level b before the src header), identity
    // does not: b keeps index 1, a keeps index 0.
    expect(tabs.map((row) => [row.tab.target, row.index])).toEqual([
      ["b.md", 1],
      ["src/a.py", 0],
    ]);
  });

  it("lists directories in the order a tab first mentioned them", () => {
    const rows = groupTabsByFolder([
      tab("generated", "zebra/z.py"),
      tab("generated", "alpha/a.py"),
    ]);
    expect(rows.filter((row) => row.kind === "header").map((r) => r.name)).toEqual([
      "zebra",
      "alpha",
    ]);
  });
});

describe("gridTracks and dividerLocked: strips are fixed tracks", () => {
  const collapsed = { ...emptyPane(), collapsed: true };
  const openPane = emptyPane();
  const children: Node[] = [collapsed, openPane];

  it("gives a collapsed pane a fixed pixel track and the survivor the whole fr pool", () => {
    // Regression: `0.7fr` alone sums below 1, and per the grid spec a flex
    // sum < 1 takes only that fraction of the free space — the collapsed
    // pane's "freed" 30% went to nobody. Renormalized, the survivor gets 1fr.
    expect(gridTracks(children, [0.3, 0.7])).toEqual([`${STRIP_PX}px`, "1fr"]);
  });

  it("renormalizes the survivors of a three-way split to sum to 1", () => {
    // The shape the bug was reported in: collapse one pane of a wider split
    // and the remaining fr values must still fill the whole container.
    const three: Node[] = [emptyPane(), collapsed, emptyPane()];
    const tracks = gridTracks(three, [0.25, 0.25, 0.5]);
    expect(tracks[1]).toBe(`${STRIP_PX}px`);
    const fr = (track: string) => Number(track.replace("fr", ""));
    expect(fr(tracks[0]) + fr(tracks[2])).toBeCloseTo(1);
    // ...and keep their RELATIVE shares (0.25 : 0.5 stays 1 : 2).
    expect(fr(tracks[2]) / fr(tracks[0])).toBeCloseTo(2);
  });

  it("keeps a fully expanded split exactly as sized", () => {
    const two: Node[] = [emptyPane(), emptyPane()];
    expect(gridTracks(two, [0.3, 0.7])).toEqual(["0.3fr", "0.7fr"]);
  });

  it("gives nested splits their renormalized fraction too", () => {
    // A split child (not a pane) beside a strip: the split is a survivor and
    // must inherit the whole fr pool the same way a pane would.
    const inner: Node = {
      type: "split",
      id: "s-inner",
      direction: "row",
      children: [emptyPane(), emptyPane()],
      sizes: [0.5, 0.5],
    };
    expect(gridTracks([collapsed, inner], [0.2, 0.8])).toEqual([`${STRIP_PX}px`, "1fr"]);
  });

  it("handles every child collapsed without dividing by zero", () => {
    const strips: Node[] = [collapsed, { ...emptyPane(), collapsed: true }];
    expect(gridTracks(strips, [0.5, 0.5])).toEqual([`${STRIP_PX}px`, `${STRIP_PX}px`]);
  });

  it("locks the divider on either side of a strip, and no others", () => {
    expect(dividerLocked(children, 0)).toBe(true);
    const three: Node[] = [openPane, emptyPane(), collapsed];
    expect(dividerLocked(three, 0)).toBe(false);
    expect(dividerLocked(three, 1)).toBe(true);
  });
});
