import { describe, expect, it } from "vitest";
import { parseRoute } from "./router";

describe("routes", () => {
  it("routes the bare domain to the landing decision, never a chooser", () => {
    expect(parseRoute("")).toEqual({ name: "landing" });
    expect(parseRoute("#")).toEqual({ name: "landing" });
    expect(parseRoute("#/")).toEqual({ name: "landing" });
  });

  it("routes a document", () => {
    expect(parseRoute("#/docs/abc")).toEqual({ name: "doc", id: "abc" });
  });

  it("lands the retired documents-list route like any stale bookmark", () => {
    // The folder's files are a pane now (FolderTreePane), not a page.
    expect(parseRoute("#/documents")).toEqual({ name: "landing" });
  });

  it("routes the in-memory untitled document", () => {
    expect(parseRoute("#/new")).toEqual({ name: "new" });
  });

  it("routes settings (LLM API keys)", () => {
    expect(parseRoute("#/settings")).toEqual({ name: "settings" });
  });

  it("routes retired hosted-product paths to the landing decision", () => {
    // These routes no longer exist (local-only.md); a stale bookmark should
    // land somewhere useful rather than on a blank screen.
    expect(parseRoute("#/login")).toEqual({ name: "landing" });
    expect(parseRoute("#/verify?token=abc")).toEqual({ name: "landing" });
  });
});

describe("the lineage route", () => {
  it("is project-scoped, because a chain crosses documents", () => {
    expect(parseRoute("#/projects/abc123/lineage")).toEqual({ name: "lineage", id: "abc123" });
  });
});
