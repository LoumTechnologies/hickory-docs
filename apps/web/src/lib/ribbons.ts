// Ribbon DATA derivation for the Split (lineage) view: pure functions from an
// output file's provenance to the list of source→output ribbons. Geometry
// (screen anchors, paths) lives in ribbonGeometry.ts; DOM measurement in
// views/SplitView.tsx. Kept separate so this is unit-testable.

import type { OutputFile, Provenance } from "../api/types";
import { samePath } from "./paths";
import { byteToChar } from "./offsets";

/** Number of distinct ribbon colors (see --ribbon-0 … in styles.css). */
export const RIBBON_PALETTE_SIZE = 6;

export interface Ribbon {
  /** Stable identity: file path + provenance index. */
  key: string;
  /** Provenance origin kind (never "synthetic" — those get no ribbon). */
  kind: Exclude<Provenance["origin"], { kind: "synthetic" }>["kind"];
  /** Source span, CHAR offsets into the doc source. */
  sourceSpan: [number, number];
  /** Source span as wire bytes (for labels / server round-trips). */
  sourceByteSpan: [number, number];
  /** Output range, CHAR offsets into the output file content. */
  outputRange: [number, number];
  /** Output byte length — drives ribbon thickness. */
  bytes: number;
  /** Palette index, stable per distinct source fragment: the same copy slot
   * pasted twice yields two ribbons of one color. */
  color: number;
}

/**
 * One ribbon per non-synthetic provenance entry whose origin is `docPath`.
 * Colors are assigned per distinct source fragment (identified by its doc
 * span) in order of first appearance, cycling through the palette.
 */
export function deriveRibbons(
  file: OutputFile,
  docPath: string,
  docSource: string,
): Ribbon[] {
  const fragmentColors = new Map<string, number>();
  const ribbons: Ribbon[] = [];
  file.provenance.forEach((p, i) => {
    if (p.origin.kind === "synthetic") return; // synthetic ranges: no ribbon
    if (!samePath(p.origin.doc_path, docPath)) return; // other docs' spans: no anchor here
    if (p.end <= p.start) return;
    const fragKey = `${p.origin.span[0]}:${p.origin.span[1]}`;
    let color = fragmentColors.get(fragKey);
    if (color === undefined) {
      color = fragmentColors.size % RIBBON_PALETTE_SIZE;
      fragmentColors.set(fragKey, color);
    }
    ribbons.push({
      key: `${file.path}:${i}`,
      kind: p.origin.kind,
      sourceSpan: [
        byteToChar(docSource, p.origin.span[0]),
        byteToChar(docSource, p.origin.span[1]),
      ],
      sourceByteSpan: p.origin.span,
      outputRange: [byteToChar(file.content, p.start), byteToChar(file.content, p.end)],
      bytes: p.end - p.start,
      color,
    });
  });
  return ribbons;
}
