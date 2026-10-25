<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/tree.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `e183453cce8f857a9d02566b2a58767ce45ecc078652dbd0d55d1eed864eae2a`.

<book:entry path="apps/web/src/lib/tree.test.ts">
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
</book:entry>

</book:doc>
