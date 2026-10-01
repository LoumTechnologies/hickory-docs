import { describe, expect, it } from "vitest";
import { sourcePositionAt, type OutputProvenance } from "./outputMapping";

// gen.rs is woven from two copy blocks in doc.hick. Byte spans below index
// the document source; charFrom/charTo index the generated output.
const DOC = ['<hick:doc>', '<hick:copy id="a">fn alpha() {}', '</hick:copy>', '</hick:doc>'].join(
  "\n",
);
const ALPHA = "fn alpha() {}";
const start = DOC.indexOf(ALPHA);

function prov(over: Partial<OutputProvenance> = {}): OutputProvenance {
  return {
    start: 0,
    end: ALPHA.length,
    charFrom: 0,
    charTo: ALPHA.length,
    origin: { kind: "paste", doc_path: "doc.md", span: [start, start + ALPHA.length] },
    ...over,
  } as OutputProvenance;
}

describe("sourcePositionAt", () => {
  it("maps an output offset onto the document line/character that produced it", () => {
    // Offset 3 is the 'a' that starts "alpha" in the output; in the document
    // that byte lives on line 1 (0-based), after the `<hick:copy …>` open tag.
    const pos = sourcePositionAt(3, [prov()], "doc.md", DOC);
    expect(pos).not.toBeNull();
    expect(pos!.line).toBe(1);
    const line = DOC.split("\n")[1];
    expect(line.slice(pos!.character, pos!.character + 5)).toBe("alpha");
  });

  it("refuses synthetic bytes — the weaver made them, no source to ask about", () => {
    const synthetic = prov({ origin: { kind: "synthetic" } });
    expect(sourcePositionAt(3, [synthetic], "doc.md", DOC)).toBeNull();
  });

  it("refuses ranges owned by a different document", () => {
    const other = prov({
      origin: { kind: "paste", doc_path: "other.md", span: [0, 4] },
    });
    expect(sourcePositionAt(3, [other], "doc.md", DOC)).toBeNull();
  });

  it("returns null outside every provenance range instead of guessing", () => {
    expect(sourcePositionAt(999, [prov()], "doc.md", DOC)).toBeNull();
  });

  it("honours a mapping for edits made since the buffer was loaded", () => {
    // Two characters were inserted ahead of the range, so the same source byte
    // now sits two offsets later in the buffer.
    const shift = (o: number) => o + 2;
    const shifted = sourcePositionAt(5, [prov()], "doc.md", DOC, shift);
    const unshifted = sourcePositionAt(3, [prov()], "doc.md", DOC);
    expect(shifted).toEqual(unshifted);
  });
});
