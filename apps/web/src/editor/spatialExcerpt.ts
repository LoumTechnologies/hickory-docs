import { StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView, WidgetType } from "@codemirror/view";

interface Excerpt { first: number; last: number; focal: number }
const excerptEffect = StateEffect.define<Excerpt | null>();

class ExcerptEdge extends WidgetType {
  constructor(readonly range: Excerpt, readonly count: number, readonly above: boolean, readonly contract: boolean) { super(); }
  eq(other: ExcerptEdge) { return this.range.first === other.range.first && this.range.last === other.range.last && this.count === other.count && this.above === other.above && this.contract === other.contract; }
  toDOM(view: EditorView) {
    const row = document.createElement("div"); row.className = "spatial-excerpt-edge";
    const more = document.createElement("button"); more.type = "button";
    more.textContent = `${this.above ? "↑" : "↓"} ${this.count} hidden`;
    more.setAttribute("aria-label", `Show three more lines ${this.above ? "above" : "below"}`);
    more.onclick = () => { const next = { ...this.range }; if (this.above) next.first = Math.max(1, next.first - 3); else next.last = Math.min(view.state.doc.lines, next.last + 3); view.dispatch({ effects: excerptEffect.of(next) }); };
    row.append(more);
    if (this.contract) {
      const less = document.createElement("button"); less.type = "button"; less.textContent = this.above ? "↓" : "↑";
      less.setAttribute("aria-label", `Hide three lines ${this.above ? "above" : "below"}`);
      less.onclick = () => { const next = { ...this.range }; if (this.above) next.first = Math.min(Math.max(1, next.focal - 2), next.first + 3); else next.last = Math.max(Math.min(view.state.doc.lines, next.focal + 3), next.last - 3); view.dispatch({ effects: excerptEffect.of(next) }); }; row.append(less);
    }
    return row;
  }
  ignoreEvent() { return true; }
}

/** A reading window over the unchanged buffer, independent of language folds. */
const excerptField = StateField.define<{ range: Excerpt | null; decorations: ReturnType<typeof Decoration.set> }>({
  create: () => ({ range: null, decorations: Decoration.none }),
  update(value, transaction) {
    let range = value.range;
    for (const effect of transaction.effects) if (effect.is(excerptEffect)) range = effect.value;
    if (!range) return { range, decorations: Decoration.none };
    const count = transaction.state.doc.lines, first = Math.max(1, Math.min(count, range.first)), last = Math.max(first, Math.min(count, range.last)), marks = [];
    range = { ...range, first, last };
    if (first > 1) marks.push(Decoration.replace({ block: true, widget: new ExcerptEdge(range, first - 1, true, first < Math.max(1, range.focal - 2)) }).range(0, transaction.state.doc.line(first - 1).to));
    if (last < count) marks.push(Decoration.replace({ block: true, widget: new ExcerptEdge(range, count - last, false, last > range.focal + 3) }).range(transaction.state.doc.line(last + 1).from, transaction.state.doc.length));
    return { range, decorations: Decoration.set(marks, true) };
  },
  provide: field => EditorView.decorations.from(field, value => value.decorations),
});

const applied = new WeakMap<EditorView, string>();
export function showSpatialExcerpt(view: EditorView, focal: number | null, expanded = false) {
  const current = view.state.field(excerptField, false), line = focal === null ? null : Math.max(1, Math.min(view.state.doc.lines, view.state.doc.lineAt(Math.min(focal, view.state.doc.length)).number)), key = line === null ? "full" : `${line}:${expanded}`;
  if (applied.get(view) === key) return;
  if (!current) view.dispatch({ effects: StateEffect.appendConfig.of(excerptField) });
  view.dispatch({ effects: excerptEffect.of(line === null || !expanded ? null : { focal: line, first: Math.max(1, line - 2), last: Math.min(view.state.doc.lines, line + 3) }) });
  applied.set(view, key);
}

export function clearSpatialExcerpt(view: EditorView) { showSpatialExcerpt(view, null); }
