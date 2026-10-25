<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/multiCursor.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Several Cursors Edit At Once](../../../../../contract.md)

Captured SHA-256: `e61e03077073819b3824b2fd3cae54631786822687928182fee70d96aff29fe7`.

<book:entry path="apps/web/src/editor/multiCursor.test.ts">
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
</book:entry>

</book:doc>
