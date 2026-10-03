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

  it("routes a deliberately folderless window", () => {
    expect(parseRoute("#/blank")).toEqual({ name: "blank" });
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

describe("the scratchpad route", () => {
  it("is its own route, so it can be linked and bookmarked", () => {
    expect(parseRoute("#/scratchpad")).toEqual({ name: "scratchpad" });
  });
});

// Protects docs/guarantees/authoring/new-document-is-an-act.md: asking for a
// new document works however many times, including when the address already
// says #/new — which is exactly when `navigate("/new")` alone did nothing.
describe("newDocument", () => {
  it("fires the event even when the hash is already #/new", async () => {
    const { NEW_DOCUMENT_EVENT, newDocument } = await import("./router");
    location.hash = "#/new";
    let fired = 0;
    const listener = () => fired++;
    window.addEventListener(NEW_DOCUMENT_EVENT, listener);
    newDocument();
    newDocument();
    window.removeEventListener(NEW_DOCUMENT_EVENT, listener);
    expect(fired).toBe(2);
    expect(location.hash).toBe("#/new");
  });
});


it("routes a selected file without opening a folder", () => {
  expect(parseRoute("#/file/%2Ftmp%2FMy%20note.md")).toEqual({ name: "new", file: "/tmp/My note.md" });
});
