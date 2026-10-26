<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/openPath.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-491aa149d07d"></a>
## `apps/web/src/lib/openPath.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-491aa149d07d-nodeForAbsolutePath"></a>
### `nodeForAbsolutePath`

File > Open File… picked something INSIDE the current folder: the shell
hands the page an absolute path, and the page finds which tree entry (and
so which document) that is. The page never knows the folder's absolute
root, so the match is by relative-path suffix — unique within one tree.

<storycode:copy id="f-491aa149d07d-nodeForAbsolutePath">
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
        // Longest suffix wins: "b/notes.md" beats "notes.md" when both
        // exist and the absolute path names the nested one.
        if (!best || node.path.length > best.path.length) best = node;
      }
      if (node.children) visit(node.children);
    }
  };
  visit(tree);
  return best;
}
</storycode:copy>

### Assemble `apps/web/src/lib/openPath.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/openPath.ts"><storycode:paste select="#f-491aa149d07d-nodeForAbsolutePath"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-ff605c3f2c75"></a>
## `apps/web/src/lib/openPath.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-ff605c3f2c75-tree"></a>
### `tree`

<storycode:copy id="f-ff605c3f2c75-tree">
import { describe, expect, it } from "vitest";

import { nodeForAbsolutePath } from "./openPath";
import type { FileNode } from "../api/types";

const tree: FileNode[] = [
  {
    name: "b",
    path: "b",
    dir: true,
    children: [{ name: "notes.md", path: "b/notes.md", dir: false, doc_id: "d2" }],
  },
  { name: "notes.md", path: "notes.md", dir: false, doc_id: "d1" },
  { name: "orders.py", path: "orders.py", dir: false },
];

</storycode:copy>
<a id="f-ff605c3f2c75-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-ff605c3f2c75-expression_statement">
describe("nodeForAbsolutePath", () => {
  it("matches by relative-path suffix", () => {
    expect(nodeForAbsolutePath(tree, "/home/me/proj/notes.md")?.doc_id).toBe("d1");
  });

  it("prefers the longest (most specific) suffix", () => {
    expect(nodeForAbsolutePath(tree, "/home/me/proj/b/notes.md")?.doc_id).toBe("d2");
  });

  it("finds non-document files too", () => {
    expect(nodeForAbsolutePath(tree, "/home/me/proj/orders.py")?.path).toBe("orders.py");
  });

  it("returns null for a path outside the tree", () => {
    expect(nodeForAbsolutePath(tree, "/etc/passwd")).toBeNull();
  });

  it("normalizes Windows separators", () => {
    expect(nodeForAbsolutePath(tree, "C:\\proj\\b\\notes.md")?.doc_id).toBe("d2");
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/openPath.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/openPath.test.ts"><storycode:paste select="#f-ff605c3f2c75-tree"/><storycode:paste select="#f-ff605c3f2c75-expression_statement"/></storycode:file>


</storycode:doc>
