<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/embedded.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-df9508b237a8"></a>
## `apps/web/src/editor/embedded.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Keystroke Redraws Only The Editor](../../../../../guarantees/authoring/a-keystroke-redraws-only-the-editor/contract.md), [Document-View Styling Never Mutates the Source](../../../../../guarantees/language/document-view-decorations-never-mutate-source/contract.md).

<a id="f-df9508b237a8-StyledSpan"></a>
### `StyledSpan`

Embedded-language syntax highlighting for verbatim hick block bodies in the
Document view: a file body highlights as its extension's language, an exec
body as shell, a copy/cut fragment as the language of the file it is pasted
into (see hickDoc.languageForBlock).
Pure computation: given the doc text and parsed structure, return styled
spans (`tok-*` classes, colored by styles.css). The view plugin turns them
into CodeMirror mark decorations for visible blocks only and caches per doc
version — highlighting never touches the text itself.

<storycode:copy id="f-df9508b237a8-StyledSpan">
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

</storycode:copy>
<a id="f-df9508b237a8-codeCache"></a>
### `codeCache`

Results by (language, code). A keystroke anywhere in the document is a new
document version, but the code inside a block you did not touch is the same
string — and re-parsing every visible block for every keystroke is what
made typing in a long note lag. Bounded and insertion-ordered, so the
blocks on screen stay hot and the ones scrolled away age out.

<storycode:copy id="f-df9508b237a8-codeCache">
// Results by (language, code). A keystroke anywhere in the document is a new
// document version, but the code inside a block you did not touch is the same
// string — and re-parsing every visible block for every keystroke is what
// made typing in a long note lag. Bounded and insertion-ordered, so the
// blocks on screen stay hot and the ones scrolled away age out.
const codeCache = new Map<string, readonly StyledSpan[]>();
</storycode:copy>
<a id="f-df9508b237a8-CODE_CACHE_SIZE"></a>
### `CODE_CACHE_SIZE`

<storycode:copy id="f-df9508b237a8-CODE_CACHE_SIZE">
const CODE_CACHE_SIZE = 256;

</storycode:copy>
<a id="f-df9508b237a8-highlightCode"></a>
### `highlightCode`

Highlight one code string in `lang`; spans are relative to the string.

<storycode:copy id="f-df9508b237a8-highlightCode">
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

</storycode:copy>
<a id="f-df9508b237a8-highlightCodeUncached"></a>
### `highlightCodeUncached`

<storycode:copy id="f-df9508b237a8-highlightCodeUncached">
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

</storycode:copy>
<a id="f-df9508b237a8-highlightBlock"></a>
### `highlightBlock`

Styled spans (absolute doc offsets) for one block's body, or [] when the
block has no known language. Nested tags/blocks are excluded from the code
ranges, so an expect body or a nested block is never mis-highlighted.

<storycode:copy id="f-df9508b237a8-highlightBlock">
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
</storycode:copy>

### Assemble `apps/web/src/editor/embedded.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/embedded.ts"><storycode:paste select="#f-df9508b237a8-StyledSpan"/><storycode:paste select="#f-df9508b237a8-codeCache"/><storycode:paste select="#f-df9508b237a8-CODE_CACHE_SIZE"/><storycode:paste select="#f-df9508b237a8-highlightCode"/><storycode:paste select="#f-df9508b237a8-highlightCodeUncached"/><storycode:paste select="#f-df9508b237a8-highlightBlock"/></storycode:file>

</storycode:doc>
