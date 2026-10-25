<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/spatialExcerpt.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Spatial Views Preserve The Editor](../../../../../../guarantees/shell/spatial-views-preserve-the-editor/contract.md)

Captured SHA-256: `7d55b6219359a5b917deb0ec2906695048e4d571dc6a1bda94b2a6cf16f4ed9c`.

<book:entry path="apps/web/src/editor/spatialExcerpt.ts">
import { editorLanguageActions } from "./languageActions";
import type { UiOutcome } from "../lib/uiActions";
import { StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView, WidgetType } from "@codemirror/view";

interface Excerpt { first: number; last: number; focal: number }
const excerptEffect = StateEffect.define<Excerpt | null>();

class ExcerptEdge extends WidgetType {
  constructor(readonly range: Excerpt, readonly count: number, readonly above: boolean, readonly contract: boolean) { super(); }
  eq(other: ExcerptEdge) { return this.range.first === other.range.first && this.range.last === other.range.last && this.range.focal === other.range.focal && this.count === other.count && this.above === other.above && this.contract === other.contract; }
  toDOM(view: EditorView) {
    const row = document.createElement("div"); row.className = "spatial-excerpt-edge";
    const more = document.createElement("button"); more.type = "button";
    more.textContent = `${this.above ? "↑" : "↓"} ${this.count} hidden`;
    more.setAttribute("aria-label", `Show three more lines ${this.above ? "above" : "below"}`);
    more.onclick = () => stepExcerpt(view, this.above, true, this.range, this.count);
    row.append(more);
    if (this.contract) {
      const less = document.createElement("button"); less.type = "button"; less.textContent = this.above ? "↓" : "↑";
      less.setAttribute("aria-label", `Hide three lines ${this.above ? "above" : "below"}`);
      less.onclick = () => stepExcerpt(view, this.above, false, this.range, this.count); row.append(less);
    }
    return row;
  }
  ignoreEvent() { return true; }
}

/** A reading window over the unchanged buffer, independent of language folds. */
const excerptField: StateField<{ range: Excerpt | null; decorations: ReturnType<typeof Decoration.set> }> = StateField.define<{ range: Excerpt | null; decorations: ReturnType<typeof Decoration.set> }>({
  create: () => ({ range: null, decorations: Decoration.none }),
  update(value, transaction) {
    let range = value.range;
    for (const effect of transaction.effects) if (effect.is(excerptEffect)) range = effect.value;
    if (!range) return { range, decorations: Decoration.none };
    const count = transaction.state.doc.lines, first = Math.max(1, Math.min(count, range.first)), last = Math.max(first, Math.min(count, range.last)), marks = [];
    range = { ...range, focal: Math.max(1, Math.min(count, range.focal)), first, last };
    if (first > 1) marks.push(Decoration.replace({ block: true, widget: new ExcerptEdge(range, first - 1, true, first < Math.max(1, range.focal - 2)) }).range(0, transaction.state.doc.line(first - 1).to));
    if (last < count) marks.push(Decoration.replace({ block: true, widget: new ExcerptEdge(range, count - last, false, last > range.focal + 3) }).range(transaction.state.doc.line(last + 1).from, transaction.state.doc.length));
    return { range, decorations: Decoration.set(marks, true) };
  },
  provide: field => [EditorView.decorations.from(field, value => value.decorations),
    editorLanguageActions.of({ id: "spatial-excerpt:read", label: "Read spatial excerpt extent", parameters: {}, visible: view => !!view.state.field(field).range, enabled: () => true,
      execute: async view => ({ status: "completed", value: { ...view.state.field(field).range, total_lines: view.state.doc.lines, hidden_above: hidden(view, true), hidden_below: hidden(view, false), content_read: false, source_changed: false } }) }),
    ...[true, false].flatMap(above => [true, false].map(show => editorLanguageActions.of({
      id: `spatial-excerpt:${above}:${show}`, label: `${show ? "Show three more" : "Hide three"} spatial excerpt lines ${above ? "above" : "below"}`,
      parameters: {}, visible: view => hidden(view, above) > 0 && (show || canContract(view, above)), enabled: () => true,
      execute: async view => stepExcerpt(view, above, show),
    }))),
  ],
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


function hidden(view: EditorView, above: boolean): number {
  const range = view.state.field(excerptField, false)?.range;
  return !range ? 0 : above ? range.first - 1 : view.state.doc.lines - range.last;
}
function canContract(view: EditorView, above: boolean): boolean {
  const range = view.state.field(excerptField, false)?.range;
  return !!range && (above ? range.first < Math.max(1, range.focal - 2) : range.last > Math.min(view.state.doc.lines, range.focal + 3));
}
function stepExcerpt(view: EditorView, above: boolean, show: boolean, expected?: Excerpt, expectedCount?: number): UiOutcome {
  const range = view.state.field(excerptField, false)?.range;
  if (!view.dom.isConnected || !range || hidden(view, above) <= 0 || (!show && !canContract(view, above))
    || (expected && (range.first !== expected.first || range.last !== expected.last || range.focal !== expected.focal || hidden(view, above) !== expectedCount))) return { status: "unavailable", reason: "Excerpt edge changed, closed or is unavailable; inspect current extent" };
  const next = { ...range };
  if (show) { if (above) next.first = Math.max(1, next.first - 3); else next.last = Math.min(view.state.doc.lines, next.last + 3); }
  else if (above) next.first = Math.min(Math.max(1, next.focal - 2), next.first + 3);
  else next.last = Math.max(Math.min(view.state.doc.lines, next.focal + 3), next.last - 3);
  view.dispatch({ effects: excerptEffect.of(next) });
  return { status: "completed", value: { ...view.state.field(excerptField).range, hidden_above: hidden(view, true), hidden_below: hidden(view, false), content_read: false, source_changed: false } };
}
</book:entry>

</book:doc>
