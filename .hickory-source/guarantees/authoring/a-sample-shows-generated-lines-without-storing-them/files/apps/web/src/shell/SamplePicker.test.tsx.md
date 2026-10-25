<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/SamplePicker.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Sample Shows Generated Lines Without Storing Them](../../../../../contract.md)

Captured SHA-256: `18bea6865379c68b57b9545b7a88d12238e213bb93c6cc0960714656da4ede77`.

<book:entry path="apps/web/src/shell/SamplePicker.test.tsx">
// Which lines a selection means — the arithmetic behind "Lines 11–18".
//
// Protects docs/guarantees/authoring/a-sample-shows-generated-lines-without-storing-them.md

import { describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";

import { selectedLines } from "./SamplePicker";

const DOC = "one\ntwo\nthree\nfour\n";

function at(anchor: number, head: number) {
  return EditorState.create({ doc: DOC, selection: { anchor, head } });
}

describe("selectedLines", () => {
  it("offers nothing for a bare caret — clicking around is not picking", () => {
    expect(selectedLines(at(5, 5))).toBeNull();
  });

  it("counts the lines a selection covers", () => {
    // From inside line 1 to inside line 3.
    expect(selectedLines(at(1, 10))).toEqual({ from: 1, to: 3 });
  });

  it("does not count a line the selection only touched the start of", () => {
    // "one\ntwo\n" — ends exactly at the first character of line 3, which is
    // two lines highlighted, not three.
    expect(selectedLines(at(0, 8))).toEqual({ from: 1, to: 2 });
  });
});
</book:entry>

</book:doc>
