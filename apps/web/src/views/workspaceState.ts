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
  withRail,
  withTree,
  type Layout,
  type Pane,
  type Region,
  type Tab,
} from "../shell/layout";

/** The tab the agent conversation lives in. */
export const CHAT_TAB = "chat";

/** The tab the commit graph lives in. */
export const GIT_TAB = "git";

/** Show the history. Opening it twice fronts the one that exists. */
export function openGitTab(layout: Layout): Layout {
  for (const pane of panes(layout.root)) {
    const index = pane.tabs.findIndex((t) => t.kind === "tool" && t.target === GIT_TAB);
    if (index >= 0) return activate(layout, pane.id, index);
  }
  return openInLayout(layout, makeTab("tool", GIT_TAB, "History"), layout.focus);
}

/** The tab the welcome page lives in. A tool tab, so it counts as furniture
 * and an arrangement holding only it is still "empty". */
export const WELCOME_TAB = "welcome";

/** Show the welcome page. Opening it twice fronts the one that exists. */
export function openWelcomeTab(layout: Layout): Layout {
  for (const pane of panes(layout.root)) {
    const index = pane.tabs.findIndex((t) => t.kind === "tool" && t.target === WELCOME_TAB);
    if (index >= 0) return activate(layout, pane.id, index);
  }
  return openInLayout(layout, makeTab("tool", WELCOME_TAB, "Welcome"), layout.focus);
}

/**
 * The layout a brand-new workspace starts as: the folder tree on the left,
 * the work in the middle, the agent on the right.
 *
 * The chat is a PANE here rather than a dock along the bottom. A dock made it
 * a mode — it covered the document it was about, and collapsing it was the
 * only way to see the thing you were asking about. As a pane it sits beside
 * the work, and can be moved, split, resized, or closed like anything else.
 */
export function initialWorkspace(): Layout {
  return withRail(
    withTree(freeform(), makeTab("tree", "folder", "Files")),
    makeTab("chat", CHAT_TAB, "Agent"),
  );
}

/** Show the agent conversation. Opening it twice fronts the one that exists. */
export function openChatTab(layout: Layout): Layout {
  for (const pane of panes(layout.root)) {
    const index = pane.tabs.findIndex((t) => t.kind === "chat");
    if (index >= 0) return activate(layout, pane.id, index);
  }
  return withRail(layout, makeTab("chat", CHAT_TAB, "Agent"));
}

/** Furniture: panes that are part of the window rather than of the work.
 *
 * The folder tree, and any other tool pane. There used to be a second entry
 * here — a pane listing every terminal session — and removing it is the point
 * of this window having ONE tree: a terminal has a working directory, the
 * tree already draws directories, and two trees meant two places to look for
 * "what is going on", neither of them the place you were already looking.
 * Terminals are icons on their directory's row now (shell/FolderTreePane). */
function isFurniture(t: Tab): boolean {
  return t.kind === "tree" || t.kind === "tool" || t.kind === "chat";
}

/**
 * Whether the workspace is still untouched: nothing open but furniture (the
 * tree pane, the terminals list). Only here may a document's declared layout
 * be applied — an arrangement someone has started filling is theirs, and a
 * declaration must never reset it (merging a declared layout into a busy
 * workspace is out of scope, deliberately).
 */
export function isWorkspaceEmpty(layout: Layout): boolean {
  return panes(layout.root).every((pane) => pane.tabs.every(isFurniture));
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

/**
 * Show a terminal session: front its tab if one exists, else add one.
 *
 * The same "ensure open" shape as a document, and for the same reason —
 * answering a prompt or clicking a session row must never rearrange the
 * window someone has built. The tab's target is the session id, so a session
 * has at most one tab however many times it is asked for.
 */
export function openTerminalTab(layout: Layout, sessionId: string, title: string): Layout {
  for (const pane of panes(layout.root)) {
    const index = pane.tabs.findIndex((t) => t.kind === "terminal" && t.target === sessionId);
    if (index >= 0) return activate(layout, pane.id, index);
  }
  const entry = makeTab("terminal", sessionId, title);
  const into = openablePane(layout);
  if (into) return openInLayout(layout, entry, into);
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

/** The pane holding a plain file's tab, when one does. Identity is the
 * path — a plain file has no id, and the path is what the tab shows. */
export function findFileTab(
  layout: Layout,
  path: string,
): { pane: Pane; tab: Tab; index: number } | null {
  for (const pane of panes(layout.root)) {
    const index = pane.tabs.findIndex((t) => t.kind === "file" && t.target === path);
    if (index >= 0) return { pane, tab: pane.tabs[index], index };
  }
  return null;
}

/**
 * Open a plain file: activate its tab wherever it already is, or add a tab
 * in the focused pane — the same "ensure open" a document gets, with no
 * docId because no document owns it.
 */
export function openFileTab(layout: Layout, path: string): Layout {
  const existing = findFileTab(layout, path);
  if (existing) return activate(layout, existing.pane.id, existing.index);
  const into = openablePane(layout);
  const entry = makeTab("file", path, path.split("/").pop());
  if (into) return openInLayout(layout, entry, into);
  const grown = split(layout, layout.focus, "row");
  return openInLayout(grown, entry, grown.focus);
}

/**
 * A plain file was adopted into a document: its tab becomes a GENERATED tab
 * owned by `docId`, in place — the same surgery adoptUntitledTab performs,
 * for the same reason: the buffer's position among its neighbours must not
 * move when the file underneath it gains an owner. The target changes from
 * the tree's root-relative path to `outputPath`, the document-relative name
 * the weave keys the output by, which is what generated panes fetch.
 */
export function adoptPlainFileTab(
  layout: Layout,
  path: string,
  docId: string,
  outputPath: string,
): Layout {
  const swap = (t: Tab): Tab =>
    t.kind === "file" && t.target === path
      ? { ...t, kind: "generated", target: outputPath, docId }
      : t;
  const apply = (node: Layout["root"]): Layout["root"] =>
    node.type === "pane"
      ? { ...node, tabs: node.tabs.map(swap) }
      : { ...node, children: node.children.map(apply) };
  return { ...layout, root: apply(layout.root) };
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

/** Open (or re-activate) the scratchpad. One at a time, like the untitled
 * buffer: a second scratchpad would split someone's train of thought across
 * two places, and the point of it is that there is one place to put a thought
 * before it has a name. */
export function openScratchpadTab(layout: Layout): Layout {
  for (const pane of panes(layout.root)) {
    const index = pane.tabs.findIndex((t) => t.kind === "scratchpad");
    if (index >= 0) return activate(layout, pane.id, index);
  }
  const into = openablePane(layout);
  const entry = makeTab("scratchpad", "scratchpad", "Scratchpad");
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
