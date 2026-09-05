// docs/guarantees/authoring/the-tree-is-a-dired.md
import { describe, expect, it } from "vitest";

import { containing, diredIntent, nextMarks, targets, toggleMark } from "./dired";

const none: ReadonlySet<string> = new Set();
const marks: ReadonlySet<string> = new Set(["a.md", "b.md"]);

describe("the dired keys", () => {
  it("mark, unmark and unmark-all are m, u and U", () => {
    expect(diredIntent({ key: "m" }, "x.md", false, none)).toEqual({ kind: "mark", path: "x.md" });
    expect(diredIntent({ key: "u" }, "x.md", false, marks)).toEqual({ kind: "unmark", path: "x.md" });
    expect(diredIntent({ key: "U" }, "", true, marks)).toEqual({ kind: "unmark-all" });
  });

  it("D, C and M act on the marks when the row is marked, else on the row", () => {
    expect(diredIntent({ key: "D" }, "a.md", false, marks)).toEqual({ kind: "delete", paths: ["a.md", "b.md"] });
    expect(diredIntent({ key: "D" }, "c.md", false, marks)).toEqual({ kind: "delete", paths: ["c.md"] });
    expect(diredIntent({ key: "Delete" }, "c.md", false, none)).toEqual({ kind: "delete", paths: ["c.md"] });
    expect(diredIntent({ key: "C" }, "a.md", false, marks)).toEqual({ kind: "copy", paths: ["a.md", "b.md"] });
    expect(diredIntent({ key: "M" }, "c.md", false, none)).toEqual({ kind: "move", paths: ["c.md"] });
  });

  it("R renames the row, never the marks: a rename is one name", () => {
    expect(diredIntent({ key: "R" }, "a.md", false, marks)).toEqual({ kind: "rename", path: "a.md" });
  });

  it("+ and n create in the row's directory, or beside a file", () => {
    expect(diredIntent({ key: "+" }, "src", true, none)).toEqual({ kind: "mkdir", dir: "src" });
    expect(diredIntent({ key: "n" }, "src/a.rs", false, none)).toEqual({ kind: "create", dir: "src" });
    expect(diredIntent({ key: "+" }, "", true, none)).toEqual({ kind: "mkdir", dir: "" });
    expect(containing("a.md", false)).toBe("");
  });

  it("is silent with a modifier held, on other keys, and on the header with nothing marked", () => {
    expect(diredIntent({ key: "m", ctrlKey: true }, "x.md", false, none)).toBeNull();
    expect(diredIntent({ key: "j" }, "x.md", false, none)).toBeNull();
    expect(diredIntent({ key: "D" }, "", true, none)).toBeNull();
    expect(diredIntent({ key: "m" }, "", true, none)).toBeNull();
  });
});

describe("the marks", () => {
  it("follow mark, unmark and unmark-all, and leave other intents alone", () => {
    const after = nextMarks(none, { kind: "mark", path: "a.md" });
    expect([...after]).toEqual(["a.md"]);
    expect([...nextMarks(after, { kind: "unmark", path: "a.md" })]).toEqual([]);
    expect([...nextMarks(marks, { kind: "unmark-all" })]).toEqual([]);
    expect([...nextMarks(marks, { kind: "delete", paths: ["a.md"] })]).toEqual([...marks]);
  });

  it("toggle with Ctrl+click, and targets pick the marks only when the row is one", () => {
    expect([...toggleMark(none, "a.md")]).toEqual(["a.md"]);
    expect([...toggleMark(marks, "a.md")]).toEqual(["b.md"]);
    expect(targets("a.md", marks)).toEqual(["a.md", "b.md"]);
    expect(targets("c.md", marks)).toEqual(["c.md"]);
  });
});
