// Historical bytes are decorations, never part of saved text or LSP positions.
import { StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView, WidgetType, type DecorationSet } from "@codemirror/view";
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
export interface ComparisonState { base: string | null; editable: boolean }
export const setComparison = StateEffect.define<ComparisonState>();
class Removed extends WidgetType {
  constructor(readonly text: string, readonly at: number, readonly canRestore: boolean) { super(); }
  eq(other: Removed) { return this.text === other.text && this.at === other.at && this.canRestore === other.canRestore; }
  toDOM(view: EditorView) {
    const box = document.createElement("details");
    box.className = "comparison-removed";
    const summary = box.appendChild(document.createElement("summary"));
    summary.textContent = `− ${splitLines(this.text).length} historical line(s) — read-only`;
    const pre = box.appendChild(document.createElement("pre")); pre.textContent = this.text;
    if (this.canRestore) {
      const restore = box.appendChild(document.createElement("button"));
      restore.type = "button"; restore.textContent = "Restore these lines";
      restore.onclick = () => view.dispatch({ changes: { from: this.at, insert: this.text }, userEvent: "input.restore" });
    }
    return box;
  }
  ignoreEvent() { return true; }
}
function decorate(view: { doc: { toString(): string; lineAt(n: number): { from: number; to: number } } }, comparison: ComparisonState): DecorationSet {
  if (comparison.base === null) return Decoration.none;
  const ranges = [];
  for (const change of comparisonChanges(comparison.base, view.doc.toString())) {
    if (change.removed) ranges.push(Decoration.widget({ widget: new Removed(change.removed, change.from, comparison.editable), block: true, side: -1 }).range(change.from));
    let at = change.from;
    while (at < change.to) {
      const line = view.doc.lineAt(at);
      ranges.push(Decoration.line({ class: "comparison-added" }).range(line.from));
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
  provide: field => EditorView.decorations.from(field, value => value.decorations),
});
