<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/offsets.test.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-dea322d4979a"></a>
## `apps/web/src/lib/offsets.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Trace conversation passages without confusing context with support](../../../../../guarantees/agent/an-answer-in-the-agent-pane-has-ribbons/contract.md), [A Keystroke Redraws Only The Editor](../../../../../guarantees/authoring/a-keystroke-redraws-only-the-editor/contract.md).

<a id="f-dea322d4979a-slowByteToChar"></a>
### `slowByteToChar`

— the ASCII fast path must answer exactly what the byte walk answers, or
a ribbon anchored through it lands on the wrong character.

<storycode:copy id="f-dea322d4979a-slowByteToChar">
// Guarantee: docs/guarantees/authoring/a-keystroke-redraws-only-the-editor.md
// — the ASCII fast path must answer exactly what the byte walk answers, or
// a ribbon anchored through it lands on the wrong character.
import { describe, expect, it } from "vitest";
import { byteLength, byteToChar, bytesToChars, charToByte } from "./offsets";

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

</storycode:copy>
<a id="f-dea322d4979a-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-dea322d4979a-expression_statement">
describe("byte ↔ char offsets", () => {
  // Guarantee: docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
  it("projects unordered repeated Unicode link endpoints without splitting surrogate pairs", () => {
    expect(bytesToChars("a📝漢λz", [8, 2, -1, 11, 1, 5, 50, 2, 0])).toEqual([4, 3, 0, 6, 1, 3, 6, 3, 0]);
    expect(bytesToChars("a📝漢λz", [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])).toEqual([0, 1, 3, 3, 3, 3, 4, 4, 4, 5, 5, 6]);
    expect(bytesToChars("abc", [3, -2, 1, 99])).toEqual([3, 0, 1, 3]);
    expect(bytesToChars("", [0, 10])).toEqual([0, 0]);
  });
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
</storycode:copy>

### Assemble `apps/web/src/lib/offsets.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/offsets.test.ts"><storycode:paste select="#f-dea322d4979a-slowByteToChar"/><storycode:paste select="#f-dea322d4979a-expression_statement"/></storycode:file>

</storycode:doc>
