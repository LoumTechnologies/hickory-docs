<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/tree.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2189a1d0541c"></a>
## `apps/web/src/lib/tree.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-2189a1d0541c-DirNode"></a>
### `DirNode`

Group flat `a/b/c.rs`-style paths into a directory tree. Shared by
OutputTree (generated files).

<storycode:copy id="f-2189a1d0541c-DirNode">
// Group flat `a/b/c.rs`-style paths into a directory tree. Shared by
// OutputTree (generated files).

export interface DirNode {
  name: string;
  dirs: Map<string, DirNode>;
  files: { name: string; path: string }[];
}

</storycode:copy>
<a id="f-2189a1d0541c-emptyDir"></a>
### `emptyDir`

<storycode:copy id="f-2189a1d0541c-emptyDir">
function emptyDir(name: string): DirNode {
  return { name, dirs: new Map(), files: [] };
}

</storycode:copy>
<a id="f-2189a1d0541c-buildTree"></a>
### `buildTree`

Group `a/b/c.rs` paths into a directory tree, directories before files.

<storycode:copy id="f-2189a1d0541c-buildTree">
/** Group `a/b/c.rs` paths into a directory tree, directories before files. */
export function buildTree(paths: string[]): DirNode {
  const root = emptyDir("");
  for (const path of paths) {
    const parts = path.split("/").filter(Boolean);
    if (parts.length === 0) continue;
    let node = root;
    for (const dir of parts.slice(0, -1)) {
      let next = node.dirs.get(dir);
      if (!next) {
        next = emptyDir(dir);
        node.dirs.set(dir, next);
      }
      node = next;
    }
    node.files.push({ name: parts[parts.length - 1], path });
  }
  return root;
}
</storycode:copy>

### Assemble `apps/web/src/lib/tree.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/tree.ts"><storycode:paste select="#f-2189a1d0541c-DirNode"/><storycode:paste select="#f-2189a1d0541c-emptyDir"/><storycode:paste select="#f-2189a1d0541c-buildTree"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-b48d9c400c66"></a>
## `apps/web/src/lib/tree.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b48d9c400c66-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b48d9c400c66-expression_statement">
import { describe, expect, it } from "vitest";
import { buildTree } from "./tree";

describe("buildTree", () => {
  it("groups paths into directories, keeping full paths on the leaves", () => {
    const root = buildTree(["src/latency.py", "src/util/io.py", "README.md"]);
    expect(root.files.map((f) => f.name)).toEqual(["README.md"]);
    const src = root.dirs.get("src")!;
    expect(src.files.map((f) => f.path)).toEqual(["src/latency.py"]);
    expect(src.dirs.get("util")!.files[0]).toEqual({ name: "io.py", path: "src/util/io.py" });
  });

  it("ignores empty and leading-slash segments rather than making blank nodes", () => {
    const root = buildTree(["/gen.rs", ""]);
    expect(root.dirs.size).toBe(0);
    expect(root.files.map((f) => f.path)).toEqual(["/gen.rs"]);
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/tree.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/tree.test.ts"><storycode:paste select="#f-b48d9c400c66-expression_statement"/></storycode:file>


</storycode:doc>
