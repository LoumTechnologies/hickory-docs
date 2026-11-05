import { afterEach, describe, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

import { assetUrl, followLink, linkAt, linksIn, mdLinks } from "./mdLinks";
import { installMockHandler } from "../api/client";

function editorWith(doc: string, options: Parameters<typeof mdLinks>[0]) {
  return new EditorView({
    state: EditorState.create({ doc, extensions: [mdLinks(options)] }),
    parent: document.body,
  });
}

const inNotes = { docPath: () => "notes/a.md" };

afterEach(() => {
  document.body.innerHTML = "";
});

describe("what counts as a link", () => {
  // Contract: .hickory-source/guarantees/authoring/a-tree-row-opens-in-the-platform/contract.md
  it("recognizes agent @file includes without making code or mentions navigable", () => {
    const text = '@AGENTS.md\n@.instructions/just.md\nRead @"notes/my plan.md"\n@include notes/other.md\nmail@example.com @someone `@literal.md`\n```md\n@fenced.md\n```\n[an @label.md](target.md)';
    const state = EditorState.create({ doc: text });
    expect(linksIn(state, inNotes).map(({ link }) => link.target)).toEqual([
      'AGENTS.md', '.instructions/just.md', 'notes/my plan.md', 'notes/other.md', 'target.md',
    ]);
    expect(linkAt(state, inNotes, text.indexOf('@.instructions') + 3)?.target).toBe('.instructions/just.md');
    expect(linksIn(state, { ...inNotes, skip: () => [[0, text.length]] })).toEqual([]);
    const view = editorWith('@AGENTS.md', inNotes);
    expect(view.contentDOM.querySelector('.cm-md-link')?.textContent).toBe('@AGENTS.md');
    view.destroy();
  });
  it("skips the ranges the caller says are not prose", () => {
    const doc = "prose [a](a.md)\ncode [b](b.md)";
    const state = EditorState.create({ doc });
    const all = linksIn(state, inNotes);
    expect(all.map((l) => l.link.target)).toEqual(["a.md", "b.md"]);
    const skipped = linksIn(state, { ...inNotes, skip: () => [[18, doc.length]] });
    expect(skipped.map((l) => l.link.target)).toEqual(["a.md"]);
  });

  it("finds the link under a position, brackets included", () => {
    const state = EditorState.create({ doc: "see [the plan](plan.md) now" });
    expect(linkAt(state, inNotes, 6)?.target).toBe("plan.md");
    expect(linkAt(state, inNotes, 27)).toBeNull();
  });
});

describe("how a link looks", () => {
  it("shows only the label until the caret enters that link", () => {
    const view = editorWith("see [the plan](plan.md)", inNotes);
    expect(view.state.doc.toString()).toBe("see [the plan](plan.md)");
    expect(view.contentDOM.querySelector(".cm-md-link")?.textContent).toBe("the plan");
    expect(view.contentDOM.textContent).toBe("see the plan");
    view.dispatch({ selection: { anchor: 8 } });
    expect(view.contentDOM.textContent).toBe("see [the plan](plan.md)");
    expect(view.contentDOM.querySelectorAll(".cm-md-link-mark").length).toBe(2);
    view.dispatch({ selection: { anchor: 0 } });
    expect(view.contentDOM.textContent).toBe("see the plan");
    view.destroy();
  });

  it("draws an image under its line, through the asset route", () => {
    const view = editorWith("![a chart](assets/c.png)", { ...inNotes, images: true });
    const img = view.contentDOM.querySelector<HTMLImageElement>(".cm-md-image img");
    expect(img?.getAttribute("src")).toBe(assetUrl("notes/assets/c.png"));
    expect(img?.alt).toBe("a chart");
    // Never instead of: the markdown is still in the buffer and on screen.
    expect(view.state.doc.toString()).toBe("![a chart](assets/c.png)");
    view.destroy();
  });

  it("draws nothing for an image when pictures are off", () => {
    const view = editorWith("![a](assets/c.png)", inNotes);
    expect(view.contentDOM.querySelector(".cm-md-image")).toBeNull();
    view.destroy();
  });
});

describe("following a link", () => {
  it("opens parsed includes through the same destination lookup", () => {
    const state = EditorState.create({ doc: '<book:include file="other.md" />' });
    expect(linkAt(state, { ...inNotes, includes: () => [{ from: 0, to: state.doc.length, target: "other.md" }] }, 12)?.target).toBe("other.md");
    expect(linkAt(state, inNotes, 12)).toBeNull();
  });
  it("asks the shell to open a document in the folder", () => {
    const seen: string[] = [];
    const listener = (e: Event) => seen.push((e as CustomEvent<string>).detail);
    window.addEventListener("hickory-open-path", listener);
    followLink("notes/weekly/mon.md", "../plan.md");
    window.removeEventListener("hickory-open-path", listener);
    expect(seen).toEqual(["notes/plan.md"]);
  });

  it("sends a web address to the default OS browser", async () => {
    const calls: unknown[] = [];
    installMockHandler(async (...args) => { calls.push(args); return { ok: true }; });
    followLink("a.md", "https://example.com");
    expect(calls).toEqual([["POST", "/api/open-external", { url: "https://example.com" }]]);
  });

  it("does nothing for a destination with nowhere to go", () => {
    const open = vi.spyOn(window, "open").mockReturnValue(null);
    const seen: string[] = [];
    const listener = (e: Event) => seen.push((e as CustomEvent<string>).detail);
    window.addEventListener("hickory-open-path", listener);
    followLink("a.md", "#section");
    window.removeEventListener("hickory-open-path", listener);
    expect(open).not.toHaveBeenCalled();
    expect(seen).toEqual([]);
    open.mockRestore();
  });
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("refreshes rendered image ownership when the document path changes without a source edit", () => {
  let path = "notes/a.md";
  const clicked: string[] = [];
  const view = editorWith("![chart](chart.png)", { docPath: () => path, images: true, onOpenImageSource: target => { clicked.push(target); return true; } });
  try {
    expect(view.contentDOM.querySelector("img")?.getAttribute("src")).toBe(assetUrl("notes/chart.png"));
    path = "renamed/a.md";
    view.dispatch({});
    expect(view.contentDOM.querySelector("img")?.getAttribute("src")).toBe(assetUrl("renamed/chart.png"));
    view.contentDOM.querySelector("img")?.click();
    expect(clicked).toEqual(["renamed/chart.png"]);
  } finally { view.destroy(); }
});
