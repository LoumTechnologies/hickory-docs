import { describe, expect, it } from "vitest";

import { absolutePath, baseName, fileManagerName, treeMenuItems,
  terminalMenuItems,
} from "./treeMenu";

const POSIX = { rootPath: "/home/me/notebook", separator: "/", fileManager: "Finder" };
const WINDOWS = { rootPath: "C:\\Users\\me\\notebook", separator: "\\", fileManager: "File Explorer" };

describe("absolutePath", () => {
  it("joins the root to a tree path", () => {
    expect(absolutePath(POSIX, "src/main.rs")).toBe("/home/me/notebook/src/main.rs");
    expect(absolutePath(POSIX, "")).toBe("/home/me/notebook");
  });

  it("rewrites separators on Windows, not just the join", () => {
    // `C:\Users\me\notebook/src/main.rs` is the almost-right answer: it
    // resolves, and it is wrong to paste anywhere a person will read it.
    expect(absolutePath(WINDOWS, "src/main.rs")).toBe("C:\\Users\\me\\notebook\\src\\main.rs");
  });

  it("does not double the separator when the root already ends in one", () => {
    expect(absolutePath({ ...POSIX, rootPath: "/home/me/notebook/" }, "a.txt")).toBe(
      "/home/me/notebook/a.txt",
    );
    expect(absolutePath({ ...WINDOWS, rootPath: "C:\\" }, "a.txt")).toBe("C:\\a.txt");
  });

  it("is null when the server did not say where the folder is", () => {
    expect(absolutePath({}, "a.txt")).toBeNull();
  });
});

describe("baseName", () => {
  it("is the last segment", () => {
    expect(baseName("src/deep/inner.hick")).toBe("inner.hick");
    expect(baseName("readme.txt")).toBe("readme.txt");
    expect(baseName("src/")).toBe("src");
  });
});

describe("terminalMenuItems", () => {
  it("offers the one verb a running session has that its own tab does not", () => {
    // Short on purpose: an icon is a small target, and a long menu on one is
    // a menu nobody reads.
    const items = terminalMenuItems({ id: "t1", title: "build" });
    expect(items.map((i) => i.id)).toEqual(["close-terminal"]);
    expect(items[0].label).toBe("Close build");
    expect(items[0].action).toEqual({ kind: "close-terminal", id: "t1", title: "build" });
  });
});

describe("treeMenuItems", () => {
  it("offers the three copies, the file manager, and the default program for a file", () => {
    const items = treeMenuItems(POSIX, "src/main.rs", false);
    expect(items.map((i) => i.id)).toEqual([
      // The dired verbs first: they are what a right-click on a row is for.
      "rename",
      "move",
      "copy-to",
      "delete",
      "copy-absolute",
      "copy-relative",
      "copy-name",
      "reveal",
      "open-external",
    ]);
    expect(items.map((i) => i.label)).toContain("Reveal in Finder");
    expect(items.find((i) => i.id === "copy-absolute")?.action).toEqual({
      kind: "copy",
      text: "/home/me/notebook/src/main.rs",
    });
    expect(items.find((i) => i.id === "copy-name")?.action).toEqual({
      kind: "copy",
      text: "main.rs",
    });
  });

  it("names the platform's file manager", () => {
    expect(treeMenuItems(WINDOWS, "a.txt", false).find((i) => i.id === "reveal")?.label).toBe(
      "Reveal in File Explorer",
    );
    expect(fileManagerName({})).toBe("file manager");
    expect(treeMenuItems({}, "a.txt", false).find((i) => i.id === "reveal")?.label).toBe(
      "Reveal in file manager",
    );
  });

  it("gives a directory no default-program item, and calls the name a folder", () => {
    const items = treeMenuItems(POSIX, "src", true);
    expect(items.map((i) => i.id)).toEqual([
      "new-file",
      "new-folder",
      "rename",
      "move",
      "copy-to",
      "delete",
      "copy-absolute",
      "copy-relative",
      "copy-name",
      "reveal",
      // Terminals live in the tree now: a directory is where you open one.
      "new-terminal",
      "new-worktree",
      "associate-github-issue",
    ]);
    expect(items.find((i) => i.id === "copy-name")?.label).toBe("Copy folder name");
  });

  it("gives the folder itself only what applies to it", () => {
    // No relative path and no name: the row IS the root, and "." is not
    // something anyone wants on their clipboard.
    const items = treeMenuItems(POSIX, "", true);
    expect(items.map((i) => i.id)).toEqual([
      // A new entry can go in the root; the root itself cannot be renamed,
      // moved or deleted from inside the app that is open on it.
      "new-file",
      "new-folder",
      "copy-absolute",
      "reveal",
      "new-terminal",
      "new-worktree",
      "associate-github-issue",
    ]);
  });

  it("leaves out the absolute path rather than copying a relative one", () => {
    // An older server, or the mock: offering "Copy absolute path" and then
    // quietly copying `src/main.rs` is the failure worth avoiding.
    expect(treeMenuItems({}, "src/main.rs", false).map((i) => i.id)).toEqual([
      "rename",
      "move",
      "copy-to",
      "delete",
      "copy-relative",
      "copy-name",
      "reveal",
      "open-external",
    ]);
  });
});

// docs/guarantees/authoring/the-tree-is-a-dired.md and
// docs/guarantees/authoring/a-file-is-ingested-from-the-tree.md
describe("the dired and ingest verbs", () => {
  it("offers Make literate and Ingest into the focused document for a plain text file only", () => {
    const active = { path: "notes/today.hick", name: "today.hick" };
    const plain = treeMenuItems(POSIX, "src/main.rs", false, { plainText: true, activeDoc: active });
    expect(plain.slice(0, 2).map((i) => i.id)).toEqual(["literate", "ingest"]);
    expect(plain[1].label).toBe("Ingest into today.hick");
    expect(plain[1].action).toEqual({ kind: "ingest", path: "src/main.rs", into: "notes/today.hick" });
    // No focused document: nothing to ingest into, but a new document is
    // always possible.
    expect(treeMenuItems(POSIX, "src/main.rs", false, { plainText: true }).map((i) => i.id)[0]).toBe("literate");
    // A generated file, a binary, a document: neither verb.
    expect(treeMenuItems(POSIX, "a.png", false, { activeDoc: active }).map((i) => i.id)).not.toContain("ingest");
    // A directory: neither.
    expect(treeMenuItems(POSIX, "src", true, { plainText: true, activeDoc: active }).map((i) => i.id)).not.toContain("literate");
  });

  it("acts on the marks when the row is marked, and says how many", () => {
    const marked = new Set(["a.md", "b.md", "c.md"]);
    const onMarked = treeMenuItems(POSIX, "a.md", false, { marked });
    expect(onMarked.find((i) => i.id === "delete")?.label).toBe("Delete (3 marked)");
    expect(onMarked.find((i) => i.id === "move")?.action).toEqual({ kind: "move", paths: ["a.md", "b.md", "c.md"] });
    expect(onMarked.find((i) => i.id === "rename")?.label).toBe("Rename… (3 marked)");
    expect(onMarked.find((i) => i.id === "rename")?.action).toEqual({ kind: "rename", paths: ["a.md", "b.md", "c.md"] });
    const offMarks = treeMenuItems(POSIX, "z.md", false, { marked });
    expect(offMarks.find((i) => i.id === "delete")?.label).toBe("Delete");
    expect(offMarks.find((i) => i.id === "delete")?.action).toEqual({ kind: "delete", paths: ["z.md"] });
  });
});
