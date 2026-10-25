<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/workspacePaths.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `4fc57f23417922a59173ba9ff93a3679a054ec784b3c6da0ed0ef7ba57e2ecc2`.

<book:entry path="apps/web/src/shell/workspacePaths.ts">
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
</book:entry>

</book:doc>
