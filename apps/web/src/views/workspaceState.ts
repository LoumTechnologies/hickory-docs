// The workspace's opening rules: what happens to ONE layout when documents
// come and go around it.
//
// Pure, like shell/layout.ts, and for the same reason. The invariant every
// function here protects is the one the old per-document view broke: opening
// a document ADDS to the arrangement — a tab in a sensible pane, or the
// activation of a tab that already exists — and never closes, rebuilds, or
// resets anything the person already has open. Navigation is "ensure open",
// not "replace the world".

import {
  activate,
  freeform,
  fromRegions,
  open as openInLayout,
  paneFor,
  panes,
  split,
  tab as makeTab,
  withTree,
  type Layout,
  type Pane,
  type Region,
  type Tab,
} from "../shell/layout";

/** The layout a brand-new workspace starts as: one empty pane and the
 * folder tree beside it. */
export function initialWorkspace(): Layout {
  return withTree(freeform(), makeTab("tree", "folder", "Files"));
}

/**
 * Whether the workspace is still untouched: nothing open but furniture (the
 * tree pane). Only here may a document's declared layout be applied — an
 * arrangement someone has started filling is theirs, and a declaration must
 * never reset it (merging a declared layout into a busy workspace is out of
 * scope, deliberately).
 */
export function isWorkspaceEmpty(layout: Layout): boolean {
  return panes(layout.root).every((pane) => pane.tabs.every((t) => t.kind === "tree"));
}

/** The pane holding a document's tab, when one does. Identity is the doc id:
 * a path can be renamed under a tab, an id cannot. */
export function findDocTab(
  layout: Layout,
  docId: string,
): { pane: Pane; tab: Tab; index: number } | null {
  for (const pane of panes(layout.root)) {
    const index = pane.tabs.findIndex((t) => t.kind === "document" && t.docId === docId);
    if (index >= 0) return { pane, tab: pane.tabs[index], index };
  }
  return null;
}

/** Every document the layout involves: doc tabs' own ids plus the owners of
 * generated tabs, in first-appearance order. These are the documents whose
 * machinery (CRDT room, runs, LSP, debugger) must be alive. */
export function docIdsIn(layout: Layout): string[] {
  const ids: string[] = [];
  for (const pane of panes(layout.root)) {
    for (const t of pane.tabs) {
      if (!t.docId) continue;
      if ((t.kind === "document" || t.kind === "generated") && !ids.includes(t.docId)) {
        ids.push(t.docId);
      }
    }
  }
  return ids;
}

/** A pane a document may open into: the focus, unless the focus is the tree
 * pane (opening over the tree would bury it) — then any other pane. */
function openablePane(layout: Layout): string | null {
  const all = panes(layout.root);
  const focused = all.find((pane) => pane.id === layout.focus);
  if (focused && !focused.tabs.some((t) => t.kind === "tree")) return focused.id;
  const other = all.find((pane) => !pane.tabs.some((t) => t.kind === "tree"));
  return other?.id ?? null;
}

/**
 * Open a document: activate its tab wherever it already is, or add a tab in
 * the focused pane. The rest of the layout is untouched — every other pane,
 * tab, size and collapse survives exactly as it was.
 */
export function openDocTab(layout: Layout, docId: string, path: string): Layout {
  const existing = findDocTab(layout, docId);
  if (existing) return activate(layout, existing.pane.id, existing.index);
  const into = openablePane(layout);
  const entry = makeTab("document", path, path.split("/").pop(), docId);
  if (into) return openInLayout(layout, entry, into);
  // Every pane is the tree's (or there are none): give the document a pane
  // of its own beside the tree rather than burying the folder under it.
  const grown = split(layout, layout.focus, "row");
  return openInLayout(grown, entry, grown.focus);
}

/** Activate a document's tab if it has one. Null when it does not — the
 * caller then knows an open (with a fetched path) is needed. */
export function activateDocTab(layout: Layout, docId: string): Layout | null {
  const existing = findDocTab(layout, docId);
  if (!existing) return null;
  // Already front-and-centre: hand back the same layout so effects that
  // compare identity can tell nothing happened.
  if (existing.pane.active === existing.index && layout.focus === existing.pane.id) return layout;
  return activate(layout, existing.pane.id, existing.index);
}

/**
 * A document opened into an EMPTY workspace, when the document declares its
 * own layout: build the declared regions, keep the tree, and put the
 * document where its globs say. The only situation a declaration may shape
 * the window — see isWorkspaceEmpty.
 */
export function openIntoDeclared(regions: readonly Region[], docId: string, path: string): Layout {
  const built = withTree(fromRegions(regions), makeTab("tree", "folder", "Files"));
  const target = paneFor(built, regions, path);
  return openInLayout(built, makeTab("document", path, path.split("/").pop(), docId), target);
}

/**
 * Open a generated file, owned by `docId`.
 *
 * Under a declared layout the regions say where. Freeform opens it BESIDE
 * the pane showing its document — a generated file on top of the document
 * that produced it hides the comparison it exists for, and no ribbon can be
 * drawn between a pane and itself.
 */
export function openGeneratedTab(
  layout: Layout,
  docId: string,
  path: string,
  regions: readonly Region[],
): Layout {
  const entry = makeTab("generated", path, path.split("/").pop(), docId);
  if (regions.length > 0) return openInLayout(layout, entry, paneFor(layout, regions, path));
  // Beside the document's own pane when it has one, else beside the focus —
  // and never onto the tree pane, which is chrome the way dragDrop.ts's
  // refusal already says: a file joined to it would bury the folder.
  const anchor = findDocTab(layout, docId)?.pane.id ?? layout.focus;
  const others = panes(layout.root).filter(
    (pane) => pane.id !== anchor && !pane.tabs.some((t) => t.kind === "tree"),
  );
  const side = (others.find((pane) => pane.tabs.length === 0) ??
    others.find((pane) => !pane.tabs.some((t) => t.kind === "document")) ??
    others[0])?.id;
  if (side) return openInLayout(layout, entry, side);
  const opened = split(layout, anchor, "row");
  return openInLayout(opened, entry, opened.focus);
}

/** Open (or re-activate) the untitled buffer. One at a time: an untitled tab
 * that already exists is what "#/new" means until its first edit names it. */
export function openUntitledTab(layout: Layout): Layout {
  for (const pane of panes(layout.root)) {
    const index = pane.tabs.findIndex((t) => t.kind === "untitled");
    if (index >= 0) return activate(layout, pane.id, index);
  }
  const into = openablePane(layout);
  const entry = makeTab("untitled", "untitled", "Untitled");
  if (into) return openInLayout(layout, entry, into);
  const grown = split(layout, layout.focus, "row");
  return openInLayout(grown, entry, grown.focus);
}

/**
 * The untitled buffer became a real document: convert its tab IN PLACE.
 *
 * Surgery on the tab rather than close-and-open, so the buffer's position
 * among its neighbours — the thing the person can see — does not move when
 * the file underneath it gains a name.
 */
export function adoptUntitledTab(layout: Layout, tabId: string, docId: string, path: string): Layout {
  const swap = (t: Tab): Tab =>
    t.id === tabId && t.kind === "untitled"
      ? { ...t, kind: "document", target: path, title: path.split("/").pop(), docId }
      : t;
  const apply = (node: Layout["root"]): Layout["root"] =>
    node.type === "pane"
      ? { ...node, tabs: node.tabs.map(swap) }
      : { ...node, children: node.children.map(apply) };
  return { ...layout, root: apply(layout.root) };
}

/**
 * Which document the keyboard would reach: the focused pane's active tab's
 * document. A focused tree (or an empty pane) answers null, and the caller
 * falls back to the document that was focused last — chrome must not make
 * "the current document" flicker to nothing.
 */
export function focusedDocId(layout: Layout): string | null {
  const pane = panes(layout.root).find((candidate) => candidate.id === layout.focus);
  const active = pane?.tabs[pane.active];
  if (!active) return null;
  if ((active.kind === "document" || active.kind === "generated") && active.docId) {
    return active.docId;
  }
  return null;
}
