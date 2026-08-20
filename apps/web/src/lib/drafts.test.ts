import { describe, expect, it } from "vitest";
import { draftDisposition } from "./drafts";

describe("what to do with a draft when the app reopens", () => {
  it("restores quietly when nobody else touched the file", () => {
    // The common case by a wide margin, and it must be silent: a dialog here
    // would train people to dismiss dialogs.
    expect(
      draftDisposition({ contents: "half a sentence", base: "# Notes\n" }, "# Notes\n"),
    ).toEqual({ kind: "restore", contents: "half a sentence" });
  });

  it("calls a draft clean when the file already says the same thing", () => {
    // Somebody saved the same text from elsewhere. Not a conflict however far
    // the file has moved from the base — there is nothing left to merge.
    expect(draftDisposition({ contents: "same", base: "old" }, "same")).toEqual({
      kind: "clean",
    });
  });

  it("asks for a merge when the file moved on and the buffer did too", () => {
    expect(
      draftDisposition({ contents: "ours", base: "original" }, "theirs"),
    ).toEqual({ kind: "merge", base: "original", ours: "ours", theirs: "theirs" });
  });

  it("still knows the base when the file was deleted under us", () => {
    // An empty file on disk is a change like any other, and the reader gets to
    // decide whether their unsaved text wins.
    expect(draftDisposition({ contents: "work", base: "was here" }, "")).toEqual({
      kind: "merge",
      base: "was here",
      ours: "work",
      theirs: "",
    });
  });

  it("treats a draft with no base as a merge against whatever is there", () => {
    // A buffer that never had a file behind it, and now a file exists at that
    // path. Two-way, and the UI says so.
    expect(draftDisposition({ contents: "typed", base: "" }, "someone else's")).toEqual({
      kind: "merge",
      base: "",
      ours: "typed",
      theirs: "someone else's",
    });
  });
});
