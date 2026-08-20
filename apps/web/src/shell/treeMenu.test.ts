import { describe, expect, it } from "vitest";

import { absolutePath, baseName, fileManagerName, treeMenuItems } from "./treeMenu";

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

describe("treeMenuItems", () => {
  it("offers the three copies, the file manager, and the default program for a file", () => {
    const items = treeMenuItems(POSIX, "src/main.rs", false);
    expect(items.map((i) => i.id)).toEqual([
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
    expect(items.map((i) => i.id)).toEqual(["copy-absolute", "copy-relative", "copy-name", "reveal"]);
    expect(items.find((i) => i.id === "copy-name")?.label).toBe("Copy folder name");
  });

  it("gives the folder itself only what applies to it", () => {
    // No relative path and no name: the row IS the root, and "." is not
    // something anyone wants on their clipboard.
    const items = treeMenuItems(POSIX, "", true);
    expect(items.map((i) => i.id)).toEqual(["copy-absolute", "reveal"]);
  });

  it("leaves out the absolute path rather than copying a relative one", () => {
    // An older server, or the mock: offering "Copy absolute path" and then
    // quietly copying `src/main.rs` is the failure worth avoiding.
    expect(treeMenuItems({}, "src/main.rs", false).map((i) => i.id)).toEqual([
      "copy-relative",
      "copy-name",
      "reveal",
      "open-external",
    ]);
  });
});
