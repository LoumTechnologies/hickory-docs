import { describe, expect, it } from "vitest";
import { decodeSemanticTokens, tokenClass, tokenRange } from "./semanticTokens";

const legend = {
  tokenTypes: ["function", "variable", "keyword"],
  tokenModifiers: ["declaration", "readonly"],
};

describe("semantic token decoding", () => {
  it("follows the delta encoding, resetting the column when the line moves", () => {
    // Two tokens on line 0, one on line 2. The second token's column is
    // relative to the first; the third resets because its line moved.
    const tokens = decodeSemanticTokens([0, 5, 3, 0, 0, 0, 4, 2, 1, 1, 2, 1, 6, 2, 0], legend);
    expect(tokens.map((t) => [t.line, t.start])).toEqual([
      [0, 5],
      [0, 9],
      [2, 1],
    ]);
  });

  it("names the type and every set modifier from the legend", () => {
    const [token] = decodeSemanticTokens([0, 0, 4, 0, 0b11], legend);
    expect(token.type).toBe("function");
    expect(token.modifiers).toEqual(["declaration", "readonly"]);
  });

  it("names a type outside the legend rather than dropping the token", () => {
    // Dropping it would make an identifier silently lose its colour, which is
    // far harder to notice than one styled generically.
    const [token] = decodeSemanticTokens([0, 0, 4, 99, 0], legend);
    expect(token.type).toBe("unknown");
  });

  it("ignores a trailing partial group instead of inventing a token", () => {
    expect(decodeSemanticTokens([0, 0, 4, 0, 0, 1, 2], legend)).toHaveLength(1);
  });

  it("turns a token into the classes the stylesheet paints", () => {
    const [token] = decodeSemanticTokens([0, 0, 4, 0, 0b01], legend);
    expect(tokenClass(token)).toBe("cm-st-function cm-stm-declaration");
  });

  it("gives a token an end column derived from its length", () => {
    const [token] = decodeSemanticTokens([3, 2, 5, 1, 0], legend);
    expect(tokenRange(token)).toEqual({
      start: { line: 3, character: 2 },
      end: { line: 3, character: 7 },
    });
  });

  it("decodes nothing from nothing", () => {
    expect(decodeSemanticTokens([], legend)).toEqual([]);
  });
});
