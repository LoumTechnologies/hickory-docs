// The tree's file verbs, as the server takes them (`POST /api/files/op`).
// See docs/guarantees/authoring/the-tree-is-a-dired.md.

/** One dired verb on the tree. */
export interface FileOpRequest {
  op: "rename" | "move" | "copy" | "delete" | "mkdir" | "create";
  /** Root-relative path the verb acts on. */
  path: string;
  /** rename: the new name; move/copy: the destination directory. */
  to?: string;
}
