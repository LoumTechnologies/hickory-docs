/** Validate a requested source replacement before the actual editor pipeline runs. */
function boundary(text: string, at: number) {
  return !(at > 0 && at < text.length && text.charCodeAt(at - 1) >= 0xd800 && text.charCodeAt(at - 1) <= 0xdbff && text.charCodeAt(at) >= 0xdc00 && text.charCodeAt(at) <= 0xdfff);
}
function wellFormed(text: string) {
  for (let i = 0; i < text.length; i++) {
    const unit = text.charCodeAt(i);
    if (unit >= 0xd800 && unit <= 0xdbff) { const low = text.charCodeAt(++i); if (!(low >= 0xdc00 && low <= 0xdfff)) return false; }
    else if (unit >= 0xdc00 && unit <= 0xdfff) return false;
  }
  return true;
}

export function prepareEditorInput(source: string, ranges: readonly { from: number; to: number }[], text: string): { expected: string } | { reason: string } {
  if (!wellFormed(text)) return { reason: "Replacement contains an unpaired UTF-16 surrogate; provide valid Unicode text" };
  if (new TextEncoder().encode(text).length * ranges.length > 1048576) return { reason: "Replacement exceeds the 1 MiB insertion budget; reduce text or select a smaller group" };
  let expected = "", cursor = 0;
  for (const range of ranges) {
    if (!Number.isInteger(range.from) || !Number.isInteger(range.to) || range.from < cursor || range.to < range.from || range.to > source.length) return { reason: "Replacement ranges must be ordered and inside the current source" };
    if (!boundary(source, range.from) || !boundary(source, range.to)) return { reason: "Range splits a UTF-16 surrogate pair; select the whole Unicode character" };
    expected += source.slice(cursor, range.from) + text; cursor = range.to;
  }
  expected += source.slice(cursor);
  return wellFormed(expected) ? { expected } : { reason: "Result would retain malformed Unicode; replace the malformed region before writing" };
}
