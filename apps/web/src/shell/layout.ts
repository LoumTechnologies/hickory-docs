// The shell's arrangement: panes of tabs, split horizontally and vertically.
//
// Pure data and pure functions, deliberately. Every hard question about a
// tiling shell — what happens to the focus when you close the pane it was in,
// where a file opens, what a split does to the sizes — is arithmetic, and
// arithmetic is answerable at a desk rather than by clicking. The React that
// renders this comes later and gets to be dull.
//
// See docs/specs/freeform/shell-layouts.md.

/** What a tab shows. Regions come from a layout; tools come from the app. */
export type ViewKind = "document" | "generated" | "tool";

export interface Tab {
  id: string;
  kind: ViewKind;
  /** The document path, generated file path, or tool name. */
  target: string;
  /** What the tab says. Defaults to the target's last segment. */
  title?: string;
}

export interface Pane {
  type: "pane";
  id: string;
  tabs: Tab[];
  /** Index into `tabs`. Always valid unless the pane is empty. */
  active: number;
  /**
   * The region this pane belongs to, in a declared layout.
   *
   * Freeform panes have none: a file opens where the focus is, because
   * nothing has been declared that would say otherwise.
   */
  region?: string;
}

export interface Split {
  type: "split";
  id: string;
  direction: "row" | "column";
  children: Node[];
  /** One fraction per child, summing to 1. */
  sizes: number[];
}

export type Node = Pane | Split;

export interface Layout {
  root: Node;
  /** The pane that takes the next open, and the keystrokes. */
  focus: string;
}

// ---------------------------------------------------------------------------
// Building
// ---------------------------------------------------------------------------

let counter = 0;
/** Ids are per-session and never persisted, so a counter is enough. */
function nextId(prefix: string): string {
  counter += 1;
  return `${prefix}-${counter}`;
}

export function emptyPane(region?: string): Pane {
  return { type: "pane", id: nextId("pane"), tabs: [], active: 0, region };
}

/** A layout with one empty pane: what opening a single file starts from. */
export function freeform(): Layout {
  const pane = emptyPane();
  return { root: pane, focus: pane.id };
}

export function tab(kind: ViewKind, target: string, title?: string): Tab {
  return { id: nextId("tab"), kind, target, title };
}

// ---------------------------------------------------------------------------
// Walking
// ---------------------------------------------------------------------------

export function panes(node: Node): Pane[] {
  return node.type === "pane" ? [node] : node.children.flatMap(panes);
}

export function paneById(layout: Layout, id: string): Pane | null {
  return panes(layout.root).find((pane) => pane.id === id) ?? null;
}

export function focused(layout: Layout): Pane | null {
  return paneById(layout, layout.focus);
}

/** Rebuild the tree with one pane replaced by whatever `make` returns. */
function replace(node: Node, id: string, make: (pane: Pane) => Node | null): Node | null {
  if (node.type === "pane") return node.id === id ? make(node) : node;
  const children: Node[] = [];
  const sizes: number[] = [];
  node.children.forEach((child, index) => {
    const next = replace(child, id, make);
    if (next === null) return;
    children.push(next);
    sizes.push(node.sizes[index] ?? 1 / node.children.length);
  });
  if (children.length === 0) return null;
  // A split with one child is not a split. Collapsing keeps the tree the
  // shape a person would draw, which is what makes "close the pane" leave
  // something recognisable behind.
  if (children.length === 1) return children[0];
  return { ...node, children, sizes: normalize(sizes) };
}

function normalize(sizes: number[]): number[] {
  const total = sizes.reduce((sum, size) => sum + size, 0);
  if (total <= 0) return sizes.map(() => 1 / sizes.length);
  return sizes.map((size) => size / total);
}

// ---------------------------------------------------------------------------
// Opening
// ---------------------------------------------------------------------------

/**
 * Open a target, or focus it where it is already open.
 *
 * Reopening a file that is already in front of you should not give you a
 * second copy of it — two tabs of one file is a state with no honest way to
 * decide which one an edit belongs to.
 */
export function open(layout: Layout, next: Tab, into?: string): Layout {
  const existing = panes(layout.root).find((pane) =>
    pane.tabs.some((candidate) => candidate.target === next.target && candidate.kind === next.kind),
  );
  if (existing) {
    const index = existing.tabs.findIndex(
      (candidate) => candidate.target === next.target && candidate.kind === next.kind,
    );
    const root = replace(layout.root, existing.id, (pane) => ({ ...pane, active: index }));
    return { root: root ?? layout.root, focus: existing.id };
  }

  const target = into ?? layout.focus;
  const root = replace(layout.root, target, (pane) => ({
    ...pane,
    tabs: [...pane.tabs, next],
    active: pane.tabs.length,
  }));
  return { root: root ?? layout.root, focus: target };
}

export function activate(layout: Layout, paneId: string, index: number): Layout {
  const root = replace(layout.root, paneId, (pane) => ({
    ...pane,
    active: Math.max(0, Math.min(index, pane.tabs.length - 1)),
  }));
  return { root: root ?? layout.root, focus: paneId };
}

/**
 * Close one tab.
 *
 * An empty pane is kept when it is the only one — there has to be somewhere
 * for the next file to go — and removed otherwise, because a strip of nothing
 * is not worth the width.
 */
export function closeTab(layout: Layout, paneId: string, tabId: string): Layout {
  const pane = paneById(layout, paneId);
  if (!pane) return layout;
  const tabs = pane.tabs.filter((candidate) => candidate.id !== tabId);
  const onlyPane = panes(layout.root).length === 1;

  if (tabs.length === 0 && !onlyPane) {
    const root = replace(layout.root, paneId, () => null);
    if (root === null) return layout;
    const remaining = panes(root);
    return { root, focus: remaining[0]?.id ?? layout.focus };
  }

  const removed = pane.tabs.findIndex((candidate) => candidate.id === tabId);
  const root = replace(layout.root, paneId, (current) => ({
    ...current,
    tabs,
    // Stay where you were looking: closing a tab before the active one must
    // not scroll the selection along with it.
    active: Math.max(0, Math.min(removed <= current.active ? current.active - 1 : current.active, tabs.length - 1)),
  }));
  return { root: root ?? layout.root, focus: paneId };
}

/**
 * Close a whole pane, tabs and all.
 *
 * The last pane stays, emptied: there has to be somewhere for the next file
 * to go, and an app with no panes has nothing to show and no way back.
 */
export function closePane(layout: Layout, paneId: string): Layout {
  if (panes(layout.root).length === 1) {
    const root = replace(layout.root, paneId, (pane) => ({ ...pane, tabs: [], active: 0 }));
    return { root: root ?? layout.root, focus: paneId };
  }
  const root = replace(layout.root, paneId, () => null);
  if (root === null) return layout;
  const remaining = panes(root);
  return { root, focus: remaining[0]?.id ?? layout.focus };
}

// ---------------------------------------------------------------------------
// Splitting
// ---------------------------------------------------------------------------

/**
 * Split a pane, putting a new empty pane beside it.
 *
 * The new pane takes the focus, because splitting is how a person says "and
 * also show me…", and what they mean by it is next.
 */
export function split(layout: Layout, paneId: string, direction: "row" | "column"): Layout {
  const fresh = emptyPane(paneById(layout, paneId)?.region);
  const root = replace(layout.root, paneId, (pane) => ({
    type: "split",
    id: nextId("split"),
    direction,
    children: [pane, fresh],
    sizes: [0.5, 0.5],
  }));
  return { root: root ?? layout.root, focus: fresh.id };
}

/** Drag a divider: `sizes` are fractions and always sum to 1. */
export function resize(layout: Layout, splitId: string, sizes: number[]): Layout {
  const apply = (node: Node): Node =>
    node.type === "pane"
      ? node
      : node.id === splitId
        ? { ...node, sizes: normalize(sizes.slice(0, node.children.length)) }
        : { ...node, children: node.children.map(apply) };
  return { ...layout, root: apply(layout.root) };
}

export function focus(layout: Layout, paneId: string): Layout {
  return paneById(layout, paneId) ? { ...layout, focus: paneId } : layout;
}

// ---------------------------------------------------------------------------
// Declared layouts
// ---------------------------------------------------------------------------

/** One box of a declared layout: a name and what belongs to it. */
export interface Region {
  name: string;
  /** Globs, in the order they were declared. First match wins. */
  match: string[];
}

/**
 * Which region a path belongs to.
 *
 * First match wins rather than most-specific: the order is written down in the
 * document, so it is something a person chose and can see, where "most
 * specific" is a rule they would have to work out.
 */
export function regionOf(regions: readonly Region[], path: string): string | null {
  for (const region of regions) {
    if (region.match.some((pattern) => matches(pattern, path))) return region.name;
  }
  return null;
}

/**
 * Glob matching, in the small dialect a layout needs: `*` within a segment,
 * `**` across segments, `?` for one character.
 *
 * Written segment by segment rather than by rewriting the whole string:
 * `**` behaves differently depending on where it sits — trailing, it takes
 * the separator with it, so `docs/**` matches `docs/spec.md` AND `docs`
 * itself — and a single pass of replacements cannot say that.
 */
export function matches(pattern: string, path: string): boolean {
  const parts = pattern.split("/");
  let out = "";
  parts.forEach((part, index) => {
    const last = index === parts.length - 1;
    if (part === "**") {
      if (last) {
        // `**` on its own is everything; trailing, it eats the slash before
        // it so the directory itself matches too.
        out = out === "" ? ".*" : `${out.replace(/\/$/, "")}(?:/.*)?`;
      } else {
        out += "(?:[^/]+/)*";
      }
      return;
    }
    out += part
      .replace(/[.+^${}()|[\]\\]/g, "\\$&")
      .replace(/\*/g, "[^/]*")
      .replace(/\?/g, "[^/]");
    if (!last) out += "/";
  });
  return new RegExp(`^${out}$`).test(path);
}

/** Build a layout from declared regions: one pane each, side by side. */
export function fromRegions(regions: readonly Region[]): Layout {
  if (regions.length === 0) return freeform();
  const children = regions.map((region) => emptyPane(region.name));
  if (children.length === 1) return { root: children[0], focus: children[0].id };
  const root: Split = {
    type: "split",
    id: nextId("split"),
    direction: "row",
    children,
    sizes: children.map(() => 1 / children.length),
  };
  return { root, focus: children[0].id };
}

/**
 * A pane that is not this one, preferring a neighbour that is empty.
 *
 * "Open to the side" exists because two things you want to compare have to be
 * on screen at once: a generated file opened over the document that produced
 * it hides the very thing it should sit beside, and no ribbon can be drawn
 * between a pane and itself.
 */
export function besides(layout: Layout, paneId: string): string | null {
  const others = panes(layout.root).filter((pane) => pane.id !== paneId);
  if (others.length === 0) return null;
  const empty = others.find((pane) => pane.tabs.length === 0);
  // Never over a document: that is the thing being compared against.
  const spare = others.find((pane) => !pane.tabs.some((tab) => tab.kind === "document"));
  return (empty ?? spare ?? others[0]).id;
}

/**
 * Where a file should open under a declared layout.
 *
 * This is the question a tiling shell otherwise has to invent an answer to,
 * and the declaration answers it: the pane whose region claims the path. A
 * path no region claims goes to the focus, which is the freeform behaviour and
 * the only honest fallback — refusing to open it would punish the person for
 * an incomplete declaration.
 */
export function paneFor(layout: Layout, regions: readonly Region[], path: string): string {
  const region = regionOf(regions, path);
  if (region === null) return layout.focus;
  return panes(layout.root).find((pane) => pane.region === region)?.id ?? layout.focus;
}
