<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/spatialExcerpt.test.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a83150248eb7"></a>
## `apps/web/src/editor/spatialExcerpt.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Spatial Views Preserve The Editor](../../../../../contract.md).

<a id="f-a83150248eb7-view"></a>
### `view`

<storycode:copy id="f-a83150248eb7-view">
import { afterEach, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView, lineNumbers } from "@codemirror/view";
import { showSpatialExcerpt } from "./spatialExcerpt";

// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
let view: EditorView;
</storycode:copy>
<a id="f-a83150248eb7-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-a83150248eb7-expression_statement">
afterEach(() => { view?.destroy(); document.body.replaceChildren(); });
</storycode:copy>
<a id="f-a83150248eb7-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-a83150248eb7-expression_statement-2">
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
</storycode:copy>

### Assemble `apps/web/src/editor/spatialExcerpt.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/spatialExcerpt.test.ts"><storycode:paste select="#f-a83150248eb7-view"/><storycode:paste select="#f-a83150248eb7-expression_statement"/><storycode:paste select="#f-a83150248eb7-expression_statement-2"/></storycode:file>

</storycode:doc>
