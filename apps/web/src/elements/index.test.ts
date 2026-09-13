// Protects docs/guarantees/language/an-element-is-drawn-by-its-view.md:
// one view per kind, and the walk asks the views which blocks they draw.
import { describe, expect, it } from "vitest";

import { parseHickDoc } from "../editor/hickDoc";
import { elementViews, renderableBlocks, slotKindOf } from "./index";
import type { SlotKind } from "./types";

const KINDS: SlotKind[] = [
  "exec",
  "output",
  "diagram",
  "math",
  "table",
  "picture",
  // The conversation's elements: a session is a document, drawn by these.
  "session-user",
  "session-assistant",
  "session-tool",
  "session-tool-result",
  "session-read",
  "session-wrote",
  "session-context",
  "session-observation",
  "session-action",
  "session-reasoning",
  "session-input",
  "session-meta",
];

describe("the element views", () => {
  it("has exactly one view per kind, and each view names its kind", () => {
    expect(Object.keys(elementViews).sort()).toEqual([...KINDS].sort());
    for (const kind of KINDS) expect(elementViews[kind].kind).toBe(kind);
  });

  it("claims a file only when it writes a picture", () => {
    const { blocks } = parseHickDoc(
      '<hick:file path="chart.svg">x</hick:file>\n<hick:file path="a.py">y</hick:file>\n',
    );
    expect(slotKindOf(blocks[0])).toBe("picture");
    expect(slotKindOf(blocks[1])).toBeNull();
  });

  it("lists renderable blocks in document order, and nothing else", () => {
    const src =
      '<hick:copy id="c">z</hick:copy>\n<hick:exec container="c">a</hick:exec>\n<hick:math>x^2</hick:math>\n<hick:table>1,2</hick:table>\n<hick:diagram>g</hick:diagram>\n';
    const structure = parseHickDoc(src);
    expect(renderableBlocks(structure).map((b) => b.name)).toEqual([
      "exec",
      "math",
      "table",
      "diagram",
    ]);
  });

  it("draws fenced exec and derived output blocks", () => {
    const structure = parseHickDoc(
      "```shell container=\"reporter\" show=\"output\"\necho hello\n```\n\n```output-for=\"#reporter-1\" hash=\"x\" input-hash=\"y\" exit=\"0\"\nhello\n```\n",
    );
    expect(renderableBlocks(structure).map((block) => block.name)).toEqual(["exec", "output"]);
  });
});
