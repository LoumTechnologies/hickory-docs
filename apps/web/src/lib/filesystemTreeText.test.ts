import { describe, expect, it } from "vitest";

import {
  filesystemTextEntries,
  filesystemTreeOperations,
  filesystemTreeText,
  parseFilesystemTreeText,
  reconcileFilesystemTreeText,
} from "./filesystemTreeText";

// Guarantee: docs/guarantees/authoring/the-workspace-tree-edits-like-an-editor.md
describe("the filesystem as significant-whitespace text", () => {
  const nodes = [{ name: "src", path: "src", dir: true, children: [
    { name: "one.ts", path: "src/one.ts", dir: false },
  ] }, { name: "notes.txt", path: "notes.txt", dir: false }];

  it("renders hierarchy as ordinary indented lines", () => {
    expect(filesystemTreeText(filesystemTextEntries(nodes))).toBe("src/\n  one.ts\nnotes.txt");
  });

  it("turns names and indentation into filesystem destinations", () => {
    const original = filesystemTextEntries(nodes);
    const parsed = parseFilesystemTreeText("source/\n  first.ts\n  notes.txt", original);
    expect(filesystemTreeOperations(parsed.edits!)).toEqual([
      { path: "src", to: "source" },
      { path: "source/one.ts", to: "source/first.ts" },
      { path: "notes.txt", to: "source/notes.txt" },
    ]);
  });

  it("treats reordered unchanged entries as presentation, not filesystem edits", () => {
    const result = reconcileFilesystemTreeText(
      "notes.txt\nsrc/\n  one.ts",
      filesystemTextEntries(nodes),
    );
    expect(result.reconciliation).toEqual({ renames: [], creates: [], deletes: [] });
  });

  it("matches unchanged reordered entries before finding the one real rename", () => {
    const result = reconcileFilesystemTreeText(
      "notes.txt\nsrc/\n  first.ts",
      filesystemTextEntries(nodes),
    );
    expect(result.reconciliation?.renames).toEqual([
      { path: "src/one.ts", to: "src/first.ts" },
    ]);
  });

  it("keeps reordered row identity through a column-style prefix edit", () => {
    const original = filesystemTextEntries([
      { name: "one.txt", path: "one.txt", dir: false },
      { name: "two.txt", path: "two.txt", dir: false },
    ]);
    const result = reconcileFilesystemTreeText("renamed-two.txt\nrenamed-one.txt", original);
    expect(result.reconciliation?.renames).toEqual([
      { path: "two.txt", to: "renamed-two.txt" },
      { path: "one.txt", to: "renamed-one.txt" },
    ]);
  });

  it("refuses malformed or type-changing structure", () => {
    const original = filesystemTextEntries(nodes);
    expect(parseFilesystemTreeText("src/\n one.ts\nnotes.txt", original).error).toContain("two spaces");
    expect(parseFilesystemTreeText("src\n  one.ts\nnotes.txt", original).error).toContain("trailing slash");
  });

  it("turns inserted lines into parent-first creates", () => {
    const result = reconcileFilesystemTreeText(
      "src/\n  one.ts\n  fixtures/\n    sample.json\nnotes.txt",
      filesystemTextEntries(nodes),
    );
    expect(result.reconciliation?.creates).toEqual([
      { path: "src/fixtures", dir: true },
      { path: "src/fixtures/sample.json", dir: false },
    ]);
  });

  it("collapses a removed folder subtree into one reviewed delete", () => {
    const result = reconcileFilesystemTreeText("notes.txt", filesystemTextEntries(nodes));
    expect(result.reconciliation?.deletes).toEqual(["src"]);
  });

  it("refuses a create combined with a rename because line identity is ambiguous", () => {
    const result = reconcileFilesystemTreeText("source/\n  one.ts\n  new.ts\nnotes.txt", filesystemTextEntries(nodes));
    expect(result.error).toContain("separate save");
  });
});
