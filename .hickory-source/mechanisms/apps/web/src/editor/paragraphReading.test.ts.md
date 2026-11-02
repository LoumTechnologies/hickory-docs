<static:doc xmlns:static="http://www.hickorydocs.com/1.0">
# Keep paragraph reading separate from source bytes

Check the display joins against real Markdown and Hickory payloads, then replace the source to prove the decoration plan follows the current document. A read-only view must never rewrite its source merely to read as one paragraph. The browser witness separately proves that replacement decorations actually join the rendered lines.

<static:copy id="static-apps-web-src-editor-paragraphReading-test-ts-0">
import { it, expect } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { paragraphReading } from "./paragraphReading";
import { loadHickLang } from "./hickLang";

it("joins prose visually without changing code, explicit breaks or document offsets", async () => {
  await loadHickLang();
  const source = '<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\nFirst part\n  continues here.\n\nAnother paragraph.\n\nHard break  \nstays.\n\n<h:file path="a.ts">\nconst count = 1;\ncount;\n</h:file>\n</h:doc>';
  const view = new EditorView({ state: EditorState.create({ doc: source, extensions: [EditorState.readOnly.of(true), paragraphReading()] }) });
  try {
    const ranges: [number, number][] = [];
    for (const decorations of view.state.facet(EditorView.decorations)) {
      if (typeof decorations !== "function") decorations.between(0, source.length, (from, to) => { ranges.push([from, to]); });
    }
    expect(ranges).toEqual([[source.indexOf("\n  continues"), source.indexOf("continues here.")]]);
    expect(view.state.doc.toString()).toBe(source);
    expect(view.state.doc.lineAt(source.indexOf("count;")).text).toBe("count;");
    view.dispatch({ changes: { from: 0, to: source.length, insert: "New paragraph\ncontinues.\n\nSeparate." } });
    const changed: [number, number][] = [];
    for (const decorations of view.state.facet(EditorView.decorations)) {
      if (typeof decorations !== "function") decorations.between(0, view.state.doc.length, (from, to) => { changed.push([from, to]); });
    }
    expect(changed).toEqual([[13, 14]]);
    expect(view.state.doc.toString()).toBe("New paragraph\ncontinues.\n\nSeparate.");
  } finally { view.destroy(); }
});
</static:copy>

<static:file path="apps/web/src/editor/paragraphReading.test.ts"><static:paste select="#static-apps-web-src-editor-paragraphReading-test-ts-0"/></static:file>
</static:doc>
