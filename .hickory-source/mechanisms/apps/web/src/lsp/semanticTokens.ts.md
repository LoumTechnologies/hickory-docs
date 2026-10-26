<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lsp/semanticTokens.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6fcc65d3b9df"></a>
## `apps/web/src/lsp/semanticTokens.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-6fcc65d3b9df-SemanticToken"></a>
### `SemanticToken`

Decoding the semantic-token wire format into things to paint.
The server sends a flat array of five-number groups, each *relative to the
token before it*:
[ Δline, Δstart, length, tokenType, tokenModifiers, … ]
A non-zero line delta resets the column; a zero one continues from the
previous token's start. Getting that backwards shifts every token on a
line, which looks like the highlighter having an opinion rather than a bug.
`tokenType` is an index into the legend the server advertised, which is why
the bridge forwards its capabilities: the same integer means different
things to different servers, and hick-lsp re-indexes every child's tokens
against one legend before they get here.

<storycode:copy id="f-6fcc65d3b9df-SemanticToken">
// Decoding the semantic-token wire format into things to paint.
//
// The server sends a flat array of five-number groups, each *relative to the
// token before it*:
//
//   [ Δline, Δstart, length, tokenType, tokenModifiers, … ]
//
// A non-zero line delta resets the column; a zero one continues from the
// previous token's start. Getting that backwards shifts every token on a
// line, which looks like the highlighter having an opinion rather than a bug.
//
// `tokenType` is an index into the legend the server advertised, which is why
// the bridge forwards its capabilities: the same integer means different
// things to different servers, and hick-lsp re-indexes every child's tokens
// against one legend before they get here.

import type { LspPosition } from "./positions";

export interface SemanticToken {
  line: number;
  start: number;
  length: number;
  /** Legend name, e.g. "function" — never the raw index. */
  type: string;
  modifiers: string[];
}

</storycode:copy>
<a id="f-6fcc65d3b9df-SemanticLegend"></a>
### `SemanticLegend`

<storycode:copy id="f-6fcc65d3b9df-SemanticLegend">
export interface SemanticLegend {
  tokenTypes: string[];
  tokenModifiers: string[];
}

</storycode:copy>
<a id="f-6fcc65d3b9df-decodeSemanticTokens"></a>
### `decodeSemanticTokens`

<storycode:copy id="f-6fcc65d3b9df-decodeSemanticTokens">
export function decodeSemanticTokens(data: number[], legend: SemanticLegend): SemanticToken[] {
  const tokens: SemanticToken[] = [];
  let line = 0;
  let start = 0;
  for (let i = 0; i + 4 < data.length; i += 5) {
    const [deltaLine, deltaStart, length, typeIndex, modifierBits] = data.slice(i, i + 5);
    if (deltaLine > 0) {
      line += deltaLine;
      start = deltaStart;
    } else {
      start += deltaStart;
    }
    const modifiers: string[] = [];
    for (let bit = 0; bit < legend.tokenModifiers.length; bit++) {
      if (modifierBits & (1 << bit)) modifiers.push(legend.tokenModifiers[bit]);
    }
    tokens.push({
      line,
      start,
      length,
      // A type outside the legend is named rather than dropped: the editor
      // can style it generically, and a silent disappearance would be much
      // harder to notice than an unstyled identifier.
      type: legend.tokenTypes[typeIndex] ?? "unknown",
      modifiers,
    });
  }
  return tokens;
}

</storycode:copy>
<a id="f-6fcc65d3b9df-tokenClass"></a>
### `tokenClass`

The CSS class a token gets, e.g. `cm-st-function cm-stm-declaration`.

<storycode:copy id="f-6fcc65d3b9df-tokenClass">
/** The CSS class a token gets, e.g. `cm-st-function cm-stm-declaration`. */
export function tokenClass(token: SemanticToken): string {
  const classes = [`cm-st-${token.type}`];
  for (const modifier of token.modifiers) classes.push(`cm-stm-${modifier}`);
  return classes.join(" ");
}

</storycode:copy>
<a id="f-6fcc65d3b9df-tokenRange"></a>
### `tokenRange`

Where a token starts and ends, as LSP positions.

<storycode:copy id="f-6fcc65d3b9df-tokenRange">
/** Where a token starts and ends, as LSP positions. */
export function tokenRange(token: SemanticToken): { start: LspPosition; end: LspPosition } {
  return {
    start: { line: token.line, character: token.start },
    end: { line: token.line, character: token.start + token.length },
  };
}
</storycode:copy>

### Assemble `apps/web/src/lsp/semanticTokens.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/semanticTokens.ts"><storycode:paste select="#f-6fcc65d3b9df-SemanticToken"/><storycode:paste select="#f-6fcc65d3b9df-SemanticLegend"/><storycode:paste select="#f-6fcc65d3b9df-decodeSemanticTokens"/><storycode:paste select="#f-6fcc65d3b9df-tokenClass"/><storycode:paste select="#f-6fcc65d3b9df-tokenRange"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-ba579175e4ee"></a>
## `apps/web/src/lsp/semanticTokens.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-ba579175e4ee-legend"></a>
### `legend`

<storycode:copy id="f-ba579175e4ee-legend">
import { describe, expect, it } from "vitest";
import { decodeSemanticTokens, tokenClass, tokenRange } from "./semanticTokens";

const legend = {
  tokenTypes: ["function", "variable", "keyword"],
  tokenModifiers: ["declaration", "readonly"],
};

</storycode:copy>
<a id="f-ba579175e4ee-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-ba579175e4ee-expression_statement">
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
</storycode:copy>

### Assemble `apps/web/src/lsp/semanticTokens.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/semanticTokens.test.ts"><storycode:paste select="#f-ba579175e4ee-legend"/><storycode:paste select="#f-ba579175e4ee-expression_statement"/></storycode:file>


</storycode:doc>
