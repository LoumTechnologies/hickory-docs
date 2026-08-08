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
