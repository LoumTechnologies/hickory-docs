<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/FolderTreePane.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9c0af3061e25"></a>
## `apps/web/src/shell/FolderTreePane.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A File Is Ingested From The Tree](../../../../../guarantees/authoring/a-file-is-ingested-from-the-tree/contract.md), [A Plain File Opens, Edits, And Never Silently Loses A Byte](../../../../../guarantees/authoring/a-plain-file-opens-and-saves/contract.md), [A Tree Row Hands Its File To The Rest Of The Machine](../../../../../guarantees/authoring/a-tree-row-opens-in-the-platform/contract.md), [Keep divergent output until an explicit decision names the write](../../../../../guarantees/authoring/an-output-that-cannot-be-carried-back-is-held/contract.md), [Open File Tabs Follow Renames](../../../../../guarantees/authoring/open-file-tabs-follow-renames/contract.md), [The Workspace Tree Is An Editor Buffer](../../../../../guarantees/authoring/the-workspace-tree-edits-like-an-editor/contract.md), [The Workspace Tree Is A Lens](../../../../../guarantees/authoring/the-workspace-tree-is-a-lens/contract.md), [A GitHub Issue Belongs To Its Associated Folder](../../../../../guarantees/integrations/a-github-issue-belongs-to-its-associated-folder/contract.md), [An Unread Item Notifies At Its Nearest Visible Ancestor](../../../../../guarantees/integrations/an-unread-item-notifies-at-its-nearest-visible-ancestor/contract.md), [A Session Appears Where It Is Working](../../../../../guarantees/terminal/a-session-appears-where-it-is-working/contract.md).

<a id="f-9c0af3061e25-FolderTree"></a>
### `FolderTree`

<storycode:copy id="f-9c0af3061e25-FolderTree">
import { useUiReading } from "../lib/uiReading";
// The folder tree: the open folder's files, as a pane.
//
// There is no "list of documents" anywhere in this app — a folder was opened,
// and this is what is in it. One root today; the shape is a list of roots
// because "open another folder beside this one" must be an addition to an
// array, not a rewrite of this file.
//
// The pane itself is dumb about the shell: it renders roots it is given and
// reports clicks. What opening a file MEANS (a route, a generated pane,
// nothing) is the mounting view's decision, expressed through `fileAction`.

import { useCallback, useEffect, useId, useMemo, useRef, useState, type MouseEvent } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";

import { api } from "../api/client";
import type { FileNode, FilesResponse } from "../api/types";
import { TreeContextMenu } from "./TreeContextMenu";
import { TreePrompt } from "./TreePrompt";
import { FileActionPrompts, runFileMenuItem } from "./fileMenuActions";
import { useDired } from "./useDired";
import { useTreeNavigation } from "./useTreeNavigation";
import { TreeFindReplace } from "./TreeFindReplace";
import { FilesystemTreeEditor, type WorkspaceTextExtra } from "./FilesystemTreeEditor";
import {
  terminalMenuItems,
  treeMenuItems,
  type TreeFolder,
  type TreeMenuItem,
} from "./treeMenu";
import { rankSessions } from "../lib/treeTerminals";
import {
  GithubIssueRows,
  GithubProviderStatusRow,
  GithubReviewRows,
  parseGithubIssueReference,
  useGithubWorkspaceNodes,
} from "./GithubTreeNodes";

/** One open folder: the server's FilesResponse, kept whole. */
export type FolderTree = FilesResponse;

</storycode:copy>
<a id="f-9c0af3061e25-FileAction"></a>
### `FileAction`

<storycode:copy id="f-9c0af3061e25-FileAction">
// ---------------------------------------------------------------------------
// What clicking a file does
// ---------------------------------------------------------------------------

export type FileAction =
  /** A `.md` document: navigate to its route. */
  | { kind: "doc"; id: string }
  /** A file some document generates: open a generated pane. `docId` names the
   * owner when the server knew it, which it does for any document in the
   * folder — not only for the ones already open. */
  | { kind: "generated"; path: string; docId?: string }
  /** Any other text file: open a plain-file pane. */
  | { kind: "file"; path: string }
  /** A file this app cannot show (binary, by extension): named, and inert. */
  | { kind: "inert" };

</storycode:copy>
<a id="f-9c0af3061e25-BINARY_EXTENSIONS"></a>
### `BINARY_EXTENSIONS`

Extensions that name bytes, not text. A click on one stays inert rather
than opening a pane whose only content would be an encoding error. The
list is advisory — a mislabelled file still gets an honest refusal from
the server's UTF-8 check.

<storycode:copy id="f-9c0af3061e25-BINARY_EXTENSIONS">
/** Extensions that name bytes, not text. A click on one stays inert rather
 * than opening a pane whose only content would be an encoding error. The
 * list is advisory — a mislabelled file still gets an honest refusal from
 * the server's UTF-8 check. */
const BINARY_EXTENSIONS = new Set([
  "png", "jpg", "jpeg", "gif", "webp", "avif", "ico", "bmp", "tiff",
  "pdf", "zip", "gz", "tgz", "bz2", "xz", "7z", "rar", "jar",
  "exe", "dll", "so", "dylib", "a", "o", "class", "wasm", "bin",
  "woff", "woff2", "ttf", "otf", "eot",
  "mp3", "mp4", "mov", "avi", "mkv", "webm", "ogg", "wav", "flac",
  "sqlite", "db",
]);

</storycode:copy>
<a id="f-9c0af3061e25-isLikelyBinaryPath"></a>
### `isLikelyBinaryPath`

<storycode:copy id="f-9c0af3061e25-isLikelyBinaryPath">
export function isLikelyBinaryPath(path: string): boolean {
  const name = path.split("/").pop() ?? path;
  const dot = name.lastIndexOf(".");
  if (dot <= 0) return false; // no extension, or a dotfile: assume text
  return BINARY_EXTENSIONS.has(name.slice(dot + 1).toLowerCase());
}

</storycode:copy>
<a id="f-9c0af3061e25-fileAction"></a>
### `fileAction`

What a click on `node` should do, given which non-document paths are
openable as generated files (the open documents' outputs). Everything
else that looks like text opens as a plain file.

<storycode:copy id="f-9c0af3061e25-fileAction">
/**
 * What a click on `node` should do, given which non-document paths are
 * openable as generated files (the open documents' outputs). Everything
 * else that looks like text opens as a plain file.
 */
export function fileAction(node: FileNode, openable: ReadonlySet<string>): FileAction {
  if (node.doc_id) return { kind: "doc", id: node.doc_id };
  // `openable` is what the OPEN documents weave; `generated_by` is what every
  // document in the folder declares. The second is the reason a woven file
  // opens as the generated thing it is even when its document is closed —
  // and the reason the app stops offering to make `cards.md` literate when
  // `cards.md` has been writing it all along.
  if (node.generated_by) return { kind: "generated", path: node.path, docId: node.generated_by };
  if (openable.has(node.path)) return { kind: "generated", path: node.path };
  if (isLikelyBinaryPath(node.path)) return { kind: "inert" };
  return { kind: "file", path: node.path };
}

</storycode:copy>
<a id="f-9c0af3061e25-docPathOf"></a>
### `docPathOf`

The path of the document with this id, anywhere in the tree.

<storycode:copy id="f-9c0af3061e25-docPathOf">
/** The path of the document with this id, anywhere in the tree. */
export function docPathOf(nodes: readonly FileNode[], docId: string): string | null {
  for (const node of nodes) {
    if (node.doc_id === docId) return node.path;
    const inner = node.children ? docPathOf(node.children, docId) : null;
    if (inner) return inner;
  }
  return null;
}

</storycode:copy>
<a id="f-9c0af3061e25-fileNodeAt"></a>
### `fileNodeAt`

<storycode:copy id="f-9c0af3061e25-fileNodeAt">
function fileNodeAt(nodes: readonly FileNode[], path: string): FileNode | null {
  for (const node of nodes) {
    if (node.path === path) return node;
    const child = node.children ? fileNodeAt(node.children, path) : null;
    if (child) return child;
  }
  return null;
}

</storycode:copy>
<a id="f-9c0af3061e25-expandKey"></a>
### `expandKey`

Expand state: which directories are open, per root, across sessions

<storycode:copy id="f-9c0af3061e25-expandKey">
// ---------------------------------------------------------------------------
// Expand state: which directories are open, per root, across sessions
// ---------------------------------------------------------------------------

function expandKey(root: string): string {
  return `hickory.tree.expanded:${root}`;
}

</storycode:copy>
<a id="f-9c0af3061e25-loadExpanded"></a>
### `loadExpanded`

<storycode:copy id="f-9c0af3061e25-loadExpanded">
export function loadExpanded(root: string): Set<string> {
  try {
    const raw = localStorage.getItem(expandKey(root));
    const parsed: unknown = raw ? JSON.parse(raw) : [];
    return new Set(Array.isArray(parsed) ? parsed.filter((p): p is string => typeof p === "string") : []);
  } catch {
    // A corrupt entry collapses the tree; it must never break the pane.
    return new Set();
  }
}

</storycode:copy>
<a id="f-9c0af3061e25-saveExpanded"></a>
### `saveExpanded`

<storycode:copy id="f-9c0af3061e25-saveExpanded">
export function saveExpanded(root: string, expanded: ReadonlySet<string>): void {
  try {
    localStorage.setItem(expandKey(root), JSON.stringify([...expanded]));
  } catch {
    // Storage full or denied: the tree still works, it just forgets.
  }
}

</storycode:copy>
<a id="f-9c0af3061e25-toggleExpanded"></a>
### `toggleExpanded`

<storycode:copy id="f-9c0af3061e25-toggleExpanded">
export function toggleExpanded(expanded: ReadonlySet<string>, path: string): Set<string> {
  const next = new Set(expanded);
  if (next.has(path)) next.delete(path);
  else next.add(path);
  return next;
}

</storycode:copy>
<a id="f-9c0af3061e25-FILES_CHANGED_EVENT"></a>
### `FILES_CHANGED_EVENT`

Fetching
The event anything that changes files on disk may dispatch to refresh
every mounted tree without a page-wide state channel.

<storycode:copy id="f-9c0af3061e25-FILES_CHANGED_EVENT">
// ---------------------------------------------------------------------------
// Fetching
// ---------------------------------------------------------------------------

/** The event anything that changes files on disk may dispatch to refresh
 * every mounted tree without a page-wide state channel. */
export const FILES_CHANGED_EVENT = "hickory:files-changed";
</storycode:copy>
<a id="f-9c0af3061e25-FILES_DELETED_EVENT"></a>
### `FILES_DELETED_EVENT`

Successful deletions announce their exact paths so open editors can retire
their backing tabs without mistaking ordinary saves for removals.

<storycode:copy id="f-9c0af3061e25-FILES_DELETED_EVENT">
/** Successful deletions announce their exact paths so open editors can retire
 * their backing tabs without mistaking ordinary saves for removals. */
export const FILES_DELETED_EVENT = "hickory:files-deleted";

</storycode:copy>
<a id="f-9c0af3061e25-useFolderTrees"></a>
### `useFolderTrees`

The open folder's tree, kept fresh: fetched on mount, refetched when the
window regains focus (the cheap way to notice out-of-app edits) and when
anything dispatches FILES_CHANGED_EVENT (runs, saves).

<storycode:copy id="f-9c0af3061e25-useFolderTrees">
/**
 * The open folder's tree, kept fresh: fetched on mount, refetched when the
 * window regains focus (the cheap way to notice out-of-app edits) and when
 * anything dispatches FILES_CHANGED_EVENT (runs, saves).
 */
export function useFolderTrees(): { roots: FolderTree[]; error: string | null; folderOpen: boolean | null } {
  const [roots, setRoots] = useState<FolderTree[]>([]);
  const [folderOpen, setFolderOpen] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    api.files().then(
      (response) => {
        const open = response.folder_open !== false;
        setFolderOpen(open);
        setRoots(open ? [response] : []);
        setError(null);
      },
      (e) => setError(e instanceof Error ? e.message : String(e)),
    );
  }, []);

  useEffect(() => {
    load();
    window.addEventListener("focus", load);
    window.addEventListener(FILES_CHANGED_EVENT, load);
    return () => {
      window.removeEventListener("focus", load);
      window.removeEventListener(FILES_CHANGED_EVENT, load);
    };
  }, [load]);

  return { roots, error, folderOpen };
}

</storycode:copy>
<a id="f-9c0af3061e25-TreeSession"></a>
### `TreeSession`

The pane
A terminal session as the tree needs to know it.

<storycode:copy id="f-9c0af3061e25-TreeSession">
// ---------------------------------------------------------------------------
// The pane
// ---------------------------------------------------------------------------

/** A terminal session as the tree needs to know it. */
export interface TreeSession {
  id: string;
  title: string;
  cwd: string;
  state: string;
  monitor: boolean;
  /** False when `cwd` is only where the session was started — see
   * `SessionSummary::cwd_is_live`. */
  cwdIsLive: boolean;
}

</storycode:copy>
<a id="f-9c0af3061e25-fileWorkspaceKey"></a>
### `fileWorkspaceKey`

<storycode:copy id="f-9c0af3061e25-fileWorkspaceKey">
export function fileWorkspaceKey(path: string): string {
  return `filesystem:${encodeURIComponent(path)}`;
}

</storycode:copy>
<a id="f-9c0af3061e25-normalizeDir"></a>
### `normalizeDir`

Strip trailing slashes so `/a/b/` and `/a/b` are one directory.

<storycode:copy id="f-9c0af3061e25-normalizeDir">
/** Strip trailing slashes so `/a/b/` and `/a/b` are one directory. */
function normalizeDir(path: string): string {
  const trimmed = path.replace(/\/+$/, "");
  return trimmed === "" ? "/" : trimmed;
}

</storycode:copy>
<a id="f-9c0af3061e25-relativeCwd"></a>
### `relativeCwd`

A session's working directory, relative to the folder being viewed.
`null` when it is not inside that folder at all. Node paths in a listing are
relative to the root and session directories are absolute, so something has
to do this conversion; doing it in one named place is what keeps the two
from being compared directly, which silently matches nothing.

<storycode:copy id="f-9c0af3061e25-relativeCwd">
/**
 * A session's working directory, relative to the folder being viewed.
 *
 * `null` when it is not inside that folder at all. Node paths in a listing are
 * relative to the root and session directories are absolute, so something has
 * to do this conversion; doing it in one named place is what keeps the two
 * from being compared directly, which silently matches nothing.
 */
export function relativeCwd(cwd: string, root: string): string | null {
  const rootPath = normalizeDir(root);
  const path = normalizeDir(cwd);
  if (path === rootPath) return "";
  // The slash matters: `/w-other` starts with `/w` and is not inside it.
  if (!path.startsWith(`${rootPath}/`)) return null;
  return path.slice(rootPath.length + 1);
}

</storycode:copy>
<a id="f-9c0af3061e25-placeSessions"></a>
### `placeSessions`

Which directory row each session belongs under, keyed the way the tree keys
its nodes: relative to the root, with `""` for the root itself.
A session is shown at the deepest directory the listing contains — usually
its own working directory, and its nearest listed ancestor when that
directory was truncated away or is simply not part of this listing. Sessions
working outside the folder are left out: the question this answers is "what
is running *in here*".
Pure, and separate from the rendering, because the interesting cases are the
ones that are annoying to reach through a UI: a `cd` out of the tree, a
directory nobody listed, and a sibling whose name shares a prefix.

<storycode:copy id="f-9c0af3061e25-placeSessions">
/**
 * Which directory row each session belongs under, keyed the way the tree keys
 * its nodes: relative to the root, with `""` for the root itself.
 *
 * A session is shown at the deepest directory the listing contains — usually
 * its own working directory, and its nearest listed ancestor when that
 * directory was truncated away or is simply not part of this listing. Sessions
 * working outside the folder are left out: the question this answers is "what
 * is running *in here*".
 *
 * Pure, and separate from the rendering, because the interesting cases are the
 * ones that are annoying to reach through a UI: a `cd` out of the tree, a
 * directory nobody listed, and a sibling whose name shares a prefix.
 */
export function placeSessions(
  sessions: readonly TreeSession[],
  root: string,
  directories: ReadonlySet<string>,
): Map<string, TreeSession[]> {
  const out = new Map<string, TreeSession[]>();

  for (const session of sessions) {
    let path = relativeCwd(session.cwd, root);
    if (path === null) continue;

    // Walk up to the first directory this tree actually lists.
    while (path !== "" && !directories.has(path)) {
      const slash = path.lastIndexOf("/");
      path = slash === -1 ? "" : path.slice(0, slash);
    }
    const at = out.get(path);
    if (at) at.push(session);
    else out.set(path, [session]);
  }
  return out;
}

</storycode:copy>
<a id="f-9c0af3061e25-sessionsUnder"></a>
### `sessionsUnder`

How many sessions are running anywhere beneath `dir`.
A collapsed directory would otherwise hide them completely, which defeats
the point: the reason to show processes in the tree is to see the ones you
had forgotten about.

<storycode:copy id="f-9c0af3061e25-sessionsUnder">
/**
 * How many sessions are running anywhere beneath `dir`.
 *
 * A collapsed directory would otherwise hide them completely, which defeats
 * the point: the reason to show processes in the tree is to see the ones you
 * had forgotten about.
 */
export function sessionsUnder(
  sessions: readonly TreeSession[],
  root: string,
  dir: string,
): number {
  const path = normalizeDir(dir);
  return sessions.filter((session) => {
    const cwd = relativeCwd(session.cwd, root);
    return cwd !== null && (cwd === path || cwd.startsWith(`${path}/`));
  }).length;
}

</storycode:copy>
<a id="f-9c0af3061e25-directoryPaths"></a>
### `directoryPaths`

Every directory path in a listing, root-relative, for `placeSessions`.

<storycode:copy id="f-9c0af3061e25-directoryPaths">
/** Every directory path in a listing, root-relative, for `placeSessions`. */
export function directoryPaths(nodes: readonly FileNode[]): Set<string> {
  const out = new Set<string>();
  const walk = (list: readonly FileNode[]) => {
    for (const node of list) {
      if (node.dir) {
        out.add(node.path.replace(/\/+$/, ""));
        walk(node.children ?? []);
      }
    }
  };
  walk(nodes);
  return out;
}

</storycode:copy>
<a id="f-9c0af3061e25-OnRowMenu"></a>
### `OnRowMenu`

A right-click on a row, reported with what was clicked.
Every row kind reports the same three things — where the pointer was, which
root-relative path the row names, and whether it is a directory — so the
menu is built in one place from one shape rather than per row type.

<storycode:copy id="f-9c0af3061e25-OnRowMenu">
/**
 * A right-click on a row, reported with what was clicked.
 *
 * Every row kind reports the same three things — where the pointer was, which
 * root-relative path the row names, and whether it is a directory — so the
 * menu is built in one place from one shape rather than per row type.
 */
export type OnRowMenu = (event: MouseEvent, path: string, dir: boolean, plainText?: boolean) => void;

</storycode:copy>
<a id="f-9c0af3061e25-FolderTreePaneProps"></a>
### `FolderTreePaneProps`

<storycode:copy id="f-9c0af3061e25-FolderTreePaneProps">
export interface FolderTreePaneProps {
  roots: readonly FolderTree[];
  /** Non-document paths that a click can open (generated files). */
  openable: ReadonlySet<string>;
  /** A click on something openable. Inert files never call this. */
  onOpen: (action: Exclude<FileAction, { kind: "inert" }>) => void;
  /** The + in the header: a new document (#/new). */
  onNewDocument: () => void;
  /** The document currently on screen, to mark its row. */
  activeDocId?: string;
  /** Project-relative paths whose open buffers differ from explicit Save. */
  dirtyPaths?: ReadonlySet<string>;
  /** Terminal sessions, shown as child nodes of the directory each one is working
   * in. There is no separate list of terminals any more: a terminal has a
   * working directory, this tree already draws directories, and two trees
   * meant two places to look for "what is going on". */
  sessions?: readonly TreeSession[];
  /** A click on a terminal node: show that terminal. */
  onOpenTerminal?: (id: string) => void;
  /** "Open terminal here" on a directory's menu, with its root-relative
   * path. */
  onNewTerminal?: (path: string) => void;
  /** "New worktree here…" on a directory's menu. */
  onNewWorktree?: (path: string) => void;
  /** "Close" on a terminal node's own menu — the only place a session can be
   * stopped now that terminals have no list of their own. */
  onCloseTerminal?: (id: string) => void;
  /** Jump to a find hit: a root-relative path and a 1-based line. */
  onOpenHit?: (path: string, line: number) => void;
  /** Incremented when the shell's Show Files command should put point here. */
  focusRequest?: number;
  error?: string | null;
}

</storycode:copy>
<a id="f-9c0af3061e25-TerminalRows"></a>
### `TerminalRows`

The terminals running in one directory, as visible child nodes beside its
files. A terminal is the first non-file workspace node; hiding it in a
folder-row decoration made the implemented feature indistinguishable from
no feature at all. See lib/treeTerminals.ts for the stable urgency order.

<storycode:copy id="f-9c0af3061e25-TerminalRows">
/**
 * The terminals running in one directory, as visible child nodes beside its
 * files. A terminal is the first non-file workspace node; hiding it in a
 * folder-row decoration made the implemented feature indistinguishable from
 * no feature at all. See lib/treeTerminals.ts for the stable urgency order.
 */
function TerminalRows({
  sessions,
  depth,
  parentPath,
  onOpen,
  onMenu,
}: {
  sessions: readonly TreeSession[];
  depth: number;
  parentPath?: string;
  onOpen?: (id: string) => void;
  onMenu?: (event: MouseEvent, session: TreeSession) => void;
}) {
  if (sessions.length === 0) return null;
  return (
    <>
      {rankSessions(sessions).map((session) => {
        // Where the shell says it is, versus where it was started, is a real
        // difference in how much to trust this icon's placement — so the
        // tooltip says which one it is rather than presenting both as one
        // fact.
        const where = session.cwdIsLive
          ? session.cwd
          : `${session.cwd} — started here; this shell does not report its directory`;
        return (
          <li key={session.id} role="treeitem">
            <button
              type="button"
              className={`folder-tree__terminal mono state-${session.state}${
                session.monitor ? " folder-tree__terminal--monitor" : ""
              }`}
              style={{ paddingLeft: `${depth * 0.85 + 0.4}rem` }}
              data-session-id={session.id}
              data-workspace-node-key={`terminal:${session.id}`}
              data-workspace-node-kind="terminal"
              data-workspace-parent={parentPath ? fileWorkspaceKey(parentPath) : undefined}
              data-tip={`${session.title} — ${session.state.replace("-", " ")}\n${where}`}
              aria-label={`${session.title}, terminal, ${session.state.replace("-", " ")}`}
              onClick={() => onOpen?.(session.id)}
              onContextMenu={(event) => onMenu?.(event, session)}
            >
              <span className="folder-tree__terminal-mark" aria-hidden>▮</span>
              <span className="folder-tree__terminal-title">{session.title}</span>
              <span className="folder-tree__terminal-state">{session.state.replace("-", " ")}</span>
            </button>
          </li>
        );
      })}
    </>
  );
}

</storycode:copy>
<a id="f-9c0af3061e25-FolderTreePane"></a>
### `FolderTreePane`

<storycode:copy id="f-9c0af3061e25-FolderTreePane">
export function FolderTreePane({
  roots,
  openable,
  onOpen,
  onNewDocument,
  activeDocId,
  dirtyPaths = new Set(),
  sessions = [],
  onOpenTerminal,
  onNewTerminal,
  onNewWorktree,
  onCloseTerminal,
  onOpenHit,
  focusRequest = 0,
  error,
}: FolderTreePaneProps) {
  const pane = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (focusRequest === 0) return;
    const frame = requestAnimationFrame(() =>
      pane.current?.querySelector<HTMLElement>('.filesystem-editor .cm-content')?.focus(),
    );
    return () => cancelAnimationFrame(frame);
  }, [focusRequest, roots]);
  useUiReading("Files pane load state", { root_count: roots.length, error: error || null, message: error || (roots.length ? null : "Reading folder…"), file_contents_read: false });
  if (error) return <p className="error folder-tree__error">{error}</p>;
  if (roots.length === 0) return <p className="muted folder-tree__loading">Reading folder…</p>;
  return (
    <div
      ref={pane}
      className="folder-tree"
      onClick={(event) => {
        const target = event.target as HTMLElement;
        if (!target.closest("button, a, input, textarea, .cm-editor, [data-workspace-node-key]")) {
          pane.current?.querySelector<HTMLElement>('[role="tree"]')?.focus();
        }
      }}
    >
      {/* Find and replace across everything, above the tree it acts on. Not
          an overlay: this is a list you work through while the tree stays
          where it is. */}
      <TreeFindReplace
        onOpenHit={(path, line) => onOpenHit?.(path, line)}
        onReplaced={() => window.dispatchEvent(new Event(FILES_CHANGED_EVENT))}
      />
      {roots.map((folder) => (
        <FolderRoot
          key={folder.root}
          folder={folder}
          openable={openable}
          onOpen={onOpen}
          onNewDocument={onNewDocument}
          activeDocId={activeDocId}
          dirtyPaths={dirtyPaths}
          sessions={sessions}
          onOpenTerminal={onOpenTerminal}
          onNewTerminal={onNewTerminal}
          onNewWorktree={onNewWorktree}
          onCloseTerminal={onCloseTerminal}
        />
      ))}
    </div>
  );
}

</storycode:copy>
<a id="f-9c0af3061e25-FolderRoot"></a>
### `FolderRoot`

<storycode:copy id="f-9c0af3061e25-FolderRoot">
function FolderRoot({
  folder,
  openable,
  onOpen,
  onNewDocument,
  activeDocId,
  dirtyPaths,
  sessions,
  onOpenTerminal,
  onNewTerminal,
  onNewWorktree,
  onCloseTerminal,
}: {
  folder: FolderTree;
  openable: ReadonlySet<string>;
  onOpen: FolderTreePaneProps["onOpen"];
  onNewDocument: () => void;
  activeDocId?: string;
  dirtyPaths: ReadonlySet<string>;
  sessions: readonly TreeSession[];
  onOpenTerminal?: (id: string) => void;
  onNewTerminal?: (path: string) => void;
  onNewWorktree?: (path: string) => void;
  onCloseTerminal?: (id: string) => void;
}) {
  const instance = useId();
  const placement = useMemo(
    () => placeSessions(sessions, folder.root, directoryPaths(folder.tree)),
    [sessions, folder.root, folder.tree],
  );
  const atRoot = placement.get("") ?? [];
  const github = useGithubWorkspaceNodes();
  const [detailFolder, setDetailFolder] = useState<string | null>(null);
  const editorExtras = useMemo<WorkspaceTextExtra[]>(() => [
    ...[...placement].flatMap(([parentPath, placed]) => parentPath === "" ? [] : placed.map((session) => ({
      key: `terminal:${session.id}`,
      parentPath,
      label: `terminal: ${session.title} · ${session.state.replace("-", " ")}`,
      activate: onOpenTerminal ? () => onOpenTerminal(session.id) : undefined,
    }))),
    ...(github.value?.issues ?? []).filter((issue) => issue.folder !== "").map((issue) => ({
      key: `github:${issue.repository}#${issue.number}`,
      parentPath: issue.folder,
      label: `issue #${issue.number}: ${issue.title} · ${issue.state.toLowerCase()}${issue.unread ? ` · ${issue.unread} unread` : ""}`,
      activate: () => setDetailFolder(issue.folder),
      edit: {
        prefix: `issue #${issue.number}: `,
        value: issue.title,
        suffix: ` · ${issue.state.toLowerCase()}${issue.unread ? ` · ${issue.unread} unread` : ""}`,
        apply: async (title: string) => {
          await api.editGithubObject("issue", issue.repository, issue.number, "title", title);
          github.refresh();
        },
      },
    })),
  ], [placement, github.value?.issues, onOpenTerminal]);

  // The folder as the context menu needs to know it: where it is on this
  // machine, how paths are spelled there, and what the file manager is called.
  const info: TreeFolder = useMemo(
    () => ({
      rootPath: folder.root_path,
      separator: folder.separator,
      fileManager: folder.file_manager,
    }),
    [folder.root_path, folder.separator, folder.file_manager],
  );

  const menuOwner = JSON.stringify([folder, activeDocId, [...openable], atRoot]);
  const [menu, setMenu] = useState<{
    owner: string;
    x: number;
    y: number;
    path: string;
    dir: boolean;
    plainText: boolean;
  } | null>(null);
  // A failed reveal (no file manager on a headless Linux box, a file deleted
  // between the listing and the click) says so in the pane. The alternative is
  // a menu item that appears to do nothing at all.
  const [notice, setNotice] = useState<string | null>(null);
  const [issueAssociation, setIssueAssociation] = useState<{ folder: string; error?: string; busy?: boolean } | null>(null);

  const showMenu = useCallback((position: { x: number; y: number }, path: string, dir: boolean, plainText = false) => {
    setTermMenu(null); setNotice(null); setMenu({ owner: menuOwner, ...position, path, dir, plainText });
  }, [menuOwner]);
  const openMenu = useCallback<OnRowMenu>((event, path, dir, plainText = false) => {
    event.preventDefault(); event.stopPropagation();
    showMenu({ x: event.clientX, y: event.clientY }, path, dir, plainText);
  }, [showMenu]);

  // Dired: the marks, the prompt a verb asks through, and the verbs.
  const dired = useDired(setNotice);
  const navigation = useTreeNavigation({
    replacePaths: dired.replaceMarks,
    addPaths: dired.addMarks,
  });
  // The focused document, for "Ingest into …": found in this tree by id
  // rather than passed in, because the tree already knows every document.
  const activeDoc = useMemo(() => {
    const path = activeDocId ? docPathOf(folder.tree, activeDocId) : null;
    return path ? { path, name: path.split("/").pop() ?? path } : undefined;
  }, [folder.tree, activeDocId]);

  /** A right-click on a terminal node: its own short menu, not the folder's. */
  const [termMenu, setTermMenu] = useState<{
    owner: string;
    x: number;
    y: number;
    session: TreeSession;
  } | null>(null);
  const showTermMenu = (position: { x: number; y: number }, id: string) => {
    const session = atRoot.find(session => session.id === id);
    if (!session) return false;
    setMenu(null); setTermMenu({ owner: menuOwner, ...position, session }); return true;
  };
  const openTermMenu = (event: MouseEvent, session: TreeSession) => {
    event.preventDefault(); event.stopPropagation();
    showTermMenu({ x: event.clientX, y: event.clientY }, session.id);
  };
  const priorMenuOwner = useRef(menuOwner);
  useEffect(() => {
    if (priorMenuOwner.current !== menuOwner) { priorMenuOwner.current = menuOwner; setMenu(null); setTermMenu(null); }
  }, [menuOwner]);

  const runItem = useCallback((item: TreeMenuItem) => {
    setMenu(null);
    const action = item.action;
    if (runFileMenuItem(item, dired, setNotice, id => onOpen({ kind: "doc", id }))) return;
    // Terminal verbs are the pane's owner's business: it holds the session
    // list and the layout a new terminal opens into.
    if (action.kind === "terminal") {
      onNewTerminal?.(action.path);
      return;
    }
    if (action.kind === "worktree") {
      onNewWorktree?.(action.path);
      return;
    }
    if (action.kind === "associate-github-issue") {
      setIssueAssociation({ folder: action.path });
      return;
    }
    if (action.kind === "close-terminal") {
      onCloseTerminal?.(action.id);
      return;
    }
  }, [onNewTerminal, onNewWorktree, onCloseTerminal, onOpen, dired]);

  const newFromHeader = () => { setMenu(null); setTermMenu(null); onNewDocument(); };
  const name = folder.root.replace(/\/+$/, "").split("/").pop() || folder.root;
  const actionParent = `folder-tree:${instance}`;
  const treeRevision = useUiRevision(() => [folder, [...dirtyPaths], [...openable], notice, atRoot, menu, termMenu]);
  const actionRead = () => ({ revision: treeRevision(), enabled: true });
  useUiCommands([
    { key: `${actionParent}/root-menu`, command: { label: "Open workspace folder header menu", parent: actionParent, subject: actionParent, parameters: {}, prerequisites: [], read: actionRead, execute: async () => { showMenu({ x: 16, y: 16 }, "", true); return { status: "completed", value: { menu_open_requested: true } }; } } },
    { key: `${actionParent}/new-document`, command: { label: "New document from folder header", parent: actionParent, subject: actionParent, parameters: {}, prerequisites: [], read: actionRead, execute: async () => { newFromHeader(); return { status: "completed", value: { callback_requested: true, document_created: null } }; } } },
    { key: `${actionParent}/read-terminals`, command: { label: "Read root terminal row page", parent: actionParent, subject: `${actionParent}:terminal-reading`, parameters: { first: { type: "number", integer: true, minimum: 0, maximum: 100000000 }, count: { type: "number", integer: true, minimum: 1, maximum: 64 } }, prerequisites: [], read: actionRead, execute: async args => {
      const first = Number(args.first), count = Number(args.count), rows = rankSessions(atRoot);
      if (first > rows.length) return { status: "unavailable", reason: "Index exceeds terminal rows in this root" };
      const value = { rows: rows.slice(first, first + count).map(session => ({ id: session.id, title: session.title, state: session.state, cwd: session.cwd, cwd_is_live: session.cwdIsLive, monitor: session.monitor })), total: rows.length, next: first + count < rows.length ? first + count : null, output_read: false };
      if (new TextEncoder().encode(JSON.stringify(value)).length > 8192) return { status: "unavailable", reason: "Terminal row page exceeds 8192 UTF-8 bytes; request fewer rows" };
      return { status: "completed", value };
    } } },
    ...[false, true].map(menu => ({ key: `${actionParent}/terminal-${menu ? "menu" : "open"}`, command: { label: menu ? "Open root terminal row menu" : "Open root terminal row", parent: actionParent, subject: actionParent, parameters: { id: { type: "string" as const, maxLength: 4096 } }, prerequisites: ["Current terminal row placed in this folder root"], read: actionRead, execute: async (args: Record<string, import("../lib/uiActions").UiValue>) => {
      const session = atRoot.find(session => session.id === args.id);
      if (!session) return { status: "unavailable" as const, reason: "Terminal moved, closed or is outside this root" };
      if (menu) showTermMenu({ x: 16, y: 16 }, session.id);
      else if (onOpenTerminal) onOpenTerminal(session.id);
      else return { status: "requires_input" as const, reason: "This host has no terminal-opening callback", request: "workspace-terminal-owner" };
      return { status: "completed" as const, value: { callback_requested: true, menu_open_requested: menu } };
    } } })),
    { key: `${actionParent}/read`, command: { label: `Read folder entries: ${name}`, parent: actionParent, subject: actionParent, parameters: {
      directory: { type: "string", maxLength: 8192 }, offset: { type: "number", minimum: 0, maximum: 1000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 64, integer: true },
    }, prerequisites: ["Explicit directory disclosure; returns only immediate children and no file content"], read: actionRead, execute: async args => {
      const directory = String(args.directory);
      const node = directory ? fileNodeAt(folder.tree, directory) : null;
      if (directory && (!node || !node.dir)) return { status: "unavailable", reason: "Directory is absent from the observed folder tree" };
      const nodes = directory ? node!.children ?? [] : folder.tree;
      const offset = Number(args.offset), limit = Number(args.limit);
      if (offset > nodes.length) return { status: "unavailable", reason: "Offset exceeds this directory's entry count" };
      return { status: "completed", value: { root: folder.root, directory, truncated: !!folder.truncated, total: nodes.length, next: offset + limit < nodes.length ? offset + limit : null, notice, entries: nodes.slice(offset, offset + limit).map(item => ({ path: item.path, name: item.name, directory: item.dir, document: item.doc_id ?? null, generated_by: item.generated_by ?? null, dirty: dirtyPaths.has(item.path), children: item.children?.length ?? 0 })) } };
    } } },
    { key: `${actionParent}/open`, command: { label: "Open observed workspace file", parent: actionParent, subject: actionParent, parameters: { path: { type: "string", maxLength: 8192 } }, prerequisites: ["File must belong to the current observed folder tree"], read: actionRead, execute: async args => {
      const node = fileNodeAt(folder.tree, String(args.path));
      if (!node || node.dir) return { status: "unavailable", reason: "Choose an existing file from this tree" };
      const action = fileAction(node, openable);
      if (action.kind === "inert") return { status: "unavailable", reason: "This file type has no in-app text view; use its existing external-open control" };
      onOpen(action); return { status: "completed", value: { dispatch: "open requested", path: node.path, result: "Observe the actual pane or loading error" } };
    } } },
    { key: `${actionParent}/new`, command: { label: "Create untitled document", parent: actionParent, subject: actionParent, parameters: {}, prerequisites: [], read: actionRead, execute: async () => { onNewDocument(); return { status: "completed", value: { dispatch: "requested", saved: false } }; } } },
  ]);
  return (
    <section className="folder-tree__root">
      <header className="folder-tree__header">
        <span
          className="folder-tree__name"
          data-tip={folder.root}
          onContextMenu={(event) => openMenu(event, "", true)}
        >
          {name}
        </span>
        <button
          type="button"
          className="folder-tree__new"
          data-tip="New document"
          aria-label="New document"
          onClick={newFromHeader}
        >
          +
        </button>
      </header>
      {folder.truncated && (
        <p className="muted folder-tree__truncated">Large folder — not everything is listed.</p>
      )}
      {notice && <p className="error folder-tree__notice">{notice}</p>}
      <FileActionPrompts dired={dired} inlineRename />
      {issueAssociation && (
        <TreePrompt
          label={`GitHub issue for ${issueAssociation.folder || "this folder"} (owner/repo#123 or URL)`}
          verb="Associate"
          initial={github.value?.repository ? `${github.value.repository}#` : ""}
          busy={Boolean(issueAssociation.busy)}
          error={issueAssociation.error ?? null}
          onCancel={() => setIssueAssociation(null)}
          onSubmit={(value) => {
            const reference = parseGithubIssueReference(value ?? "", github.value?.repository);
            if (!reference) {
              setIssueAssociation((current) => current && { ...current, error: "Use owner/repo#123 or a GitHub issue URL." });
              return;
            }
            setIssueAssociation((current) => current && { ...current, busy: true, error: undefined });
            void api.associateGithubIssue(reference.repository, reference.number, issueAssociation.folder).then(
              () => { setIssueAssociation(null); github.refresh(); window.dispatchEvent(new Event(FILES_CHANGED_EVENT)); },
              (cause) => setIssueAssociation((current) => current && { ...current, busy: false, error: cause instanceof Error ? cause.message : String(cause) }),
            );
          }}
        />
      )}
      <ul
        className="folder-tree__list"
        role="tree"
        tabIndex={0}
        onFocus={navigation.onFocus}
        onFocusCapture={navigation.onFocusCapture}
        // The dired keys — m, u, U, D, R, C, M, +, n — act on the focused
        // row. Read off the row's own attributes so the tree, not each row,
        // owns the one handler.
        onKeyDown={(event) => {
          if (navigation.onKeyDown(event)) return;
          const row = (event.target as HTMLElement).closest<HTMLElement>("[data-tree-path]");
          if (!row) return;
          dired.onKey(event, row.dataset.treePath ?? "", row.dataset.treeDir === "true");
        }}
      >
        <GithubProviderStatusRow workspace={github.value} />
        <GithubReviewRows workspace={github.value} onChanged={github.refresh} />
        <GithubIssueRows workspace={github.value} folder="" onChanged={github.refresh} />
        <TerminalRows
          sessions={atRoot}
          depth={0}
          onOpen={onOpenTerminal}
          onMenu={openTermMenu}
        />
      </ul>
      <FilesystemTreeEditor
        nodes={folder.tree}
        onDeleted={(paths) => window.dispatchEvent(new CustomEvent(FILES_DELETED_EVENT, { detail: { paths } }))}
        onChanged={(renames) =>
          window.dispatchEvent(
            renames?.length
              ? new CustomEvent(FILES_CHANGED_EVENT, {
                  detail: { renames: renames.map(({ path, to }) => ({ from: path, to })) },
                })
              : new Event(FILES_CHANGED_EVENT),
          )
        }
        onOpenPath={(path) => {
          const node = fileNodeAt(folder.tree, path);
          if (!node) return;
          const action = fileAction(node, openable);
          if (action.kind !== "inert") onOpen(action);
        }}
        onContextPath={(event, path, dir) => {
          showMenu({ x: event.clientX, y: event.clientY }, path, dir, !dir && !isLikelyBinaryPath(path));
        }}
        kindOfPath={(path, dir) => {
          if (dir) return "directory";
          const node = fileNodeAt(folder.tree, path);
          return node ? fileAction(node, openable).kind : "file";
        }}
        extras={editorExtras}
        dirtyPaths={dirtyPaths}
      />
      {detailFolder !== null && <ul className="folder-tree__list" role="tree">
        <GithubIssueRows workspace={github.value} folder={detailFolder} onChanged={github.refresh} />
      </ul>}
      {menu && menu.owner === menuOwner && (
        <TreeContextMenu
          x={menu.x}
          y={menu.y}
          subject={menu.path === "" ? name : menu.path}
          items={treeMenuItems(info, menu.path, menu.dir, {
            marked: dired.marked,
            activeDoc,
            plainText: menu.plainText,
          })}
          onPick={runItem}
          onClose={() => setMenu(null)}
        />
      )}
      {termMenu && termMenu.owner === menuOwner && (
        <TreeContextMenu
          x={termMenu.x}
          y={termMenu.y}
          subject={termMenu.session.title}
          items={terminalMenuItems(termMenu.session)}
          onPick={(item) => {
            setTermMenu(null);
            runItem(item);
          }}
          onClose={() => setTermMenu(null)}
        />
      )}
    </section>
  );
}
</storycode:copy>

### Assemble `apps/web/src/shell/FolderTreePane.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/FolderTreePane.tsx"><storycode:paste select="#f-9c0af3061e25-FolderTree"/><storycode:paste select="#f-9c0af3061e25-FileAction"/><storycode:paste select="#f-9c0af3061e25-BINARY_EXTENSIONS"/><storycode:paste select="#f-9c0af3061e25-isLikelyBinaryPath"/><storycode:paste select="#f-9c0af3061e25-fileAction"/><storycode:paste select="#f-9c0af3061e25-docPathOf"/><storycode:paste select="#f-9c0af3061e25-fileNodeAt"/><storycode:paste select="#f-9c0af3061e25-expandKey"/><storycode:paste select="#f-9c0af3061e25-loadExpanded"/><storycode:paste select="#f-9c0af3061e25-saveExpanded"/><storycode:paste select="#f-9c0af3061e25-toggleExpanded"/><storycode:paste select="#f-9c0af3061e25-FILES_CHANGED_EVENT"/><storycode:paste select="#f-9c0af3061e25-FILES_DELETED_EVENT"/><storycode:paste select="#f-9c0af3061e25-useFolderTrees"/><storycode:paste select="#f-9c0af3061e25-TreeSession"/><storycode:paste select="#f-9c0af3061e25-fileWorkspaceKey"/><storycode:paste select="#f-9c0af3061e25-normalizeDir"/><storycode:paste select="#f-9c0af3061e25-relativeCwd"/><storycode:paste select="#f-9c0af3061e25-placeSessions"/><storycode:paste select="#f-9c0af3061e25-sessionsUnder"/><storycode:paste select="#f-9c0af3061e25-directoryPaths"/><storycode:paste select="#f-9c0af3061e25-OnRowMenu"/><storycode:paste select="#f-9c0af3061e25-FolderTreePaneProps"/><storycode:paste select="#f-9c0af3061e25-TerminalRows"/><storycode:paste select="#f-9c0af3061e25-FolderTreePane"/><storycode:paste select="#f-9c0af3061e25-FolderRoot"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-508e32e299fd"></a>
## `apps/web/src/shell/FolderTreePane.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A File Is Ingested From The Tree](../../../../../guarantees/authoring/a-file-is-ingested-from-the-tree/contract.md), [A Plain File Opens, Edits, And Never Silently Loses A Byte](../../../../../guarantees/authoring/a-plain-file-opens-and-saves/contract.md), [A Tree Row Hands Its File To The Rest Of The Machine](../../../../../guarantees/authoring/a-tree-row-opens-in-the-platform/contract.md), [Filesystem Acts Keep Dired's Safety](../../../../../guarantees/authoring/the-tree-is-a-dired/contract.md), [The Workspace Tree Is An Editor Buffer](../../../../../guarantees/authoring/the-workspace-tree-edits-like-an-editor/contract.md), [The Workspace Tree Is A Lens](../../../../../guarantees/authoring/the-workspace-tree-is-a-lens/contract.md), [Close or recover a document without losing the wrong draft](../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md), [A GitHub Issue Belongs To Its Associated Folder](../../../../../guarantees/integrations/a-github-issue-belongs-to-its-associated-folder/contract.md), [A Session Appears Where It Is Working](../../../../../guarantees/terminal/a-session-appears-where-it-is-working/contract.md).

<a id="f-508e32e299fd-TREE"></a>
### `TREE`

<storycode:copy id="f-508e32e299fd-TREE">
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { EditorView } from "@codemirror/view";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { installMockHandler } from "../api/client";
import type { FileNode, FilesResponse } from "../api/types";
import {
  FolderTreePane,
  fileAction,
  isLikelyBinaryPath,
  loadExpanded,
  saveExpanded,
  toggleExpanded,
  useFolderTrees,
  directoryPaths,
  placeSessions,
  relativeCwd,
  sessionsUnder,
  type FileAction,
  type TreeSession,
} from "./FolderTreePane";

// The /api/files contract, as the backend serves it: dirs first, each level
// already alphabetical — the pane renders the order it is given.
const TREE: FileNode[] = [
  {
    name: "src",
    path: "src",
    dir: true,
    children: [
      {
        name: "gen",
        path: "src/gen",
        dir: true,
        children: [{ name: "orders.py", path: "src/gen/orders.py", dir: false }],
      },
      { name: "main.rs", path: "src/main.rs", dir: false },
    ],
  },
  { name: "paper.md", path: "paper.md", dir: false, doc_id: "d1" },
  { name: "readme.txt", path: "readme.txt", dir: false },
  { name: "logo.png", path: "logo.png", dir: false },
];

</storycode:copy>
<a id="f-508e32e299fd-RESPONSE"></a>
### `RESPONSE`

<storycode:copy id="f-508e32e299fd-RESPONSE">
const RESPONSE: FilesResponse = { root: "/home/me/notebook", tree: TREE };

</storycode:copy>
<a id="f-508e32e299fd-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-508e32e299fd-expression_statement">
it("discloses immediate folder children and opens through the actual pane callback", async () => {
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  const onOpen = vi.fn();
  render(<UiActionContext.Provider value={registry}><FolderTreePane roots={[RESPONSE]} openable={new Set()} onOpen={onOpen} onNewDocument={() => {}} /></UiActionContext.Provider>);
  const parent = registry.groups().groups.find(group => group.startsWith("folder-tree:"))!;
  let n = 0;
  const invoke = async (label: string, arguments_: Record<string, string | number>) => {
    const action = registry.observe(parent).actions.find(action => action.label.startsWith(label))!;
    let result;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: arguments_, request_id: String(++n) }); });
    return result;
  };
  expect(await invoke("Read folder entries", { directory: "src", offset: 0, limit: 1 })).toMatchObject({ status: "result", outcome: { value: { directory: "src", total: 2, next: 1, entries: [{ path: "src/gen", children: 1 }] } } });
  expect(await invoke("Read folder entries", { directory: "missing", offset: 0, limit: 1 })).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  await invoke("Open observed workspace file", { path: "paper.md" }); expect(onOpen).toHaveBeenCalledWith({ kind: "doc", id: "d1" });
  expect(await invoke("Open observed workspace file", { path: "logo.png" })).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
});

</storycode:copy>
<a id="f-508e32e299fd-mockFiles"></a>
### `mockFiles`

<storycode:copy id="f-508e32e299fd-mockFiles">
function mockFiles(response: FilesResponse = RESPONSE) {
  installMockHandler(async (method, path) => {
    if (method === "GET" && path === "/api/files") return response;
    throw new Error(`unexpected ${method} ${path}`);
  });
}

</storycode:copy>
<a id="f-508e32e299fd-Harness"></a>
### `Harness`

<storycode:copy id="f-508e32e299fd-Harness">
function Harness({
  onOpen = () => {},
  openable = new Set<string>(),
  onNew = () => {},
  sessions = [],
  onOpenTerminal,
  onCloseTerminal,
  dirtyPaths = new Set<string>(),
  focusRequest = 0,
}: {
  onOpen?: (action: Exclude<FileAction, { kind: "inert" }>) => void;
  openable?: Set<string>;
  onNew?: () => void;
  sessions?: readonly TreeSession[];
  onOpenTerminal?: (id: string) => void;
  onCloseTerminal?: (id: string) => void;
  dirtyPaths?: ReadonlySet<string>;
  focusRequest?: number;
}) {
  const { roots, error } = useFolderTrees();
  return (
    <FolderTreePane
      roots={roots}
      error={error}
      openable={openable}
      onOpen={onOpen}
      onNewDocument={onNew}
      sessions={sessions}
      onOpenTerminal={onOpenTerminal}
      onCloseTerminal={onCloseTerminal}
      dirtyPaths={dirtyPaths}
      focusRequest={focusRequest}
    />
  );
}

</storycode:copy>
<a id="f-508e32e299fd-filesEditor"></a>
### `filesEditor`

<storycode:copy id="f-508e32e299fd-filesEditor">
function filesEditor(): EditorView {
  const element = document.querySelector<HTMLElement>(".filesystem-editor .cm-editor");
  const editor = element && EditorView.findFromDOM(element);
  if (!editor) throw new Error("Files editor did not mount");
  return editor;
}

</storycode:copy>
<a id="f-508e32e299fd-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-508e32e299fd-expression_statement-2">
beforeEach(() => {
  localStorage.clear();
  mockFiles();
});
</storycode:copy>
<a id="f-508e32e299fd-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-508e32e299fd-expression_statement-3">
afterEach(cleanup);

</storycode:copy>
<a id="f-508e32e299fd-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-508e32e299fd-expression_statement-4">
describe("rendering the folder", () => {
  // Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
  it("marks a file dirty from the buffer state supplied by the workspace", async () => {
    render(<Harness dirtyPaths={new Set(["paper.md"])} />);
    await screen.findByText("paper.md");
    expect(document.querySelector('[data-tree-path="paper.md"]')?.className).toContain("filesystem-editor__line--dirty");
    expect(document.querySelector('[data-tree-path="readme.txt"]')?.className).not.toContain("filesystem-editor__line--dirty");
  });
  it("names the folder after its last path segment and offers a new document", async () => {
    const onNew = vi.fn();
    render(<Harness onNew={onNew} />);
    expect(await screen.findByText("notebook")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "New document" }));
    expect(onNew).toHaveBeenCalledOnce();
  });

  it("renders the server's order untouched: dirs first, then files", async () => {
    render(<Harness />);
    await screen.findByText("paper.md");
    expect(filesEditor().state.doc.toString()).toBe(
      "src/\n  gen/\n    orders.py\n  main.rs\npaper.md\nreadme.txt\nlogo.png",
    );
  });

  it("represents hierarchy as editable, significant whitespace", async () => {
    render(<Harness />);
    await screen.findByText("paper.md");
    expect(filesEditor().state.doc.line(1).text).toBe("src/");
    expect(filesEditor().state.doc.line(2).text).toBe("  gen/");
    expect(filesEditor().state.doc.line(3).text).toBe("    orders.py");
  });

  it("tags every visible file row for the ribbon overlay, by path", async () => {
    // The contract with shell/Ribbons.tsx: a connection whose target file is
    // visible as a buffer line (and not open as a tab) terminates ON that line,
    // found by data-tree-path.
    render(<Harness openable={new Set(["src/main.rs"])} />);
    await screen.findByText("paper.md");
    expect(document.querySelector('[data-tree-path="paper.md"]')?.getAttribute("data-tree-kind")).toBe("doc");
    expect(document.querySelector('[data-tree-path="readme.txt"]')?.getAttribute("data-tree-kind")).toBe("file");
    expect(document.querySelector('[data-tree-path="logo.png"]')?.getAttribute("data-tree-kind")).toBe("inert");
    expect(document.querySelector('[data-tree-path="src/main.rs"]')?.getAttribute("data-tree-kind")).toBe("generated");
  });

  it("says when the walk was truncated", async () => {
    mockFiles({ ...RESPONSE, truncated: true });
    render(<Harness />);
    expect(await screen.findByText(/not everything is listed/i)).toBeTruthy();
  });
});

</storycode:copy>
<a id="f-508e32e299fd-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-508e32e299fd-expression_statement-5">
// Guarantee: docs/guarantees/authoring/the-workspace-tree-edits-like-an-editor.md
describe("editor-grade tree navigation", () => {
  it("moves point into the tree when the shell asks to focus Files", async () => {
    const view = render(<Harness />);
    await screen.findByText("paper.md");
    view.rerender(<Harness focusRequest={1} />);
    await vi.waitFor(() => expect(document.activeElement).toBe(filesEditor().contentDOM));
  });

  it("uses ordinary editor arrows, Home, and Shift-selection", async () => {
    render(<Harness />);
    await screen.findByText("paper.md");
    const editor = filesEditor();
    editor.focus();
    editor.dispatch({ selection: { anchor: 0 } });
    fireEvent.keyDown(editor.contentDOM, { key: "ArrowRight" });
    expect(editor.state.selection.main.head).toBe(1);
    fireEvent.keyDown(editor.contentDOM, { key: "Home" });
    expect(editor.state.selection.main.head).toBe(0);
    fireEvent.keyDown(editor.contentDOM, { key: "ArrowRight", shiftKey: true });
    expect(editor.state.selection.main.empty).toBe(false);
  });
});

</storycode:copy>
<a id="f-508e32e299fd-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-508e32e299fd-expression_statement-6">
// docs/guarantees/integrations/a-github-issue-belongs-to-its-associated-folder.md
describe("GitHub issues in folders", () => {
  it("indents an associated issue beneath its folder and opens its details", async () => {
    installMockHandler(async (method, path) => {
      if (method === "GET" && path === "/api/files") return RESPONSE;
      if (method === "GET" && path === "/api/workspace/github") return {
        status: "available", repository: "acme/widget", branch: "main", reviews: [],
        issues: [{ repository: "acme/widget", number: 7, folder: "src", title: "Fix retry", state: "OPEN", freshness: "live" }],
      };
      throw new Error(`unexpected ${method} ${path}`);
    });
    render(<Harness />);
    const issue = await screen.findByText(/issue #7: Fix retry/);
    expect(filesEditor().state.doc.toString()).toContain("src/\n  issue #7: Fix retry · open\n  gen/");
    fireEvent.doubleClick(issue);
    expect(await screen.findByRole("button", { name: /Issue #7: Fix retry/ })).toBeTruthy();
  });

  it("associates an issue from a folder's context menu", async () => {
    const associated: unknown[] = [];
    installMockHandler(async (method, path, body) => {
      if (method === "GET" && path === "/api/files") return RESPONSE;
      if (method === "GET" && path === "/api/workspace/github") return {
        status: "available", repository: "acme/widget", branch: "main", reviews: [], issues: [],
      };
      if (method === "POST" && path === "/api/workspace/github/issues") {
        associated.push(body); return body;
      }
      throw new Error(`unexpected ${method} ${path}`);
    });
    render(<Harness />);
    const src = await screen.findByText("src/");
    fireEvent.contextMenu(src);
    fireEvent.click(document.querySelector<HTMLElement>('[data-menu-item="associate-github-issue"]')!);
    const input = await screen.findByRole("textbox", { name: /GitHub issue for src/ });
    fireEvent.change(input, { target: { value: "acme/widget#19" } });
    fireEvent.click(screen.getByRole("button", { name: "Associate" }));
    await vi.waitFor(() => expect(associated).toEqual([{ repository: "acme/widget", number: 19, folder: "src" }]));
  });
});

</storycode:copy>
<a id="f-508e32e299fd-expression_statement-7"></a>
### `expression_statement`

The right-click menu: the tree's door to the rest of the machine.
Guards docs/guarantees/authoring/a-tree-row-opens-in-the-platform.md.

<storycode:copy id="f-508e32e299fd-expression_statement-7">
// The right-click menu: the tree's door to the rest of the machine.
// Guards docs/guarantees/authoring/a-tree-row-opens-in-the-platform.md.
describe("the right-click menu", () => {
  const PLATFORM: FilesResponse = {
    ...RESPONSE,
    root_path: "/home/me/notebook",
    separator: "/",
    file_manager: "Finder",
  };

  function menuItem(id: string): HTMLElement {
    const el = document.querySelector<HTMLElement>(`[data-menu-item="${id}"]`);
    if (!el) throw new Error(`no menu item ${id} in ${document.body.innerHTML}`);
    return el;
  }

  it("copies the absolute path, the relative path, and the name", async () => {
    const written: string[] = [];
    Object.assign(navigator, {
      clipboard: { writeText: (t: string) => (written.push(t), Promise.resolve()) },
    });
    mockFiles(PLATFORM);
    render(<Harness />);
    const row = await screen.findByText("readme.txt");

    for (const id of ["copy-absolute", "copy-relative", "copy-name"]) {
      fireEvent.contextMenu(row);
      fireEvent.click(menuItem(id));
    }
    expect(written).toEqual(["/home/me/notebook/readme.txt", "readme.txt", "readme.txt"]);
  });

  it("hands a file to the file manager and to its default program", async () => {
    const calls: [string, unknown][] = [];
    installMockHandler(async (method, path, body) => {
      if (method === "GET" && path === "/api/files") return PLATFORM;
      if (method === "GET" && path === "/api/workspace/github") {
        return { status: "not_repository", reviews: [], issues: [] };
      }
      calls.push([`${method} ${path}`, body]);
      return { ok: true };
    });
    render(<Harness />);
    const row = await screen.findByText("readme.txt");

    fireEvent.contextMenu(row);
    expect(menuItem("reveal").textContent).toBe("Reveal in Finder");
    fireEvent.click(menuItem("reveal"));
    fireEvent.contextMenu(row);
    fireEvent.click(menuItem("open-external"));

    expect(calls).toEqual([
      ["POST /api/reveal", { path: "readme.txt" }],
      ["POST /api/open-external", { path: "readme.txt" }],
    ]);
  });

  it("offers the menu on a directory and on a binary the app cannot open", async () => {
    mockFiles(PLATFORM);
    render(<Harness />);
    // A binary row is inert to a click and still has a machine that can open
    // it — that is the whole reason the menu is on every row.
    fireEvent.contextMenu(await screen.findByText("logo.png"));
    expect(menuItem("open-external")).toBeTruthy();
    fireEvent.keyDown(window, { key: "Escape" });

    fireEvent.contextMenu(screen.getByText("src/"));
    // A directory has no default program of its own worth naming.
    expect(document.querySelector('[data-menu-item="open-external"]')).toBeNull();
    expect(menuItem("copy-name").textContent).toBe("Copy folder name");
    // ...and right-clicking it did not change the editable buffer.
    expect(filesEditor().state.doc.line(1).text).toBe("src/");
  });

  it("says so when the platform refuses, instead of appearing to do nothing", async () => {
    installMockHandler(async (method, path) => {
      if (method === "GET" && path === "/api/files") return PLATFORM;
      throw new Error("could not start the file manager (`xdg-open`)");
    });
    render(<Harness />);
    fireEvent.contextMenu(await screen.findByText("readme.txt"));
    fireEvent.click(menuItem("reveal"));
    expect(await screen.findByText(/could not start the file manager/)).toBeTruthy();
  });

  it("closes on Escape and on a click elsewhere", async () => {
    mockFiles(PLATFORM);
    render(<Harness />);
    const row = await screen.findByText("readme.txt");

    fireEvent.contextMenu(row);
    fireEvent.keyDown(window, { key: "Escape" });
    expect(document.querySelector(".tree-menu")).toBeNull();

    fireEvent.contextMenu(row);
    fireEvent.mouseDown(document.body);
    expect(document.querySelector(".tree-menu")).toBeNull();
  });
});

</storycode:copy>
<a id="f-508e32e299fd-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-508e32e299fd-expression_statement-8">
describe("what clicking a file does", () => {
  it("maps documents, generated files, plain text files, and binaries apart", () => {
    const openable = new Set(["src/gen/orders.py"]);
    expect(fileAction(TREE[1], openable)).toEqual({ kind: "doc", id: "d1" });
    expect(fileAction({ name: "orders.py", path: "src/gen/orders.py", dir: false }, openable)).toEqual({
      kind: "generated",
      path: "src/gen/orders.py",
    });
    // Not a document, not woven by an open document: still a text file in
    // the folder, so it opens as a plain file rather than sitting inert.
    expect(fileAction(TREE[2], openable)).toEqual({ kind: "file", path: "readme.txt" });
    expect(fileAction(TREE[3], openable)).toEqual({ kind: "inert" });
  });

  it("judges binary-ness by extension, with no-extension and dotfiles assumed text", () => {
    expect(isLikelyBinaryPath("assets/logo.png")).toBe(true);
    expect(isLikelyBinaryPath("dist/app.WASM")).toBe(true);
    expect(isLikelyBinaryPath("justfile")).toBe(false);
    expect(isLikelyBinaryPath(".gitignore")).toBe(false);
    expect(isLikelyBinaryPath("Cargo.lock")).toBe(false);
  });

  it("opens text entries on double-click while a single click only places the caret", async () => {
    const onOpen = vi.fn();
    render(<Harness onOpen={onOpen} openable={new Set(["src/main.rs"])} />);
    fireEvent.click(await screen.findByText("paper.md"));
    expect(onOpen).not.toHaveBeenCalled();
    fireEvent.doubleClick(screen.getByText("paper.md"));
    expect(onOpen).toHaveBeenLastCalledWith({ kind: "doc", id: "d1" });
    fireEvent.doubleClick(screen.getByText("main.rs"));
    expect(onOpen).toHaveBeenLastCalledWith({ kind: "generated", path: "src/main.rs" });
    fireEvent.doubleClick(screen.getByText("readme.txt"));
    expect(onOpen).toHaveBeenLastCalledWith({ kind: "file", path: "readme.txt" });
    fireEvent.doubleClick(screen.getByText("logo.png"));
    expect(onOpen).toHaveBeenCalledTimes(3);
  });
});

</storycode:copy>
<a id="f-508e32e299fd-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-508e32e299fd-expression_statement-9">
describe("expand state, across sessions", () => {
  it("round-trips through localStorage, keyed by root", () => {
    saveExpanded("/a", new Set(["src", "src/gen"]));
    saveExpanded("/b", new Set(["docs"]));
    expect(loadExpanded("/a")).toEqual(new Set(["src", "src/gen"]));
    expect(loadExpanded("/b")).toEqual(new Set(["docs"]));
    expect(loadExpanded("/c")).toEqual(new Set());
  });

  it("toggles without mutating what it was given", () => {
    const before = new Set(["src"]);
    expect(toggleExpanded(before, "docs")).toEqual(new Set(["src", "docs"]));
    expect(toggleExpanded(before, "src")).toEqual(new Set());
    expect(before).toEqual(new Set(["src"]));
  });

  it("shrugs at a corrupt entry instead of breaking the pane", () => {
    localStorage.setItem("hickory.tree.expanded:/a", "{not json");
    expect(loadExpanded("/a")).toEqual(new Set());
  });

  it("keeps a directory open across unmount and remount", async () => {
    const first = render(<Harness />);
    fireEvent.click(await screen.findByText(/src/));
    expect(screen.getByText("main.rs")).toBeTruthy();
    first.unmount();

    render(<Harness />);
    // Re-fetched, re-mounted — and src is still expanded, from localStorage.
    expect(await screen.findByText("main.rs")).toBeTruthy();
  });
});

</storycode:copy>
<a id="f-508e32e299fd-session"></a>
### `session`

<storycode:copy id="f-508e32e299fd-session">
// ---------------------------------------------------------------------------
// Terminals, shown where they are running
// ---------------------------------------------------------------------------

function session(id: string, cwd: string, over: Partial<TreeSession> = {}): TreeSession {
  return {
    id,
    title: id,
    cwd,
    state: "running",
    monitor: false,
    cwdIsLive: true,
    ...over,
  };
}

</storycode:copy>
<a id="f-508e32e299fd-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-508e32e299fd-expression_statement-10">
describe("placing sessions in the tree", () => {
  // Node paths in a listing are RELATIVE to the root; session directories are
  // absolute. Keeping the fixture honest about that is the whole point — the
  // first version of this compared the two directly and matched nothing.
  const root = "/w";
  const dirs = new Set(["src", "src/deep", "docs"]);

  it("puts a session at its own directory", () => {
    const placed = placeSessions([session("a", "/w/src")], root, dirs);
    expect(placed.get("src")?.map((s) => s.id)).toEqual(["a"]);
  });

  it("puts a session working in the root at the root", () => {
    const placed = placeSessions([session("a", "/w")], root, dirs);
    expect(placed.get("")?.map((s) => s.id)).toEqual(["a"]);
  });

  it("climbs to the nearest directory the tree actually lists", () => {
    // A large folder is truncated, and a session may be working somewhere the
    // listing never mentioned. Showing it at the nearest ancestor beats not
    // showing it at all.
    const placed = placeSessions([session("a", "/w/src/deep/unlisted/x")], root, dirs);
    expect(placed.get("src/deep")?.map((s) => s.id)).toEqual(["a"]);
  });

  it("leaves out a session working outside the folder", () => {
    // The question is "what is running IN HERE".
    const placed = placeSessions([session("a", "/elsewhere")], root, dirs);
    expect(placed.size).toBe(0);
  });

  it("is not fooled by a sibling with a shared prefix", () => {
    // `/w-other` starts with `/w` as a string and is not inside it.
    const placed = placeSessions([session("a", "/w-other/src")], root, dirs);
    expect(placed.size).toBe(0);
  });

  it("treats a trailing slash as the same directory", () => {
    const placed = placeSessions([session("a", "/w/src/")], "/w/", dirs);
    expect(placed.get("src")?.map((s) => s.id)).toEqual(["a"]);
  });

  it("keeps several sessions in one directory", () => {
    const placed = placeSessions([session("a", "/w/src"), session("b", "/w/src")], root, dirs);
    expect(placed.get("src")).toHaveLength(2);
  });
});

</storycode:copy>
<a id="f-508e32e299fd-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-508e32e299fd-expression_statement-11">
describe("counting what a collapsed directory hides", () => {
  it("counts the whole subtree, not just the directory itself", () => {
    const sessions = [session("a", "/w/src"), session("b", "/w/src/deep"), session("c", "/w")];
    expect(sessionsUnder(sessions, "/w", "src")).toBe(2);
    expect(sessionsUnder(sessions, "/w", "src/deep")).toBe(1);
  });

  it("does not count a sibling with a shared prefix", () => {
    expect(sessionsUnder([session("a", "/w/srcother")], "/w", "src")).toBe(0);
  });

  it("does not count a session outside the folder", () => {
    expect(sessionsUnder([session("a", "/elsewhere/src")], "/w", "src")).toBe(0);
  });
});

</storycode:copy>
<a id="f-508e32e299fd-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-508e32e299fd-expression_statement-12">
describe("directoryPaths", () => {
  it("collects directories, root-relative, and no files", () => {
    const paths = directoryPaths(TREE);
    expect(paths.has("src")).toBe(true);
    expect([...paths].every((p) => !p.endsWith(".md") && !p.startsWith("/"))).toBe(true);
  });
});

</storycode:copy>
<a id="f-508e32e299fd-expression_statement-13"></a>
### `expression_statement`

<storycode:copy id="f-508e32e299fd-expression_statement-13">
describe("relativeCwd", () => {
  it("answers empty for the root itself and null for anything outside", () => {
    expect(relativeCwd("/w", "/w")).toBe("");
    expect(relativeCwd("/w/src/deep", "/w")).toBe("src/deep");
    expect(relativeCwd("/w-other", "/w")).toBeNull();
    expect(relativeCwd("/", "/w")).toBeNull();
  });
});

</storycode:copy>
<a id="f-508e32e299fd-expression_statement-14"></a>
### `expression_statement`

- docs/guarantees/terminal/a-session-appears-where-it-is-working.md
- docs/guarantees/authoring/the-workspace-tree-is-a-lens.md

<storycode:copy id="f-508e32e299fd-expression_statement-14">
// Guarantees:
// - docs/guarantees/terminal/a-session-appears-where-it-is-working.md
// - docs/guarantees/authoring/the-workspace-tree-is-a-lens.md
describe("showing a terminal from the tree", () => {
  it("lists a session at the folder it is working in, and opening it names it", async () => {
    const onOpenTerminal = vi.fn();
    render(
      <Harness
        sessions={[session("t1", "/home/me/notebook", { title: "cargo test", state: "working" })]}
        onOpenTerminal={onOpenTerminal}
      />,
    );
    const row = await screen.findByRole("button", { name: /cargo test/ });
    expect(row.getAttribute("data-workspace-node-key")).toBe("terminal:t1");
    expect(row.className).toContain("folder-tree__terminal");
    expect(row.textContent).toContain("working");
    fireEvent.click(row);
    expect(onOpenTerminal).toHaveBeenCalledWith("t1");
  });

  it("indents a nested terminal beneath its cwd and activates it from the buffer", async () => {
    const onOpenTerminal = vi.fn();
    render(<Harness sessions={[session("t1", "/home/me/notebook/src", { title: "cargo watch" })]} onOpenTerminal={onOpenTerminal} />);
    const terminal = await screen.findByText("terminal: cargo watch · running");
    expect(filesEditor().state.doc.toString()).toContain("src/\n  terminal: cargo watch · running\n  gen/");
    fireEvent.doubleClick(terminal);
    expect(onOpenTerminal).toHaveBeenCalledWith("t1");
  });

  it("offers close on the visible terminal node's own menu", async () => {
    const onCloseTerminal = vi.fn();
    render(
      <Harness
        sessions={[session("t1", "/home/me/notebook", { title: "cargo test" })]}
        onCloseTerminal={onCloseTerminal}
      />,
    );
    fireEvent.contextMenu(await screen.findByRole("button", { name: /cargo test/ }));
    fireEvent.click(screen.getByText("Close cargo test"));
    expect(onCloseTerminal).toHaveBeenCalledWith("t1");
  });

  it("keeps a nested session visible as text even before a folder is folded", async () => {
    render(<Harness sessions={[session("t1", "/home/me/notebook/src")]} />);
    await screen.findByText("terminal: t1 · running");
    expect(filesEditor().state.doc.toString()).toContain("  terminal: t1 · running");
  });

  it("shows nothing for a session working outside the folder", async () => {
    render(<Harness sessions={[session("t1", "/somewhere/else", { title: "elsewhere" })]} />);
    await screen.findByText("notebook");
    expect(screen.queryByRole("button", { name: /elsewhere/ })).toBeNull();
  });
});

</storycode:copy>
<a id="f-508e32e299fd-expression_statement-15"></a>
### `expression_statement`

<storycode:copy id="f-508e32e299fd-expression_statement-15">
// docs/guarantees/authoring/the-tree-is-a-dired.md
describe("the tree as dired", () => {
  it("keeps destructive deletion behind the existing confirmation", async () => {
    const ops: unknown[] = [];
    installMockHandler(async (method, path, body) => {
      if (method === "GET" && path === "/api/files") return RESPONSE;
      if (method === "POST" && path === "/api/files/op") {
        ops.push(body);
        return { op: "delete" };
      }
      throw new Error(`unexpected ${method} ${path}`);
    });
    render(<Harness />);
    const readme = await screen.findByText("readme.txt");
    fireEvent.contextMenu(readme);
    fireEvent.click(await screen.findByText("Delete"));
    const prompt = await screen.findByRole("dialog");
    expect(prompt.textContent).toContain("Delete readme.txt?");
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    await vi.waitFor(() => expect(ops).toEqual([{ op: "delete", path: "readme.txt" }]));
    await vi.waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });

  it("renames by editing the buffer text and saving it", async () => {
    const ops: unknown[] = [];
    installMockHandler(async (method, path, body) => {
      if (method === "GET" && path === "/api/files") return RESPONSE;
      if (method === "POST" && path === "/api/files/op") {
        ops.push(body);
        return { op: "rename" };
      }
      throw new Error(`unexpected ${method} ${path}`);
    });
    render(<Harness />);
    await screen.findByText("readme.txt");
    const editor = filesEditor();
    const line = editor.state.doc.line(6);
    editor.dispatch({ changes: { from: line.from, to: line.to, insert: "notes.txt" } });
    fireEvent.keyDown(editor.contentDOM, { key: "s", ctrlKey: true });
    await vi.waitFor(() => expect(ops).toEqual([{ op: "rename", path: "readme.txt", to: "notes.txt" }]));
    expect(await screen.findByText("Applied 1 filesystem edit.")).toBeTruthy();
  });
});

</storycode:copy>
<a id="f-508e32e299fd-expression_statement-16"></a>
### `expression_statement`

<storycode:copy id="f-508e32e299fd-expression_statement-16">
// docs/guarantees/authoring/a-file-is-ingested-from-the-tree.md
describe("ingest from the tree", () => {
  it("makes a plain file literate and opens the document it became", async () => {
    const adopted: unknown[] = [];
    installMockHandler(async (method, path, body) => {
      if (method === "GET" && path === "/api/files") return RESPONSE;
      if (method === "POST" && path === "/api/adopt") {
        adopted.push(body);
        return { doc_id: "d9", doc_path: "readme.md", file_path: "readme.txt" };
      }
      throw new Error(`unexpected ${method} ${path}`);
    });
    const onOpen = vi.fn();
    render(<Harness onOpen={onOpen} />);
    fireEvent.contextMenu(await screen.findByText("readme.txt"));
    fireEvent.click(await screen.findByText(/Make literate/));
    await vi.waitFor(() => expect(onOpen).toHaveBeenCalledWith({ kind: "doc", id: "d9" }));
    expect(adopted).toEqual([{ path: "readme.txt" }]);
    // A binary and a document get no such verb.
    fireEvent.contextMenu(screen.getByText("logo.png"));
    expect(screen.queryByText(/Make literate/)).toBeNull();
  });
});

</storycode:copy>
<a id="f-508e32e299fd-expression_statement-17"></a>
### `expression_statement`

<storycode:copy id="f-508e32e299fd-expression_statement-17">
// Guarantee: docs/guarantees/terminal/a-session-appears-where-it-is-working.md
it("shares folder header/root terminal menu disclosure and retires a removed terminal's menu", async () => {
  const registry = new UiActions({ workspace: "practice", window: "root-menus" }, () => true, async () => {});
  const onOpenTerminal = vi.fn(), onCloseTerminal = vi.fn(), onNewDocument = vi.fn();
  const terminal = session("root-t", "/home/me/notebook", { title: "root witness" });
  const renderPane = (sessions: TreeSession[]) => <UiActionContext.Provider value={registry}><FolderTreePane roots={[RESPONSE]} sessions={sessions} openable={new Set()} onOpen={() => {}} onNewDocument={onNewDocument} onOpenTerminal={onOpenTerminal} onCloseTerminal={onCloseTerminal} /></UiActionContext.Provider>;
  const mounted = render(renderPane([terminal]));
  await screen.findByRole("button", { name: /root witness/ });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const invoke = async (label: string, args = {}) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "acp:root", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return reply; };
  expect(await invoke("Read root terminal row page", { first: 0, count: 64 })).toMatchObject({ outcome: { value: { rows: [{ id: "root-t", title: "root witness" }], output_read: false } } });
  await invoke("New document from folder header"); expect(onNewDocument).toHaveBeenCalledTimes(1);
  await invoke("Open workspace folder header menu"); expect(screen.getByRole("menuitem", { name: "New folder…" })).toBeTruthy();
  await invoke("Open root terminal row", { id: "root-t" }); expect(onOpenTerminal).toHaveBeenCalledWith("root-t");
  await invoke("Open root terminal row menu", { id: "root-t" });
  expect(screen.getByText("Close root witness")).toBeTruthy();
  const old = actions().find(action => action.label === "Close root witness")!;
  mounted.rerender(renderPane([]));
  expect(screen.queryByText("Close root witness")).toBeNull();
  expect(await registry.act(registry.scope, { id: "acp:root", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "retired" })).not.toMatchObject({ outcome: { status: "completed" } });
  expect(onCloseTerminal).not.toHaveBeenCalled();
  expect(await invoke("Open root terminal row menu", { id: "root-t" })).toMatchObject({ outcome: { status: "unavailable" } });
});
</storycode:copy>

### Assemble `apps/web/src/shell/FolderTreePane.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/FolderTreePane.test.tsx"><storycode:paste select="#f-508e32e299fd-TREE"/><storycode:paste select="#f-508e32e299fd-RESPONSE"/><storycode:paste select="#f-508e32e299fd-expression_statement"/><storycode:paste select="#f-508e32e299fd-mockFiles"/><storycode:paste select="#f-508e32e299fd-Harness"/><storycode:paste select="#f-508e32e299fd-filesEditor"/><storycode:paste select="#f-508e32e299fd-expression_statement-2"/><storycode:paste select="#f-508e32e299fd-expression_statement-3"/><storycode:paste select="#f-508e32e299fd-expression_statement-4"/><storycode:paste select="#f-508e32e299fd-expression_statement-5"/><storycode:paste select="#f-508e32e299fd-expression_statement-6"/><storycode:paste select="#f-508e32e299fd-expression_statement-7"/><storycode:paste select="#f-508e32e299fd-expression_statement-8"/><storycode:paste select="#f-508e32e299fd-expression_statement-9"/><storycode:paste select="#f-508e32e299fd-session"/><storycode:paste select="#f-508e32e299fd-expression_statement-10"/><storycode:paste select="#f-508e32e299fd-expression_statement-11"/><storycode:paste select="#f-508e32e299fd-expression_statement-12"/><storycode:paste select="#f-508e32e299fd-expression_statement-13"/><storycode:paste select="#f-508e32e299fd-expression_statement-14"/><storycode:paste select="#f-508e32e299fd-expression_statement-15"/><storycode:paste select="#f-508e32e299fd-expression_statement-16"/><storycode:paste select="#f-508e32e299fd-expression_statement-17"/></storycode:file>


</storycode:doc>
