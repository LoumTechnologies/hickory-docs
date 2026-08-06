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

export interface LspRange {
  start: LspPosition;
  end: LspPosition;
}

function utf8Len(codePoint: number): number {
  if (codePoint < 0x80) return 1;
  if (codePoint < 0x800) return 2;
  if (codePoint < 0x10000) return 3;
  return 4;
}

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

/** UTF-8 byte offset of an LSP position. */
export function positionToByteOffset(text: string, pos: LspPosition): number {
  return utf16ToByteOffset(text, positionToUtf16(text, pos));
}

/** LSP position of a UTF-8 byte offset. */
export function byteOffsetToPosition(text: string, byteOffset: number): LspPosition {
  return utf16ToPosition(text, byteOffsetToUtf16(text, byteOffset));
}

/** LSP range for a byte span. */
export function byteSpanToRange(text: string, start: number, end: number): LspRange {
  return {
    start: byteOffsetToPosition(text, start),
    end: byteOffsetToPosition(text, end),
  };
}

/** Byte span of an LSP range. */
export function rangeToByteSpan(text: string, range: LspRange): [number, number] {
  return [positionToByteOffset(text, range.start), positionToByteOffset(text, range.end)];
}
