import { describe, expect, it } from "vitest";
import { untitledDraftKey, untitledPath, untitledSaveName, wrapUntitled } from "./newDoc";

describe("naming the untitled document", () => {
  // Guarantee: docs/guarantees/authoring/new-document-is-an-act.md
  it("suggests the first heading as the Save-dialog filename", () => {
    expect(untitledSaveName("# Meeting notes\n\nDetails", [])).toBe("Meeting notes.md");
    expect(untitledSaveName("before\n## Plan #\n", [])).toBe("Plan.md");
  });

  it("falls back to an untitled name when there is no heading", () => {
    expect(untitledSaveName("just a thought", ["untitled.md"])).toBe("untitled-2.md");
  });
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
