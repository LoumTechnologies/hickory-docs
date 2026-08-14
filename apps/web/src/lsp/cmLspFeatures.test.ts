import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { describe, expect, it } from "vitest";
import { diagnosticRanges, diagnosticsAt, lspDiagnosticField, setLspDiagnostics } from "./cmLsp";
import {
  editsForUri,
  inlayHintField,
  inlayText,
  semanticTokenField,
  setInlayHints,
  setSemanticTokens,
  signatureLabel,
  urisInEdit,
} from "./cmLspFeatures";

function viewWith(doc: string, extensions = [semanticTokenField, inlayHintField]) {
  return new EditorView({ state: EditorState.create({ doc, extensions }) });
}

function decorationCount(view: EditorView, field: typeof semanticTokenField) {
  let count = 0;
  view.state.field(field).between(0, view.state.doc.length, () => {
    count++;
  });
  return count;
}

describe("semantic token decorations", () => {
  it("paints the spans it is given", () => {
    const view = viewWith("def load(path):\n    return path\n");
    view.dispatch({
      effects: setSemanticTokens.of([
        { from: 0, to: 3, class: "cm-st-keyword" },
        { from: 4, to: 8, class: "cm-st-function" },
      ]),
    });
    expect(decorationCount(view, semanticTokenField)).toBe(2);
  });

  it("accepts spans in any order, because a server does not sort for us", () => {
    // RangeSetBuilder throws on descending ranges, so an unsorted reply used
    // to take the whole editor down rather than losing a colour.
    const view = viewWith("abcdefghij");
    expect(() =>
      view.dispatch({
        effects: setSemanticTokens.of([
          { from: 6, to: 8, class: "cm-st-variable" },
          { from: 0, to: 3, class: "cm-st-keyword" },
        ]),
      }),
    ).not.toThrow();
    expect(decorationCount(view, semanticTokenField)).toBe(2);
  });

  it("clamps a span that outlived the text it described", () => {
    // Tokens arrive after a debounce, so the document may have shrunk in the
    // meantime. Painting past the end would throw.
    const view = viewWith("short");
    view.dispatch({ effects: setSemanticTokens.of([{ from: 2, to: 900, class: "cm-st-type" }]) });
    expect(decorationCount(view, semanticTokenField)).toBe(1);
  });

  it("drops an empty span rather than painting a zero-width mark", () => {
    const view = viewWith("hello");
    view.dispatch({ effects: setSemanticTokens.of([{ from: 3, to: 3, class: "cm-st-type" }]) });
    expect(decorationCount(view, semanticTokenField)).toBe(0);
  });
});

describe("inlay hints", () => {
  it("places a widget at each hint position", () => {
    const view = viewWith("x = load()\n");
    view.dispatch({ effects: setInlayHints.of([{ at: 1, text: ": str" }]) });
    expect(decorationCount(view, inlayHintField)).toBe(1);
  });

  it("joins a label given as parts", () => {
    expect(inlayText({ position: { line: 0, character: 0 }, label: [{ value: ": " }, { value: "int" }] })).toBe(
      ": int",
    );
    expect(inlayText({ position: { line: 0, character: 0 }, label: "-> bool" })).toBe("-> bool");
  });
});

describe("diagnostics", () => {
  it("carries the server's message through to the decoration", () => {
    // An underline tells you WHERE the problem is and nothing about what it
    // is. The message rides on the decoration so the hover can read it back
    // without a second structure to keep in step.
    const view = viewWith("const x: number = 'oops';", [lspDiagnosticField]);
    view.dispatch({
      effects: setLspDiagnostics.of([
        { from: 18, to: 24, severity: 1, message: "Type 'string' is not assignable", source: "typescript" },
      ]),
    });
    const found = diagnosticsAt(view.state, 20);
    expect(found).toHaveLength(1);
    expect(found[0].message).toContain("not assignable");
    expect(found[0].source).toBe("typescript");
    expect(found[0].severity).toBe(1);
  });

  it("finds nothing where there is nothing wrong", () => {
    const view = viewWith("fine", [lspDiagnosticField]);
    view.dispatch({ effects: setLspDiagnostics.of([]) });
    expect(diagnosticsAt(view.state, 2)).toEqual([]);
  });

  it("keeps a zero-width diagnostic hoverable", () => {
    // An insertion-point diagnostic has start == end; left alone it would be
    // an underline with no width, which is a squiggle nobody can point at.
    const spans = diagnosticRanges(
      [{ range: r(0, 3, 0, 3), severity: 1, message: "expected `;`" }],
      (position) => position.character,
    );
    expect(spans[0].to).toBeGreaterThan(spans[0].from);
    expect(spans[0].message).toBe("expected `;`");
  });
});

describe("workspace edits", () => {
  const edit = {
    changes: { "hick:///a.hick": [{ range: r(0, 0, 0, 4), newText: "next" }] },
    documentChanges: [
      {
        textDocument: { uri: "hick:///a.hick", version: 1 },
        edits: [{ range: r(2, 0, 2, 4), newText: "next" }],
      },
      {
        textDocument: { uri: "hick:///b.hick", version: 1 },
        edits: [{ range: r(0, 0, 0, 4), newText: "next" }],
      },
    ],
  };

  it("takes this document's edits from both shapes a server may use", () => {
    expect(editsForUri(edit, "hick:///a.hick")).toHaveLength(2);
  });

  it("ignores edits addressed to another document", () => {
    // Applying another file's ranges to this buffer would corrupt it at
    // coordinates that happen to exist here.
    expect(editsForUri(edit, "hick:///a.hick").every((e) => e.newText === "next")).toBe(true);
    expect(editsForUri(edit, "hick:///c.hick")).toEqual([]);
  });

  it("names every document a rename touches, so the rest can be reported", () => {
    expect(urisInEdit(edit).sort()).toEqual(["hick:///a.hick", "hick:///b.hick"]);
  });

  it("treats a refused rename as no edits rather than an error", () => {
    expect(editsForUri(null, "hick:///a.hick")).toEqual([]);
    expect(urisInEdit(null)).toEqual([]);
  });
});

describe("signature help", () => {
  it("shows the active signature, not the first", () => {
    expect(
      signatureLabel({
        signatures: [{ label: "(a: int)" }, { label: "(a: str, b: int)" }],
        activeSignature: 1,
      }),
    ).toBe("(a: str, b: int)");
  });

  it("is empty when a server answers with no signatures", () => {
    expect(signatureLabel({ signatures: [] })).toBe("");
  });
});

function r(sl: number, sc: number, el: number, ec: number) {
  return { start: { line: sl, character: sc }, end: { line: el, character: ec } };
}
