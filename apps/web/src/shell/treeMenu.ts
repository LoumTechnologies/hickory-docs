// What a right-click on a tree row offers, as data.
//
// The menu is a list of items with labels and actions, computed here and only
// rendered by the pane. Two reasons for the split: the interesting decisions
// are textual (what an absolute path looks like on Windows, what the file
// manager is called on this desktop, which items a directory has that a file
// does not), and a right-click is one of the more annoying things to reach
// through a UI test.
//
// Nothing here talks to the OS. Copying is the clipboard; revealing and
// opening are two server routes (crates/hickory-cli/src/serve/reveal.rs) —
// the machine whose Finder opens is the machine the engine runs on, which is
// the user's own.

/** The folder a menu is being built inside, as the server described it. */
export interface TreeFolder {
  /** The open folder's absolute path, in the platform's spelling. */
  rootPath?: string;
  /** What joins `rootPath` to a node path. Defaults to `/`. */
  separator?: string;
  /** "Finder" / "File Explorer" / "file manager". */
  fileManager?: string;
}

export type TreeMenuAction =
  /** Put `text` on the clipboard. */
  | { kind: "copy"; text: string }
  /** Show this root-relative path in the file manager. */
  | { kind: "reveal"; path: string }
  /** Open this root-relative path in its default program. */
  | { kind: "open"; path: string }
  /** Start a terminal working in this root-relative directory. Terminals live
   * in the folder tree now — they have a working directory, and the tree
   * already draws directories, so "open one here" is a verb of the row. */
  | { kind: "terminal"; path: string }
  /** Start a terminal in a fresh git worktree, on its own branch. */
  | { kind: "worktree"; path: string }
  /** Stop this session. The only place a terminal can be closed now that
   * they have no list of their own. */
  | { kind: "close-terminal"; id: string; title: string }
  // The dired verbs (docs/guarantees/authoring/the-tree-is-a-dired.md).
  // Each acts on `paths`: the marked rows when the row is marked, else the
  // row alone — the way `D` in dired acts on the marks if there are any.
  | { kind: "rename"; path: string }
  | { kind: "move"; paths: string[] }
  | { kind: "copy-to"; paths: string[] }
  | { kind: "delete"; paths: string[] }
  /** New file / new folder inside this directory (`""` for the root). */
  | { kind: "create"; dir: string }
  | { kind: "mkdir"; dir: string }
  // Ingest (docs/guarantees/authoring/a-file-is-ingested-from-the-tree.md).
  /** A new document beside the file, owning its bytes — "Make literate". */
  | { kind: "literate"; path: string }
  /** The file's bytes appended to an existing document as a block. */
  | { kind: "ingest"; path: string; into: string };

export interface TreeMenuItem {
  /** Stable across renders and platforms; what a test clicks by. */
  id: string;
  label: string;
  action: TreeMenuAction;
  /** A separator line is drawn above this item. */
  group?: boolean;
}

/** The file manager's name, with the generic fallback the server also uses. */
export function fileManagerName(folder: TreeFolder): string {
  return folder.fileManager?.trim() || "file manager";
}

/**
 * A node's absolute path, or `null` when the server did not say where the
 * folder is.
 *
 * The node path is always forward-slashed (it is a tree key, not an OS path),
 * so on Windows the separators have to be rewritten as well as joined —
 * copying `C:\notes/src/main.rs` into a terminal is the kind of almost-right
 * that wastes a minute.
 */
export function absolutePath(folder: TreeFolder, path: string): string | null {
  const root = folder.rootPath;
  if (!root) return null;
  const sep = folder.separator || "/";
  const trimmedRoot = root.length > 1 ? root.replace(/[/\\]+$/, "") : root;
  if (path === "") return trimmedRoot;
  const native = sep === "/" ? path : path.replace(/\//g, sep);
  return `${trimmedRoot}${trimmedRoot.endsWith(sep) ? "" : sep}${native}`;
}

/** The last segment of a root-relative path: the file or folder's own name. */
export function baseName(path: string): string {
  return path.split("/").filter(Boolean).pop() ?? path;
}

/**
 * The menu for one row.
 *
 * `path` is root-relative (`""` for the folder itself). `dir` says which of
 * the two "open" verbs makes sense: a directory has no default program of its
 * own worth naming, so it only offers the file manager.
 *
 * "Copy absolute path" is left out entirely when the server did not say where
 * the folder is, rather than offered and then quietly copying a relative path.
 */
/** What else the menu knows about the row: the marks, and the document a
 * file could be ingested into. */
export interface TreeMenuContext {
  /** Every marked row, root-relative. */
  marked?: ReadonlySet<string>;
  /** The focused document, if there is one, for "Ingest into …". */
  activeDoc?: { path: string; name: string };
  /** Whether this row is a plain text file some document does not already
   * write — the only kind that can be made literate or ingested. */
  plainText?: boolean;
}

/** The rows a verb acts on: the marks when this row is one of them, else
 * the row alone. */
export function subjects(path: string, marked?: ReadonlySet<string>): string[] {
  return marked && marked.has(path) ? [...marked] : [path];
}

export function treeMenuItems(
  folder: TreeFolder,
  path: string,
  dir: boolean,
  context: TreeMenuContext = {},
): TreeMenuItem[] {
  const items: TreeMenuItem[] = [];
  const many = subjects(path, context.marked);
  const count = many.length > 1 ? ` (${many.length} marked)` : "";
  if (path !== "" && !dir && context.plainText) {
    items.push({
      id: "literate",
      label: "Make literate — a new document owning this file",
      action: { kind: "literate", path },
    });
    if (context.activeDoc && context.activeDoc.path !== path) {
      items.push({
        id: "ingest",
        label: `Ingest into ${context.activeDoc.name}`,
        action: { kind: "ingest", path, into: context.activeDoc.path },
      });
    }
  }
  if (dir) {
    items.push({ id: "new-file", label: "New file…", action: { kind: "create", dir: path }, group: items.length > 0 });
    items.push({ id: "new-folder", label: "New folder…", action: { kind: "mkdir", dir: path } });
  }
  if (path !== "") {
    items.push({ id: "rename", label: "Rename…", action: { kind: "rename", path }, group: true });
    items.push({ id: "move", label: `Move to…${count}`, action: { kind: "move", paths: many } });
    items.push({ id: "copy-to", label: `Copy to…${count}`, action: { kind: "copy-to", paths: many } });
    items.push({ id: "delete", label: `Delete${count}`, action: { kind: "delete", paths: many } });
  }
  const absolute = absolutePath(folder, path);
  if (absolute) {
    items.push({
      id: "copy-absolute",
      label: "Copy absolute path",
      action: { kind: "copy", text: absolute },
      group: items.length > 0,
    });
  }
  if (path !== "") {
    items.push({
      id: "copy-relative",
      label: "Copy relative path",
      action: { kind: "copy", text: path },
    });
    items.push({
      id: "copy-name",
      label: dir ? "Copy folder name" : "Copy file name",
      action: { kind: "copy", text: baseName(path) },
    });
  }
  items.push({
    id: "reveal",
    label: `Reveal in ${fileManagerName(folder)}`,
    action: { kind: "reveal", path },
    group: true,
  });
  if (!dir) {
    items.push({
      id: "open-external",
      label: "Open in default program",
      action: { kind: "open", path },
    });
  }
  if (dir) {
    items.push({
      id: "new-terminal",
      label: "Open terminal here",
      action: { kind: "terminal", path },
      group: true,
    });
    items.push({
      id: "new-worktree",
      label: "New worktree here…",
      action: { kind: "worktree", path },
    });
  }
  return items;
}

/**
 * The menu for one terminal icon.
 *
 * Short on purpose. An icon is a small target and a long menu on one is a
 * menu nobody reads; the two verbs a running session has are "show me" and
 * "stop".
 */
export function terminalMenuItems(session: { id: string; title: string }): TreeMenuItem[] {
  return [
    {
      id: "close-terminal",
      label: `Close ${session.title}`,
      action: { kind: "close-terminal", id: session.id, title: session.title },
    },
  ];
}

/**
 * Put text on the clipboard.
 *
 * The async Clipboard API is what a modern browser and the desktop shell both
 * have; the textarea dance is for the case where it is absent or refused
 * (an insecure origin, a denied permission), because "copy path" silently
 * doing nothing is worse than a slightly ugly fallback.
 */
export async function copyText(text: string): Promise<void> {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return;
    }
  } catch {
    // Fall through to the textarea below.
  }
  const field = document.createElement("textarea");
  field.value = text;
  field.setAttribute("readonly", "");
  field.style.position = "fixed";
  field.style.opacity = "0";
  document.body.appendChild(field);
  field.select();
  try {
    document.execCommand("copy");
  } finally {
    field.remove();
  }
}
