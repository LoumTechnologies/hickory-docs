<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/workspacePaths.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d566a235cf7e"></a>
## `apps/web/src/shell/workspacePaths.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-d566a235cf7e-findNodeByPath"></a>
### `findNodeByPath`

<storycode:copy id="f-d566a235cf7e-findNodeByPath">
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

</storycode:copy>
<a id="f-d566a235cf7e-joinRel"></a>
### `joinRel`

`dir` (ending in "/" or empty) joined with a relative `path`, with `./`
and `../` folded — the path the tree knows a file by.

<storycode:copy id="f-d566a235cf7e-joinRel">
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
</storycode:copy>

### Assemble `apps/web/src/shell/workspacePaths.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/workspacePaths.ts"><storycode:paste select="#f-d566a235cf7e-findNodeByPath"/><storycode:paste select="#f-d566a235cf7e-joinRel"/></storycode:file>

</storycode:doc>
