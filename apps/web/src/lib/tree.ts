// Group flat `a/b/c.rs`-style paths into a directory tree. Shared by
// OutputTree (generated files) and ProjectDocTree (project documents).

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
