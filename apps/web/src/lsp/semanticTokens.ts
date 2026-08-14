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

export interface SemanticLegend {
  tokenTypes: string[];
  tokenModifiers: string[];
}

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

/** The CSS class a token gets, e.g. `cm-st-function cm-stm-declaration`. */
export function tokenClass(token: SemanticToken): string {
  const classes = [`cm-st-${token.type}`];
  for (const modifier of token.modifiers) classes.push(`cm-stm-${modifier}`);
  return classes.join(" ");
}

/** Where a token starts and ends, as LSP positions. */
export function tokenRange(token: SemanticToken): { start: LspPosition; end: LspPosition } {
  return {
    start: { line: token.line, character: token.start },
    end: { line: token.line, character: token.start + token.length },
  };
}
