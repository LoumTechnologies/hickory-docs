<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/openPath.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `12681ea0d5d2af01ad745badbed562006bcd90fb235215335d756be6cb5cbcca`.

<book:entry path="apps/web/src/lib/openPath.test.ts">
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
</book:entry>

</book:doc>
