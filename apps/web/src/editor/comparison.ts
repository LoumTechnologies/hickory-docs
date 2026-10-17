import { accessibleGutterActions } from "./gutterActions";
// Historical bytes are decorations, never part of saved text or LSP positions.
import { RangeSet, StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView, GutterMarker, gutter, keymap, showTooltip, type Tooltip, type DecorationSet } from "@codemirror/view";
import { matchedLines, splitLines } from "../lib/merge";

export interface Change { from: number; to: number; removed: string }
export function comparisonChanges(base: string, current: string): Change[] {
  const a = splitLines(base), b = splitLines(current);
  const matches = [...matchedLines(a, b), [a.length, b.length]];
  const offsets = [0];
  b.forEach(line => offsets.push(offsets[offsets.length - 1] + line.length));
  let old = 0, next = 0;
  const changes: Change[] = [];
  for (const [i, j] of matches) {
    if (i > old || j > next) changes.push({ from: offsets[next], to: offsets[j], removed: a.slice(old, i).join("") });
    old = i + 1; next = j + 1;
  }
  return changes;
}
export interface ComparisonState { base: string | null; editable: boolean; presentation?: "inline" | "old" | "new" }
export const setComparison = StateEffect.define<ComparisonState>();
const openRemoved = StateEffect.define<{ text: string; at: number; canRestore: boolean } | null>();
class Removed extends GutterMarker {
  constructor(readonly text: string, readonly at: number, readonly canRestore: boolean) { super(); }
  eq(other: Removed) { return this.text === other.text && this.at === other.at && this.canRestore === other.canRestore; }
  toDOM(view: EditorView) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "comparison-deletion";
    button.textContent = "−";
    const count = splitLines(this.text).length;
    button.setAttribute("aria-label", `Show ${count} removed ${count === 1 ? "line" : "lines"}`);
    button.dataset.tip = button.getAttribute("aria-label")!;
    button.onclick = () => view.dispatch({ effects: openRemoved.of({ text: this.text, at: this.at, canRestore: this.canRestore }) });
    return button;
  }
}
const removedPreview = StateField.define<Tooltip | null>({
  create: () => null,
  update(value, tr) {
    if (tr.docChanged || tr.selection || tr.effects.some(effect => effect.is(setComparison))) value = null;
    for (const effect of tr.effects) if (effect.is(openRemoved)) {
      const removed = effect.value;
      value = removed ? { pos: removed.at, above: false, create(view) {
        const dom = document.createElement("div");
        dom.className = "comparison-removed";
        const title = dom.appendChild(document.createElement("strong"));
        title.textContent = "Removed lines · previous version";
        const close = dom.appendChild(document.createElement("button"));
        close.type = "button"; close.textContent = "×"; close.setAttribute("aria-label", "Close removed lines");
        close.onclick = () => view.dispatch({ effects: openRemoved.of(null) });
        const pre = dom.appendChild(document.createElement("pre"));
        pre.textContent = removed.text;
        if (removed.canRestore) {
          const restore = dom.appendChild(document.createElement("button"));
          restore.type = "button"; restore.textContent = "Restore these lines";
          restore.onclick = () => view.dispatch({ changes: { from: removed.at, insert: removed.text }, userEvent: "input.restore" });
        }
        return { dom };
      } } : null;
    }
    return value;
  },
  provide: field => showTooltip.from(field),
});
function decorate(view: { doc: { toString(): string; lineAt(n: number): { from: number; to: number } } }, comparison: ComparisonState): DecorationSet {
  if (comparison.base === null) return Decoration.none;
  const ranges = [];
  for (const change of comparisonChanges(comparison.base, view.doc.toString())) {
    let at = change.from;
    while (at < change.to) {
      const line = view.doc.lineAt(at);
      ranges.push(Decoration.line({ class: comparison.presentation === "old" ? "comparison-deleted" : "comparison-added" }).range(line.from));
      at = line.to + 1;
    }
  }
  return Decoration.set(ranges, true);
}
export const comparisonField = StateField.define<{ comparison: ComparisonState; decorations: DecorationSet }>({
  create: () => ({ comparison: { base: null, editable: true }, decorations: Decoration.none }),
  update(value, tr) {
    let comparison = value.comparison;
    for (const effect of tr.effects) if (effect.is(setComparison)) comparison = effect.value;
    if (!tr.docChanged && comparison === value.comparison) return value;
    return { comparison, decorations: decorate(tr.state, comparison) };
  },
  provide: field => [EditorView.decorations.from(field, value => value.decorations), removedPreview, accessibleGutterActions,
    keymap.of([{ key: "Escape", run: view => {
      if (!view.state.field(removedPreview)) return false;
      view.dispatch({ effects: openRemoved.of(null) });
      view.focus();
      return true;
    } }]),
    gutter({ class: "comparison-gutter", markers: view => {
      const { comparison } = view.state.field(field);
      if (comparison.base === null || (comparison.presentation && comparison.presentation !== "inline")) return RangeSet.empty;
      // Several edits on one line share one preview.
      const marks = new Map<number, { text: string; at: number }>();
      for (const change of comparisonChanges(comparison.base, view.state.doc.toString())) {
        if (!change.removed) continue;
        const line = view.state.doc.lineAt(change.from).from;
        const previous = marks.get(line);
        marks.set(line, { text: (previous?.text ?? "") + change.removed, at: previous?.at ?? change.from });
      }
      return RangeSet.of([...marks].map(([line, removed]) => new Removed(removed.text, removed.at, comparison.editable).range(line)), true);
    } }),
  ],
});
