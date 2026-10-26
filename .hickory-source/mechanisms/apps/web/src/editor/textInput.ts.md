<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/textInput.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b0fc8ae7492b"></a>
## `apps/web/src/editor/textInput.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b0fc8ae7492b-boundary"></a>
### `boundary`

Validate a requested source replacement before the actual editor pipeline runs.

<storycode:copy id="f-b0fc8ae7492b-boundary">
/** Validate a requested source replacement before the actual editor pipeline runs. */
function boundary(text: string, at: number) {
  return !(at > 0 && at < text.length && text.charCodeAt(at - 1) >= 0xd800 && text.charCodeAt(at - 1) <= 0xdbff && text.charCodeAt(at) >= 0xdc00 && text.charCodeAt(at) <= 0xdfff);
}
</storycode:copy>
<a id="f-b0fc8ae7492b-wellFormed"></a>
### `wellFormed`

<storycode:copy id="f-b0fc8ae7492b-wellFormed">
function wellFormed(text: string) {
  for (let i = 0; i < text.length; i++) {
    const unit = text.charCodeAt(i);
    if (unit >= 0xd800 && unit <= 0xdbff) { const low = text.charCodeAt(++i); if (!(low >= 0xdc00 && low <= 0xdfff)) return false; }
    else if (unit >= 0xdc00 && unit <= 0xdfff) return false;
  }
  return true;
}

</storycode:copy>
<a id="f-b0fc8ae7492b-prepareEditorChanges"></a>
### `prepareEditorChanges`

<storycode:copy id="f-b0fc8ae7492b-prepareEditorChanges">
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

</storycode:copy>
<a id="f-b0fc8ae7492b-prepareEditorInput"></a>
### `prepareEditorInput`

<storycode:copy id="f-b0fc8ae7492b-prepareEditorInput">
export function prepareEditorInput(source: string, ranges: readonly { from: number; to: number }[], text: string): { expected: string } | { reason: string } {
  return prepareEditorChanges(source, ranges.map(range => ({ ...range, insert: text })));
}
</storycode:copy>

### Assemble `apps/web/src/editor/textInput.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/textInput.ts"><storycode:paste select="#f-b0fc8ae7492b-boundary"/><storycode:paste select="#f-b0fc8ae7492b-wellFormed"/><storycode:paste select="#f-b0fc8ae7492b-prepareEditorChanges"/><storycode:paste select="#f-b0fc8ae7492b-prepareEditorInput"/></storycode:file>

</storycode:doc>
