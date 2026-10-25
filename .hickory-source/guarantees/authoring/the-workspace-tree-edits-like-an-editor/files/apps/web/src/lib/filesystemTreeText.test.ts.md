<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/filesystemTreeText.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Filesystem Acts Keep Dired's Safety](../../../../../../the-tree-is-a-dired/contract.md)
- [The Workspace Tree Is An Editor Buffer](../../../../../contract.md)

Captured SHA-256: `f8dc6af4a90b8bc7454b5adda9aa278ae27873f3af140bd803c4c8e1d1fa326d`.

<book:entry path="apps/web/src/lib/filesystemTreeText.test.ts">
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
    expect(parseFilesystemTreeText("folder\n  child.txt", filesystemTextEntries([
      { name: "one.txt", path: "one.txt", dir: false },
      { name: "two.txt", path: "two.txt", dir: false },
    ])).error).toContain("Folder lines must end in /");
    expect(parseFilesystemTreeText("one.txt\n\n", filesystemTextEntries([
      { name: "one.txt", path: "one.txt", dir: false },
      { name: "two.txt", path: "two.txt", dir: false },
    ])).error).toContain("Line 2 is blank");
  });

  it("ignores one terminal newline rather than treating it as a blank file", () => {
    const original = filesystemTextEntries(nodes);
    expect(reconcileFilesystemTreeText("src/\n  one.ts\nnotes.txt\n", original).reconciliation)
      .toEqual({ renames: [], creates: [], deletes: [] });
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

  it("creates a folder and moves an unchanged uniquely named file into it in one apply", () => {
    const original = filesystemTextEntries([
      { name: "test53.py", path: "test53.py", dir: false },
    ]);
    const result = reconcileFilesystemTreeText("test 1/\n  test53.py", original);
    expect(result.reconciliation).toEqual({
      renames: [{ path: "test53.py", to: "test 1/test53.py" }],
      creates: [{ path: "test 1", dir: true }],
      deletes: [],
    });
  });

  it("collapses a removed folder subtree into one reviewed delete", () => {
    const result = reconcileFilesystemTreeText("notes.txt", filesystemTextEntries(nodes));
    expect(result.reconciliation?.deletes).toEqual(["src"]);
  });

  it("refuses a new entry combined with a renamed existing one", () => {
    const result = reconcileFilesystemTreeText("source/\n  one.ts\n  new.ts\nnotes.txt", filesystemTextEntries(nodes));
    expect(result.error).toContain("Cannot create entries and rename existing ones in the same Apply");
  });
});
</book:entry>

</book:doc>
