import { afterEach, describe, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

import { assetUrl, followLink, linkAt, linksIn, mdLinks } from "./mdLinks";

function editorWith(doc: string, options: Parameters<typeof mdLinks>[0]) {
  return new EditorView({
    state: EditorState.create({ doc, extensions: [mdLinks(options)] }),
    parent: document.body,
  });
}

const inNotes = { docPath: () => "notes/a.hick" };

afterEach(() => {
  document.body.innerHTML = "";
});

describe("what counts as a link", () => {
  it("skips the ranges the caller says are not prose", () => {
    const doc = "prose [a](a.hick)\ncode [b](b.hick)";
    const state = EditorState.create({ doc });
    const all = linksIn(state, inNotes);
    expect(all.map((l) => l.link.target)).toEqual(["a.hick", "b.hick"]);
    const skipped = linksIn(state, { ...inNotes, skip: () => [[18, doc.length]] });
    expect(skipped.map((l) => l.link.target)).toEqual(["a.hick"]);
  });

  it("finds the link under a position, brackets included", () => {
    const state = EditorState.create({ doc: "see [the plan](plan.hick) now" });
    expect(linkAt(state, inNotes, 6)?.target).toBe("plan.hick");
    expect(linkAt(state, inNotes, 27)).toBeNull();
  });
});

describe("how a link looks", () => {
  it("lights the label and dims the punctuation without hiding anything", () => {
    const view = editorWith("see [the plan](plan.hick)", inNotes);
    expect(view.state.doc.toString()).toBe("see [the plan](plan.hick)");
    expect(view.contentDOM.querySelector(".cm-md-link")?.textContent).toBe("the plan");
    expect(view.contentDOM.querySelectorAll(".cm-md-link-mark").length).toBe(2);
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
  it("asks the shell to open a document in the folder", () => {
    const seen: string[] = [];
    const listener = (e: Event) => seen.push((e as CustomEvent<string>).detail);
    window.addEventListener("hickory-open-path", listener);
    followLink("notes/weekly/mon.hick", "../plan.hick");
    window.removeEventListener("hickory-open-path", listener);
    expect(seen).toEqual(["/notes/plan.hick"]);
  });

  it("sends a web address to the browser", () => {
    const open = vi.spyOn(window, "open").mockReturnValue(null);
    followLink("a.hick", "https://example.com");
    expect(open).toHaveBeenCalledWith(
      "https://example.com",
      "_blank",
      "noopener,noreferrer",
    );
    open.mockRestore();
  });

  it("does nothing for a destination with nowhere to go", () => {
    const open = vi.spyOn(window, "open").mockReturnValue(null);
    const seen: string[] = [];
    const listener = (e: Event) => seen.push((e as CustomEvent<string>).detail);
    window.addEventListener("hickory-open-path", listener);
    followLink("a.hick", "#section");
    window.removeEventListener("hickory-open-path", listener);
    expect(open).not.toHaveBeenCalled();
    expect(seen).toEqual([]);
    open.mockRestore();
  });
});
