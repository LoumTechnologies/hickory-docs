<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lsp/positions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-86e96c7d2d4f"></a>
## `apps/web/src/lsp/positions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-86e96c7d2d4f-LspPosition"></a>
### `LspPosition`

Position math for the LSP bridge (api.md "Editor intelligence — LSP bridge
(v0.3)"): conversions between
- LSP positions ({line, character} with character in UTF-16 code units),
- UTF-16 string indices (what JS strings and the editor use),
- UTF-8 byte offsets (what spans/provenance in the API use),
all computed over one source string.

<storycode:copy id="f-86e96c7d2d4f-LspPosition">
// Position math for the LSP bridge (api.md "Editor intelligence — LSP bridge
// (v0.3)"): conversions between
//   - LSP positions ({line, character} with character in UTF-16 code units),
//   - UTF-16 string indices (what JS strings and the editor use),
//   - UTF-8 byte offsets (what spans/provenance in the API use),
// all computed over one source string.

export interface LspPosition {
  line: number;
  character: number;
}

</storycode:copy>
<a id="f-86e96c7d2d4f-LspRange"></a>
### `LspRange`

<storycode:copy id="f-86e96c7d2d4f-LspRange">
export interface LspRange {
  start: LspPosition;
  end: LspPosition;
}

</storycode:copy>
<a id="f-86e96c7d2d4f-utf8Len"></a>
### `utf8Len`

<storycode:copy id="f-86e96c7d2d4f-utf8Len">
function utf8Len(codePoint: number): number {
  if (codePoint < 0x80) return 1;
  if (codePoint < 0x800) return 2;
  if (codePoint < 0x10000) return 3;
  return 4;
}

</storycode:copy>
<a id="f-86e96c7d2d4f-positionToUtf16"></a>
### `positionToUtf16`

UTF-16 index of an LSP position. Out-of-range positions clamp (per LSP).

<storycode:copy id="f-86e96c7d2d4f-positionToUtf16">
/** UTF-16 index of an LSP position. Out-of-range positions clamp (per LSP). */
export function positionToUtf16(text: string, pos: LspPosition): number {
  let lineStart = 0;
  for (let line = 0; line < pos.line; line++) {
    const nl = text.indexOf("\n", lineStart);
    if (nl === -1) return text.length;
    lineStart = nl + 1;
  }
  const nl = text.indexOf("\n", lineStart);
  const lineEnd = nl === -1 ? text.length : nl;
  return Math.min(lineStart + pos.character, lineEnd);
}

</storycode:copy>
<a id="f-86e96c7d2d4f-utf16ToPosition"></a>
### `utf16ToPosition`

LSP position of a UTF-16 index. Indices past the end clamp.

<storycode:copy id="f-86e96c7d2d4f-utf16ToPosition">
/** LSP position of a UTF-16 index. Indices past the end clamp. */
export function utf16ToPosition(text: string, index: number): LspPosition {
  const i = Math.max(0, Math.min(index, text.length));
  let line = 0;
  let lineStart = 0;
  for (let j = 0; j < i; j++) {
    if (text.charCodeAt(j) === 10) {
      line++;
      lineStart = j + 1;
    }
  }
  return { line, character: i - lineStart };
}

</storycode:copy>
<a id="f-86e96c7d2d4f-utf16ToByteOffset"></a>
### `utf16ToByteOffset`

<storycode:copy id="f-86e96c7d2d4f-utf16ToByteOffset">
/** UTF-8 byte offset of a UTF-16 index. */
export function utf16ToByteOffset(text: string, index: number): number {
  const end = Math.max(0, Math.min(index, text.length));
  let bytes = 0;
  let i = 0;
  while (i < end) {
    const cp = text.codePointAt(i)!;
    const units = cp > 0xffff ? 2 : 1;
    // A surrogate half split by `index` counts as its own 3-byte unit; in
    // practice indices come from the editor and sit on code-point boundaries.
    if (i + units > end) return bytes + 3;
    bytes += utf8Len(cp);
    i += units;
  }
  return bytes;
}

</storycode:copy>
<a id="f-86e96c7d2d4f-byteOffsetToUtf16"></a>
### `byteOffsetToUtf16`

UTF-16 index of a UTF-8 byte offset (clamps into the string).

<storycode:copy id="f-86e96c7d2d4f-byteOffsetToUtf16">
/** UTF-16 index of a UTF-8 byte offset (clamps into the string). */
export function byteOffsetToUtf16(text: string, byteOffset: number): number {
  let bytes = 0;
  let i = 0;
  while (i < text.length) {
    if (bytes >= byteOffset) return i;
    const cp = text.codePointAt(i)!;
    bytes += utf8Len(cp);
    i += cp > 0xffff ? 2 : 1;
  }
  return text.length;
}

</storycode:copy>
<a id="f-86e96c7d2d4f-positionToByteOffset"></a>
### `positionToByteOffset`

<storycode:copy id="f-86e96c7d2d4f-positionToByteOffset">
/** UTF-8 byte offset of an LSP position. */
export function positionToByteOffset(text: string, pos: LspPosition): number {
  return utf16ToByteOffset(text, positionToUtf16(text, pos));
}

</storycode:copy>
<a id="f-86e96c7d2d4f-byteOffsetToPosition"></a>
### `byteOffsetToPosition`

<storycode:copy id="f-86e96c7d2d4f-byteOffsetToPosition">
/** LSP position of a UTF-8 byte offset. */
export function byteOffsetToPosition(text: string, byteOffset: number): LspPosition {
  return utf16ToPosition(text, byteOffsetToUtf16(text, byteOffset));
}

</storycode:copy>
<a id="f-86e96c7d2d4f-byteSpanToRange"></a>
### `byteSpanToRange`

<storycode:copy id="f-86e96c7d2d4f-byteSpanToRange">
/** LSP range for a byte span. */
export function byteSpanToRange(text: string, start: number, end: number): LspRange {
  return {
    start: byteOffsetToPosition(text, start),
    end: byteOffsetToPosition(text, end),
  };
}

</storycode:copy>
<a id="f-86e96c7d2d4f-rangeToByteSpan"></a>
### `rangeToByteSpan`

<storycode:copy id="f-86e96c7d2d4f-rangeToByteSpan">
/** Byte span of an LSP range. */
export function rangeToByteSpan(text: string, range: LspRange): [number, number] {
  return [positionToByteOffset(text, range.start), positionToByteOffset(text, range.end)];
}
</storycode:copy>

### Assemble `apps/web/src/lsp/positions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/positions.ts"><storycode:paste select="#f-86e96c7d2d4f-LspPosition"/><storycode:paste select="#f-86e96c7d2d4f-LspRange"/><storycode:paste select="#f-86e96c7d2d4f-utf8Len"/><storycode:paste select="#f-86e96c7d2d4f-positionToUtf16"/><storycode:paste select="#f-86e96c7d2d4f-utf16ToPosition"/><storycode:paste select="#f-86e96c7d2d4f-utf16ToByteOffset"/><storycode:paste select="#f-86e96c7d2d4f-byteOffsetToUtf16"/><storycode:paste select="#f-86e96c7d2d4f-positionToByteOffset"/><storycode:paste select="#f-86e96c7d2d4f-byteOffsetToPosition"/><storycode:paste select="#f-86e96c7d2d4f-byteSpanToRange"/><storycode:paste select="#f-86e96c7d2d4f-rangeToByteSpan"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-b4ff4dd97222"></a>
## `apps/web/src/lsp/positions.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b4ff4dd97222-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b4ff4dd97222-expression_statement">
import { describe, expect, it } from "vitest";
import {
  byteOffsetToPosition,
  byteOffsetToUtf16,
  byteSpanToRange,
  positionToByteOffset,
  positionToUtf16,
  rangeToByteSpan,
  utf16ToByteOffset,
  utf16ToPosition,
} from "./positions";

describe("LSP position <-> UTF-16 index", () => {
  const text = "abc\ndef\nghi";

  it("maps positions to indices", () => {
    expect(positionToUtf16(text, { line: 0, character: 0 })).toBe(0);
    expect(positionToUtf16(text, { line: 1, character: 2 })).toBe(6);
    expect(positionToUtf16(text, { line: 2, character: 3 })).toBe(11);
  });

  it("clamps beyond line and document ends", () => {
    expect(positionToUtf16(text, { line: 0, character: 99 })).toBe(3);
    expect(positionToUtf16(text, { line: 99, character: 0 })).toBe(text.length);
  });

  it("maps indices to positions", () => {
    expect(utf16ToPosition(text, 0)).toEqual({ line: 0, character: 0 });
    expect(utf16ToPosition(text, 6)).toEqual({ line: 1, character: 2 });
    expect(utf16ToPosition(text, 999)).toEqual({ line: 2, character: 3 });
  });

  it("round-trips every index", () => {
    for (let i = 0; i <= text.length; i++) {
      expect(positionToUtf16(text, utf16ToPosition(text, i))).toBe(i);
    }
  });
});

</storycode:copy>
<a id="f-b4ff4dd97222-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-b4ff4dd97222-expression_statement-2">
describe("UTF-16 index <-> UTF-8 byte offset", () => {
  // "€" = 3 bytes / 1 unit, "𝄞" = 4 bytes / 2 units (surrogate pair).
  const text = "a€b\n𝄞c";

  it("counts UTF-8 bytes", () => {
    expect(utf16ToByteOffset(text, 0)).toBe(0);
    expect(utf16ToByteOffset(text, 1)).toBe(1); // after "a"
    expect(utf16ToByteOffset(text, 2)).toBe(4); // after "€"
    expect(utf16ToByteOffset(text, 4)).toBe(6); // after "\n"
    expect(utf16ToByteOffset(text, 6)).toBe(10); // after "𝄞"
    expect(utf16ToByteOffset(text, 7)).toBe(11);
  });

  it("inverts byte offsets", () => {
    expect(byteOffsetToUtf16(text, 0)).toBe(0);
    expect(byteOffsetToUtf16(text, 4)).toBe(2);
    expect(byteOffsetToUtf16(text, 10)).toBe(6);
    expect(byteOffsetToUtf16(text, 999)).toBe(text.length);
  });

  it("matches TextEncoder for every boundary", () => {
    const enc = new TextEncoder();
    for (let i = 0; i <= text.length; i++) {
      // Skip the middle of the surrogate pair (not a code-point boundary).
      if (i > 0 && text.charCodeAt(i - 1) >= 0xd800 && text.charCodeAt(i - 1) < 0xdc00) continue;
      expect(utf16ToByteOffset(text, i)).toBe(enc.encode(text.slice(0, i)).length);
    }
  });
});

</storycode:copy>
<a id="f-b4ff4dd97222-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-b4ff4dd97222-expression_statement-3">
describe("LSP position <-> byte offset composites", () => {
  const text = "x\n€€\nend";

  it("position to byte offset and back", () => {
    const pos = { line: 1, character: 1 }; // between the two €
    const byte = positionToByteOffset(text, pos);
    expect(byte).toBe(5); // "x\n" (2) + "€" (3)
    expect(byteOffsetToPosition(text, byte)).toEqual(pos);
  });

  it("byte spans to ranges and back", () => {
    const start = text.indexOf("end"); // ASCII prefix bytes == chars until here? no: € are 3 bytes
    void start;
    const span: [number, number] = [9, 12]; // "end" starts at byte 2+6+1=9
    const range = byteSpanToRange(text, span[0], span[1]);
    expect(range.start).toEqual({ line: 2, character: 0 });
    expect(range.end).toEqual({ line: 2, character: 3 });
    expect(rangeToByteSpan(text, range)).toEqual(span);
  });
});
</storycode:copy>

### Assemble `apps/web/src/lsp/positions.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/positions.test.ts"><storycode:paste select="#f-b4ff4dd97222-expression_statement"/><storycode:paste select="#f-b4ff4dd97222-expression_statement-2"/><storycode:paste select="#f-b4ff4dd97222-expression_statement-3"/></storycode:file>


</storycode:doc>
