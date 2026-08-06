// The server contract (docs/specs/freeform/api.md) expresses provenance and
// output-edit ranges in BYTE offsets (UTF-8); JavaScript strings index UTF-16
// code units. These helpers convert between the two.

/** UTF-8 byte length of the code point starting at UTF-16 index `i`. */
function byteLenAt(text: string, i: number): { bytes: number; units: number } {
  const cp = text.codePointAt(i)!;
  if (cp < 0x80) return { bytes: 1, units: 1 };
  if (cp < 0x800) return { bytes: 2, units: 1 };
  if (cp < 0x10000) return { bytes: 3, units: 1 };
  return { bytes: 4, units: 2 };
}

/** UTF-8 byte offset of UTF-16 index `charIndex` in `text`. */
export function charToByte(text: string, charIndex: number): number {
  let bytes = 0;
  let i = 0;
  while (i < charIndex && i < text.length) {
    const { bytes: b, units } = byteLenAt(text, i);
    bytes += b;
    i += units;
  }
  return bytes;
}

/** UTF-16 index for UTF-8 byte offset `byteIndex` in `text` (clamped). */
export function byteToChar(text: string, byteIndex: number): number {
  let bytes = 0;
  let i = 0;
  while (i < text.length) {
    if (bytes >= byteIndex) return i;
    const { bytes: b, units } = byteLenAt(text, i);
    bytes += b;
    i += units;
  }
  return text.length;
}

/** Total UTF-8 byte length of `text`. */
export function byteLength(text: string): number {
  return charToByte(text, text.length);
}
