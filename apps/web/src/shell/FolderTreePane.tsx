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

import { useCallback, useEffect, useState } from "react";

import { api } from "../api/client";
import type { FileNode, FilesResponse } from "../api/types";

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
  error?: string | null;
}

export function FolderTreePane({
  roots,
  openable,
  onOpen,
  onNewDocument,
  activeDocId,
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
}: {
  folder: FolderTree;
  openable: ReadonlySet<string>;
  onOpen: FolderTreePaneProps["onOpen"];
  onNewDocument: () => void;
  activeDocId?: string;
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

  const name = folder.root.replace(/\/+$/, "").split("/").pop() || folder.root;
  return (
    <section className="folder-tree__root">
      <header className="folder-tree__header">
        <span className="folder-tree__name" data-tip={folder.root}>
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
          />
        ))}
      </ul>
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
}: {
  node: FileNode;
  depth: number;
  expanded: ReadonlySet<string>;
  onToggle: (path: string) => void;
  openable: ReadonlySet<string>;
  onOpen: FolderTreePaneProps["onOpen"];
  activeDocId?: string;
}) {
  const indent = { paddingLeft: `${depth * 0.85 + 0.4}rem` };
  if (node.dir) {
    const open = expanded.has(node.path);
    return (
      <li role="treeitem" aria-expanded={open}>
        <button
          type="button"
          className="folder-tree__dir mono"
          style={indent}
          onClick={() => onToggle(node.path)}
          data-tip={node.path}
        >
          <span className="folder-tree__disclosure" aria-hidden>
            {open ? "▾" : "▸"}
          </span>
          {node.name}
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
      >
        {node.name}
      </button>
    </li>
  );
}
