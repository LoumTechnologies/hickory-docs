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

import { useCallback, useEffect, useMemo, useState, type MouseEvent } from "react";

import { api } from "../api/client";
import type { FileNode, FilesResponse } from "../api/types";
import { TreeContextMenu } from "./TreeContextMenu";
import { copyText, treeMenuItems, type TreeFolder, type TreeMenuItem } from "./treeMenu";

/** One open folder: the server's FilesResponse, kept whole. */
export type FolderTree = FilesResponse;

// ---------------------------------------------------------------------------
// What clicking a file does
// ---------------------------------------------------------------------------

export type FileAction =
  /** A `.hick` document: navigate to its route. */
  | { kind: "doc"; id: string }
  /** A file the current document generates: open a generated pane. */
  | { kind: "generated"; path: string }
  /** Any other text file: open a plain-file pane. */
  | { kind: "file"; path: string }
  /** A file this app cannot show (binary, by extension): named, and inert. */
  | { kind: "inert" };

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

export function isLikelyBinaryPath(path: string): boolean {
  const name = path.split("/").pop() ?? path;
  const dot = name.lastIndexOf(".");
  if (dot <= 0) return false; // no extension, or a dotfile: assume text
  return BINARY_EXTENSIONS.has(name.slice(dot + 1).toLowerCase());
}

/**
 * What a click on `node` should do, given which non-document paths are
 * openable as generated files (the open documents' outputs). Everything
 * else that looks like text opens as a plain file.
 */
export function fileAction(node: FileNode, openable: ReadonlySet<string>): FileAction {
  if (node.doc_id) return { kind: "doc", id: node.doc_id };
  if (openable.has(node.path)) return { kind: "generated", path: node.path };
  if (isLikelyBinaryPath(node.path)) return { kind: "inert" };
  return { kind: "file", path: node.path };
}

// ---------------------------------------------------------------------------
// Expand state: which directories are open, per root, across sessions
// ---------------------------------------------------------------------------

function expandKey(root: string): string {
  return `hickory.tree.expanded:${root}`;
}

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

export function saveExpanded(root: string, expanded: ReadonlySet<string>): void {
  try {
    localStorage.setItem(expandKey(root), JSON.stringify([...expanded]));
  } catch {
    // Storage full or denied: the tree still works, it just forgets.
  }
}

export function toggleExpanded(expanded: ReadonlySet<string>, path: string): Set<string> {
  const next = new Set(expanded);
  if (next.has(path)) next.delete(path);
  else next.add(path);
  return next;
}

// ---------------------------------------------------------------------------
// Fetching
// ---------------------------------------------------------------------------

/** The event anything that changes files on disk may dispatch to refresh
 * every mounted tree without a page-wide state channel. */
export const FILES_CHANGED_EVENT = "hickory:files-changed";

/**
 * The open folder's tree, kept fresh: fetched on mount, refetched when the
 * window regains focus (the cheap way to notice out-of-app edits) and when
 * anything dispatches FILES_CHANGED_EVENT (runs, saves).
 */
export function useFolderTrees(): { roots: FolderTree[]; error: string | null } {
  const [roots, setRoots] = useState<FolderTree[]>([]);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    api.files().then(
      (response) => {
        setRoots([response]);
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

  return { roots, error };
}

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

/** Strip trailing slashes so `/a/b/` and `/a/b` are one directory. */
function normalizeDir(path: string): string {
  const trimmed = path.replace(/\/+$/, "");
  return trimmed === "" ? "/" : trimmed;
}

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

/**
 * A right-click on a row, reported with what was clicked.
 *
 * Every row kind reports the same three things — where the pointer was, which
 * root-relative path the row names, and whether it is a directory — so the
 * menu is built in one place from one shape rather than per row type.
 */
export type OnRowMenu = (event: MouseEvent, path: string, dir: boolean) => void;

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
  /** Terminal sessions, shown at the directory each one is working in. */
  sessions?: readonly TreeSession[];
  /** A click on a session row: show that terminal. */
  onOpenTerminal?: (id: string) => void;
  error?: string | null;
}

/** One running session, at the directory it is running in. */
function SessionRow({
  session,
  depth,
  onOpen,
}: {
  session: TreeSession;
  depth: number;
  onOpen?: (id: string) => void;
}) {
  const indent = { paddingLeft: `${depth * 0.85 + 0.4}rem` };
  // Where the shell says it is, versus where it was started, is a real
  // difference in how much to trust this row's placement — so the tooltip
  // says which one it is rather than presenting both as the same fact.
  const where = session.cwdIsLive
    ? session.cwd
    : `${session.cwd} — started here; this shell does not report its directory`;
  return (
    <li role="treeitem">
      <button
        type="button"
        className={`folder-tree__session mono state-${session.state}`}
        style={indent}
        data-tip={where}
        data-session-id={session.id}
        onClick={() => onOpen?.(session.id)}
      >
        <span className="folder-tree__session-dot" aria-hidden />
        {session.title}
        {session.monitor && <span className="folder-tree__session-monitor">monitor</span>}
      </button>
    </li>
  );
}

export function FolderTreePane({
  roots,
  openable,
  onOpen,
  onNewDocument,
  activeDocId,
  sessions = [],
  onOpenTerminal,
  error,
}: FolderTreePaneProps) {
  if (error) return <p className="error folder-tree__error">{error}</p>;
  if (roots.length === 0) return <p className="muted folder-tree__loading">Reading folder…</p>;
  return (
    <div className="folder-tree">
      {roots.map((folder) => (
        <FolderRoot
          key={folder.root}
          folder={folder}
          openable={openable}
          onOpen={onOpen}
          onNewDocument={onNewDocument}
          activeDocId={activeDocId}
          sessions={sessions}
          onOpenTerminal={onOpenTerminal}
        />
      ))}
    </div>
  );
}

function FolderRoot({
  folder,
  openable,
  onOpen,
  onNewDocument,
  activeDocId,
  sessions,
  onOpenTerminal,
}: {
  folder: FolderTree;
  openable: ReadonlySet<string>;
  onOpen: FolderTreePaneProps["onOpen"];
  onNewDocument: () => void;
  activeDocId?: string;
  sessions: readonly TreeSession[];
  onOpenTerminal?: (id: string) => void;
}) {
  const [expanded, setExpanded] = useState<Set<string>>(() => loadExpanded(folder.root));
  const toggle = useCallback(
    (path: string) => {
      setExpanded((current) => {
        const next = toggleExpanded(current, path);
        saveExpanded(folder.root, next);
        return next;
      });
    },
    [folder.root],
  );

  const placement = useMemo(
    () => placeSessions(sessions, folder.root, directoryPaths(folder.tree)),
    [sessions, folder.root, folder.tree],
  );
  const atRoot = placement.get("") ?? [];

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

  const [menu, setMenu] = useState<{
    x: number;
    y: number;
    path: string;
    dir: boolean;
  } | null>(null);
  // A failed reveal (no file manager on a headless Linux box, a file deleted
  // between the listing and the click) says so in the pane. The alternative is
  // a menu item that appears to do nothing at all.
  const [notice, setNotice] = useState<string | null>(null);

  const openMenu = useCallback<OnRowMenu>((event, path, dir) => {
    event.preventDefault();
    event.stopPropagation();
    setNotice(null);
    setMenu({ x: event.clientX, y: event.clientY, path, dir });
  }, []);

  const runItem = useCallback((item: TreeMenuItem) => {
    setMenu(null);
    const action = item.action;
    if (action.kind === "copy") {
      void copyText(action.text).catch((e: unknown) =>
        setNotice(`Could not copy: ${e instanceof Error ? e.message : String(e)}`),
      );
      return;
    }
    const call = action.kind === "reveal" ? api.reveal(action.path) : api.openExternal(action.path);
    void call.catch((e: unknown) =>
      setNotice(e instanceof Error ? e.message : String(e)),
    );
  }, []);

  const name = folder.root.replace(/\/+$/, "").split("/").pop() || folder.root;
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
          onClick={onNewDocument}
        >
          +
        </button>
      </header>
      {folder.truncated && (
        <p className="muted folder-tree__truncated">Large folder — not everything is listed.</p>
      )}
      {notice && <p className="error folder-tree__notice">{notice}</p>}
      <ul className="folder-tree__list" role="tree">
        {atRoot.map((session) => (
          <SessionRow key={session.id} session={session} depth={0} onOpen={onOpenTerminal} />
        ))}
        {folder.tree.map((node) => (
          <TreeRow
            key={node.path}
            node={node}
            depth={0}
            expanded={expanded}
            onToggle={toggle}
            openable={openable}
            onOpen={onOpen}
            activeDocId={activeDocId}
            sessions={sessions}
            placement={placement}
            onOpenTerminal={onOpenTerminal}
            root={folder.root}
            onRowMenu={openMenu}
          />
        ))}
      </ul>
      {menu && (
        <TreeContextMenu
          x={menu.x}
          y={menu.y}
          subject={menu.path === "" ? name : menu.path}
          items={treeMenuItems(info, menu.path, menu.dir)}
          onPick={runItem}
          onClose={() => setMenu(null)}
        />
      )}
    </section>
  );
}

function TreeRow({
  node,
  depth,
  expanded,
  onToggle,
  openable,
  onOpen,
  activeDocId,
  sessions,
  placement,
  onOpenTerminal,
  root,
  onRowMenu,
}: {
  node: FileNode;
  depth: number;
  expanded: ReadonlySet<string>;
  onToggle: (path: string) => void;
  openable: ReadonlySet<string>;
  onOpen: FolderTreePaneProps["onOpen"];
  activeDocId?: string;
  sessions: readonly TreeSession[];
  placement: ReadonlyMap<string, TreeSession[]>;
  onOpenTerminal?: (id: string) => void;
  root: string;
  /** A right-click anywhere on this row (or its children). */
  onRowMenu: OnRowMenu;
}) {
  const indent = { paddingLeft: `${depth * 0.85 + 0.4}rem` };
  if (node.dir) {
    const open = expanded.has(node.path);
    const here = placement.get(node.path.replace(/\/+$/, "")) ?? [];
    // A collapsed directory would hide what is running inside it, which is
    // exactly the thing worth seeing — so it says how many instead.
    const hidden = open ? 0 : sessionsUnder(sessions, root, node.path);
    return (
      <li role="treeitem" aria-expanded={open}>
        <button
          type="button"
          className="folder-tree__dir mono"
          style={indent}
          onClick={() => onToggle(node.path)}
          onContextMenu={(event) => onRowMenu(event, node.path, true)}
          data-tip={node.path}
        >
          <span className="folder-tree__disclosure" aria-hidden>
            {open ? "▾" : "▸"}
          </span>
          {node.name}
          {hidden > 0 && (
            <span
              className="folder-tree__session-count"
              data-tip={`${hidden} terminal${hidden === 1 ? "" : "s"} running in here`}
            >
              {hidden}
            </span>
          )}
        </button>
        {open && (
          <ul className="folder-tree__list" role="group">
            {here.map((session) => (
              <SessionRow
                key={session.id}
                session={session}
                depth={depth + 1}
                onOpen={onOpenTerminal}
              />
            ))}
            {(node.children ?? []).map((child) => (
              <TreeRow
                key={child.path}
                node={child}
                depth={depth + 1}
                expanded={expanded}
                onToggle={onToggle}
                openable={openable}
                onOpen={onOpen}
                activeDocId={activeDocId}
                sessions={sessions}
                placement={placement}
                onOpenTerminal={onOpenTerminal}
                root={root}
                onRowMenu={onRowMenu}
              />
            ))}
          </ul>
        )}
      </li>
    );
  }

  const action = fileAction(node, openable);
  if (action.kind === "inert") {
    // Named, present, and not pretending to be a link: a file this app has
    // no way to show is still part of the folder's truth. The data
    // attributes are the contract with the ribbon overlay — a connection
    // whose file is visible as a row terminates on the row's near edge —
    // and an inert row is still an honest place to point.
    return (
      <li role="treeitem">
        <span
          className="folder-tree__file folder-tree__file--inert mono"
          style={indent}
          data-tip={node.path}
          data-tree-path={node.path}
          data-tree-kind="inert"
          onContextMenu={(event) => onRowMenu(event, node.path, false)}
        >
          {node.name}
        </span>
      </li>
    );
  }
  const active = action.kind === "doc" && action.id === activeDocId;
  return (
    <li role="treeitem">
      <button
        type="button"
        className={`folder-tree__file mono${active ? " on" : ""}`}
        style={indent}
        data-tip={node.path}
        // The ribbon overlay finds this row by path: a connection to a file
        // that is not open but IS visible here lands on this row's edge.
        data-tree-path={node.path}
        data-tree-kind={action.kind}
        onClick={() => onOpen(action)}
        // A file this app cannot or will not open in a pane still has a
        // machine that can: the same menu is on every row, inert ones
        // included.
        onContextMenu={(event) => onRowMenu(event, node.path, false)}
      >
        {node.name}
      </button>
    </li>
  );
}
