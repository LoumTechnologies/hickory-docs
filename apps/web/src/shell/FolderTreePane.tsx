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
import { TreeFindReplace } from "./TreeFindReplace";
import {
  copyText,
  terminalMenuItems,
  treeMenuItems,
  type TreeFolder,
  type TreeMenuItem,
} from "./treeMenu";
import { alwaysVisible, hiddenSummary, rankSessions, urgentCount } from "../lib/treeTerminals";

/** One open folder: the server's FilesResponse, kept whole. */
export type FolderTree = FilesResponse;

// ---------------------------------------------------------------------------
// What clicking a file does
// ---------------------------------------------------------------------------

export type FileAction =
  /** A `.hick` document: navigate to its route. */
  | { kind: "doc"; id: string }
  /** A file some document generates: open a generated pane. `docId` names the
   * owner when the server knew it, which it does for any document in the
   * folder — not only for the ones already open. */
  | { kind: "generated"; path: string; docId?: string }
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
  // `openable` is what the OPEN documents weave; `generated_by` is what every
  // document in the folder declares. The second is the reason a woven file
  // opens as the generated thing it is even when its document is closed —
  // and the reason the app stops offering to make `cards.md` literate when
  // `cards.hick` has been writing it all along.
  if (node.generated_by) return { kind: "generated", path: node.path, docId: node.generated_by };
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
  /** Terminal sessions, shown as icons on the directory each one is working
   * in. There is no separate list of terminals any more: a terminal has a
   * working directory, this tree already draws directories, and two trees
   * meant two places to look for "what is going on". */
  sessions?: readonly TreeSession[];
  /** A click on a terminal icon: show that terminal. */
  onOpenTerminal?: (id: string) => void;
  /** "Open terminal here" on a directory's menu, with its root-relative
   * path. */
  onNewTerminal?: (path: string) => void;
  /** "New worktree here…" on a directory's menu. */
  onNewWorktree?: (path: string) => void;
  /** "Close" on a terminal icon's own menu — the only place a session can be
   * stopped now that terminals have no list of their own. */
  onCloseTerminal?: (id: string) => void;
  /** Jump to a find hit: a root-relative path and a 1-based line. */
  onOpenHit?: (path: string, line: number) => void;
  error?: string | null;
}

/** One running session, at the directory it is running in. */
/**
 * The terminals running in one directory, as icons on that directory's own
 * row.
 *
 * Icons rather than rows: a row per session pushes the folder's contents down
 * and turns a busy project's tree into mostly-not-files, which inverts what
 * the tree is for. These ride a row that already exists and cost no vertical
 * space at all. See lib/treeTerminals.ts for the ordering, and for why a
 * session that needs you does not wait to be hovered.
 */
function TerminalIcons({
  sessions,
  onOpen,
  onMenu,
}: {
  sessions: readonly TreeSession[];
  onOpen?: (id: string) => void;
  onMenu?: (event: MouseEvent, session: TreeSession) => void;
}) {
  if (sessions.length === 0) return null;
  return (
    <span className="folder-tree__terms" role="group" aria-label="Terminals here">
      {rankSessions(sessions).map((session) => {
        // Where the shell says it is, versus where it was started, is a real
        // difference in how much to trust this icon's placement — so the
        // tooltip says which one it is rather than presenting both as one
        // fact.
        const where = session.cwdIsLive
          ? session.cwd
          : `${session.cwd} — started here; this shell does not report its directory`;
        return (
          <span
            key={session.id}
            role="button"
            tabIndex={0}
            className={`folder-tree__term state-${session.state}${
              alwaysVisible(session.state) ? " folder-tree__term--urgent" : ""
            }${session.monitor ? " folder-tree__term--monitor" : ""}`}
            data-session-id={session.id}
            data-tip={`${session.title} — ${session.state.replace("-", " ")}\n${where}`}
            aria-label={`${session.title}, ${session.state.replace("-", " ")}`}
            // The icon sits INSIDE the directory's own button, whose click
            // toggles the folder. Both handlers stop propagation, or opening
            // a terminal would fold the directory it is in.
            onClick={(event) => {
              event.stopPropagation();
              onOpen?.(session.id);
            }}
            onKeyDown={(event) => {
              if (event.key !== "Enter" && event.key !== " ") return;
              event.preventDefault();
              event.stopPropagation();
              onOpen?.(session.id);
            }}
            onContextMenu={(event) => onMenu?.(event, session)}
          >
            ▮
          </span>
        );
      })}
    </span>
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
  onNewTerminal,
  onNewWorktree,
  onCloseTerminal,
  onOpenHit,
  error,
}: FolderTreePaneProps) {
  if (error) return <p className="error folder-tree__error">{error}</p>;
  if (roots.length === 0) return <p className="muted folder-tree__loading">Reading folder…</p>;
  return (
    <div className="folder-tree">
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

function FolderRoot({
  folder,
  openable,
  onOpen,
  onNewDocument,
  activeDocId,
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
  sessions: readonly TreeSession[];
  onOpenTerminal?: (id: string) => void;
  onNewTerminal?: (path: string) => void;
  onNewWorktree?: (path: string) => void;
  onCloseTerminal?: (id: string) => void;
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

  /** A right-click on a terminal icon: its own short menu, not the row's. */
  const [termMenu, setTermMenu] = useState<{
    x: number;
    y: number;
    session: TreeSession;
  } | null>(null);
  const openTermMenu = useCallback((event: MouseEvent, session: TreeSession) => {
    event.preventDefault();
    event.stopPropagation();
    setTermMenu({ x: event.clientX, y: event.clientY, session });
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
    if (action.kind === "close-terminal") {
      onCloseTerminal?.(action.id);
      return;
    }
    const call = action.kind === "reveal" ? api.reveal(action.path) : api.openExternal(action.path);
    void call.catch((e: unknown) =>
      setNotice(e instanceof Error ? e.message : String(e)),
    );
  }, [onNewTerminal, onNewWorktree, onCloseTerminal]);

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
        {/* Terminals working in the folder itself ride its header, the same
            way a subdirectory's ride its row. */}
        <TerminalIcons sessions={atRoot} onOpen={onOpenTerminal} onMenu={openTermMenu} />
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
            onTermMenu={openTermMenu}
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
      {termMenu && (
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
  onTermMenu,
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
  /** A right-click on one of this row's terminal icons. */
  onTermMenu?: (event: MouseEvent, session: TreeSession) => void;
}) {
  const indent = { paddingLeft: `${depth * 0.85 + 0.4}rem` };
  if (node.dir) {
    const open = expanded.has(node.path);
    const here = placement.get(node.path.replace(/\/+$/, "")) ?? [];
    // A collapsed directory would hide what is running inside it, which is
    // exactly the thing worth seeing — so it says how many instead.
    const hidden = open ? 0 : sessionsUnder(sessions, root, node.path);
    // Whether any of those hidden ones is asking a question, so a folded
    // directory can say "and one of them needs you" rather than just a count.
    const hiddenUrgent = open
      ? 0
      : urgentCount(
          sessions.filter((session) => {
            const cwd = relativeCwd(session.cwd, root);
            const dir = node.path.replace(/\/+$/, "");
            return cwd !== null && (cwd === dir || cwd.startsWith(`${dir}/`));
          }),
        );
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
          {/* Open: the terminals working in THIS directory, as icons.
              Collapsed: a count, because a folded directory would otherwise
              hide the processes running inside it — which is exactly the
              thing worth seeing. */}
          {open ? (
            <TerminalIcons sessions={here} onOpen={onOpenTerminal} onMenu={onTermMenu} />
          ) : (
            hidden > 0 && (
              <span
                className={`folder-tree__session-count${
                  hiddenUrgent > 0 ? " folder-tree__session-count--urgent" : ""
                }`}
                data-tip={hiddenSummary(hidden, hiddenUrgent)}
              >
                {hidden}
              </span>
            )
          )}
        </button>
        {open && (
          <ul className="folder-tree__list" role="group">
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
                onTermMenu={onTermMenu}
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
        {node.held && (
          // The loop is leaving this file as it is on disk — somebody's
          // bytes, not the document's — and says so here rather than
          // silently. The pane says why; this says that.
          <span className="folder-tree__held" data-tip={`Held: ${node.held}`}>
            held
          </span>
        )}
      </button>
    </li>
  );
}
