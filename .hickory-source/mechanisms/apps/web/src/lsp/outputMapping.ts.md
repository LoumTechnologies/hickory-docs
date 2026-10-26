<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lsp/outputMapping.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-14ad78d142d1"></a>
## `apps/web/src/lsp/outputMapping.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-14ad78d142d1-OutputProvenance"></a>
### `OutputProvenance`

Asking a language question from inside generated output.
The LSP bridge only speaks document coordinates: it weaves the `.md`
source into virtual files, delegates to the real language servers, and maps
their answers back through provenance. So when you Cmd-click a symbol in
woven output, the position has to travel the same road in reverse — output
offset → the provenance range covering it → the source bytes that produced
it → a line/character position in the document.
Synthetic bytes (separators, weaver-generated values) have no source origin,
and neither do ranges belonging to a different document. Both return null:
there is no honest question to ask, so nothing is asked.

<storycode:copy id="f-14ad78d142d1-OutputProvenance">
// Asking a language question from inside generated output.
//
// The LSP bridge only speaks document coordinates: it weaves the `.md`
// source into virtual files, delegates to the real language servers, and maps
// their answers back through provenance. So when you Cmd-click a symbol in
// woven output, the position has to travel the same road in reverse — output
// offset → the provenance range covering it → the source bytes that produced
// it → a line/character position in the document.
//
// Synthetic bytes (separators, weaver-generated values) have no source origin,
// and neither do ranges belonging to a different document. Both return null:
// there is no honest question to ask, so nothing is asked.

import type { Provenance } from "../api/types";
import { samePath } from "../lib/paths";
import { byteToChar } from "../lib/offsets";
import { utf16ToPosition, type LspPosition } from "./positions";

export interface OutputProvenance extends Provenance {
  /** Provenance range in output CHAR offsets. */
  charFrom: number;
  charTo: number;
}

</storycode:copy>
<a id="f-14ad78d142d1-sourcePositionAt"></a>
### `sourcePositionAt`

Map an offset in an output buffer to a position in `docSource`.
`mapOffset` lets callers account for edits made since the buffer was loaded
(provenance indexes the server's copy, the buffer may have moved on).

<storycode:copy id="f-14ad78d142d1-sourcePositionAt">
/**
 * Map an offset in an output buffer to a position in `docSource`.
 *
 * `mapOffset` lets callers account for edits made since the buffer was loaded
 * (provenance indexes the server's copy, the buffer may have moved on).
 */
export function sourcePositionAt(
  offset: number,
  provenance: OutputProvenance[],
  docPath: string,
  docSource: string,
  mapOffset: (offset: number) => number = (o) => o,
): LspPosition | null {
  for (const p of provenance) {
    const from = mapOffset(p.charFrom);
    const to = mapOffset(p.charTo);
    if (to <= from) continue;
    if (offset < from || offset >= to) continue;
    if (p.origin.kind === "synthetic") return null;
    if (!samePath(p.origin.doc_path, docPath)) return null;
    // The output text and its source text are the same bytes, so the distance
    // into the range carries across unchanged.
    const delta = offset - from;
    const sourceStart = byteToChar(docSource, p.origin.span[0]);
    const sourceEnd = byteToChar(docSource, p.origin.span[1]);
    return utf16ToPosition(docSource, Math.min(sourceStart + delta, sourceEnd));
  }
  return null;
}
</storycode:copy>

### Assemble `apps/web/src/lsp/outputMapping.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/outputMapping.ts"><storycode:paste select="#f-14ad78d142d1-OutputProvenance"/><storycode:paste select="#f-14ad78d142d1-sourcePositionAt"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-3b77135a6839"></a>
## `apps/web/src/lsp/outputMapping.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-3b77135a6839-DOC"></a>
### `DOC`

<storycode:copy id="f-3b77135a6839-DOC">
import { describe, expect, it } from "vitest";
import { sourcePositionAt, type OutputProvenance } from "./outputMapping";

// gen.rs is woven from two copy blocks in doc.md. Byte spans below index
// the document source; charFrom/charTo index the generated output.
const DOC = ['<hick:doc>', '<hick:copy id="a">fn alpha() {}', '</hick:copy>', '</hick:doc>'].join(
  "\n",
);
</storycode:copy>
<a id="f-3b77135a6839-ALPHA"></a>
### `ALPHA`

<storycode:copy id="f-3b77135a6839-ALPHA">
const ALPHA = "fn alpha() {}";
</storycode:copy>
<a id="f-3b77135a6839-start"></a>
### `start`

<storycode:copy id="f-3b77135a6839-start">
const start = DOC.indexOf(ALPHA);

</storycode:copy>
<a id="f-3b77135a6839-prov"></a>
### `prov`

<storycode:copy id="f-3b77135a6839-prov">
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

</storycode:copy>
<a id="f-3b77135a6839-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-3b77135a6839-expression_statement">
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
</storycode:copy>

### Assemble `apps/web/src/lsp/outputMapping.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/outputMapping.test.ts"><storycode:paste select="#f-3b77135a6839-DOC"/><storycode:paste select="#f-3b77135a6839-ALPHA"/><storycode:paste select="#f-3b77135a6839-start"/><storycode:paste select="#f-3b77135a6839-prov"/><storycode:paste select="#f-3b77135a6839-expression_statement"/></storycode:file>


</storycode:doc>
