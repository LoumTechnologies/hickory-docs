// File > Open File… picked something INSIDE the current folder: the shell
// hands the page an absolute path, and the page finds which tree entry (and
// so which document) that is. The page never knows the folder's absolute
// root, so the match is by relative-path suffix — unique within one tree.

import type { FileNode } from "../api/types";

/** The tree node whose root-relative path the absolute path ends with. */
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
