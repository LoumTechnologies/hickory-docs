import { describe, expect, it } from "vitest";

import { windowTitle } from "./windowTitle";

describe("window title precedence", () => {
  it("uses the custom override verbatim when one is set", () => {
    expect(
      windowTitle({ custom: "My Lab Notebook", folder: "notes", file: "a.hick" }),
    ).toBe("My Lab Notebook");
  });

  it("falls back to the project folder name when there is no override", () => {
    expect(windowTitle({ custom: null, folder: "notes", file: "a.hick" })).toBe(
      "notes — Hickory Docs",
    );
  });

  it("reduces a folder path to its last segment", () => {
    expect(windowTitle({ folder: "/home/mock/notebook", file: "a.hick" })).toBe(
      "notebook — Hickory Docs",
    );
    expect(windowTitle({ folder: "C:\\Users\\me\\notes" })).toBe("notes — Hickory Docs");
  });

  it("falls back to the focused file name when there is no folder either", () => {
    expect(windowTitle({ custom: "", folder: null, file: "weave-demo.hick" })).toBe(
      "weave-demo.hick — Hickory Docs",
    );
  });

  it("treats a blank override as unset, not as an empty title", () => {
    expect(windowTitle({ custom: "   ", folder: "notes" })).toBe("notes — Hickory Docs");
  });

  it("is just the app when nothing at all is known", () => {
    expect(windowTitle({})).toBe("Hickory Docs");
  });
});
