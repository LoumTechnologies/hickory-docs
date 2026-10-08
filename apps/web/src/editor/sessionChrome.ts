// The live conversation uses the document editor's native folds. Newly
// arriving work starts folded once; opening it survives subsequent streaming.
import { foldEffect, foldedRanges } from "@codemirror/language";
import { ViewPlugin, type EditorView } from "@codemirror/view";
import { foldRangesOf, workFoldRanges } from "./folding";

export const sessionBookkeeping = ViewPlugin.define((view: EditorView) => {
  let seen = new Set<number>();
  let alive = true;
  const attempt = () => {
    const fresh = workFoldRanges(foldRangesOf(view.state)).filter(range => !seen.has(range.from));
    fresh.forEach(range => seen.add(range.from));
    if (!fresh.length) return;
    queueMicrotask(() => {
      if (!alive) return;
      const effects = fresh.filter(range => range.to <= view.state.doc.length).map(range => foldEffect.of(range));
      if (effects.length) view.dispatch({ effects });
    });
  };
  attempt();
  return {
    update(update) {
      if (!update.docChanged) return;
      const ranges = foldRangesOf(update.state);
      seen = new Set([...seen].flatMap(from => {
        const mapped = update.changes.mapPos(from, -1);
        return ranges.some(range => range.from === mapped) ? [mapped] : [];
      }));
      // Only newly arrived work folds. Existing folds map with CodeMirror.
      attempt();
    },
    destroy() { alive = false; },
  };
});

/** Resolve hidden receipts to their native fold's visible opening line. */
export function conversationFold(view: EditorView, from: number, to: number) {
  let header: number | null = null;
  foldedRanges(view.state).between(0, view.state.doc.length, (start, end) => {
    if (start < to && end > from) {
      const line = view.state.doc.lineAt(start).number;
      header = header === null ? line : Math.min(header, line);
    }
  });
  return header;
}
