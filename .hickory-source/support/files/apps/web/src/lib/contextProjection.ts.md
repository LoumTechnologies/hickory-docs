<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/contextProjection.ts

For: maintainers changing this file and reviewers following the related requirements.

Each file has one constructor. These links establish navigation, not verified behavioral coverage.

- [Selected reading scope remains separate from complete edit state](../../../../../../guarantees/agent/expanded-reading-projection/contract.md)
- [Select model context without saving open buffers](../../../../../../guarantees/agent/the-agent-sees-the-open-editors/contract.md)

<book:entry path="apps/web/src/lib/contextProjection.ts">
import { foldedRanges, foldEffect } from "@codemirror/language";
import type { EditorState } from "@codemirror/state";
import type { EditorView } from "@codemirror/view";

export interface ReadingProjection {
  version: 1;
  documents: { id: string; revision: string; coordinate_unit: "utf8_bytes"; selection: "expanded" | "pinned" | "unavailable"; segments: { start: number; end: number; line_start: number; line_end: number; text: string; omitted: boolean }[] }[];
  observations: unknown[];
  estimate: { utf8_bytes: number; approximate_tokens: number; method: string };
}
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
export function projectionEstimate(value: unknown) {
  const bytes = new TextEncoder().encode(JSON.stringify(value)).length;
  return { utf8_bytes: bytes, approximate_tokens: Math.ceil(bytes / 4), method: "UTF-8 bytes / 4 heuristic; not provider tokenization or total request usage" };
}

// Keep reading intent when tab rendering unmounts an editor. Equal bytes in two
// documents never share a key. Changed text makes the old fold mapping unavailable.
const retained = new Map<string, EditorState>();
export function rememberReadingState(key: string, view?: EditorView | null) {
  if (view) retained.set(key, view.state);
}
export function retainedReadingState(key: string) {
  const state = retained.get(key);
  return state ? { state } : undefined;
}
export function restoreReadingState(key: string, view: EditorView) {
  const state = retained.get(key);
  if (!state || !state.doc.eq(view.state.doc)) return;
  const effects: ReturnType<typeof foldEffect.of>[] = [];
  foldedRanges(state).between(0, state.doc.length, (from, to) => { effects.push(foldEffect.of({ from, to })); });
  if (effects.length) view.dispatch({ effects });
}
</book:entry>

</book:doc>
