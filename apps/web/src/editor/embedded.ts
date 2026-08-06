// Embedded-language syntax highlighting for verbatim hick block bodies in the
// Document view: a file body highlights as its extension's language, an exec
// body as shell, a copy/cut fragment as the language of the file it is pasted
// into (see hickDoc.languageForBlock).
//
// Pure computation: given the doc text and parsed structure, return styled
// spans (`tok-*` classes, colored by styles.css). The view plugin turns them
// into CodeMirror mark decorations for visible blocks only and caches per doc
// version — highlighting never touches the text itself.

import { highlightTree, classHighlighter } from "@lezer/highlight";
import { codeRangesOf, languageForBlock } from "./hickDoc";
import type { HickBlock, HickDocStructure } from "./hickDoc";
import { parserForLanguage } from "./languages";

export interface StyledSpan {
  from: number;
  to: number;
  /** Space-separated `tok-*` classes. */
  cls: string;
}

/** Highlight one code string in `lang`; spans are relative to the string. */
export function highlightCode(code: string, lang: string): StyledSpan[] {
  const parser = parserForLanguage(lang);
  if (!parser) return [];
  const out: StyledSpan[] = [];
  try {
    const tree = parser.parse(code);
    highlightTree(tree, classHighlighter, (from, to, cls) => {
      if (to > from && cls) out.push({ from, to, cls });
    });
  } catch {
    // Malformed input must never break the document view.
    return [];
  }
  return out;
}

/**
 * Styled spans (absolute doc offsets) for one block's body, or [] when the
 * block has no known language. Nested tags/blocks are excluded from the code
 * ranges, so an expect body or a nested block is never mis-highlighted.
 */
export function highlightBlock(
  text: string,
  structure: HickDocStructure,
  block: HickBlock,
): StyledSpan[] {
  const lang = languageForBlock(structure, block);
  if (!lang) return [];
  const out: StyledSpan[] = [];
  for (const [from, to] of codeRangesOf(structure, block)) {
    for (const span of highlightCode(text.slice(from, to), lang)) {
      out.push({ from: from + span.from, to: from + span.to, cls: span.cls });
    }
  }
  return out;
}
