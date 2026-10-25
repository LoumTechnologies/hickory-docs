<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/workspaceTreeFiles.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ef54efe69ea95945558c76ed393ddffa92328468efd718529000e666a61581a7`.

<book:entry path="apps/web/src/views/workspaceTreeFiles.ts">
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
</book:entry>

</book:doc>
