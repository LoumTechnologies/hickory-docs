<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Keystroke Redraws Only The Editor

For: Authors editing executable documents and the files those documents produce.

Given a document open in the workspace beside any number of other panes, when
a character is typed into it, then the only React render that keystroke causes
is inside the editor pane that received it — the workspace chrome (the other
panes, the folder tree, the rulers, the rails, the ribbon overlay, the status
bar's other fields) is not re-rendered, and nothing that was already on screen
is re-measured against the DOM.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-keystroke-redraws-only-the-editor.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-8f044116e767"></a>
## `apps/web/src/lib/offsets.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Keystroke Redraws Only The Editor](contract.md).

<a id="f-8f044116e767-byteLenAt"></a>
### `byteLenAt`

The server contract (docs/specs/freeform/api.md) expresses provenance and
output-edit ranges in BYTE offsets (UTF-8); JavaScript strings index UTF-16
code units. These helpers convert between the two.
UTF-8 byte length of the code point starting at UTF-16 index `i`.

<storycode:copy id="f-8f044116e767-byteLenAt">
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

</storycode:copy>
<a id="f-8f044116e767-asciiCache"></a>
### `asciiCache`

Whether a text is pure ASCII, remembered for the last few texts seen. In
ASCII, byte offsets and UTF-16 offsets are the same number, and the
provenance of one file asks this question hundreds of times per redraw
against the same two strings — so the answer is worth keeping. The regex is
native and linear; the walk it replaces is interpreted and quadratic over a
file's provenance list. Keyed by the string itself: V8 caches a string's
hash, so a repeated lookup of the same (large) string is a pointer check.

<storycode:copy id="f-8f044116e767-asciiCache">
// Whether a text is pure ASCII, remembered for the last few texts seen. In
// ASCII, byte offsets and UTF-16 offsets are the same number, and the
// provenance of one file asks this question hundreds of times per redraw
// against the same two strings — so the answer is worth keeping. The regex is
// native and linear; the walk it replaces is interpreted and quadratic over a
// file's provenance list. Keyed by the string itself: V8 caches a string's
// hash, so a repeated lookup of the same (large) string is a pointer check.
const asciiCache = new Map<string, boolean>();
</storycode:copy>
<a id="f-8f044116e767-ASCII_CACHE_SIZE"></a>
### `ASCII_CACHE_SIZE`

<storycode:copy id="f-8f044116e767-ASCII_CACHE_SIZE">
const ASCII_CACHE_SIZE = 16;
</storycode:copy>
<a id="f-8f044116e767-isAscii"></a>
### `isAscii`

<storycode:copy id="f-8f044116e767-isAscii">
function isAscii(text: string): boolean {
  let known = asciiCache.get(text);
  if (known === undefined) {
    // eslint-disable-next-line no-control-regex
    known = !/[^\x00-\x7f]/.test(text);
    if (asciiCache.size >= ASCII_CACHE_SIZE) {
      asciiCache.delete(asciiCache.keys().next().value as string);
    }
    asciiCache.set(text, known);
  }
  return known;
}

</storycode:copy>
<a id="f-8f044116e767-charToByte"></a>
### `charToByte`

UTF-8 byte offset of UTF-16 index `charIndex` in `text`.

<storycode:copy id="f-8f044116e767-charToByte">
/** UTF-8 byte offset of UTF-16 index `charIndex` in `text`. */
export function charToByte(text: string, charIndex: number): number {
  if (isAscii(text)) return Math.max(0, Math.min(charIndex, text.length));
  let bytes = 0;
  let i = 0;
  while (i < charIndex && i < text.length) {
    const { bytes: b, units } = byteLenAt(text, i);
    bytes += b;
    i += units;
  }
  return bytes;
}

</storycode:copy>
<a id="f-8f044116e767-byteToChar"></a>
### `byteToChar`

UTF-16 index for UTF-8 byte offset `byteIndex` in `text` (clamped).

<storycode:copy id="f-8f044116e767-byteToChar">
/** UTF-16 index for UTF-8 byte offset `byteIndex` in `text` (clamped). */
export function byteToChar(text: string, byteIndex: number): number {
  if (isAscii(text)) return Math.max(0, Math.min(byteIndex, text.length));
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

</storycode:copy>
<a id="f-8f044116e767-bytesToChars"></a>
### `bytesToChars`

Convert a provenance list in one forward scan, retaining input order.
Repeated byteToChar calls rescan a Unicode session from its beginning for
every link, which can block token-bound UI replies as a session grows.

<storycode:copy id="f-8f044116e767-bytesToChars">
/** Convert a provenance list in one forward scan, retaining input order.
 * Repeated byteToChar calls rescan a Unicode session from its beginning for
 * every link, which can block token-bound UI replies as a session grows. */
export function bytesToChars(text: string, offsets: readonly number[]): number[] {
  if (isAscii(text)) return offsets.map(offset => Math.max(0, Math.min(offset, text.length)));
  const ordered = offsets.map((offset, index) => ({ offset: Number.isNaN(offset) ? Infinity : offset, index }))
    .sort((left, right) => left.offset - right.offset);
  const result = new Array<number>(offsets.length);
  let bytes = 0, units = 0;
  for (const target of ordered) {
    while (units < text.length && bytes < target.offset) {
      const next = byteLenAt(text, units);
      bytes += next.bytes; units += next.units;
    }
    result[target.index] = units;
  }
  return result;
}

</storycode:copy>
<a id="f-8f044116e767-byteLength"></a>
### `byteLength`

<storycode:copy id="f-8f044116e767-byteLength">
/** Total UTF-8 byte length of `text`. */
export function byteLength(text: string): number {
  return charToByte(text, text.length);
}
</storycode:copy>

### Assemble `apps/web/src/lib/offsets.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/offsets.ts"><storycode:paste select="#f-8f044116e767-byteLenAt"/><storycode:paste select="#f-8f044116e767-asciiCache"/><storycode:paste select="#f-8f044116e767-ASCII_CACHE_SIZE"/><storycode:paste select="#f-8f044116e767-isAscii"/><storycode:paste select="#f-8f044116e767-charToByte"/><storycode:paste select="#f-8f044116e767-byteToChar"/><storycode:paste select="#f-8f044116e767-bytesToChars"/><storycode:paste select="#f-8f044116e767-byteLength"/></storycode:file>

<a id="output-22de632562cd"></a>
## `apps/web/src/views/sessionRegistry.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Keystroke Redraws Only The Editor](contract.md).

<a id="f-22de632562cd-session"></a>
### `session`

— a session hook re-runs on every keystroke; the registry must not tell the
workspace about a publish whose every field is the one it already holds.

<storycode:copy id="f-22de632562cd-session">
// Guarantee: docs/guarantees/authoring/a-keystroke-redraws-only-the-editor.md
// — a session hook re-runs on every keystroke; the registry must not tell the
// workspace about a publish whose every field is the one it already holds.
import { describe, expect, it, vi } from "vitest";
import { SessionRegistry, type DocSession } from "./documentSession";

const session = (over: Partial<DocSession> = {}): DocSession =>
  ({ docId: "d1", doc: null, syncState: "idle", ...over }) as unknown as DocSession;

</storycode:copy>
<a id="f-22de632562cd-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-22de632562cd-expression_statement">
describe("the session registry", () => {
  it("emits on the first publish and on a real change", () => {
    const registry = new SessionRegistry();
    const heard = vi.fn();
    registry.subscribe(heard);
    registry.publish(session());
    expect(heard).toHaveBeenCalledTimes(1);
    registry.publish(session({ syncState: "editing" }));
    expect(heard).toHaveBeenCalledTimes(2);
    expect(registry.get("d1")?.syncState).toBe("editing");
  });

  it("stays silent when a fresh object carries the same fields", () => {
    const registry = new SessionRegistry();
    const heard = vi.fn();
    registry.subscribe(heard);
    const first = session();
    registry.publish(first);
    const version = registry.version;
    registry.publish(session()); // a new object, the same session
    expect(heard).toHaveBeenCalledTimes(1);
    expect(registry.version).toBe(version);
    // And the held object is the first one: a subscriber reading the snapshot
    // sees a stable identity, which is what lets it skip its own render.
    expect(registry.get("d1")).toBe(first);
  });

  it("notices a field added or removed, not only one replaced", () => {
    const registry = new SessionRegistry();
    const heard = vi.fn();
    registry.subscribe(heard);
    registry.publish(session());
    registry.publish(session({ banner: { kind: "pass", text: "ok" } }));
    expect(heard).toHaveBeenCalledTimes(2);
  });
});
</storycode:copy>

### Assemble `apps/web/src/views/sessionRegistry.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/sessionRegistry.test.ts"><storycode:paste select="#f-22de632562cd-session"/><storycode:paste select="#f-22de632562cd-expression_statement"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/debug/useDebugger.ts](../../../mechanisms/apps/web/src/debug/useDebugger.ts.md#output-597e991b795b)
- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/editor/EditorRuler.tsx](../../../mechanisms/apps/web/src/editor/EditorRuler.tsx.md#output-75b1988d3607)
- [apps/web/src/editor/embedded.ts](../../../mechanisms/apps/web/src/editor/embedded.ts.md#output-df9508b237a8)
- [apps/web/src/lib/offsets.ts](contract.md#output-8f044116e767)
- [apps/web/src/shell/Ribbons.tsx](../../../mechanisms/apps/web/src/shell/Ribbons.tsx.md#output-b916e85d3b0b)
- apps/web/src/shell/StatusBar.tsx
- apps/web/src/views/WorkspaceView.tsx
- [apps/web/src/views/documentSession.tsx](../../../mechanisms/apps/web/src/views/documentSession.tsx.md#output-bfb59577f48b)

### Tests and fixtures

- [apps/web/src/editor/embedded.test.ts](../../../shared/files/apps/web/src/editor/embedded.test.ts.md)
- [apps/web/src/lib/offsets.test.ts](../../../mechanisms/apps/web/src/lib/offsets.test.ts.md#output-dea322d4979a)
- apps/web/src/shell/StatusBar.test.tsx
- [apps/web/src/views/sessionRegistry.test.ts](contract.md#output-22de632562cd)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
