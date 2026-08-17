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

// ---------------------------------------------------------------------------
// Icons for the collapsed strip
// ---------------------------------------------------------------------------

/**
 * What a strip icon draws: a folder glyph for the tree pane, a document
 * glyph for the prose, and a small text monogram — the file extension —
 * for everything with a path but no picture that would say more.
 */
export type TabIcon = { kind: "folder" } | { kind: "doc" } | { kind: "mono"; text: string };

export function tabIcon(tab: Tab): TabIcon {
  if (tab.kind === "tree") return { kind: "folder" };
  if (tab.kind === "document" || tab.kind === "untitled") return { kind: "doc" };
  return { kind: "mono", text: monogram(tab.target) };
}

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

interface Dir {
  name: string;
  dirs: Map<string, Dir>;
  tabs: { tab: Tab; index: number }[];
}

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

/**
 * Whether the divider after `index` must refuse to drag: a strip has a
 * fixed width, so a divider beside one has nothing it may resize.
 */
export function dividerLocked(children: readonly Node[], index: number): boolean {
  const strip = (node: Node | undefined) =>
    !!node && node.type === "pane" && !!node.collapsed;
  return strip(children[index]) || strip(children[index + 1]);
}
