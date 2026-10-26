<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Folder Remembers Its Window

For: Authors editing executable documents and the files those documents produce.

Given a folder previously opened in Hickory Docs,
when it is opened again,
then its open documents, files, active view, pane arrangement, spatial tiles,
drawer pins and widths, prose measures, table sizes and tab zoom return.
Files stays closed if the remembered arrangement left it closed.
A fresh folder uses the normal introductory defaults.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-folder-remembers-its-window.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `folder_layout_follows_the_first_folder_attached_to_a_blank_window`

This is the maintained test fragment from `apps/desktop/src-tauri/tests/serves_one_origin.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-e7ec95ab75f6-folder_layout_follows_the_first_folder_attached_to_a_blank_window"></a>
### `folder_layout_follows_the_first_folder_attached_to_a_blank_window`

<storycode:copy id="f-e7ec95ab75f6-folder_layout_follows_the_first_folder_attached_to_a_blank_window">
/// Guarantee: docs/guarantees/authoring/a-folder-remembers-its-window.md
#[tokio::test(flavor = "multi_thread")]
async fn folder_layout_follows_the_first_folder_attached_to_a_blank_window() {
    let config = tempfile::tempdir().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    let store = hickory_workspace::WorkspaceStore::for_project(folder.path()).unwrap();
    let remembered = serde_json::json!({"version": 1, "layout": {"open": ["notes.md"]}});
    store.save_ui(&remembered).unwrap();
    let session = hickory_desktop_lib::server::start_blank_with_config(Some(config.path()), None)
        .await
        .unwrap();
    let http = reqwest::Client::new();
    let endpoint = format!("{}/api/workspace/ui", session.url);
    assert!(
        http.put(&endpoint)
            .json(&serde_json::json!({"state": {"blank": true}}))
            .send()
            .await
            .unwrap()
            .status()
            .is_success()
    );
    assert!(
        session
            .attach_folder(folder.path(), Some(config.path()), None, None)
            .await
            .unwrap()
    );
    let response: serde_json::Value = http
        .get(&endpoint)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(response["state"], remembered);
    let changed = serde_json::json!({"version": 1, "layout": {"open": ["notes.md", "todo.md"]}});
    assert!(
        http.put(&endpoint)
            .json(&serde_json::json!({"state": changed}))
            .send()
            .await
            .unwrap()
            .status()
            .is_success()
    );
    assert_eq!(store.load_ui(), Some(changed));
    assert!(
        hickory_workspace::WorkspaceStore::for_project(other.path())
            .unwrap()
            .load_ui()
            .is_none()
    );
    store.clear_ui().unwrap();
}
</storycode:copy>

<a id="output-7c7c598c04b7"></a>
## `apps/web/src/shell/layout.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Folder Remembers Its Window](contract.md).

<a id="f-7c7c598c04b7-ViewKind"></a>
### `ViewKind`

The shell's arrangement: panes of tabs, split horizontally and vertically.
Pure data and pure functions, deliberately. Every hard question about a
tiling shell — what happens to the focus when you close the pane it was in,
where a file opens, what a split does to the sizes — is arithmetic, and
arithmetic is answerable at a desk rather than by clicking. The React that
renders this comes later and gets to be dull.
See docs/specs/freeform/shell-layouts.md.
What a tab shows. Regions come from a layout; tools come from the app.
"tree" is the folder tree pane — a distinct kind, because drops treat it
differently (see dragDrop.ts). "untitled" is a document that does not
exist yet: a durable draft, adopted into a "document" tab when it is saved.
"file" is a plain file — in the folder, but neither a document nor a
woven output: it has no owning docId and edits save to disk directly.

<storycode:copy id="f-7c7c598c04b7-ViewKind">
// The shell's arrangement: panes of tabs, split horizontally and vertically.
//
// Pure data and pure functions, deliberately. Every hard question about a
// tiling shell — what happens to the focus when you close the pane it was in,
// where a file opens, what a split does to the sizes — is arithmetic, and
// arithmetic is answerable at a desk rather than by clicking. The React that
// renders this comes later and gets to be dull.
//
// See docs/specs/freeform/shell-layouts.md.

/** What a tab shows. Regions come from a layout; tools come from the app.
 * "tree" is the folder tree pane — a distinct kind, because drops treat it
 * differently (see dragDrop.ts). "untitled" is a document that does not
 * exist yet: a durable draft, adopted into a "document" tab when it is saved.
 * "file" is a plain file — in the folder, but neither a document nor a
 * woven output: it has no owning docId and edits save to disk directly. */
export type ViewKind =
  | "document"
  | "generated"
  | "file"
  | "tool"
  | "tree"
  | "untitled"
  // Text on its way to becoming a note: the typed way into a notes folder,
  // beside downloading a file into the inbox and copying one there. Not a
  // document — it owns no file until it is saved.
  | "scratchpad"
  // A terminal session, addressed by its server-side id. The pane shows the
  // session; it does not own it, which is why closing the tab leaves the work
  // running. See src/terminal/.
  | "terminal"
  // The agent conversation. A PANE, not a dock along the bottom: the chat is
  // a thing you read and scroll beside your work, and a dock made it a mode
  // that covered the document it was about. As a pane it can be moved,
  // split, resized, and closed like everything else.
  | "chat";

</storycode:copy>
<a id="f-7c7c598c04b7-Tab"></a>
### `Tab`

<storycode:copy id="f-7c7c598c04b7-Tab">
export interface Tab {
  id: string;
  kind: ViewKind;
  /** The document path, generated file path, or tool name. */
  target: string;
  /** What the tab says. Defaults to the target's last segment. */
  title?: string;
  /**
   * Which document this tab belongs to. On a "document" tab, the document's
   * own id; on a "generated" tab, the id of the document that produced the
   * file. The workspace holds several documents at once, and a generated
   * file's fetches and saves must go to the document that owns it — the
   * target path alone cannot say which one that is.
   */
  docId?: string;
}

</storycode:copy>
<a id="f-7c7c598c04b7-Pane"></a>
### `Pane`

<storycode:copy id="f-7c7c598c04b7-Pane">
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
  /**
   * Folded down to an icon strip (see collapsePane). The tabs stay exactly
   * where they were — collapsing is presentation, not tree surgery, which is
   * what makes expanding a perfect round-trip.
   */
  collapsed?: boolean;
}

</storycode:copy>
<a id="f-7c7c598c04b7-Split"></a>
### `Split`

<storycode:copy id="f-7c7c598c04b7-Split">
export interface Split {
  type: "split";
  id: string;
  direction: "row" | "column";
  children: Node[];
  /** One fraction per child, summing to 1. */
  sizes: number[];
}

</storycode:copy>
<a id="f-7c7c598c04b7-Node"></a>
### `Node`

<storycode:copy id="f-7c7c598c04b7-Node">
export type Node = Pane | Split;

</storycode:copy>
<a id="f-7c7c598c04b7-Layout"></a>
### `Layout`

<storycode:copy id="f-7c7c598c04b7-Layout">
export interface Layout {
  root: Node;
  /** The pane that takes the next open, and the keystrokes. */
  focus: string;
}

</storycode:copy>
<a id="f-7c7c598c04b7-counter"></a>
### `counter`

<storycode:copy id="f-7c7c598c04b7-counter">
// ---------------------------------------------------------------------------
// Building
// ---------------------------------------------------------------------------

let counter = 0;
</storycode:copy>
<a id="f-7c7c598c04b7-nextId"></a>
### `nextId`

Ids are per-session and never persisted, so a counter is enough.

<storycode:copy id="f-7c7c598c04b7-nextId">
/** Ids are per-session and never persisted, so a counter is enough. */
function nextId(prefix: string): string {
  counter += 1;
  return `${prefix}-${counter}`;
}

</storycode:copy>
<a id="f-7c7c598c04b7-emptyPane"></a>
### `emptyPane`

<storycode:copy id="f-7c7c598c04b7-emptyPane">
export function emptyPane(region?: string): Pane {
  return { type: "pane", id: nextId("pane"), tabs: [], active: 0, region };
}

</storycode:copy>
<a id="f-7c7c598c04b7-freeform"></a>
### `freeform`

A layout with one empty pane: what opening a single file starts from.

<storycode:copy id="f-7c7c598c04b7-freeform">
/** A layout with one empty pane: what opening a single file starts from. */
export function freeform(): Layout {
  const pane = emptyPane();
  return { root: pane, focus: pane.id };
}

</storycode:copy>
<a id="f-7c7c598c04b7-tab"></a>
### `tab`

<storycode:copy id="f-7c7c598c04b7-tab">
export function tab(kind: ViewKind, target: string, title?: string, docId?: string): Tab {
  return { id: nextId("tab"), kind, target, title, docId };
}

</storycode:copy>
<a id="f-7c7c598c04b7-withTree"></a>
### `withTree`

Put a tree pane at the left edge of an arrangement, modest width.
A wrapper rather than a region: the tree describes the folder, not a part
of the codebase, and it must survive whichever layout the person picks.
The pane it makes is a normal pane — closable, resizable, a drop target
for nothing (dragDrop.ts refuses joins into it).

<storycode:copy id="f-7c7c598c04b7-withTree">
/**
 * Put a tree pane at the left edge of an arrangement, modest width.
 *
 * A wrapper rather than a region: the tree describes the folder, not a part
 * of the codebase, and it must survive whichever layout the person picks.
 * The pane it makes is a normal pane — closable, resizable, a drop target
 * for nothing (dragDrop.ts refuses joins into it).
 */
export function withTree(layout: Layout, entry: Tab, size = 0.2): Layout {
  const pane: Pane = { ...emptyPane(), tabs: [entry], active: 0 };
  const root: Split = {
    type: "split",
    id: nextId("split"),
    direction: "row",
    children: [pane, layout.root],
    sizes: [size, 1 - size],
  };
  // The focus stays where it was: the tree is furniture, not what you came
  // here to edit.
  return { root, focus: layout.focus };
}

</storycode:copy>
<a id="f-7c7c598c04b7-withRail"></a>
### `withRail`

Put a pane at the RIGHT edge of an arrangement.
`withTree`'s mirror image, and separate rather than parameterised because
the two differ in more than a side: the tree takes no focus (it is
furniture you navigate with), and neither does this — opening the chat
should not take the caret out of the document you are asking about.

<storycode:copy id="f-7c7c598c04b7-withRail">
/**
 * Put a pane at the RIGHT edge of an arrangement.
 *
 * `withTree`'s mirror image, and separate rather than parameterised because
 * the two differ in more than a side: the tree takes no focus (it is
 * furniture you navigate with), and neither does this — opening the chat
 * should not take the caret out of the document you are asking about.
 */
export function withRail(layout: Layout, entry: Tab, size = 0.26): Layout {
  const pane: Pane = { ...emptyPane(), tabs: [entry], active: 0 };
  const root: Split = {
    type: "split",
    id: nextId("split"),
    direction: "row",
    children: [layout.root, pane],
    sizes: [1 - size, size],
  };
  return { root, focus: layout.focus };
}

</storycode:copy>
<a id="f-7c7c598c04b7-treePane"></a>
### `treePane`

The pane holding the folder tree, when one is open.

<storycode:copy id="f-7c7c598c04b7-treePane">
/** The pane holding the folder tree, when one is open. */
export function treePane(layout: Layout): Pane | null {
  return panes(layout.root).find((pane) => pane.tabs.some((t) => t.kind === "tree")) ?? null;
}

</storycode:copy>
<a id="f-7c7c598c04b7-reid"></a>
### `reid`

Walking
The same arrangement with every id freshly minted.
Ids come from a per-session counter that restarts at zero, which was fine
while a layout only ever lived in memory. A RESTORED layout brings its old
ids back with it, and the counter would happily hand `pane-1` out again to
the next split — two panes with one id, and every lookup finding whichever
came first. Re-minting on the way in keeps the invariant that every id in a
live layout came from this one generator.

<storycode:copy id="f-7c7c598c04b7-reid">
// ---------------------------------------------------------------------------
// Walking
// ---------------------------------------------------------------------------

/**
 * The same arrangement with every id freshly minted.
 *
 * Ids come from a per-session counter that restarts at zero, which was fine
 * while a layout only ever lived in memory. A RESTORED layout brings its old
 * ids back with it, and the counter would happily hand `pane-1` out again to
 * the next split — two panes with one id, and every lookup finding whichever
 * came first. Re-minting on the way in keeps the invariant that every id in a
 * live layout came from this one generator.
 */
export function reid(layout: Layout): Layout {
  let focus = layout.focus;
  const walk = (node: Node): Node => {
    if (node.type === "pane") {
      const id = nextId("pane");
      if (node.id === layout.focus) focus = id;
      return { ...node, id, tabs: node.tabs.map((t) => ({ ...t, id: t.kind === "untitled" ? t.id : nextId("tab") })) };
    }
    return { ...node, id: nextId("split"), children: node.children.map(walk) };
  };
  const root = walk(layout.root);
  // A focus naming a pane that is no longer there would leave every "open
  // here" with nowhere to go.
  const live = panes(root);
  if (!live.some((p) => p.id === focus)) focus = live[0]?.id ?? focus;
  return { root, focus };
}

</storycode:copy>
<a id="f-7c7c598c04b7-panes"></a>
### `panes`

<storycode:copy id="f-7c7c598c04b7-panes">
export function panes(node: Node): Pane[] {
  return node.type === "pane" ? [node] : node.children.flatMap(panes);
}

</storycode:copy>
<a id="f-7c7c598c04b7-paneById"></a>
### `paneById`

<storycode:copy id="f-7c7c598c04b7-paneById">
export function paneById(layout: Layout, id: string): Pane | null {
  return panes(layout.root).find((pane) => pane.id === id) ?? null;
}

</storycode:copy>
<a id="f-7c7c598c04b7-focused"></a>
### `focused`

<storycode:copy id="f-7c7c598c04b7-focused">
export function focused(layout: Layout): Pane | null {
  return paneById(layout, layout.focus);
}

</storycode:copy>
<a id="f-7c7c598c04b7-replace"></a>
### `replace`

Rebuild the tree with one pane replaced by whatever `make` returns.

<storycode:copy id="f-7c7c598c04b7-replace">
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

</storycode:copy>
<a id="f-7c7c598c04b7-normalize"></a>
### `normalize`

<storycode:copy id="f-7c7c598c04b7-normalize">
function normalize(sizes: number[]): number[] {
  const total = sizes.reduce((sum, size) => sum + size, 0);
  if (total <= 0) return sizes.map(() => 1 / sizes.length);
  return sizes.map((size) => size / total);
}

</storycode:copy>
<a id="f-7c7c598c04b7-open"></a>
### `open`

Opening
Open a target, or focus it where it is already open.
Reopening a file that is already in front of you should not give you a
second copy of it — two tabs of one file is a state with no honest way to
decide which one an edit belongs to.

<storycode:copy id="f-7c7c598c04b7-open">
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

</storycode:copy>
<a id="f-7c7c598c04b7-activate"></a>
### `activate`

<storycode:copy id="f-7c7c598c04b7-activate">
export function activate(layout: Layout, paneId: string, index: number): Layout {
  const root = replace(layout.root, paneId, (pane) => ({
    ...pane,
    active: Math.max(0, Math.min(index, pane.tabs.length - 1)),
  }));
  return { root: root ?? layout.root, focus: paneId };
}

</storycode:copy>
<a id="f-7c7c598c04b7-closeTab"></a>
### `closeTab`

Close one tab.
An empty pane is kept when it is the only one — there has to be somewhere
for the next file to go — and removed otherwise, because a strip of nothing
is not worth the width.

<storycode:copy id="f-7c7c598c04b7-closeTab">
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
    return { root, focus: layout.focus !== paneId ? layout.focus : remaining[Math.min(panes(layout.root).findIndex(item => item.id === paneId), remaining.length - 1)]?.id ?? layout.focus };
  }

  const removed = pane.tabs.findIndex((candidate) => candidate.id === tabId);
  const root = replace(layout.root, paneId, (current) => ({
    ...current,
    tabs,
    // Stay where you were looking: closing a tab before the active one must
    // not scroll the selection along with it.
    active: Math.max(0, Math.min(removed <= current.active ? current.active - 1 : current.active, tabs.length - 1)),
  }));
  return { root: root ?? layout.root, focus: layout.focus };
}

</storycode:copy>
<a id="f-7c7c598c04b7-closePane"></a>
### `closePane`

Close a whole pane, tabs and all.
The last pane stays, emptied: there has to be somewhere for the next file
to go, and an app with no panes has nothing to show and no way back.

<storycode:copy id="f-7c7c598c04b7-closePane">
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

</storycode:copy>
<a id="f-7c7c598c04b7-collapsePane"></a>
### `collapsePane`

Collapsing
Fold a pane down to an icon strip.
Nothing about the pane changes but the flag: the tabs, the active index,
and the split fraction all stay, so expanding restores exactly what was
there. The last visible pane refuses to collapse — a shell that is all
strips has nothing left to show and no obvious way back — and a collapsed
pane gives up the focus, because keystrokes aimed at a pane you cannot see
would edit an arrangement you cannot check.

<storycode:copy id="f-7c7c598c04b7-collapsePane">
// ---------------------------------------------------------------------------
// Collapsing
// ---------------------------------------------------------------------------

/**
 * Fold a pane down to an icon strip.
 *
 * Nothing about the pane changes but the flag: the tabs, the active index,
 * and the split fraction all stay, so expanding restores exactly what was
 * there. The last visible pane refuses to collapse — a shell that is all
 * strips has nothing left to show and no obvious way back — and a collapsed
 * pane gives up the focus, because keystrokes aimed at a pane you cannot see
 * would edit an arrangement you cannot check.
 */
export function collapsePane(layout: Layout, paneId: string): Layout {
  const expanded = panes(layout.root).filter((pane) => !pane.collapsed);
  if (expanded.length <= 1 && expanded[0]?.id === paneId) return layout;
  const root = replace(layout.root, paneId, (pane) => ({ ...pane, collapsed: true }));
  if (root === null) return layout;
  const focusNext =
    layout.focus === paneId
      ? (panes(root).find((pane) => !pane.collapsed && pane.id !== paneId)?.id ?? layout.focus)
      : layout.focus;
  return { root, focus: focusNext };
}

</storycode:copy>
<a id="f-7c7c598c04b7-expandPane"></a>
### `expandPane`

Unfold a collapsed pane. The pane takes the focus: expanding is how a
person says "show me this again", and what they mean by it is next.

<storycode:copy id="f-7c7c598c04b7-expandPane">
/** Unfold a collapsed pane. The pane takes the focus: expanding is how a
 * person says "show me this again", and what they mean by it is next. */
export function expandPane(layout: Layout, paneId: string): Layout {
  const pane = paneById(layout, paneId);
  if (!pane || !pane.collapsed) return layout;
  const root = replace(layout.root, paneId, (current) => ({ ...current, collapsed: false }));
  return { root: root ?? layout.root, focus: paneId };
}

</storycode:copy>
<a id="f-7c7c598c04b7-split"></a>
### `split`

Splitting
Split a pane, putting a new empty pane beside it.
The new pane takes the focus, because splitting is how a person says "and
also show me…", and what they mean by it is next.

<storycode:copy id="f-7c7c598c04b7-split">
// ---------------------------------------------------------------------------
// Splitting
// ---------------------------------------------------------------------------

/**
 * Split a pane, putting a new empty pane beside it.
 *
 * The new pane takes the focus, because splitting is how a person says "and
 * also show me…", and what they mean by it is next.
 */
export function split(layout: Layout, paneId: string, direction: "row" | "column", before = false): Layout {
  const fresh = emptyPane(paneById(layout, paneId)?.region);
  const root = replace(layout.root, paneId, (pane) => ({
    type: "split",
    id: nextId("split"),
    direction,
    children: before ? [fresh, pane] : [pane, fresh],
    sizes: [0.5, 0.5],
  }));
  return { root: root ?? layout.root, focus: fresh.id };
}

</storycode:copy>
<a id="f-7c7c598c04b7-resize"></a>
### `resize`

Drag a divider: `sizes` are fractions and always sum to 1.

<storycode:copy id="f-7c7c598c04b7-resize">
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

</storycode:copy>
<a id="f-7c7c598c04b7-focus"></a>
### `focus`

<storycode:copy id="f-7c7c598c04b7-focus">
export function focus(layout: Layout, paneId: string): Layout {
  return paneById(layout, paneId) ? { ...layout, focus: paneId } : layout;
}

</storycode:copy>
<a id="f-7c7c598c04b7-Region"></a>
### `Region`

Declared layouts
One box of a declared layout: a name and what belongs to it.

<storycode:copy id="f-7c7c598c04b7-Region">
// ---------------------------------------------------------------------------
// Declared layouts
// ---------------------------------------------------------------------------

/** One box of a declared layout: a name and what belongs to it. */
export interface Region {
  name: string;
  /** Globs, in the order they were declared. First match wins. */
  match: string[];
}

</storycode:copy>
<a id="f-7c7c598c04b7-regionOf"></a>
### `regionOf`

Which region a path belongs to.
First match wins rather than most-specific: the order is written down in the
document, so it is something a person chose and can see, where "most
specific" is a rule they would have to work out.

<storycode:copy id="f-7c7c598c04b7-regionOf">
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

</storycode:copy>
<a id="f-7c7c598c04b7-matches"></a>
### `matches`

Glob matching, in the small dialect a layout needs: `*` within a segment,
`**` across segments, `?` for one character.
Written segment by segment rather than by rewriting the whole string:
`**` behaves differently depending on where it sits — trailing, it takes
the separator with it, so `docs/**` matches `docs/spec.md` AND `docs`
itself — and a single pass of replacements cannot say that.

<storycode:copy id="f-7c7c598c04b7-matches">
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

</storycode:copy>
<a id="f-7c7c598c04b7-fromRegions"></a>
### `fromRegions`

Build a layout from declared regions: one pane each, side by side.

<storycode:copy id="f-7c7c598c04b7-fromRegions">
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

</storycode:copy>
<a id="f-7c7c598c04b7-besides"></a>
### `besides`

A pane that is not this one, preferring a neighbour that is empty.
"Open to the side" exists because two things you want to compare have to be
on screen at once: a generated file opened over the document that produced
it hides the very thing it should sit beside, and no ribbon can be drawn
between a pane and itself.

<storycode:copy id="f-7c7c598c04b7-besides">
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

</storycode:copy>
<a id="f-7c7c598c04b7-paneFor"></a>
### `paneFor`

Where a file should open under a declared layout.
This is the question a tiling shell otherwise has to invent an answer to,
and the declaration answers it: the pane whose region claims the path. A
path no region claims goes to the focus, which is the freeform behaviour and
the only honest fallback — refusing to open it would punish the person for
an incomplete declaration.

<storycode:copy id="f-7c7c598c04b7-paneFor">
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
</storycode:copy>

### Assemble `apps/web/src/shell/layout.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/layout.ts"><storycode:paste select="#f-7c7c598c04b7-ViewKind"/><storycode:paste select="#f-7c7c598c04b7-Tab"/><storycode:paste select="#f-7c7c598c04b7-Pane"/><storycode:paste select="#f-7c7c598c04b7-Split"/><storycode:paste select="#f-7c7c598c04b7-Node"/><storycode:paste select="#f-7c7c598c04b7-Layout"/><storycode:paste select="#f-7c7c598c04b7-counter"/><storycode:paste select="#f-7c7c598c04b7-nextId"/><storycode:paste select="#f-7c7c598c04b7-emptyPane"/><storycode:paste select="#f-7c7c598c04b7-freeform"/><storycode:paste select="#f-7c7c598c04b7-tab"/><storycode:paste select="#f-7c7c598c04b7-withTree"/><storycode:paste select="#f-7c7c598c04b7-withRail"/><storycode:paste select="#f-7c7c598c04b7-treePane"/><storycode:paste select="#f-7c7c598c04b7-reid"/><storycode:paste select="#f-7c7c598c04b7-panes"/><storycode:paste select="#f-7c7c598c04b7-paneById"/><storycode:paste select="#f-7c7c598c04b7-focused"/><storycode:paste select="#f-7c7c598c04b7-replace"/><storycode:paste select="#f-7c7c598c04b7-normalize"/><storycode:paste select="#f-7c7c598c04b7-open"/><storycode:paste select="#f-7c7c598c04b7-activate"/><storycode:paste select="#f-7c7c598c04b7-closeTab"/><storycode:paste select="#f-7c7c598c04b7-closePane"/><storycode:paste select="#f-7c7c598c04b7-collapsePane"/><storycode:paste select="#f-7c7c598c04b7-expandPane"/><storycode:paste select="#f-7c7c598c04b7-split"/><storycode:paste select="#f-7c7c598c04b7-resize"/><storycode:paste select="#f-7c7c598c04b7-focus"/><storycode:paste select="#f-7c7c598c04b7-Region"/><storycode:paste select="#f-7c7c598c04b7-regionOf"/><storycode:paste select="#f-7c7c598c04b7-matches"/><storycode:paste select="#f-7c7c598c04b7-fromRegions"/><storycode:paste select="#f-7c7c598c04b7-besides"/><storycode:paste select="#f-7c7c598c04b7-paneFor"/></storycode:file>

<a id="output-7589fbbd1963"></a>
## `apps/web/src/views/restoreFolderLayout.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Folder Remembers Its Window](contract.md).

<a id="f-7589fbbd1963-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-7589fbbd1963-expression_statement">
import { expect, it } from "vitest";
import { freeform, open, panes, tab } from "../shell/layout";
import { normalizeUi } from "../lib/uiState";
import { restoreFolderLayout } from "./restoreFolderLayout";

// Guarantee: docs/guarantees/authoring/a-folder-remembers-its-window.md
it("restores the folder while retaining an edited unnamed note", () => {
  const draft = tab("untitled", "untitled", "My draft");
  const current = open(freeform(), draft);
  const stored = open(freeform(), tab("file", "notes.txt"));
  const restored = restoreFolderLayout(current, stored, true);
  expect(panes(restored.root).flatMap(pane => pane.tabs)).toEqual([panes(stored.root)[0].tabs[0], draft]);
});

</storycode:copy>
<a id="f-7589fbbd1963-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-7589fbbd1963-expression_statement-2">
it("replaces only the untouched introduction and preserves the saved focus", () => {
  const intro = tab("untitled", "untitled");
  const stored = open(freeform(), tab("file", "notes.txt"));
  expect(restoreFolderLayout(open(freeform(), intro), stored, true, intro.id)).toBe(stored);
});

</storycode:copy>
<a id="f-7589fbbd1963-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-7589fbbd1963-expression_statement-3">
it("retains unnamed recovery identities when normalizing a stored layout", () => {
  const draft = tab("untitled", "untitled");
  const stored = open(freeform(), draft);
  const normalized = normalizeUi({ version: 1, layout: stored }).layout!;
  expect(panes(normalized.root)[0].tabs[0].id).toBe(draft.id);
});
</storycode:copy>

### Assemble `apps/web/src/views/restoreFolderLayout.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/restoreFolderLayout.test.ts"><storycode:paste select="#f-7589fbbd1963-expression_statement"/><storycode:paste select="#f-7589fbbd1963-expression_statement-2"/><storycode:paste select="#f-7589fbbd1963-expression_statement-3"/></storycode:file>

<a id="output-177e5ee0e73a"></a>
## `apps/web/src/views/restoreFolderLayout.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Folder Remembers Its Window](contract.md).

<a id="f-177e5ee0e73a-restoreFolderLayout"></a>
### `restoreFolderLayout`

<storycode:copy id="f-177e5ee0e73a-restoreFolderLayout">
import { open, panes, type Layout } from "../shell/layout";
import { isWorkspaceEmpty } from "./workspaceState";

/** Restore folder geometry while keeping editors opened before the folder. */
export function restoreFolderLayout(current: Layout, restored: Layout, folder: boolean, introduction?: string): Layout {
  if (!folder) return introduction || !isWorkspaceEmpty(current) ? current : restored;
  const existing = panes(restored.root).flatMap(pane => pane.tabs);
  const carried = panes(current.root).flatMap(pane => pane.tabs).filter(tab =>
    tab.kind !== "tree" && tab.id !== introduction && !existing.some(saved =>
      saved.kind === tab.kind && (tab.kind === "untitled" ? saved.id === tab.id : saved.target === tab.target)));
  return carried.reduce((layout, tab) => open(layout, tab), restored);
}
</storycode:copy>

### Assemble `apps/web/src/views/restoreFolderLayout.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/restoreFolderLayout.ts"><storycode:paste select="#f-177e5ee0e73a-restoreFolderLayout"/></storycode:file>

<a id="output-307090b7547a"></a>
## `apps/web/src/views/useWorkspaceUi.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Folder Remembers Its Window](contract.md).

<a id="f-307090b7547a-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-307090b7547a-expression_statement">
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { useState } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { freeform, open, panes, split, tab, type Layout } from "../shell/layout";
import { useWorkspaceUi, UI_SAVE_DEBOUNCE_MS } from "./useWorkspaceUi";

afterEach(() => { cleanup(); vi.useRealTimers(); vi.restoreAllMocks(); });

</storycode:copy>
<a id="f-307090b7547a-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-307090b7547a-expression_statement-2">
// Guarantee: docs/guarantees/authoring/a-folder-remembers-its-window.md
it("loads each folder before saving, restoring its tabs and split proportions", async () => {
  let stored = open(freeform(), tab("file", "a.txt"));
  stored = split(stored, stored.focus, "column");
  stored = open(stored, tab("file", "b.txt"));
  const load = vi.spyOn(api, "workspaceUi").mockResolvedValue({ state: { version: 1, layout: stored } });
  const save = vi.spyOn(api, "saveWorkspaceUi").mockResolvedValue({ ok: true });
  const { result, rerender } = renderHook(({ scope }: { scope: string | null }) => {
    const [layout, setLayout] = useState<Layout>(freeform);
    return { layout, ...useWorkspaceUi(layout, setLayout, scope) };
  }, { initialProps: { scope: null as string | null } });
  expect(load).not.toHaveBeenCalled();
  rerender({ scope: "/notes/a" });
  await waitFor(() => expect(result.current.hydrated).toBe(true));
  expect(result.current.layout.root.type).toBe("split");
  expect(panes(result.current.layout.root).flatMap(pane => pane.tabs.map(tab => tab.target))).toEqual(["a.txt", "b.txt"]);
  expect(save).not.toHaveBeenCalled();
  const other = open(freeform(), tab("file", "other.txt"));
  load.mockResolvedValue({ state: { version: 1, layout: other } });
  rerender({ scope: "/notes/b" });
  await waitFor(() => expect(panes(result.current.layout.root)[0].tabs[0].target).toBe("other.txt"));
  await act(() => result.current.flush());
  expect(save).toHaveBeenLastCalledWith(expect.objectContaining({ layout: result.current.layout }), false);
});

</storycode:copy>
<a id="f-307090b7547a-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-307090b7547a-expression_statement-3">
it("flushes the latest layout immediately on close or pagehide, including empty splits", async () => {
  vi.spyOn(api, "workspaceUi").mockResolvedValue({ state: null });
  const save = vi.spyOn(api, "saveWorkspaceUi").mockResolvedValue({ ok: true });
  const { result } = renderHook(() => {
    const [layout, setLayout] = useState<Layout>(freeform);
    return { setLayout, layout, ...useWorkspaceUi(layout, setLayout) };
  });
  await waitFor(() => expect(result.current.hydrated).toBe(true));
  vi.useFakeTimers();
  act(() => result.current.setLayout(current => split(current, current.focus, "row")));
  expect(save).not.toHaveBeenCalled();
  await act(() => result.current.flush());
  expect(save).toHaveBeenLastCalledWith(expect.objectContaining({ layout: result.current.layout }), false);
  await act(async () => { window.dispatchEvent(new Event("pagehide")); });
  expect(save).toHaveBeenLastCalledWith(expect.objectContaining({ layout: result.current.layout }), true);
  await act(async () => { vi.advanceTimersByTime(UI_SAVE_DEBOUNCE_MS); });
});

</storycode:copy>
<a id="f-307090b7547a-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-307090b7547a-expression_statement-4">
it("ignores a late response from the previous folder", async () => {
  let answer!: (value: { state: null }) => void;
  const load = vi.spyOn(api, "workspaceUi").mockReturnValueOnce(new Promise(resolve => { answer = resolve; }));
  load.mockResolvedValue({ state: { version: 1, wrap: { "b.txt": 60 } } });
  vi.spyOn(api, "saveWorkspaceUi").mockResolvedValue({ ok: true });
  const apply = vi.fn();
  const { result, rerender } = renderHook(({ scope }) => useWorkspaceUi(freeform(), apply, scope), { initialProps: { scope: "a" } });
  rerender({ scope: "b" });
  await waitFor(() => expect(result.current.hydrated).toBe(true));
  await act(async () => answer({ state: null }));
  expect(result.current.wrapFor("b.txt")).toBe(60);
});
</storycode:copy>

### Assemble `apps/web/src/views/useWorkspaceUi.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useWorkspaceUi.test.tsx"><storycode:paste select="#f-307090b7547a-expression_statement"/><storycode:paste select="#f-307090b7547a-expression_statement-2"/><storycode:paste select="#f-307090b7547a-expression_statement-3"/><storycode:paste select="#f-307090b7547a-expression_statement-4"/></storycode:file>

<a id="output-71ffc8e32ec3"></a>
## `apps/web/src/views/useWorkspaceUi.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Folder Remembers Its Window](contract.md).

<a id="f-71ffc8e32ec3-UI_SAVE_DEBOUNCE_MS"></a>
### `UI_SAVE_DEBOUNCE_MS`

Hydrating the window from what it looked like last time, and writing that
down again as it changes.
The ordering here is the whole difficulty, and it is worth stating before
the code. Two things want to arrange the workspace at launch:
1. The **stored layout**, which arrives over a fetch — so, late.
2. The **route** (`#/docs/<id>`), which is a request against the workspace
and runs on the first render — so, early.
If they race, the route wins by arriving first, the workspace is no longer
empty when the layout lands, and the restore is silently dropped. So the
route's opener WAITS: `hydrated` gates it, and it is set whether the fetch
succeeded, failed, or found nothing. A window that will not open because
its layout store is unreachable would be a far worse bug than a window that
opens with default tabs, so there is no path here that leaves `hydrated`
false.
Folder identity gates both loading and saving. A late response from an old
folder is ignored; the caller preserves any editors already in use.

<storycode:copy id="f-71ffc8e32ec3-UI_SAVE_DEBOUNCE_MS">
// Hydrating the window from what it looked like last time, and writing that
// down again as it changes.
//
// The ordering here is the whole difficulty, and it is worth stating before
// the code. Two things want to arrange the workspace at launch:
//
//  1. The **stored layout**, which arrives over a fetch — so, late.
//  2. The **route** (`#/docs/<id>`), which is a request against the workspace
//     and runs on the first render — so, early.
//
// If they race, the route wins by arriving first, the workspace is no longer
// empty when the layout lands, and the restore is silently dropped. So the
// route's opener WAITS: `hydrated` gates it, and it is set whether the fetch
// succeeded, failed, or found nothing. A window that will not open because
// its layout store is unreachable would be a far worse bug than a window that
// opens with default tabs, so there is no path here that leaves `hydrated`
// false.
//
// Folder identity gates both loading and saving. A late response from an old
// folder is ignored; the caller preserves any editors already in use.

import { useCallback, useEffect, useRef, useState } from "react";

import { api } from "../api/client";
import {
  emptyUi,
  normalizeUi,
  withTableLayout,
  withWrap,
  withZoom,
  wrapFor,
  zoomFor,
  type WorkspaceUi,
} from "../lib/uiState";
import type { SpatialState } from "../shell/spatialLayout";
import type { Layout } from "../shell/layout";
import type { TableLayout } from "../components/TablePanel";

/** How long the arrangement must sit still before it is written down. */
export const UI_SAVE_DEBOUNCE_MS = 600;

</storycode:copy>
<a id="f-71ffc8e32ec3-WorkspaceUiHandle"></a>
### `WorkspaceUiHandle`

<storycode:copy id="f-71ffc8e32ec3-WorkspaceUiHandle">
export interface WorkspaceUiHandle {
  /** False until the stored state has been asked for and answered. The
   * route's opener must not touch the layout before this. */
  hydrated: boolean;
  restored: boolean;
  spatial: SpatialState | undefined;
  setSpatial: (state: SpatialState) => void;
  flush: () => Promise<void>;
  /** The prose measure for a tab, by its target path. */
  wrapFor: (target: string) => number;
  /** Record a measure the reader dragged on a ruler. */
  setWrap: (target: string, column: number) => void;
  /** How far one tab is zoomed, and a way to move it. The whole-window level
   * is not here — it belongs to the screen, not the project. */
  zoomFor: (target: string) => number;
  setZoom: (target: string, level: number) => void;
  /** How big every table was left, keyed by `tableKey` — the whole record,
   * because the editor names its own tables and a per-key getter would make
   * every table re-render whenever any of them moved. */
  tables: Record<string, TableLayout>;
  setTableLayout: (key: string, size: TableLayout) => void;
}

</storycode:copy>
<a id="f-71ffc8e32ec3-useWorkspaceUi"></a>
### `useWorkspaceUi`

Load each folder's stored window state, then keep it up to date.
The caller decides which existing editors to carry into the arrangement.

<storycode:copy id="f-71ffc8e32ec3-useWorkspaceUi">
/**
 * Load each folder's stored window state, then keep it up to date.
 *
 * The caller decides which existing editors to carry into the arrangement.
 */
export function useWorkspaceUi(
  layout: Layout,
  applyLayout: (restored: Layout) => void,
  scope: string | null = "window",
): WorkspaceUiHandle {
  const [ui, setUi] = useState<WorkspaceUi>(emptyUi);
  const [loadedScope, setLoadedScope] = useState<string | null>(null);
  const [restored, setRestored] = useState(false);
  const hydrated = scope !== null && loadedScope === scope;
  const applyRef = useRef(applyLayout);
  applyRef.current = applyLayout;
  const uiRef = useRef(ui);
  uiRef.current = ui;
  const pending = useRef(Promise.resolve());
  const scopeRef = useRef(scope); scopeRef.current = scope;
  // The layout is read through a ref by the save timer, so a change to the
  // arrangement does not restart the timer on every keystroke that happens to
  // re-render this view.
  const layoutRef = useRef(layout);
  layoutRef.current = layout;

  useEffect(() => {
    if (scope === null) return;
    let cancelled = false;
    setRestored(false);
    void api.workspaceUi().then(
      (response) => {
        if (cancelled) return;
        const stored = normalizeUi(response.state);
        setUi(stored);
        if (stored.layout) applyRef.current(stored.layout);
        setRestored(stored.layout !== null);
        setLoadedScope(scope);
      },
      () => {
        // No store, no home directory, a read-only disk: the window opens
        // with default tabs and says nothing. There is nothing here a person
        // could act on, and the route must not stay blocked.
        if (!cancelled) { setUi(emptyUi()); setLoadedScope(scope); }
      },
    );
    return () => {
      cancelled = true;
    };
  }, [scope]);

  const flush = useCallback(async (keepalive = false) => {
    if (!hydrated) return;
    const state: WorkspaceUi = { ...uiRef.current, layout: layoutRef.current };
    pending.current = pending.current.then(() => {
      if (scopeRef.current === scope) return api.saveWorkspaceUi(state, keepalive);
    }).then(() => {}, () => {});
    await pending.current;
  }, [hydrated, scope]);

  useEffect(() => {
    const hide = () => { void flush(true); };
    window.addEventListener("pagehide", hide);
    return () => window.removeEventListener("pagehide", hide);
  }, [flush]);

  // Write it down, debounced, once hydration has happened — saving before the
  // load returns would overwrite the stored layout with the default one.
  useEffect(() => {
    if (!hydrated) return;
    const timer = window.setTimeout(() => { void flush(); }, UI_SAVE_DEBOUNCE_MS);
    return () => window.clearTimeout(timer);
  }, [hydrated, ui, layout, flush]);

  const setWrap = useCallback((target: string, column: number) => {
    setUi((current) => withWrap(current, target, column));
  }, []);

  const setZoom = useCallback((target: string, level: number) => {
    setUi((current) => withZoom(current, target, level));
  }, []);

  const setTableLayout = useCallback((key: string, size: TableLayout) => {
    setUi((current) => withTableLayout(current, key, size));
  }, []);

  return {
    hydrated,
    restored,
    spatial: ui.spatial,
    setSpatial: useCallback((spatial: SpatialState) => {
      uiRef.current = { ...uiRef.current, spatial };
      setUi(uiRef.current);
    }, []),
    flush,
    wrapFor: useCallback((target: string) => wrapFor(ui, target), [ui]),
    setWrap,
    zoomFor: useCallback((target: string) => zoomFor(ui, target), [ui]),
    setZoom,
    tables: ui.tables,
    setTableLayout,
  };
}
</storycode:copy>

### Assemble `apps/web/src/views/useWorkspaceUi.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useWorkspaceUi.ts"><storycode:paste select="#f-71ffc8e32ec3-UI_SAVE_DEBOUNCE_MS"/><storycode:paste select="#f-71ffc8e32ec3-WorkspaceUiHandle"/><storycode:paste select="#f-71ffc8e32ec3-useWorkspaceUi"/></storycode:file>

## Implementation review

### Implementation

- [apps/desktop/src-tauri/src/folder_session.rs](../../../mechanisms/apps/desktop/src-tauri/src/folder_session.rs.md#output-46ac251403d6)
- [apps/web/src/lib/uiState.ts](../../../mechanisms/apps/web/src/lib/uiState.ts.md#output-7183338fb20a)
- [apps/web/src/shell/SpatialShellView.tsx](../../../mechanisms/apps/web/src/shell/SpatialShellView.tsx.md#output-64456f67b44e)
- [apps/web/src/shell/layout.ts](contract.md#output-7c7c598c04b7)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [apps/web/src/views/restoreFolderLayout.ts](contract.md#output-177e5ee0e73a)
- [apps/web/src/views/useUnsavedLifecycle.ts](../../../mechanisms/apps/web/src/views/useUnsavedLifecycle.ts.md#output-0a84f897aa4c)
- [apps/web/src/views/useUntitledSave.ts](../../../mechanisms/apps/web/src/views/useUntitledSave.ts.md#output-7589eb95cb0f)
- [apps/web/src/views/useWorkspaceUi.ts](contract.md#output-71ffc8e32ec3)
- [apps/web/src/views/workspaceTabs.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-b5a48fd337ae)
- [crates/hickory-workspace/src/lib.rs](../../../mechanisms/crates/hickory-workspace/src/lib.rs.md#output-a9e0f815de73)

### Tests and fixtures

- [apps/desktop/src-tauri/tests/serves_one_origin.rs](../../../mechanisms/apps/desktop/src-tauri/tests/serves_one_origin.rs.md#output-e7ec95ab75f6)
- [apps/web/src/App.test.tsx](../../../mechanisms/apps/web/src/App.tsx.md#output-65ac24c24624)
- [apps/web/src/shell/SpatialShellView.test.tsx](../../../mechanisms/apps/web/src/shell/SpatialShellView.tsx.md#output-57fb3ab47480)
- [apps/web/src/views/folderPane.test.tsx](../../../mechanisms/apps/web/src/views/folderPane.ts.md#output-c29616a4a7b1)
- [apps/web/src/views/restoreFolderLayout.test.ts](contract.md#output-7589fbbd1963)
- [apps/web/src/views/useUnsavedLifecycle.test.tsx](../../../mechanisms/apps/web/src/views/useUnsavedLifecycle.ts.md#output-18ff98d49ede)
- [apps/web/src/views/useUntitledSave.test.tsx](../../../mechanisms/apps/web/src/views/useUntitledSave.ts.md#output-1a7b240c6588)
- [apps/web/src/views/useWorkspaceUi.test.tsx](contract.md#output-307090b7547a)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-5370130a675d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
