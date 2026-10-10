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
