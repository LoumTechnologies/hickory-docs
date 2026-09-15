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

import { useCallback, useEffect, useMemo, useRef, useState, type MouseEvent } from "react";

import { api } from "../api/client";
import type { FileNode, FilesResponse } from "../api/types";
import { TreeContextMenu } from "./TreeContextMenu";
import { TreePrompt } from "./TreePrompt";
import { fileRenameError, TreeRenameEditor } from "./TreeRenameEditor";
import { useDired } from "./useDired";
import { useTreeNavigation } from "./useTreeNavigation";
import { TreeFindReplace } from "./TreeFindReplace";
import { FilesystemTreeEditor, type WorkspaceTextExtra } from "./FilesystemTreeEditor";
import {
  copyText,
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

/** The path of the document with this id, anywhere in the tree. */
export function docPathOf(nodes: readonly FileNode[], docId: string): string | null {
  for (const node of nodes) {
    if (node.doc_id === docId) return node.path;
    const inner = node.children ? docPathOf(node.children, docId) : null;
    if (inner) return inner;
  }
  return null;
}

function fileNodeAt(nodes: readonly FileNode[], path: string): FileNode | null {
  for (const node of nodes) {
    if (node.path === path) return node;
    const child = node.children ? fileNodeAt(node.children, path) : null;
    if (child) return child;
  }
  return null;
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

export function fileWorkspaceKey(path: string): string {
  return `filesystem:${encodeURIComponent(path)}`;
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
export type OnRowMenu = (event: MouseEvent, path: string, dir: boolean, plainText?: boolean) => void;

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

  const [menu, setMenu] = useState<{
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

  const openMenu = useCallback<OnRowMenu>((event, path, dir, plainText = false) => {
    event.preventDefault();
    event.stopPropagation();
    setNotice(null);
    setMenu({ x: event.clientX, y: event.clientY, path, dir, plainText });
  }, []);

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
    if (action.kind === "associate-github-issue") {
      setIssueAssociation({ folder: action.path });
      return;
    }
    if (action.kind === "close-terminal") {
      onCloseTerminal?.(action.id);
      return;
    }
    if (action.kind === "literate" || action.kind === "ingest") {
      // Byte-exact adoption, then the document it landed in opens: the
      // person asked for a document, not a file that changed under them.
      const into = action.kind === "ingest" ? action.into : undefined;
      void api.adopt(action.path, into).then(
        (outcome) => {
          window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
          onOpen({ kind: "doc", id: outcome.doc_id });
        },
        (e: unknown) => setNotice(e instanceof Error ? e.message : String(e)),
      );
      return;
    }
    if (action.kind === "rename") {
      dired.run({ kind: "rename", paths: action.paths });
      return;
    }
    if (action.kind === "move" || action.kind === "delete") {
      dired.run({ kind: action.kind, paths: action.paths });
      return;
    }
    if (action.kind === "copy-to") {
      dired.run({ kind: "copy", paths: action.paths });
      return;
    }
    if (action.kind === "create" || action.kind === "mkdir") {
      dired.run({ kind: action.kind, dir: action.dir });
      return;
    }
    const call = action.kind === "reveal" ? api.reveal(action.path) : api.openExternal(action.path);
    void call.catch((e: unknown) =>
      setNotice(e instanceof Error ? e.message : String(e)),
    );
  }, [onNewTerminal, onNewWorktree, onCloseTerminal, onOpen, dired]);

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
      {dired.prompt && (
        <TreePrompt
          label={dired.prompt.label}
          verb={dired.prompt.verb}
          initial={dired.prompt.initial}
          onSubmit={dired.answer}
          onCancel={dired.cancel}
          error={dired.promptError}
          busy={dired.busy}
        />
      )}
      {dired.renamePaths && (
        <TreeRenameEditor
          items={dired.renamePaths.map((path) => ({
            key: `filesystem:${encodeURIComponent(path)}`,
            context: path,
            value: path.split("/").pop() ?? path,
          }))}
          validate={fileRenameError}
          inlinePath={dired.renamePaths.length === 1 ? dired.renamePaths[0] : undefined}
          apply={async (item, value) => {
            await api.fileOp({ op: "rename", path: item.context, to: value });
          }}
          onClose={dired.closeRename}
        />
      )}
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
        onChanged={() => window.dispatchEvent(new Event(FILES_CHANGED_EVENT))}
        onOpenPath={(path) => {
          const node = fileNodeAt(folder.tree, path);
          if (!node) return;
          const action = fileAction(node, openable);
          if (action.kind !== "inert") onOpen(action);
        }}
        onContextPath={(event, path, dir) => {
          setNotice(null);
          setMenu({ x: event.clientX, y: event.clientY, path, dir, plainText: !dir && !isLikelyBinaryPath(path) });
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
      {menu && (
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
