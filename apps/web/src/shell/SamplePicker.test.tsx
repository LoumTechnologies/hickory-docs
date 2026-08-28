// Which lines a selection means — the arithmetic behind "Lines 11–18".
//
// Protects docs/guarantees/documents/a-sample-shows-generated-lines-without-storing-them.md

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
