// Protects docs/guarantees/authoring/several-cursors-edit-at-once.md

import { EditorSelection, EditorState } from "@codemirror/state";
import { describe, expect, it } from "vitest";
import { multipleCursors } from "./multiCursor";

describe("several cursors", () => {
  it("lets a state hold more than one selection", () => {
    const state = EditorState.create({ doc: "one\ntwo\nthree", extensions: multipleCursors() });
    expect(state.facet(EditorState.allowMultipleSelections)).toBe(true);
    const two = state.update({
      selection: EditorSelection.create([EditorSelection.cursor(0), EditorSelection.cursor(4)]),
    }).state;
    expect(two.selection.ranges).toHaveLength(2);
  });

  it("types at every cursor at once", () => {
    const state = EditorState.create({ doc: "one\ntwo", extensions: multipleCursors() });
    const two = state.update({
      selection: EditorSelection.create([EditorSelection.cursor(0), EditorSelection.cursor(4)]),
    }).state;
    const typed = two.update(two.replaceSelection("# ")).state;
    expect(typed.doc.toString()).toBe("# one\n# two");
  });

  it("collapses to one selection without the extension, which is the default this replaces", () => {
    const state = EditorState.create({ doc: "one\ntwo" });
    const two = state.update({
      selection: EditorSelection.create([EditorSelection.cursor(0), EditorSelection.cursor(4)]),
    }).state;
    expect(two.selection.ranges).toHaveLength(1);
  });
});
