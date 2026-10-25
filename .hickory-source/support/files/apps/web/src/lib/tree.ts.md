<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/tree.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `0bd1aea913d1156de0f6840a097c76f7cae15da67da4e9c2278ae70b3d9acdb8`.

<book:entry path="apps/web/src/lib/tree.ts">
// Group flat `a/b/c.rs`-style paths into a directory tree. Shared by
// OutputTree (generated files).

export interface DirNode {
  name: string;
  dirs: Map<string, DirNode>;
  files: { name: string; path: string }[];
}

function emptyDir(name: string): DirNode {
  return { name, dirs: new Map(), files: [] };
}

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
</book:entry>

</book:doc>
