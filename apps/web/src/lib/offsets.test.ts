// Guarantee: docs/guarantees/authoring/a-keystroke-redraws-only-the-editor.md
// — the ASCII fast path must answer exactly what the byte walk answers, or
// a ribbon anchored through it lands on the wrong character.
import { describe, expect, it } from "vitest";
import { byteLength, byteToChar, charToByte } from "./offsets";

/** The plain walk, kept here as the oracle the fast path is checked against. */
function slowByteToChar(text: string, byteIndex: number): number {
  let bytes = 0;
  let i = 0;
  while (i < text.length) {
    if (bytes >= byteIndex) return i;
    const cp = text.codePointAt(i)!;
    bytes += cp < 0x80 ? 1 : cp < 0x800 ? 2 : cp < 0x10000 ? 3 : 4;
    i += cp >= 0x10000 ? 2 : 1;
  }
  return text.length;
}

describe("byte ↔ char offsets", () => {
  it("agrees with the byte walk on pure ASCII, where the two scales coincide", () => {
    const text = "def f(x):\n    return x + 1\n";
    for (let b = 0; b <= text.length + 2; b++) {
      expect(byteToChar(text, b)).toBe(slowByteToChar(text, b));
    }
    expect(charToByte(text, 5)).toBe(5);
    expect(charToByte(text, 999)).toBe(text.length);
    expect(byteLength(text)).toBe(text.length);
  });

  it("still walks bytes the moment a text holds anything outside ASCII", () => {
    const text = "héllo — 日本 🎉 end";
    for (let b = 0; b <= byteLength(text) + 2; b++) {
      expect(byteToChar(text, b)).toBe(slowByteToChar(text, b));
    }
    // "h" is one byte, "é" two: char 2 starts at byte 3.
    expect(charToByte(text, 2)).toBe(3);
    expect(byteToChar(text, 3)).toBe(2);
    // The emoji is one code point, two UTF-16 units, four bytes.
    const emoji = text.indexOf("🎉");
    expect(charToByte(text, emoji + 2) - charToByte(text, emoji)).toBe(4);
  });

  it("does not confuse two texts of the same length", () => {
    // The ASCII answer is remembered per text; a non-ASCII text of the same
    // shape asked next must not inherit it.
    expect(byteToChar("abcd", 3)).toBe(3);
    // "日" is bytes 2..5; a byte inside a character answers the boundary
    // after it, as the walk always has.
    expect(byteToChar("ab日d", 3)).toBe(3);
    expect(byteToChar("ab日d", 5)).toBe(3);
    expect(byteToChar("ab日d", 2)).toBe(2);
  });
});
