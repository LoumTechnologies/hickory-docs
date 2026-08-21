import { describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

import {
  WRAP_DEFAULT,
  WRAP_MAX,
  WRAP_MIN,
  clampWrapColumn,
  proseWrap,
  setWrapColumn,
  wrapColumnOf,
} from "./wrapColumn";
import { fencedCodeRanges } from "./markdownStyling";
import { parseHickDoc, verbatimRanges } from "./hickDoc";

describe("the prose measure", () => {
  it("refuses a measure so narrow a sentence becomes a column of words", () => {
    expect(clampWrapColumn(2)).toBe(WRAP_MIN);
  });

  it("refuses a measure so wide it has stopped meaning anything", () => {
    expect(clampWrapColumn(10_000)).toBe(WRAP_MAX);
  });

  it("falls back to the default rather than laying out with NaN", () => {
    // This is the restored-session path: a corrupt or hand-edited UI-state
    // file must not be able to produce an unlayoutable editor.
    expect(clampWrapColumn("eighty")).toBe(WRAP_DEFAULT);
    expect(clampWrapColumn(undefined)).toBe(WRAP_DEFAULT);
    expect(clampWrapColumn(Number.NaN)).toBe(WRAP_DEFAULT);
  });

  it("rounds, so a drag landing between columns still names one", () => {
    expect(clampWrapColumn(80.4)).toBe(80);
    expect(clampWrapColumn(80.6)).toBe(81);
  });
});

describe("which lines refuse to wrap", () => {
  const mount = (doc: string, ranges: [number, number][]) =>
    new EditorView({
      state: EditorState.create({ doc, extensions: [proseWrap(() => ranges)] }),
      parent: document.body,
    });

  it("marks every line of a code range, and no prose line", () => {
    const doc = "prose one\n```sh\necho hi\n```\nprose two\n";
    const view = mount(doc, fencedCodeRanges(doc) as [number, number][]);
    const lines = [...view.dom.querySelectorAll(".cm-line")];
    const code = lines.map((l) => l.classList.contains("cm-code-line"));
    // Lines 2..4 are the fence, its body, and its closer.
    expect(code).toEqual([false, true, true, true, false, false]);
    view.destroy();
  });

  it("marks an exec cell's payload in a .hick document", () => {
    const doc = '# Notes\n\n<hick:exec container="sh">\necho hi\n</hick:exec>\n';
    const view = mount(doc, verbatimRanges(parseHickDoc(doc).blocks) as [number, number][]);
    const lines = [...view.dom.querySelectorAll(".cm-line")];
    const marked = lines
      .map((l, i) => (l.classList.contains("cm-code-line") ? i + 1 : 0))
      .filter(Boolean);
    // The payload line, and the closing-tag line the range ends on.
    expect(marked).toContain(4);
    expect(marked).not.toContain(1);
    view.destroy();
  });

  it("marks nothing when nothing is code", () => {
    const view = mount("just prose\nand more\n", []);
    expect(view.dom.querySelectorAll(".cm-code-line")).toHaveLength(0);
    view.destroy();
  });
});

describe("moving the measure", () => {
  it("takes a new column from an effect and clamps it", () => {
    const state = EditorState.create({ doc: "hello", extensions: [proseWrap(() => [])] });
    expect(wrapColumnOf(state)).toBe(WRAP_DEFAULT);
    const moved = state.update({ effects: setWrapColumn.of(60) }).state;
    expect(wrapColumnOf(moved)).toBe(60);
    const absurd = moved.update({ effects: setWrapColumn.of(-5) }).state;
    expect(wrapColumnOf(absurd)).toBe(WRAP_MIN);
  });

  it("writes the measure onto the editor as a pixel custom property", () => {
    // Pixels rather than `ch`, so the ruler above the editor and the dotted
    // line inside it are drawn from one measurement.
    const view = new EditorView({
      state: EditorState.create({ doc: "hello", extensions: [proseWrap(() => [])] }),
      parent: document.body,
    });
    expect(view.dom.style.getPropertyValue("--prose-wrap")).toMatch(/^[\d.]+px$/);
    view.destroy();
  });
});
