<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/workspaceState.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-efd800f9ed5a"></a>
## `apps/web/src/views/workspaceState.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Plain File Opens, Edits, And Never Silently Loses A Byte](../../../../../guarantees/authoring/a-plain-file-opens-and-saves/contract.md), [Adopt a plain file without changing its bytes](../../../../../guarantees/authoring/adoption-is-byte-exact-or-refused/contract.md), [New Document Creates Another Unnamed Buffer](../../../../../guarantees/authoring/new-document-is-an-act/contract.md), [Open File Tabs Follow Renames](../../../../../guarantees/authoring/open-file-tabs-follow-renames/contract.md), [The App Starts As A Lightweight Editor](../../../../../guarantees/authoring/the-app-starts-as-a-lightweight-editor/contract.md), [Typed Text Is Prose, Not A Transcript](../../../../../guarantees/authoring/typed-text-is-prose-not-a-transcript/contract.md), [Pair identity without silently granting execution](../../../../../guarantees/collaboration/a-machine-is-a-keypair/contract.md), [The History Lens Reads The Repository As A Story](../../../../../guarantees/lenses/the-history-lens-reads-the-repository-as-a-story/contract.md).

<a id="f-efd800f9ed5a-CHAT_TAB"></a>
### `CHAT_TAB`

The workspace's opening rules: what happens to ONE layout when documents
come and go around it.
Pure, like shell/layout.ts, and for the same reason. The invariant every
function here protects is the one the old per-document view broke: opening
a document ADDS to the arrangement — a tab in a sensible pane, or the
activation of a tab that already exists — and never closes, rebuilds, or
resets anything the person already has open. Navigation is "ensure open",
not "replace the world".

<storycode:copy id="f-efd800f9ed5a-CHAT_TAB">
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
  closeTab,
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

</storycode:copy>
<a id="f-efd800f9ed5a-GIT_TAB"></a>
### `GIT_TAB`

<storycode:copy id="f-efd800f9ed5a-GIT_TAB">
/** The tab the commit graph lives in. */
export const GIT_TAB = "git";

</storycode:copy>
<a id="f-efd800f9ed5a-openGitTab"></a>
### `openGitTab`

Show the history. Opening it twice fronts the one that exists.

<storycode:copy id="f-efd800f9ed5a-openGitTab">
/** Show the history. Opening it twice fronts the one that exists. */
export function openGitTab(layout: Layout): Layout {
  for (const pane of panes(layout.root)) {
    const index = pane.tabs.findIndex((t) => t.kind === "tool" && t.target === GIT_TAB);
    if (index >= 0) return activate(layout, pane.id, index);
  }
  return openInLayout(layout, makeTab("tool", GIT_TAB, "History"), layout.focus);
}

</storycode:copy>
<a id="f-efd800f9ed5a-STORY_TAB"></a>
### `STORY_TAB`

The tab the history lens lives in: the same repository as `GIT_TAB`,
read as a story. docs/specs/freeform/lenses.md

<storycode:copy id="f-efd800f9ed5a-STORY_TAB">
/** The tab the history lens lives in: the same repository as `GIT_TAB`,
 * read as a story. docs/specs/freeform/lenses.md */
export const STORY_TAB = "story";

</storycode:copy>
<a id="f-efd800f9ed5a-openStoryTab"></a>
### `openStoryTab`

Read the history as a story. Opening it twice fronts the one that exists.

<storycode:copy id="f-efd800f9ed5a-openStoryTab">
/** Read the history as a story. Opening it twice fronts the one that exists. */
export function openStoryTab(layout: Layout): Layout {
  for (const pane of panes(layout.root)) {
    const index = pane.tabs.findIndex((t) => t.kind === "tool" && t.target === STORY_TAB);
    if (index >= 0) return activate(layout, pane.id, index);
  }
  return openInLayout(layout, makeTab("tool", STORY_TAB, "Story"), layout.focus);
}

</storycode:copy>
<a id="f-efd800f9ed5a-FLEET_TAB"></a>
### `FLEET_TAB`

<storycode:copy id="f-efd800f9ed5a-FLEET_TAB">
/** The tab this machine's fleet lives in. */
export const FLEET_TAB = "fleet";

</storycode:copy>
<a id="f-efd800f9ed5a-openFleetTab"></a>
### `openFleetTab`

Show the fleet. Opening it twice fronts the one that exists.

<storycode:copy id="f-efd800f9ed5a-openFleetTab">
/** Show the fleet. Opening it twice fronts the one that exists. */
export function openFleetTab(layout: Layout): Layout {
  for (const pane of panes(layout.root)) {
    const index = pane.tabs.findIndex((t) => t.kind === "tool" && t.target === FLEET_TAB);
    if (index >= 0) return activate(layout, pane.id, index);
  }
  return openInLayout(layout, makeTab("tool", FLEET_TAB, "Machines"), layout.focus);
}

</storycode:copy>
<a id="f-efd800f9ed5a-MERGED_TAB"></a>
### `MERGED_TAB`

The tab a merged view lives in. One per path, because the view IS of a
path: two files compared at once are two tabs, not one confused one.

<storycode:copy id="f-efd800f9ed5a-MERGED_TAB">
/** The tab a merged view lives in. One per path, because the view IS of a
 * path: two files compared at once are two tabs, not one confused one. */
export const MERGED_TAB = "merged:";

</storycode:copy>
<a id="f-efd800f9ed5a-openMergedTab"></a>
### `openMergedTab`

Compare one file across this repository's worktrees.

<storycode:copy id="f-efd800f9ed5a-openMergedTab">
/** Compare one file across this repository's worktrees. */
export function openMergedTab(layout: Layout, path: string): Layout {
  const target = `${MERGED_TAB}${path}`;
  for (const pane of panes(layout.root)) {
    const index = pane.tabs.findIndex((t) => t.kind === "tool" && t.target === target);
    if (index >= 0) return activate(layout, pane.id, index);
  }
  const name = path.split("/").pop() ?? path;
  return openInLayout(layout, makeTab("tool", target, `${name} · worktrees`), layout.focus);
}

</storycode:copy>
<a id="f-efd800f9ed5a-WELCOME_TAB"></a>
### `WELCOME_TAB`

The tab the welcome page lives in. A tool tab, so it counts as furniture
and an arrangement holding only it is still "empty".

<storycode:copy id="f-efd800f9ed5a-WELCOME_TAB">
/** The tab the welcome page lives in. A tool tab, so it counts as furniture
 * and an arrangement holding only it is still "empty". */
export const WELCOME_TAB = "welcome";

</storycode:copy>
<a id="f-efd800f9ed5a-openWelcomeTab"></a>
### `openWelcomeTab`

Show the welcome page. Opening it twice fronts the one that exists.

<storycode:copy id="f-efd800f9ed5a-openWelcomeTab">
/** Show the welcome page. Opening it twice fronts the one that exists. */
export function openWelcomeTab(layout: Layout): Layout {
  for (const pane of panes(layout.root)) {
    const index = pane.tabs.findIndex((t) => t.kind === "tool" && t.target === WELCOME_TAB);
    if (index >= 0) return activate(layout, pane.id, index);
  }
  return openInLayout(layout, makeTab("tool", WELCOME_TAB, "Welcome"), layout.focus);
}

</storycode:copy>
<a id="f-efd800f9ed5a-initialWorkspace"></a>
### `initialWorkspace`

A new workspace starts with only its editor. Tools open on demand.

<storycode:copy id="f-efd800f9ed5a-initialWorkspace">
/** A new workspace starts with only its editor. Tools open on demand. */
export function initialWorkspace(): Layout {
  return freeform();
}

</storycode:copy>
<a id="f-efd800f9ed5a-openChatTab"></a>
### `openChatTab`

Show the agent conversation. Opening it twice fronts the one that exists.

<storycode:copy id="f-efd800f9ed5a-openChatTab">
/** Show the agent conversation. Opening it twice fronts the one that exists. */
export function openChatTab(layout: Layout): Layout {
  for (const pane of panes(layout.root)) {
    const index = pane.tabs.findIndex((t) => t.kind === "chat");
    if (index >= 0) return activate(layout, pane.id, index);
  }
  return withRail(layout, makeTab("chat", CHAT_TAB, "Agent"));
}

</storycode:copy>
<a id="f-efd800f9ed5a-isFurniture"></a>
### `isFurniture`

Furniture: panes that are part of the window rather than of the work.
The folder tree, and any other tool pane. There used to be a second entry
here — a pane listing every terminal session — and removing it is the point
of this window having ONE tree: a terminal has a working directory, the
tree already draws directories, and two trees meant two places to look for
"what is going on", neither of them the place you were already looking.
Terminals are icons on their directory's row now (shell/FolderTreePane).

<storycode:copy id="f-efd800f9ed5a-isFurniture">
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

</storycode:copy>
<a id="f-efd800f9ed5a-isWorkspaceEmpty"></a>
### `isWorkspaceEmpty`

Whether the workspace is still untouched: nothing open but furniture (the
tree pane, the terminals list). Only here may a document's declared layout
be applied — an arrangement someone has started filling is theirs, and a
declaration must never reset it (merging a declared layout into a busy
workspace is out of scope, deliberately).

<storycode:copy id="f-efd800f9ed5a-isWorkspaceEmpty">
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

</storycode:copy>
<a id="f-efd800f9ed5a-findDocTab"></a>
### `findDocTab`

The pane holding a document's tab, when one does. Identity is the doc id:
a path can be renamed under a tab, an id cannot.

<storycode:copy id="f-efd800f9ed5a-findDocTab">
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

</storycode:copy>
<a id="f-efd800f9ed5a-docIdsIn"></a>
### `docIdsIn`

Every document the layout involves: doc tabs' own ids plus the owners of
generated tabs, in first-appearance order. These are the documents whose
machinery (CRDT room, runs, LSP, debugger) must be alive.

<storycode:copy id="f-efd800f9ed5a-docIdsIn">
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

</storycode:copy>
<a id="f-efd800f9ed5a-openablePane"></a>
### `openablePane`

A pane a document may open into: the focus, unless the focus is the tree
pane (opening over the tree would bury it) — then any other pane.

<storycode:copy id="f-efd800f9ed5a-openablePane">
/** A pane a document may open into: the focus, unless the focus is the tree
 * pane (opening over the tree would bury it) — then any other pane. */
function openablePane(layout: Layout): string | null {
  const all = panes(layout.root);
  const focused = all.find((pane) => pane.id === layout.focus);
  if (focused && !focused.tabs.some((t) => t.kind === "tree")) return focused.id;
  const other = all.find((pane) => !pane.tabs.some((t) => t.kind === "tree"));
  return other?.id ?? null;
}

</storycode:copy>
<a id="f-efd800f9ed5a-openDocTab"></a>
### `openDocTab`

Open a document: activate its tab wherever it already is, or add a tab in
the focused pane. The rest of the layout is untouched — every other pane,
tab, size and collapse survives exactly as it was.

<storycode:copy id="f-efd800f9ed5a-openDocTab">
/**
 * Open a document: activate its tab wherever it already is, or add a tab in
 * the focused pane. The rest of the layout is untouched — every other pane,
 * tab, size and collapse survives exactly as it was.
 */
export function openDocTab(layout: Layout, docId: string, path: string, preferred?: string): Layout {
  const existing = findDocTab(layout, docId);
  if (existing) return activate(layout, existing.pane.id, existing.index);
  const into = preferred && panes(layout.root).some(pane => pane.id === preferred) ? preferred : openablePane(layout);
  const entry = makeTab("document", path, path.split("/").pop(), docId);
  if (into) return openInLayout(layout, entry, into);
  // Every pane is the tree's (or there are none): give the document a pane
  // of its own beside the tree rather than burying the folder under it.
  const grown = split(layout, layout.focus, "row");
  return openInLayout(grown, entry, grown.focus);
}

</storycode:copy>
<a id="f-efd800f9ed5a-openTerminalTab"></a>
### `openTerminalTab`

Show a terminal session: front its tab if one exists, else add one.
The same "ensure open" shape as a document, and for the same reason —
answering a prompt or clicking a session row must never rearrange the
window someone has built. The tab's target is the session id, so a session
has at most one tab however many times it is asked for.

<storycode:copy id="f-efd800f9ed5a-openTerminalTab">
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

</storycode:copy>
<a id="f-efd800f9ed5a-activateDocTab"></a>
### `activateDocTab`

Activate a document's tab if it has one. Null when it does not — the
caller then knows an open (with a fetched path) is needed.

<storycode:copy id="f-efd800f9ed5a-activateDocTab">
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

</storycode:copy>
<a id="f-efd800f9ed5a-openIntoDeclared"></a>
### `openIntoDeclared`

A document opened into an EMPTY workspace, when the document declares its
own layout: build the declared regions, keep the tree, and put the
document where its globs say. The only situation a declaration may shape
the window — see isWorkspaceEmpty.

<storycode:copy id="f-efd800f9ed5a-openIntoDeclared">
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

</storycode:copy>
<a id="f-efd800f9ed5a-openGeneratedTab"></a>
### `openGeneratedTab`

Open a generated file, owned by `docId`.
Under a declared layout the regions say where. Freeform opens it BESIDE
the pane showing its document — a generated file on top of the document
that produced it hides the comparison it exists for, and no ribbon can be
drawn between a pane and itself.

<storycode:copy id="f-efd800f9ed5a-openGeneratedTab">
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
  preferred?: string,
): Layout {
  const entry = makeTab("generated", path, path.split("/").pop(), docId);
  if (preferred && panes(layout.root).some(pane => pane.id === preferred)) return openInLayout(layout, entry, preferred);
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

</storycode:copy>
<a id="f-efd800f9ed5a-findFileTab"></a>
### `findFileTab`

The pane holding a plain file's tab, when one does. Identity is the
path — a plain file has no id, and the path is what the tab shows.

<storycode:copy id="f-efd800f9ed5a-findFileTab">
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

</storycode:copy>
<a id="f-efd800f9ed5a-openFileTab"></a>
### `openFileTab`

Open a plain file: activate its tab wherever it already is, or add a tab
in the focused pane — the same "ensure open" a document gets, with no
docId because no document owns it.

<storycode:copy id="f-efd800f9ed5a-openFileTab">
/**
 * Open a plain file: activate its tab wherever it already is, or add a tab
 * in the focused pane — the same "ensure open" a document gets, with no
 * docId because no document owns it.
 */
export function openFileTab(layout: Layout, path: string, preferred?: string): Layout {
  const existing = findFileTab(layout, path);
  if (existing) return activate(layout, existing.pane.id, existing.index);
  const into = preferred && panes(layout.root).some(pane => pane.id === preferred) ? preferred : openablePane(layout);
  const entry = makeTab("file", path, path.split("/").pop());
  if (into) return openInLayout(layout, entry, into);
  const grown = split(layout, layout.focus, "row");
  return openInLayout(grown, entry, grown.focus);
}

</storycode:copy>
<a id="f-efd800f9ed5a-renameFileTab"></a>
### `renameFileTab`

A filesystem rename keeps its already-open editor exactly where it is;
only the path identity and tab caption move with the file.

<storycode:copy id="f-efd800f9ed5a-renameFileTab">
/** A filesystem rename keeps its already-open editor exactly where it is;
 * only the path identity and tab caption move with the file. */
export function renameFileTab(layout: Layout, from: string, to: string): Layout {
  const replace = (tab: Tab): Tab =>
    tab.kind === "file" && tab.target === from
      ? { ...tab, target: to, title: to.split("/").pop() ?? to }
      : tab;
  const walk = (node: Layout["root"]): Layout["root"] =>
    node.type === "pane"
      ? { ...node, tabs: node.tabs.map(replace) }
      : { ...node, children: node.children.map(walk) };
  return { ...layout, root: walk(layout.root) };
}

</storycode:copy>
<a id="f-efd800f9ed5a-closeTabsForDeletedPaths"></a>
### `closeTabsForDeletedPaths`

Remove tabs whose backing file disappeared. A deleted directory also
retires every file tab under it; drafts and tools do not have a backing
filesystem path and deliberately remain.

<storycode:copy id="f-efd800f9ed5a-closeTabsForDeletedPaths">
/** Remove tabs whose backing file disappeared. A deleted directory also
 * retires every file tab under it; drafts and tools do not have a backing
 * filesystem path and deliberately remain. */
export function closeTabsForDeletedPaths(layout: Layout, paths: readonly string[]): Layout {
  const deleted = paths.filter(Boolean);
  if (!deleted.length) return layout;
  const removed = (target: string) => deleted.some(path => target === path || target.startsWith(`${path}/`));
  let next = layout;
  for (const pane of panes(layout.root)) {
    for (const entry of pane.tabs) {
      if (["document", "generated", "file"].includes(entry.kind) && removed(entry.target)) {
        const live = panes(next.root).find(candidate => candidate.tabs.some(tab => tab.id === entry.id));
        if (live) next = closeTab(next, live.id, entry.id);
      }
    }
  }
  return next;
}

</storycode:copy>
<a id="f-efd800f9ed5a-adoptPlainFileTab"></a>
### `adoptPlainFileTab`

A plain file was adopted into a document: its tab becomes a GENERATED tab
owned by `docId`, in place — the same surgery adoptUntitledTab performs,
for the same reason: the buffer's position among its neighbours must not
move when the file underneath it gains an owner. The target changes from
the tree's root-relative path to `outputPath`, the document-relative name
the weave keys the output by, which is what generated panes fetch.

<storycode:copy id="f-efd800f9ed5a-adoptPlainFileTab">
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

</storycode:copy>
<a id="f-efd800f9ed5a-openUntitledTab"></a>
### `openUntitledTab`

Ensure the route has an unnamed buffer; an explicit New always adds one.

<storycode:copy id="f-efd800f9ed5a-openUntitledTab">
/** Ensure the route has an unnamed buffer; an explicit New always adds one. */
export function openUntitledTab(layout: Layout, create = false): Layout {
  const existing = panes(layout.root).flatMap(pane => pane.tabs).filter(tab => tab.kind === "untitled");
  if (!create) {
    const focused = panes(layout.root).find(pane => pane.id === layout.focus);
    if (focused?.tabs[focused.active]?.kind === "untitled") return layout;
    for (const pane of panes(layout.root)) {
      const index = pane.tabs.findIndex(tab => tab.kind === "untitled");
      if (index >= 0) return activate(layout, pane.id, index);
    }
  }
  const into = openablePane(layout);
  const titles = new Set(existing.map(tab => tab.title));
  let title = "Untitled";
  for (let n = 2; titles.has(title); n++) title = `Untitled ${n}`;
  const entry = { ...makeTab("untitled", "untitled", title), id: `untitled-${crypto.randomUUID()}` };
  if (existing.length) entry.target = entry.id;
  if (into) return openInLayout(layout, entry, into);
  const grown = split(layout, layout.focus, "row");
  return openInLayout(grown, entry, grown.focus);
}

</storycode:copy>
<a id="f-efd800f9ed5a-openScratchpadTab"></a>
### `openScratchpadTab`

Open (or re-activate) the scratchpad. One at a time: a second scratchpad would split someone's train of thought across
two places, and the point of it is that there is one place to put a thought
before it has a name.

<storycode:copy id="f-efd800f9ed5a-openScratchpadTab">
/** Open (or re-activate) the scratchpad. One at a time: a second scratchpad would split someone's train of thought across
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

</storycode:copy>
<a id="f-efd800f9ed5a-adoptUntitledTab"></a>
### `adoptUntitledTab`

The untitled buffer became a real document: convert its tab IN PLACE.
Surgery on the tab rather than close-and-open, so the buffer's position
among its neighbours — the thing the person can see — does not move when
the file underneath it gains a name.

<storycode:copy id="f-efd800f9ed5a-adoptUntitledTab">
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

</storycode:copy>
<a id="f-efd800f9ed5a-focusedDocId"></a>
### `focusedDocId`

Which document the keyboard would reach: the focused pane's active tab's
document. A focused tree (or an empty pane) answers null, and the caller
falls back to the document that was focused last — chrome must not make
"the current document" flicker to nothing.

<storycode:copy id="f-efd800f9ed5a-focusedDocId">
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
</storycode:copy>

### Assemble `apps/web/src/views/workspaceState.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/workspaceState.ts"><storycode:paste select="#f-efd800f9ed5a-CHAT_TAB"/><storycode:paste select="#f-efd800f9ed5a-GIT_TAB"/><storycode:paste select="#f-efd800f9ed5a-openGitTab"/><storycode:paste select="#f-efd800f9ed5a-STORY_TAB"/><storycode:paste select="#f-efd800f9ed5a-openStoryTab"/><storycode:paste select="#f-efd800f9ed5a-FLEET_TAB"/><storycode:paste select="#f-efd800f9ed5a-openFleetTab"/><storycode:paste select="#f-efd800f9ed5a-MERGED_TAB"/><storycode:paste select="#f-efd800f9ed5a-openMergedTab"/><storycode:paste select="#f-efd800f9ed5a-WELCOME_TAB"/><storycode:paste select="#f-efd800f9ed5a-openWelcomeTab"/><storycode:paste select="#f-efd800f9ed5a-initialWorkspace"/><storycode:paste select="#f-efd800f9ed5a-openChatTab"/><storycode:paste select="#f-efd800f9ed5a-isFurniture"/><storycode:paste select="#f-efd800f9ed5a-isWorkspaceEmpty"/><storycode:paste select="#f-efd800f9ed5a-findDocTab"/><storycode:paste select="#f-efd800f9ed5a-docIdsIn"/><storycode:paste select="#f-efd800f9ed5a-openablePane"/><storycode:paste select="#f-efd800f9ed5a-openDocTab"/><storycode:paste select="#f-efd800f9ed5a-openTerminalTab"/><storycode:paste select="#f-efd800f9ed5a-activateDocTab"/><storycode:paste select="#f-efd800f9ed5a-openIntoDeclared"/><storycode:paste select="#f-efd800f9ed5a-openGeneratedTab"/><storycode:paste select="#f-efd800f9ed5a-findFileTab"/><storycode:paste select="#f-efd800f9ed5a-openFileTab"/><storycode:paste select="#f-efd800f9ed5a-renameFileTab"/><storycode:paste select="#f-efd800f9ed5a-closeTabsForDeletedPaths"/><storycode:paste select="#f-efd800f9ed5a-adoptPlainFileTab"/><storycode:paste select="#f-efd800f9ed5a-openUntitledTab"/><storycode:paste select="#f-efd800f9ed5a-openScratchpadTab"/><storycode:paste select="#f-efd800f9ed5a-adoptUntitledTab"/><storycode:paste select="#f-efd800f9ed5a-focusedDocId"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-14d3dbc939ff"></a>
## `apps/web/src/views/workspaceState.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Plain File Opens, Edits, And Never Silently Loses A Byte](../../../../../guarantees/authoring/a-plain-file-opens-and-saves/contract.md), [Adopt a plain file without changing its bytes](../../../../../guarantees/authoring/adoption-is-byte-exact-or-refused/contract.md), [Open File Tabs Follow Renames](../../../../../guarantees/authoring/open-file-tabs-follow-renames/contract.md), [The App Starts As A Lightweight Editor](../../../../../guarantees/authoring/the-app-starts-as-a-lightweight-editor/contract.md), [Typed Text Is Prose, Not A Transcript](../../../../../guarantees/authoring/typed-text-is-prose-not-a-transcript/contract.md), [A Local Install Is a Launchable macOS App](../../../../../guarantees/release/a-local-install-is-a-launchable-macos-app/contract.md).

<a id="f-14d3dbc939ff-allTabs"></a>
### `allTabs`

<storycode:copy id="f-14d3dbc939ff-allTabs">
import { describe, expect, it } from "vitest";

import {
  activate,
  collapsePane,
  panes,
  split,
  tab as makeTab,
  treePane,
  withTree,
  type Layout,
  type Pane,
  type Tab, freeform } from "../shell/layout";
import {
  GIT_TAB,
  activateDocTab,
  adoptPlainFileTab,
  closeTabsForDeletedPaths,
  adoptUntitledTab,
  docIdsIn,
  findDocTab,
  findFileTab,
  focusedDocId,
  initialWorkspace,
  isWorkspaceEmpty,
  openDocTab,
  openFileTab,
  openGitTab,
  renameFileTab,
  openGeneratedTab,
  openIntoDeclared,
  openTerminalTab,
  openScratchpadTab,
  openUntitledTab,
  openChatTab,
} from "./workspaceState";

/** Every tab in the layout, flattened, for the nothing-closes invariant. */
function allTabs(layout: Layout): Tab[] {
  return panes(layout.root).flatMap((pane) => pane.tabs);
}

</storycode:copy>
<a id="f-14d3dbc939ff-editorPane"></a>
### `editorPane`

<storycode:copy id="f-14d3dbc939ff-editorPane">
function editorPane(layout: Layout): Pane {
  const pane = panes(layout.root).find((candidate) => !candidate.tabs.some((t) => t.kind === "tree"));
  if (!pane) throw new Error("no editor pane");
  return pane;
}

</storycode:copy>
<a id="f-14d3dbc939ff-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-14d3dbc939ff-expression_statement">
describe("initialWorkspace", () => {
  it("starts with one empty editor pane and no tools", () => {
    const layout = initialWorkspace();
    expect(isWorkspaceEmpty(layout)).toBe(true);
    expect(treePane(layout)).toBeNull();
    expect(panes(layout.root)).toHaveLength(1);
    expect(allTabs(layout)).toEqual([]);
  });

  it("fronts the agent pane that exists rather than opening a second", () => {
    const layout = openChatTab(initialWorkspace());
    const again = openChatTab(layout);
    expect(panes(again.root)).toHaveLength(2);
    expect(panes(again.root).filter((p) => p.tabs.some((t) => t.kind === "chat"))).toHaveLength(1);
  });

  it("opens the agent on the right when it has been closed", () => {
    const bare = freeform();
    const withChat = openChatTab(bare);
    const order = panes(withChat.root).map((pane) => pane.tabs[0]?.kind ?? "empty");
    expect(order).toEqual(["empty", "chat"]);
  });
});

</storycode:copy>
<a id="f-14d3dbc939ff-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-14d3dbc939ff-expression_statement-2">
describe("openDocTab", () => {
  it("adds a document tab in the focused pane and focuses it", () => {
    const layout = openDocTab(initialWorkspace(), "d1", "notes/a.md");
    const found = findDocTab(layout, "d1");
    expect(found).not.toBeNull();
    expect(found!.tab.kind).toBe("document");
    expect(found!.tab.target).toBe("notes/a.md");
    expect(found!.tab.docId).toBe("d1");
    expect(layout.focus).toBe(found!.pane.id);
    expect(isWorkspaceEmpty(layout)).toBe(false);
  });

  it("opening a second document ADDS a tab — the first stays exactly where it was", () => {
    const one = openDocTab(initialWorkspace(), "d1", "a.md");
    const two = openDocTab(one, "d2", "b.md");
    expect(findDocTab(two, "d1")).not.toBeNull();
    expect(findDocTab(two, "d2")).not.toBeNull();
    // Same pane, both tabs, second active.
    const pane = findDocTab(two, "d2")!.pane;
    expect(pane.tabs.map((t) => t.docId)).toEqual(["d1", "d2"]);
    expect(pane.active).toBe(1);
  });

  it("activates (never duplicates) a document that is already open — wherever it is", () => {
    // d1 in one pane, d2 in a split beside it, focus on d2's pane.
    let layout = openDocTab(initialWorkspace(), "d1", "a.md");
    const d1Pane = findDocTab(layout, "d1")!.pane.id;
    layout = split(layout, d1Pane, "row");
    layout = openDocTab(layout, "d2", "b.md");
    const before = allTabs(layout);

    const reopened = openDocTab(layout, "d1", "a.md");
    // Nothing added, nothing closed: the same tabs, to the id.
    expect(allTabs(reopened).map((t) => t.id).sort()).toEqual(before.map((t) => t.id).sort());
    // The focus moved to d1's pane.
    expect(reopened.focus).toBe(d1Pane);
  });

  it("never closes anything: every open tab survives every open", () => {
    let layout = openDocTab(withTree(initialWorkspace(), makeTab("tree", "folder", "Files")), "d1", "a.md");
    layout = openGeneratedTab(layout, "d1", "out/a.py", []);
    layout = openDocTab(layout, "d2", "b.md");
    layout = openDocTab(layout, "d3", "c.md");
    const kinds = allTabs(layout).map((t) => `${t.kind}:${t.target}`);
    expect(kinds).toContain("tree:folder");
    expect(kinds).toContain("document:a.md");
    expect(kinds).toContain("generated:out/a.py");
    expect(kinds).toContain("document:b.md");
    expect(kinds).toContain("document:c.md");
  });

  it("refuses to bury the tree: a focused tree pane sends the open elsewhere", () => {
    const base = withTree(initialWorkspace(), makeTab("tree", "folder", "Files"));
    const tree = treePane(base)!;
    const focusedOnTree: Layout = { ...base, focus: tree.id };
    const layout = openDocTab(focusedOnTree, "d1", "a.md");
    const found = findDocTab(layout, "d1")!;
    expect(found.pane.id).not.toBe(tree.id);
  });

  it("leaves collapsed panes collapsed — opening arranges nothing", () => {
    let layout = openDocTab(initialWorkspace(), "d1", "a.md");
    const d1Pane = findDocTab(layout, "d1")!.pane.id;
    layout = split(layout, d1Pane, "row");
    layout = openDocTab(layout, "d2", "b.md");
    layout = collapsePane(layout, d1Pane);
    const opened = openDocTab(layout, "d3", "c.md");
    const collapsed = panes(opened.root).find((pane) => pane.id === d1Pane);
    expect(collapsed?.collapsed).toBe(true);
    expect(findDocTab(opened, "d1")).not.toBeNull();
  });
});

</storycode:copy>
<a id="f-14d3dbc939ff-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-14d3dbc939ff-expression_statement-3">
describe("activateDocTab (navigation re-activation)", () => {
  it("activates an open document's tab without touching the rest", () => {
    let layout = openDocTab(initialWorkspace(), "d1", "a.md");
    layout = openDocTab(layout, "d2", "b.md");
    // Back/forward to d1: same tabs, d1 frontmost.
    const back = activateDocTab(layout, "d1");
    expect(back).not.toBeNull();
    const pane = findDocTab(back!, "d1")!.pane;
    expect(pane.active).toBe(pane.tabs.findIndex((t) => t.docId === "d1"));
    expect(allTabs(back!).length).toBe(allTabs(layout).length);
  });

  it("answers identically (same object) when the tab is already frontmost", () => {
    const layout = openDocTab(initialWorkspace(), "d1", "a.md");
    expect(activateDocTab(layout, "d1")).toBe(layout);
  });

  it("answers null for a document with no tab, so the caller knows to open", () => {
    expect(activateDocTab(initialWorkspace(), "nope")).toBeNull();
  });
});

</storycode:copy>
<a id="f-14d3dbc939ff-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-14d3dbc939ff-expression_statement-4">
describe("openGeneratedTab", () => {
  it("opens beside the owning document, carrying the owner's id", () => {
    const layout = openGeneratedTab(openDocTab(initialWorkspace(), "d1", "a.md"), "d1", "out/a.py", []);
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

</storycode:copy>
<a id="f-14d3dbc939ff-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-14d3dbc939ff-expression_statement-5">
describe("openFileTab", () => {
  it("adds a plain-file tab with no owning document, and focuses its pane", () => {
    const layout = openFileTab(initialWorkspace(), "README.md");
    const found = findFileTab(layout, "README.md");
    expect(found).not.toBeNull();
    expect(found!.tab.kind).toBe("file");
    expect(found!.tab.target).toBe("README.md");
    expect(found!.tab.docId).toBeUndefined();
    expect(layout.focus).toBe(found!.pane.id);
  });

  it("re-opening activates the existing tab instead of adding a second", () => {
    let layout = openFileTab(initialWorkspace(), ".github/workflows/ci.yml");
    layout = openDocTab(layout, "d1", "paper.md");
    const before = allTabs(layout).length;
    layout = openFileTab(layout, ".github/workflows/ci.yml");
    expect(allTabs(layout)).toHaveLength(before);
    const found = findFileTab(layout, ".github/workflows/ci.yml")!;
    expect(found.pane.active).toBe(found.index);
  });

  it("opening ADDS — every already-open tab survives exactly where it was", () => {
    let layout = openDocTab(initialWorkspace(), "d1", "paper.md");
    const survivors = allTabs(layout).map((t) => t.id);
    layout = openFileTab(layout, "justfile");
    for (const id of survivors) {
      expect(allTabs(layout).some((t) => t.id === id)).toBe(true);
    }
  });

  it("never opens over the tree: a tree-focused workspace grows a pane instead", () => {
    const fresh = withTree(initialWorkspace(), makeTab("tree", "folder", "Files"));
    const tree = treePane(fresh)!;
    const layout = openFileTab(activate(fresh, tree.id, 0), "README.md");
    const found = findFileTab(layout, "README.md")!;
    expect(found.pane.tabs.some((t) => t.kind === "tree")).toBe(false);
  });

  it("stays out of docIdsIn: no document machinery wakes for a plain file", () => {
    const layout = openFileTab(openDocTab(initialWorkspace(), "d1", "paper.md"), "README.md");
    expect(docIdsIn(layout)).toEqual(["d1"]);
  });
});

</storycode:copy>
<a id="f-14d3dbc939ff-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-14d3dbc939ff-expression_statement-6">
describe("renameFileTab", () => {
  // Guarantee: docs/guarantees/authoring/open-file-tabs-follow-renames.md
  it("keeps the open pane and its tab identity while changing its path and caption", () => {
    const opened = openFileTab(initialWorkspace(), "notes/first.md");
    const before = findFileTab(opened, "notes/first.md")!;
    const renamed = renameFileTab(opened, "notes/first.md", "notes/plan.md");
    const after = findFileTab(renamed, "notes/plan.md")!;
    expect(after.tab.id).toBe(before.tab.id);
    expect(after.pane.id).toBe(before.pane.id);
    expect(after.index).toBe(before.index);
    expect(after.tab.title).toBe("plan.md");
    expect(findFileTab(renamed, "notes/first.md")).toBeNull();
  });
});

</storycode:copy>
<a id="f-14d3dbc939ff-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-14d3dbc939ff-expression_statement-7">
describe("closeTabsForDeletedPaths", () => {
  it("retires every open backing-file tab, including a deleted folder's descendants", () => {
    let layout = openDocTab(initialWorkspace(), "d1", "src/readme.md");
    layout = openGeneratedTab(layout, "d1", "src/output.html", []);
    layout = openFileTab(layout, "src/plain.txt");
    layout = openFileTab(layout, "notes/kept.md");
    layout = openGitTab(layout);

    const closed = closeTabsForDeletedPaths(layout, ["src"]);
    expect(allTabs(closed).map(tab => tab.target)).toEqual(expect.arrayContaining(["notes/kept.md", GIT_TAB]));
    expect(allTabs(closed).some(tab => tab.target.startsWith("src/"))).toBe(false);
  });
});

</storycode:copy>
<a id="f-14d3dbc939ff-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-14d3dbc939ff-expression_statement-8">
describe("adoptPlainFileTab", () => {
  it("converts the file tab in place: generated kind, owning doc, weave-keyed target", () => {
    let layout = openDocTab(initialWorkspace(), "d1", "paper.md");
    layout = openFileTab(layout, "src/analysis.py");
    const before = findFileTab(layout, "src/analysis.py")!;
    const adopted = adoptPlainFileTab(layout, "src/analysis.py", "d2", "analysis.py");
    const pane = panes(adopted.root).find((p) => p.tabs.some((t) => t.id === before.tab.id))!;
    const tab = pane.tabs.find((t) => t.id === before.tab.id)!;
    expect(tab.kind).toBe("generated");
    expect(tab.docId).toBe("d2");
    // The weave keys outputs relative to the DOCUMENT's directory; the tab
    // must fetch by that name, not the tree's root-relative one.
    expect(tab.target).toBe("analysis.py");
    // Surgery, not close-and-open: same id, same position, neighbours intact.
    expect(pane.tabs.findIndex((t) => t.id === before.tab.id)).toBe(before.index);
    expect(allTabs(adopted)).toHaveLength(allTabs(layout).length);
    // The document machinery must wake for the new owner.
    expect(docIdsIn(adopted)).toContain("d2");
  });

  it("touches nothing when no tab shows the path", () => {
    const layout = openDocTab(initialWorkspace(), "d1", "paper.md");
    const adopted = adoptPlainFileTab(layout, "README.md", "d9", "README.md");
    expect(allTabs(adopted).map((t) => `${t.kind}:${t.target}`)).toEqual(
      allTabs(layout).map((t) => `${t.kind}:${t.target}`),
    );
  });
});

</storycode:copy>
<a id="f-14d3dbc939ff-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-14d3dbc939ff-expression_statement-9">
describe("declared layouts", () => {
  const regions = [
    { name: "prose", match: ["**/*.md"] },
    { name: "code", match: ["src/**"] },
  ];

  it("openIntoDeclared builds the regions, keeps the tree, and files the document", () => {
    const layout = openIntoDeclared(regions, "d1", "spec.md");
    expect(treePane(layout)).not.toBeNull();
    const found = findDocTab(layout, "d1")!;
    expect(found.pane.region).toBe("prose");
  });

  it("is only for an empty workspace: isWorkspaceEmpty is the gate the caller checks", () => {
    // The rule lives in the caller (WorkspaceView.ensureDocOpen): a declared
    // layout applies iff isWorkspaceEmpty; a busy workspace ADDS instead.
    // This pins the gate's two answers.
    expect(isWorkspaceEmpty(initialWorkspace())).toBe(true);
    expect(isWorkspaceEmpty(openDocTab(initialWorkspace(), "d1", "a.md"))).toBe(false);
    expect(isWorkspaceEmpty(openUntitledTab(initialWorkspace()))).toBe(false);
  });
});

</storycode:copy>
<a id="f-14d3dbc939ff-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-14d3dbc939ff-expression_statement-10">
describe("the scratchpad", () => {
  it("opens one scratchpad and re-activates it rather than stacking more", () => {
    // A second scratchpad would split a train of thought across two places,
    // and the point of it is that there is one place to put a thought before
    // it has a name.
    const one = openScratchpadTab(initialWorkspace());
    const again = openScratchpadTab(one);
    expect(allTabs(again).filter((t) => t.kind === "scratchpad")).toHaveLength(1);
  });

  it("coexists with the untitled buffer — they are different things", () => {
    // Untitled is a DOCUMENT that has no name yet; the scratchpad is text
    // that may never become one.
    let layout = openUntitledTab(initialWorkspace());
    layout = openScratchpadTab(layout);
    expect(allTabs(layout).filter((t) => t.kind === "untitled")).toHaveLength(1);
    expect(allTabs(layout).filter((t) => t.kind === "scratchpad")).toHaveLength(1);
  });
});

</storycode:copy>
<a id="f-14d3dbc939ff-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-14d3dbc939ff-expression_statement-11">
describe("untitled", () => {
  it("keeps route navigation idempotent but creates distinct tabs for explicit New", () => {
    const one = openUntitledTab(initialWorkspace());
    const again = openUntitledTab(one);
    expect(allTabs(again).filter((t) => t.kind === "untitled")).toHaveLength(1);
    const two = openUntitledTab(again, true);
    const buffers = allTabs(two).filter(tab => tab.kind === "untitled");
    expect(buffers.map(tab => tab.title)).toEqual(["Untitled", "Untitled 2"]);
    expect(buffers[0].id).not.toBe(buffers[1].id);
  });

  it("adoptUntitledTab converts the tab in place — same pane, same position", () => {
    let layout = openDocTab(initialWorkspace(), "d1", "a.md");
    layout = openUntitledTab(layout);
    const pane = editorPane(layout);
    const untitled = pane.tabs.find((t) => t.kind === "untitled")!;
    const position = pane.tabs.findIndex((t) => t.id === untitled.id);

    const adopted = adoptUntitledTab(layout, untitled.id, "d9", "untitled.md");
    const after = panes(adopted.root).find((candidate) => candidate.id === pane.id)!;
    const tab = after.tabs[position];
    expect(tab.id).toBe(untitled.id);
    expect(tab.kind).toBe("document");
    expect(tab.docId).toBe("d9");
    expect(tab.target).toBe("untitled.md");
    expect(findDocTab(adopted, "d1")).not.toBeNull();
  });
});

</storycode:copy>
<a id="f-14d3dbc939ff-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-14d3dbc939ff-expression_statement-12">
describe("docIdsIn / focusedDocId", () => {
  it("collects every involved document once — doc tabs and generated tabs' owners", () => {
    let layout = openDocTab(withTree(initialWorkspace(), makeTab("tree", "folder", "Files")), "d1", "a.md");
    layout = openGeneratedTab(layout, "d1", "out/a.py", []);
    layout = openDocTab(layout, "d2", "b.md");
    expect(docIdsIn(layout)).toEqual(["d1", "d2"]);
  });

  it("keeps a document's session id alive through its generated files alone", () => {
    // The doc tab is gone; its generated tab still names the owner.
    let layout = openGeneratedTab(openDocTab(initialWorkspace(), "d1", "a.md"), "d1", "out/a.py", []);
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
    let layout = openDocTab(withTree(initialWorkspace(), makeTab("tree", "folder", "Files")), "d1", "a.md");
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

</storycode:copy>
<a id="f-14d3dbc939ff-expression_statement-13"></a>
### `expression_statement`

<storycode:copy id="f-14d3dbc939ff-expression_statement-13">
describe("adding docId to tabs breaks nothing kind-agnostic", () => {
  it("makeTab carries the docId through", () => {
    const entry = makeTab("generated", "out/a.py", "a.py", "d1");
    expect(entry.docId).toBe("d1");
  });
});

</storycode:copy>
<a id="f-14d3dbc939ff-expression_statement-14"></a>
### `expression_statement`

<storycode:copy id="f-14d3dbc939ff-expression_statement-14">
describe("openTerminalTab — a session gets one tab, and keeps it", () => {
  it("adds a terminal tab beside the tree rather than over it", () => {
    const layout = openTerminalTab(withTree(initialWorkspace(), makeTab("tree", "folder", "Files")), "term-1", "build");
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
</storycode:copy>

### Assemble `apps/web/src/views/workspaceState.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/workspaceState.test.ts"><storycode:paste select="#f-14d3dbc939ff-allTabs"/><storycode:paste select="#f-14d3dbc939ff-editorPane"/><storycode:paste select="#f-14d3dbc939ff-expression_statement"/><storycode:paste select="#f-14d3dbc939ff-expression_statement-2"/><storycode:paste select="#f-14d3dbc939ff-expression_statement-3"/><storycode:paste select="#f-14d3dbc939ff-expression_statement-4"/><storycode:paste select="#f-14d3dbc939ff-expression_statement-5"/><storycode:paste select="#f-14d3dbc939ff-expression_statement-6"/><storycode:paste select="#f-14d3dbc939ff-expression_statement-7"/><storycode:paste select="#f-14d3dbc939ff-expression_statement-8"/><storycode:paste select="#f-14d3dbc939ff-expression_statement-9"/><storycode:paste select="#f-14d3dbc939ff-expression_statement-10"/><storycode:paste select="#f-14d3dbc939ff-expression_statement-11"/><storycode:paste select="#f-14d3dbc939ff-expression_statement-12"/><storycode:paste select="#f-14d3dbc939ff-expression_statement-13"/><storycode:paste select="#f-14d3dbc939ff-expression_statement-14"/></storycode:file>


</storycode:doc>
