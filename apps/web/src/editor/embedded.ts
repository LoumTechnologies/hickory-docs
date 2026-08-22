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

// Results by (language, code). A keystroke anywhere in the document is a new
// document version, but the code inside a block you did not touch is the same
// string — and re-parsing every visible block for every keystroke is what
// made typing in a long note lag. Bounded and insertion-ordered, so the
// blocks on screen stay hot and the ones scrolled away age out.
const codeCache = new Map<string, readonly StyledSpan[]>();
const CODE_CACHE_SIZE = 256;

/** Highlight one code string in `lang`; spans are relative to the string. */
export function highlightCode(code: string, lang: string): readonly StyledSpan[] {
  const key = `${lang}\u0000${code}`;
  const hit = codeCache.get(key);
  if (hit) {
    // Refresh its place in the eviction order.
    codeCache.delete(key);
    codeCache.set(key, hit);
    return hit;
  }
  const spans = highlightCodeUncached(code, lang);
  if (codeCache.size >= CODE_CACHE_SIZE) {
    codeCache.delete(codeCache.keys().next().value as string);
  }
  codeCache.set(key, spans);
  return spans;
}

function highlightCodeUncached(code: string, lang: string): StyledSpan[] {
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
