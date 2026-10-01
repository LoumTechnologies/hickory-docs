import { describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

import { renderedMath } from "./mathRender";
import { parseHickDoc, verbatimRanges } from "./hickDoc";
import { mathSpans } from "../lib/math";
import { cardsOf } from "./cards";
import { actionsFor } from "../lib/railActions";

const mount = (doc: string, cursor: number) =>
  new EditorView({
    state: EditorState.create({
      doc,
      selection: { anchor: cursor },
      extensions: [renderedMath()],
    }),
    parent: document.body,
  });

describe("maths in the editor", () => {
  it("replaces an inline equation with a rendered one", () => {
    const view = mount("the value $x^2$ is squared\n", 27);
    const drawn = view.dom.querySelectorAll(".cm-math");
    expect(drawn).toHaveLength(1);
    expect(drawn[0].classList.contains("cm-math--display")).toBe(false);
    view.destroy();
  });

  it("leaves the LaTeX as source on the line holding the caret", () => {
    // The reveal IS the source/rendered switch for inline maths.
    const view = mount("the value $x^2$ is squared\n", 3);
    expect(view.dom.querySelectorAll(".cm-math")).toHaveLength(0);
    expect(view.state.doc.toString()).toContain("$x^2$");
    view.destroy();
  });

  it("sets display maths that owns its lines as a block", () => {
    const doc = "before\n$$\na^2 + b^2 = c^2\n$$\nafter\n";
    const view = mount(doc, 0);
    const drawn = view.dom.querySelector(".cm-math");
    expect(drawn).not.toBeNull();
    expect(drawn!.classList.contains("cm-math--display")).toBe(true);
    // A block replacement is a DIV; an inline one is a span, because a span
    // is all that can sit inside a line without adding a row to it.
    expect(drawn!.tagName).toBe("DIV");
    view.destroy();
  });

  it("shows the source until the engine arrives, never an empty box", () => {
    // KaTeX is loaded lazily, so the first paint must still say something
    // true. jsdom never resolves the import here, which is exactly the case
    // this asserts.
    const view = mount("$e = mc^2$\n", 11);
    expect(view.dom.querySelector(".cm-math")!.textContent).toBe("e = mc^2");
    view.destroy();
  });

  it("draws no maths where a document says there is none", () => {
    const view = mount("it cost $5 and $6\n", 0);
    expect(view.dom.querySelectorAll(".cm-math")).toHaveLength(0);
    view.destroy();
  });
});

describe("maths in a .md document", () => {
  it("leaves a shell variable in a cell alone", () => {
    const src = '<hick:exec container="sh">\necho $PATH and $HOME\n</hick:exec>\n';
    const structure = parseHickDoc(src);
    expect(mathSpans(src, verbatimRanges(structure.blocks))).toEqual([]);
  });

  it("gives a <hick:math> block a rail card whose only verb is its source", () => {
    const src = "# Notes\n\n<hick:math>\ne = mc^2\n</hick:math>\n";
    const cards = cardsOf(parseHickDoc(src), { text: src });
    const math = cards.filter((c) => c.kind === "math");
    expect(math).toHaveLength(1);
    expect(math[0].label).toBe("Equation 1");
    expect(actionsFor("math")).toEqual(["source"]);
  });
});
