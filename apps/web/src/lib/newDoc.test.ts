import { describe, expect, it } from "vitest";
import { landingTarget, untitledDraftKey, untitledPath, wrapUntitled } from "./newDoc";

const doc = (id: string, updated_at: string) => ({
  id,
  path: `${id}.hick`,
  updated_at,
});

describe("where the app lands", () => {
  it("lands on a fresh untitled buffer when the folder is empty", () => {
    expect(landingTarget([])).toEqual({ kind: "new" });
  });

  it("opens the most recently updated document", () => {
    expect(
      landingTarget([
        doc("a", "2026-01-01T00:00:00Z"),
        doc("b", "2026-03-01T00:00:00Z"),
        doc("c", "2026-02-01T00:00:00Z"),
      ]),
    ).toEqual({ kind: "doc", id: "b" });
  });

  it("never lands on a chooser for a single document", () => {
    expect(landingTarget([doc("only", "2026-01-01T00:00:00Z")])).toEqual({
      kind: "doc",
      id: "only",
    });
  });

  it("tolerates an unparseable timestamp", () => {
    // A bad stamp must not decide the landing by throwing or by winning.
    expect(
      landingTarget([doc("bad", "not a date"), doc("good", "2026-01-01T00:00:00Z")]),
    ).toEqual({ kind: "doc", id: "good" });
  });
});

describe("naming the untitled document", () => {
  it("starts at untitled.md in an empty folder", () => {
    expect(untitledPath([])).toBe("untitled.md");
  });

  it("counts past taken names", () => {
    expect(untitledPath(["untitled.md"])).toBe("untitled-2.md");
    expect(untitledPath(["untitled.md", "untitled-2.md"])).toBe(
      "untitled-3.md",
    );
  });

  it("ignores unrelated documents", () => {
    expect(untitledPath(["report.md", "untitled-2.md"])).toBe(
      "untitled.md",
    );
  });

  it("judges taken-ness on the final path segment", () => {
    // Two files a person can only tell apart by directory is worse than
    // skipping a number.
    expect(untitledPath(["notes/untitled.md"])).toBe("untitled-2.md");
    expect(untitledPath(["notes\\untitled.md"])).toBe("untitled-2.md");
  });
});

describe("the unsaved buffer", () => {
  it("uses an opaque workspace key, not a path in the project", () => {
    expect(untitledDraftKey("tab-42")).toBe("untitled:tab-42");
  });
});

describe("the created document's source", () => {
  // docs/specs/freeform/bare-documents.md — the root element is optional, and
  // the new-note path is the exact case that spec was adopted for.
  it("is the typed prose and nothing else — no wrapper", () => {
    expect(wrapUntitled("# Hello")).toBe("# Hello\n");
  });

  it("does not double a newline the prose already ends with", () => {
    expect(wrapUntitled("# Hello\n")).toBe("# Hello\n");
  });

  it("creates an empty document without inventing content", () => {
    expect(wrapUntitled("")).toBe("\n");
  });
});
