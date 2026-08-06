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
