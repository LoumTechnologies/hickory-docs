<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/workspaceTreeFiles.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a643937ccda2"></a>
## `apps/web/src/views/workspaceTreeFiles.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-a643937ccda2-workspaceTreeFiles"></a>
### `workspaceTreeFiles`

<storycode:copy id="f-a643937ccda2-workspaceTreeFiles">
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
</storycode:copy>

### Assemble `apps/web/src/views/workspaceTreeFiles.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/workspaceTreeFiles.ts"><storycode:paste select="#f-a643937ccda2-workspaceTreeFiles"/></storycode:file>

</storycode:doc>
