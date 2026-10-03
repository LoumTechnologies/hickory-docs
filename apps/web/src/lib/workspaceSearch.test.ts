import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { describe, expect, it, vi } from "vitest";
import { openPlainSearchFiles, registerSearchEditor, searchWorkspace } from "./workspaceSearch";

const hit = (path: string, snippet: string) => ({ path, snippet, start_line: 1, end_line: 1, score: 1 });

describe("searching open files and the folder", () => {
  it("searches every open file without a folder", async () => {
    const found = await searchWorkspace("invoice", 20, [
      { path: "/notes/one.md", content: "Introduction\nInvoice pending" },
      { path: "/elsewhere/two.txt", content: "Invoice paid" },
    ]);
    expect(found.hits.map((h) => h.path).sort()).toEqual(["/elsewhere/two.txt", "/notes/one.md"]);
    expect(found.hits.find((h) => h.path === "/notes/one.md")?.start_line).toBe(2);
  });

  it("includes folder hits and open files outside the folder", async () => {
    const folder = vi.fn().mockResolvedValue({ semantic: true, hits: [hit("closed.md", "invoice")] });
    const found = await searchWorkspace("invoice", 20, [{ path: "/outside/open.txt", content: "invoice" }], folder);
    expect(found.hits.map((h) => h.path)).toEqual(["/outside/open.txt", "closed.md"]);
    expect(found.semantic).toBe(true);
    expect(folder).toHaveBeenCalledWith("invoice", 50);
  });

  it("replaces disk hits with unsaved contents, even when the new contents no longer match", async () => {
    const folder = async () => ({ semantic: false, hits: [hit("open.md", "old invoice"), hit("closed.md", "invoice")] });
    const found = await searchWorkspace("invoice", 20, [{ path: "/project/open.md", content: "No bills here" }], folder);
    expect(found.hits.map((h) => h.path)).toEqual(["closed.md"]);
    const updated = await searchWorkspace("invoice", 20, [{ path: "/project/open.md", content: "New invoice" }], folder);
    expect(updated.hits.filter((h) => h.path.endsWith("open.md"))).toEqual([hit("/project/open.md", "New invoice")].map((h) => ({ ...h, score: expect.any(Number) })));
  });

  it("keeps an external file distinct from a folder file with the same name", async () => {
    const folder = async () => ({ semantic: false, hits: [hit("open.md", "folder invoice")] });
    const found = await searchWorkspace("invoice", 20, [
      { path: "/outside/open.md", content: "external invoice" },
    ], folder, "/project");
    expect(found.hits.map((h) => h.path)).toEqual(["/outside/open.md", "open.md"]);
  });

  it("reads live editor contents and drops closed editors", () => {
    const view = new EditorView({ state: EditorState.create({ doc: "saved" }) });
    const unregister = registerSearchEditor("open.txt", view);
    try {
      view.dispatch({ changes: { from: 0, to: 5, insert: "unsaved invoice" } });
      expect(openPlainSearchFiles()).toContainEqual({ path: "open.txt", content: "unsaved invoice" });
      unregister();
      expect(openPlainSearchFiles()).toEqual([]);
    } finally {
      unregister();
      view.destroy();
    }
  });

  it("deduplicates open paths and respects the result limit", async () => {
    const found = await searchWorkspace("invoice", 1, [
      { path: "/project/open.md", content: "invoice" },
      { path: "open.md", content: "invoice" },
    ]);
    expect(found.hits).toHaveLength(1);
  });
});
