<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/spatialExcerpt.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Spatial Views Preserve The Editor](../../../../../contract.md)

Captured SHA-256: `36b1e826f1ff3a9fa37613fc8e16886421380d9f5d428cb8f67c5d6dbe402dcc`.

<book:entry path="apps/web/src/editor/spatialExcerpt.test.ts">
import { afterEach, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView, lineNumbers } from "@codemirror/view";
import { showSpatialExcerpt } from "./spatialExcerpt";

// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
let view: EditorView;
afterEach(() => { view?.destroy(); document.body.replaceChildren(); });
it("expands and contracts context without changing source bytes, selection, or line addresses", () => {
  const source = Array.from({ length: 40 }, (_, index) => `line ${index + 1}`).join("\n");
  view = new EditorView({ parent: document.body, state: EditorState.create({ doc: source, extensions: [lineNumbers()] }) });
  const focal = view.state.doc.line(20).from;
  view.dispatch({ selection: { anchor: focal + 2 } });
  showSpatialExcerpt(view, focal, true);
  expect(view.state.doc.toString()).toBe(source);
  expect(view.state.selection.main.anchor).toBe(focal + 2);
  const more = document.querySelector<HTMLButtonElement>('[aria-label="Show three more lines above"]')!;
  expect(more).not.toBeNull(); more.click();
  expect(document.querySelector('[aria-label="Hide three lines above"]')).not.toBeNull();
  document.querySelector<HTMLButtonElement>('[aria-label="Hide three lines above"]')!.click();
  expect(view.state.doc.line(20).from).toBe(focal);
  showSpatialExcerpt(view, null);
  expect(document.querySelector(".spatial-excerpt-edge")).toBeNull();
  expect(view.state.doc.toString()).toBe(source);
});
</book:entry>

</book:doc>
