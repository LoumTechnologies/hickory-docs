<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/tabStrip.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-585bd5e52e7b"></a>
## `apps/web/src/shell/tabStrip.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-585bd5e52e7b-STRIP_PX"></a>
### `STRIP_PX`

How a pane's tabs present themselves, as arithmetic.
Three questions live here, all pure so they are answerable at a desk:
which icon a tab becomes when its pane collapses to a strip, what folder
tree the tabs group into when they render as a sidebar, and which grid
tracks a split hands to CSS when some of its children are strips. The
React that consumes these stays in ShellView.tsx and stays dull.
See docs/specs/freeform/shell-layouts.md.

<storycode:copy id="f-585bd5e52e7b-STRIP_PX">
// How a pane's tabs present themselves, as arithmetic.
//
// Three questions live here, all pure so they are answerable at a desk:
// which icon a tab becomes when its pane collapses to a strip, what folder
// tree the tabs group into when they render as a sidebar, and which grid
// tracks a split hands to CSS when some of its children are strips. The
// React that consumes these stays in ShellView.tsx and stays dull.
//
// See docs/specs/freeform/shell-layouts.md.

import type { Node, Tab } from "./layout";

/** Width of a collapsed pane's icon strip, and the fixed grid track it
 * takes. ~a VS Code activity bar: enough for one icon and its padding. */
export const STRIP_PX = 36;

</storycode:copy>
<a id="f-585bd5e52e7b-TabIcon"></a>
### `TabIcon`

Icons for the collapsed strip
What a strip icon draws: a folder glyph for the tree pane, a document
glyph for the prose, and a small text monogram — the file extension —
for everything with a path but no picture that would say more.

<storycode:copy id="f-585bd5e52e7b-TabIcon">
// ---------------------------------------------------------------------------
// Icons for the collapsed strip
// ---------------------------------------------------------------------------

/**
 * What a strip icon draws: a folder glyph for the tree pane, a document
 * glyph for the prose, and a small text monogram — the file extension —
 * for everything with a path but no picture that would say more.
 */
export type TabIcon = { kind: "folder" } | { kind: "doc" } | { kind: "mono"; text: string };

</storycode:copy>
<a id="f-585bd5e52e7b-tabIcon"></a>
### `tabIcon`

<storycode:copy id="f-585bd5e52e7b-tabIcon">
export function tabIcon(tab: Tab): TabIcon {
  if (tab.kind === "tree") return { kind: "folder" };
  if (tab.kind === "document" || tab.kind === "untitled") return { kind: "doc" };
  return { kind: "mono", text: monogram(tab.target) };
}

</storycode:copy>
<a id="f-585bd5e52e7b-monogram"></a>
### `monogram`

The badge text for a target: its extension ("py", "md"), lowercased and
capped at three characters. A name with no extension — a tool, a Makefile
— falls back to its first two letters, which is at least ITS letters.

<storycode:copy id="f-585bd5e52e7b-monogram">
/**
 * The badge text for a target: its extension ("py", "md"), lowercased and
 * capped at three characters. A name with no extension — a tool, a Makefile
 * — falls back to its first two letters, which is at least ITS letters.
 */
export function monogram(target: string): string {
  const last = target.split("/").pop() ?? target;
  // A dotfile's leading dot is a convention, not an extension.
  const name = last.startsWith(".") ? last.slice(1) : last;
  const dot = name.lastIndexOf(".");
  const ext = dot > 0 ? name.slice(dot + 1) : "";
  const text = ext !== "" ? ext.slice(0, 3) : name.slice(0, 2);
  const lowered = text.toLowerCase();
  return lowered === "" ? "?" : lowered;
}

</storycode:copy>
<a id="f-585bd5e52e7b-SideRow"></a>
### `SideRow`

Side tabs, grouped by folder
One row of the side-tab tree: a directory header (small, non-interactive)
or a tab. `index` on a tab row is its position in the PANE's tab array —
the visual order regroups, the identity does not.

<storycode:copy id="f-585bd5e52e7b-SideRow">
// ---------------------------------------------------------------------------
// Side tabs, grouped by folder
// ---------------------------------------------------------------------------

/**
 * One row of the side-tab tree: a directory header (small, non-interactive)
 * or a tab. `index` on a tab row is its position in the PANE's tab array —
 * the visual order regroups, the identity does not.
 */
export type SideRow =
  | { kind: "header"; name: string; depth: number }
  | { kind: "tab"; tab: Tab; index: number; depth: number };

</storycode:copy>
<a id="f-585bd5e52e7b-Dir"></a>
### `Dir`

<storycode:copy id="f-585bd5e52e7b-Dir">
interface Dir {
  name: string;
  dirs: Map<string, Dir>;
  tabs: { tab: Tab; index: number }[];
}

</storycode:copy>
<a id="f-585bd5e52e7b-groupTabsByFolder"></a>
### `groupTabsByFolder`

Group a pane's tabs by the directory of their path, into a tree.
The tree pane's tab stays ungrouped at the top — it names the folder the
rest live in, and filing it under itself would be a joke, not a tree.
Targets without a slash (root-level files, tool names) sit at the top
level too. Directories appear in the order a tab first mentioned them,
which is the order the person opened things — a sort would move tabs
around under a hand that did not ask for it.

<storycode:copy id="f-585bd5e52e7b-groupTabsByFolder">
/**
 * Group a pane's tabs by the directory of their path, into a tree.
 *
 * The tree pane's tab stays ungrouped at the top — it names the folder the
 * rest live in, and filing it under itself would be a joke, not a tree.
 * Targets without a slash (root-level files, tool names) sit at the top
 * level too. Directories appear in the order a tab first mentioned them,
 * which is the order the person opened things — a sort would move tabs
 * around under a hand that did not ask for it.
 */
export function groupTabsByFolder(tabs: readonly Tab[]): SideRow[] {
  const rows: SideRow[] = [];
  const root: Dir = { name: "", dirs: new Map(), tabs: [] };

  tabs.forEach((tab, index) => {
    if (tab.kind === "tree") {
      rows.push({ kind: "tab", tab, index, depth: 0 });
      return;
    }
    const segments = tab.target.split("/").slice(0, -1);
    let at = root;
    for (const segment of segments) {
      let next = at.dirs.get(segment);
      if (!next) {
        next = { name: segment, dirs: new Map(), tabs: [] };
        at.dirs.set(segment, next);
      }
      at = next;
    }
    at.tabs.push({ tab, index });
  });

  const flatten = (dir: Dir, depth: number) => {
    for (const entry of dir.tabs) {
      rows.push({ kind: "tab", tab: entry.tab, index: entry.index, depth });
    }
    for (const child of dir.dirs.values()) {
      rows.push({ kind: "header", name: child.name, depth });
      flatten(child, depth + 1);
    }
  };
  flatten(root, 0);
  return rows;
}

</storycode:copy>
<a id="f-585bd5e52e7b-gridTracks"></a>
### `gridTracks`

Grid tracks around collapsed panes
The track each child of a split occupies: its fraction as `fr`, except a
collapsed pane, which is a fixed strip. The fraction is REMEMBERED, not
spent — it stays in `sizes`, so expanding gives the pane its old share
back without anyone re-dragging a divider.
The `fr` values handed to CSS are RENORMALIZED over the non-collapsed
children so they always sum to 1. This is not cosmetic: per the grid spec,
flex factors that sum to less than 1 take only that fraction of the free
space (`0.2fr` alone fills 20% of the row and leaves the rest EMPTY). The
un-normalized sizes made exactly that happen the moment a pane collapsed —
its fraction left the fr pool, the survivors summed below 1, and the
"freed" space went to nobody.

<storycode:copy id="f-585bd5e52e7b-gridTracks">
// ---------------------------------------------------------------------------
// Grid tracks around collapsed panes
// ---------------------------------------------------------------------------

/**
 * The track each child of a split occupies: its fraction as `fr`, except a
 * collapsed pane, which is a fixed strip. The fraction is REMEMBERED, not
 * spent — it stays in `sizes`, so expanding gives the pane its old share
 * back without anyone re-dragging a divider.
 *
 * The `fr` values handed to CSS are RENORMALIZED over the non-collapsed
 * children so they always sum to 1. This is not cosmetic: per the grid spec,
 * flex factors that sum to less than 1 take only that fraction of the free
 * space (`0.2fr` alone fills 20% of the row and leaves the rest EMPTY). The
 * un-normalized sizes made exactly that happen the moment a pane collapsed —
 * its fraction left the fr pool, the survivors summed below 1, and the
 * "freed" space went to nobody.
 */
export function gridTracks(children: readonly Node[], sizes: readonly number[]): string[] {
  const share = (index: number) => sizes[index] ?? 1 / children.length;
  const isStrip = (child: Node) => child.type === "pane" && !!child.collapsed;
  const expandedTotal = children.reduce(
    (sum, child, index) => sum + (isStrip(child) ? 0 : share(index)),
    0,
  );
  return children.map((child, index) =>
    isStrip(child)
      ? `${STRIP_PX}px`
      : `${expandedTotal > 0 ? share(index) / expandedTotal : 1}fr`,
  );
}

</storycode:copy>
<a id="f-585bd5e52e7b-dividerLocked"></a>
### `dividerLocked`

Whether the divider after `index` must refuse to drag: a strip has a
fixed width, so a divider beside one has nothing it may resize.

<storycode:copy id="f-585bd5e52e7b-dividerLocked">
/**
 * Whether the divider after `index` must refuse to drag: a strip has a
 * fixed width, so a divider beside one has nothing it may resize.
 */
export function dividerLocked(children: readonly Node[], index: number): boolean {
  const strip = (node: Node | undefined) =>
    !!node && node.type === "pane" && !!node.collapsed;
  return strip(children[index]) || strip(children[index + 1]);
}
</storycode:copy>

### Assemble `apps/web/src/shell/tabStrip.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/tabStrip.ts"><storycode:paste select="#f-585bd5e52e7b-STRIP_PX"/><storycode:paste select="#f-585bd5e52e7b-TabIcon"/><storycode:paste select="#f-585bd5e52e7b-tabIcon"/><storycode:paste select="#f-585bd5e52e7b-monogram"/><storycode:paste select="#f-585bd5e52e7b-SideRow"/><storycode:paste select="#f-585bd5e52e7b-Dir"/><storycode:paste select="#f-585bd5e52e7b-groupTabsByFolder"/><storycode:paste select="#f-585bd5e52e7b-gridTracks"/><storycode:paste select="#f-585bd5e52e7b-dividerLocked"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-05f3d6e3d23d"></a>
## `apps/web/src/shell/tabStrip.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-05f3d6e3d23d-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-05f3d6e3d23d-expression_statement">
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
    expect(tabIcon(tab("document", "guide.md"))).toEqual({ kind: "doc" });
  });

  it("gives a generated file its extension as a monogram", () => {
    expect(tabIcon(tab("generated", "src/app.py"))).toEqual({ kind: "mono", text: "py" });
    expect(tabIcon(tab("generated", "README.md"))).toEqual({ kind: "mono", text: "md" });
  });
});

</storycode:copy>
<a id="f-05f3d6e3d23d-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-05f3d6e3d23d-expression_statement-2">
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

</storycode:copy>
<a id="f-05f3d6e3d23d-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-05f3d6e3d23d-expression_statement-3">
describe("groupTabsByFolder: side tabs as a folder tree", () => {
  it("groups tabs under directory headers, indenting children", () => {
    const rows = groupTabsByFolder([
      tab("document", "guide.md"),
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

</storycode:copy>
<a id="f-05f3d6e3d23d-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-05f3d6e3d23d-expression_statement-4">
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
</storycode:copy>

### Assemble `apps/web/src/shell/tabStrip.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/tabStrip.test.ts"><storycode:paste select="#f-05f3d6e3d23d-expression_statement"/><storycode:paste select="#f-05f3d6e3d23d-expression_statement-2"/><storycode:paste select="#f-05f3d6e3d23d-expression_statement-3"/><storycode:paste select="#f-05f3d6e3d23d-expression_statement-4"/></storycode:file>


</storycode:doc>
