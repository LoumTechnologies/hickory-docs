<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/api/files.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Filesystem Acts Keep Dired's Safety](../../../../../contract.md)

Captured SHA-256: `22fb511086a632dd3f2936756ace4a211f340cc86cd816c1913bf749ff37b6b3`.

<book:entry path="apps/web/src/api/files.ts">
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
</book:entry>

</book:doc>
