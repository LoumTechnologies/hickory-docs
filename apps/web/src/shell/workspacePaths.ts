import type { FileNode } from "../api/types";

/** The tree node for a root-relative path, across every open root. */
export function findNodeByPath(
  roots: readonly { tree: FileNode[] }[],
  path: string,
): FileNode | null {
  const walk = (nodes: readonly FileNode[]): FileNode | null => {
    for (const node of nodes) {
      if (node.path === path) return node;
      const found = node.children ? walk(node.children) : null;
      if (found) return found;
    }
    return null;
  };
  for (const root of roots) {
    const found = walk(root.tree);
    if (found) return found;
  }
  return null;
}

/** `dir` (ending in "/" or empty) joined with a relative `path`, with `./`
 * and `../` folded — the path the tree knows a file by. */
export function joinRel(dir: string, path: string): string {
  if (path.startsWith("/")) return path;
  const parts: string[] = [];
  for (const seg of `${dir}${path}`.split("/")) {
    if (seg === "" || seg === ".") continue;
    if (seg === "..") parts.pop();
    else parts.push(seg);
  }
  return parts.join("/");
}
