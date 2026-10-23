import type { FileNode } from "../api/types";

/** Collect searchable files across every workspace folder, in tree order. */
export function workspaceTreeFiles(roots: readonly { tree: readonly FileNode[] }[]): FileNode[] {
  const files: FileNode[] = [];
  const walk = (nodes: readonly FileNode[]) => {
    for (const node of nodes) {
      if (node.dir) {
        if (node.children) walk(node.children);
      } else {
        files.push(node);
      }
    }
  };
  for (const root of roots) walk(root.tree);
  return files;
}
