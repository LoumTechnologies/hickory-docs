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
  | { kind: "open"; path: string };

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
export function treeMenuItems(
  folder: TreeFolder,
  path: string,
  dir: boolean,
): TreeMenuItem[] {
  const items: TreeMenuItem[] = [];
  const absolute = absolutePath(folder, path);
  if (absolute) {
    items.push({ id: "copy-absolute", label: "Copy absolute path", action: { kind: "copy", text: absolute } });
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
  return items;
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
