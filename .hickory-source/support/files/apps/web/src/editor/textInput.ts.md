<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/textInput.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `dd4005ddb6d69562599286a00af0dc6d115f17d6590e3278cbdaf3b496447e9b`.

<book:entry path="apps/web/src/editor/textInput.ts">
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

export function prepareEditorChanges(source: string, changes: readonly { from: number; to: number; insert: string }[]): { expected: string } | { reason: string } {
  let expected = "", cursor = 0, inserted = 0;
  for (const change of changes) {
    if (!wellFormed(change.insert)) return { reason: "Replacement contains an unpaired UTF-16 surrogate; provide valid Unicode text" };
    inserted += new TextEncoder().encode(change.insert).length;
    if (inserted > 1048576) return { reason: "Replacement exceeds the 1 MiB insertion budget; reduce text or select a smaller group" };
    if (!Number.isInteger(change.from) || !Number.isInteger(change.to) || change.from < cursor || change.to < change.from || change.to > source.length) return { reason: "Replacement ranges must be ordered and inside the current source" };
    if (!boundary(source, change.from) || !boundary(source, change.to)) return { reason: "Range splits a UTF-16 surrogate pair; select the whole Unicode character" };
    expected += source.slice(cursor, change.from) + change.insert; cursor = change.to;
  }
  expected += source.slice(cursor);
  return wellFormed(expected) ? { expected } : { reason: "Result would retain malformed Unicode; replace the malformed region before writing" };
}

export function prepareEditorInput(source: string, ranges: readonly { from: number; to: number }[], text: string): { expected: string } | { reason: string } {
  return prepareEditorChanges(source, ranges.map(range => ({ ...range, insert: text })));
}
</book:entry>

</book:doc>
