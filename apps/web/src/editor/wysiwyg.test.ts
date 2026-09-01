// @vitest-environment jsdom
// Guarantee: docs/guarantees/editor-intelligence/an-ingested-block-is-marked-in-the-document-view-too.md
//
// The exec card that owns a `<hick:ingested>` block stops rendering exactly
// where the ingested content begins (rendered.ts), so those bytes stay
// editable as plain text. Before this, that plain text carried no visual
// signal that it arrived from a real run — the same `cm-prov-ingested`
// treatment the Output pane already gives ingested bytes must also reach
// the Document view.
import { afterEach, describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { EnvRegistry, wysiwyg } from "./wysiwyg";

let view: EditorView | undefined;

afterEach(() => {
  view?.destroy();
  view = undefined;
});

function open(doc: string) {
  view = new EditorView({
    state: EditorState.create({ doc, extensions: wysiwyg(new EnvRegistry()) }),
    parent: document.body,
  });
  return view;
}

const DOC = `<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:exec container="c" mount="v:out">
scaffold
<hick:ingested from="#scaffold" sha256="abc123" at="2026-09-01" files="1" skipped="0">
<hick:file path="app/Program.cs">real content here</hick:file>
</hick:ingested>
</hick:exec>
</hick:doc>
`;

describe("an ingested block in the Document view", () => {
  it("carries the same cm-prov-ingested class the Output pane uses", () => {
    const v = open(DOC);
    const marked = v.dom.querySelectorAll(".cm-prov-ingested");
    expect(marked.length).toBeGreaterThan(0);
    const text = Array.from(marked)
      .map((el) => el.textContent)
      .join("");
    expect(text).toContain("real content here");
  });

  it("does not mark ordinary, non-ingested text", () => {
    const v = open('<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\nordinary prose\n</hick:doc>\n');
    expect(v.dom.querySelectorAll(".cm-prov-ingested").length).toBe(0);
  });
});
