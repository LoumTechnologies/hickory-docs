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
  /** The attributed bytes are whitespace only (blank lines, indentation
   * runs). True attribution, but no information until someone is working
   * exactly there — so the overlay draws these on hover, never by default. */
  whitespaceOnly: boolean;
}

/** Fragments of one source block share a colour and a terminal band. */
export function fragmentKey(ribbon: Ribbon): string {
  return `${ribbon.sourceByteSpan[0]}:${ribbon.sourceByteSpan[1]}`;
}

/**
 * Collapse a file's ribbons to one per SOURCE BLOCK, summing their bytes.
 *
 * Used when the ribbon cannot reach real output text (the file's tab is not
 * active, or the file is not open): one file that a block feeds is one
 * statement, not fifty, so the terminal gets one band per block. The kept
 * ribbon is the first of the group — its output range is where a click lands.
 */
export function groupBySourceBlock(
  ribbons: readonly Ribbon[],
): Map<string, { ribbon: Ribbon; bytes: number }> {
  const byBlock = new Map<string, { ribbon: Ribbon; bytes: number }>();
  for (const ribbon of ribbons) {
    const key = fragmentKey(ribbon);
    const seen = byBlock.get(key);
    if (seen) seen.bytes += ribbon.bytes;
    else byBlock.set(key, { ribbon, bytes: ribbon.bytes });
  }
  return byBlock;
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
      whitespaceOnly:
        file.content
          .slice(byteToChar(file.content, p.start), byteToChar(file.content, p.end))
          .trim().length === 0,
    });
  });
  return ribbons;
}

/**
 * Shrink a span to the lines it VISUALLY owns, for drawing.
 *
 * Ribbons and braces render line-by-line, but provenance is byte-precise: a
 * span often covers the newline (or trailing indentation) of a line whose
 * visible characters belong to a different span. Drawn at line granularity,
 * that whitespace inflates into a claim about the whole line — a brace
 * around `</hick:copy>` apparently feeding whitespace it never wrote. So a
 * line counts toward a span's drawn extent only when the span covers at
 * least one NON-whitespace character of it — or when the line has no
 * non-whitespace characters at all (a blank line belongs to whoever covers
 * it). Returns null when no line qualifies (a whitespace-only span; the
 * hover-only path owns those), and never widens: the result is clamped
 * inside the original span.
 */
export function drawnRange(
  text: string,
  from: number,
  to: number,
): [number, number] | null {
  const start = Math.max(0, Math.min(from, text.length));
  const end = Math.max(start, Math.min(to, text.length));
  if (start === end) return null;

  let first: [number, number] | null = null;
  let last: [number, number] | null = null;
  let lineStart = text.lastIndexOf("\n", start - 1) + 1;
  while (lineStart < end) {
    const newline = text.indexOf("\n", lineStart);
    const lineEnd = newline === -1 ? text.length : newline; // excl. the \n
    const coveredFrom = Math.max(start, lineStart);
    const coveredTo = Math.min(end, newline === -1 ? text.length : newline + 1);
    if (coveredFrom < coveredTo) {
      const covered = text.slice(coveredFrom, Math.min(coveredTo, lineEnd));
      const lineIsBlank = text.slice(lineStart, lineEnd).trim().length === 0;
      if (covered.trim().length > 0 || lineIsBlank) {
        const owned: [number, number] = [coveredFrom, coveredTo];
        first ??= owned;
        last = owned;
      }
    }
    if (newline === -1) break;
    lineStart = newline + 1;
  }
  return first && last ? [first[0], last[1]] : null;
}
