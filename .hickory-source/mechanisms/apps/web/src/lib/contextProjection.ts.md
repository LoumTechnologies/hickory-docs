<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/contextProjection.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5194716bada7"></a>
## `apps/web/src/lib/contextProjection.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Selected reading scope remains separate from complete edit state](../../../../../guarantees/agent/expanded-reading-projection/contract.md), [Select model context without saving open buffers](../../../../../guarantees/agent/the-agent-sees-the-open-editors/contract.md).

<a id="f-5194716bada7-ReadingProjection"></a>
### `ReadingProjection`

<storycode:copy id="f-5194716bada7-ReadingProjection">
import { foldedRanges, foldEffect } from "@codemirror/language";
import type { EditorState } from "@codemirror/state";
import type { EditorView } from "@codemirror/view";

export interface ReadingProjection {
  version: 1;
  documents: { id: string; revision: string; coordinate_unit: "utf8_bytes"; selection: "expanded" | "pinned" | "unavailable"; segments: { start: number; end: number; line_start: number; line_end: number; text: string; omitted: boolean }[] }[];
  observations: unknown[];
  estimate: { utf8_bytes: number; approximate_tokens: number; method: string };
}
</storycode:copy>
<a id="f-5194716bada7-projectDocument"></a>
### `projectDocument`

Project the supplied document revision, not the viewport. Hash its exact content, translate editor ranges into UTF-8 byte offsets and emit omitted segments without their text. A stale or unavailable editor hides the document unless it was explicitly pinned; a pin bypasses folds.

<storycode:copy id="f-5194716bada7-projectDocument">
/** Folds select source bytes, never viewport pixels or generated annotation rows. */
export async function projectDocument(id: string, content: string, view?: Pick<EditorView, "state">, pinned = false) {
  const state = view?.state;
  const available = state?.doc.toString() === content;
  const encode = (s: string) => new TextEncoder().encode(s);
  const hash = await crypto.subtle.digest("SHA-256", encode(content));
  const revision = `sha256:${Array.from(new Uint8Array(hash), b => b.toString(16).padStart(2, "0")).join("")}`;
  const hidden: { from: number; to: number }[] = [];
  if (!available && !pinned) hidden.push({ from: 0, to: content.length });
  if (state && available && !pinned) foldedRanges(state).between(0, content.length, (from, to) => { hidden.push({ from, to }); });
  const ranges: { from: number; to: number }[] = [];
  for (const range of hidden.sort((a, b) => a.from - b.from)) {
    const last = ranges.at(-1);
    if (last && range.from <= last.to) last.to = Math.max(last.to, range.to);
    else ranges.push({ ...range });
  }
  const segments: ReadingProjection["documents"][number]["segments"] = [];
  const add = (from: number, to: number, omitted: boolean) => {
    if (to <= from) return;
    segments.push({ start: encode(content.slice(0, from)).length, end: encode(content.slice(0, to)).length,
      line_start: content.slice(0, from).split("\n").length, line_end: content.slice(0, to).split("\n").length,
      text: omitted ? "" : content.slice(from, to), omitted });
  };
  let at = 0;
  for (const range of ranges) { add(at, range.from, false); add(range.from, range.to, true); at = range.to; }
  add(at, content.length, false);
  return { id, revision, selection: pinned ? "pinned" as const : available ? "expanded" as const : "unavailable" as const, coordinate_unit: "utf8_bytes" as const, segments };
}
</storycode:copy>
<a id="f-5194716bada7-projectionEstimate"></a>
### `projectionEstimate`

Label the estimate as a UTF-8-bytes/4 heuristic. It measures this serialized projection, not provider tokenization or the complete request, so the UI cannot present it as billed usage.

<storycode:copy id="f-5194716bada7-projectionEstimate">
export function projectionEstimate(value: unknown) {
  const bytes = new TextEncoder().encode(JSON.stringify(value)).length;
  return { utf8_bytes: bytes, approximate_tokens: Math.ceil(bytes / 4), method: "UTF-8 bytes / 4 heuristic; not provider tokenization or total request usage" };
}

</storycode:copy>
<a id="f-5194716bada7-retained"></a>
### `retained`

Keep reading intent when tab rendering unmounts an editor. Equal bytes in two
documents never share a key. Changed text makes the old fold mapping unavailable.

<storycode:copy id="f-5194716bada7-retained">
// Keep reading intent when tab rendering unmounts an editor. Equal bytes in two
// documents never share a key. Changed text makes the old fold mapping unavailable.
const retained = new Map<string, EditorState>();
</storycode:copy>
<a id="f-5194716bada7-rememberReadingState"></a>
### `rememberReadingState`

<storycode:copy id="f-5194716bada7-rememberReadingState">
export function rememberReadingState(key: string, view?: EditorView | null) {
  if (view) retained.set(key, view.state);
}
</storycode:copy>
<a id="f-5194716bada7-retainedReadingState"></a>
### `retainedReadingState`

<storycode:copy id="f-5194716bada7-retainedReadingState">
export function retainedReadingState(key: string) {
  const state = retained.get(key);
  return state ? { state } : undefined;
}
</storycode:copy>
<a id="f-5194716bada7-restoreReadingState"></a>
### `restoreReadingState`

Restore folds only when the retained and current editor documents have equal text. Replay fold effects rather than installing old editor state, so stale coordinates cannot hide unrelated new source.

<storycode:copy id="f-5194716bada7-restoreReadingState">
export function restoreReadingState(key: string, view: EditorView) {
  const state = retained.get(key);
  if (!state || !state.doc.eq(view.state.doc)) return;
  const effects: ReturnType<typeof foldEffect.of>[] = [];
  foldedRanges(state).between(0, state.doc.length, (from, to) => { effects.push(foldEffect.of({ from, to })); });
  if (effects.length) view.dispatch({ effects });
}
</storycode:copy>

### Assemble `apps/web/src/lib/contextProjection.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/contextProjection.ts"><storycode:paste select="#f-5194716bada7-ReadingProjection"/><storycode:paste select="#f-5194716bada7-projectDocument"/><storycode:paste select="#f-5194716bada7-projectionEstimate"/><storycode:paste select="#f-5194716bada7-retained"/><storycode:paste select="#f-5194716bada7-rememberReadingState"/><storycode:paste select="#f-5194716bada7-retainedReadingState"/><storycode:paste select="#f-5194716bada7-restoreReadingState"/></storycode:file>

</storycode:doc>
