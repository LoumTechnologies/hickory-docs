// Asking a language question from inside generated output.
//
// The LSP bridge only speaks document coordinates: it weaves the `.hick`
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
