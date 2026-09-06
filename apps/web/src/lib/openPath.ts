// File > Open File… picked something INSIDE the current folder: the shell
// hands the page an absolute path, and the page finds which tree entry (and
// so which document) that is. The page never knows the folder's absolute
// root, so the match is by relative-path suffix — unique within one tree.

import type { FileNode } from "../api/types";

/** The tree node whose root-relative path the absolute path ends with.
 * Files only — a directory is not something to open in a pane, it is
 * something to REVEAL in the tree ([`dirForAbsolutePath`]). */
export function nodeForAbsolutePath(
  tree: readonly FileNode[],
  absolute: string,
): FileNode | null {
  const normalized = absolute.replace(/\\/g, "/");
  let best: FileNode | null = null;
  const visit = (nodes: readonly FileNode[]) => {
    for (const node of nodes) {
      if (!node.dir && normalized.endsWith(`/${node.path}`)) {
        // Longest suffix wins: "b/notes.hick" beats "notes.hick" when both
        // exist and the absolute path names the nested one.
        if (!best || node.path.length > best.path.length) best = node;
      }
      if (node.children) visit(node.children);
    }
  };
  visit(tree);
  return best;
}

/**
 * The DIRECTORY node whose root-relative path the absolute path ends with.
 *
 * The same longest-suffix rule as above, for the other half of "open that":
 * an architecture diagram's node usually stands for a folder, and the useful
 * answer to a double-click on it is the folder tree expanded to that folder
 * — not a pane, because a directory has no contents to show in one.
 */
export const REVEAL_PATH_EVENT = "hickory-reveal-path";

export function dirForAbsolutePath(
  tree: readonly FileNode[],
  absolute: string,
): FileNode | null {
  const normalized = absolute.replace(/\\/g, "/").replace(/\/+$/, "");
  let best: FileNode | null = null;
  const visit = (nodes: readonly FileNode[]) => {
    for (const node of nodes) {
      const path = node.path.replace(/\/+$/, "");
      if (node.dir && normalized.endsWith(`/${path}`)) {
        if (!best || path.length > best.path.length) best = node;
      }
      if (node.children) visit(node.children);
    }
  };
  visit(tree);
  return best;
}

/**
 * Ask the folder tree to show the directory this absolute path names, and say
 * whether there was one. The tree pane answers [`REVEAL_PATH_EVENT`] by
 * expanding to that directory, scrolling to it, and lighting the row.
 */
export function revealDirectory(
  tree: readonly FileNode[],
  absolute: string,
): boolean {
  const dir = dirForAbsolutePath(tree, absolute);
  if (!dir) return false;
  window.dispatchEvent(
    new CustomEvent<string>(REVEAL_PATH_EVENT, { detail: dir.path }),
  );
  return true;
}
