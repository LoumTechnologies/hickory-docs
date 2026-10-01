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
