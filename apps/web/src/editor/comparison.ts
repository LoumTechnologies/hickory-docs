import { editorLanguageActions } from "./languageActions";
import type { UiActor, UiOutcome } from "../lib/uiActions";
import { foldedRanges } from "@codemirror/language";
import { accessibleGutterActions } from "./gutterActions";
// Historical bytes are decorations, never part of saved text or LSP positions.
import { RangeSet, StateEffect, StateField, type Extension } from "@codemirror/state";
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
type RemovedCapture = { id: string; text: string; at: number; canRestore: boolean; source: EditorView["state"]["doc"]; base: string };
const openRemoved = StateEffect.define<RemovedCapture | null>();
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
    button.onclick = () => showRemoved(view, this.at, this.text);
    return button;
  }
}
const removedPreview = StateField.define<{ removed: RemovedCapture; tooltip: Tooltip } | null>({
  create: () => null,
  update(value, tr) {
    if (tr.docChanged || tr.selection || tr.startState.readOnly !== tr.state.readOnly || tr.effects.some(effect => effect.is(setComparison))) value = null;
    for (const effect of tr.effects) if (effect.is(openRemoved)) {
      const removed = effect.value;
      const current = tr.state.field(comparisonField, false)?.comparison;
      value = removed && removed.source === tr.state.doc && current?.base === removed.base ? { removed, tooltip: { pos: removed.at, above: false, create(view) {
        const dom = document.createElement("div");
        dom.className = "comparison-removed";
        const title = dom.appendChild(document.createElement("strong"));
        title.textContent = "Removed lines · previous version";
        const close = dom.appendChild(document.createElement("button"));
        close.type = "button"; close.textContent = "×"; close.setAttribute("aria-label", "Close removed lines");
        close.onclick = () => closeRemoved(view);
        const pre = dom.appendChild(document.createElement("pre"));
        pre.textContent = removed.text;
        if (removed.canRestore) {
          const restore = dom.appendChild(document.createElement("button"));
          restore.type = "button"; restore.textContent = "Restore these lines";
          restore.onclick = () => restoreRemoved(view, removed);
        }
        return { dom };
      } } } : null;
    }
    return value;
  },
  provide: field => showTooltip.from(field, value => value?.tooltip ?? null),
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
export const comparisonField: StateField<{ comparison: ComparisonState; decorations: DecorationSet }> = StateField.define<{ comparison: ComparisonState; decorations: DecorationSet }>({
  create: () => ({ comparison: { base: null, editable: true }, decorations: Decoration.none }),
  update(value, tr) {
    let comparison = value.comparison;
    for (const effect of tr.effects) if (effect.is(setComparison)) comparison = effect.value;
    if (!tr.docChanged && comparison === value.comparison) return value;
    return { comparison, decorations: decorate(tr.state, comparison) };
  },
  provide: field => [EditorView.decorations.from(field, value => value.decorations), removedPreview, accessibleGutterActions, comparisonActions(),
    keymap.of([{ key: "Escape", run: view => {
      if (!view.state.field(removedPreview)) return false;
      closeRemoved(view);
      view.focus();
      return true;
    } }]),
    gutter({ class: "comparison-gutter", markers: view => {
      return RangeSet.of(removedMarks(view).map(mark => new Removed(mark.text, mark.at, mark.canRestore).range(mark.line)), true);
    } }),
  ],
});


function removedMarks(view: EditorView): { text: string; at: number; line: number; canRestore: boolean }[] {
  const comparison = view.state.field(comparisonField).comparison;
  if (comparison.base === null || (comparison.presentation && comparison.presentation !== "inline")) return [];
  const marks = new Map<number, { text: string; at: number }>();
  for (const change of comparisonChanges(comparison.base, view.state.doc.toString())) {
    if (!change.removed) continue;
    const line = view.state.doc.lineAt(change.from).from, previous = marks.get(line);
    marks.set(line, { text: (previous?.text ?? "") + change.removed, at: previous?.at ?? change.from });
  }
  const folded = foldedRanges(view.state);
  return [...marks].flatMap(([line, mark]) => {
    let hidden = false; folded.between(line, line, (from, to) => { if (from <= line && line < to) hidden = true; });
    return hidden ? [] : [{ ...mark, line, canRestore: comparison.editable && !view.state.readOnly }];
  });
}
function showRemoved(view: EditorView, at: number, expectedText?: string): UiOutcome {
  if (!view.dom.isConnected) return { status: "unavailable", reason: "Comparison editor was closed" };
  const mark = removedMarks(view).find(mark => mark.at === at), comparison = view.state.field(comparisonField).comparison;
  if (!mark || comparison.base === null || (expectedText !== undefined && mark.text !== expectedText)) return { status: "unavailable", reason: "The removed-line marker changed or is hidden; inspect current comparison controls" };
  view.dispatch({ effects: openRemoved.of({ ...mark, id: crypto.randomUUID(), source: view.state.doc, base: comparison.base }) });
  return { status: "completed", value: { preview_open: !!view.state.field(removedPreview), historical: true } };
}
function closeRemoved(view: EditorView): UiOutcome {
  view.dispatch({ effects: openRemoved.of(null) });
  return { status: "completed", value: { preview_open: false } };
}
function restoreRemoved(view: EditorView, captured: RemovedCapture, actor?: UiActor): UiOutcome {
  const current = view.state.field(removedPreview)?.removed, comparison = view.state.field(comparisonField).comparison;
  if (!view.dom.isConnected || view.state.readOnly || !captured.canRestore || !comparison.editable || current !== captured || view.state.doc !== captured.source || comparison.base !== captured.base) return { status: "unavailable", reason: "Removed source changed, closed or is read-only; reopen the current preview before restoring" };
  const before = view.state.doc.toString(), expected = before.slice(0, captured.at) + captured.text + before.slice(captured.at);
  view.dispatch({ changes: { from: captured.at, insert: captured.text }, userEvent: actor?.kind === "agent" ? "input.uiAgentRestore" : "input.restore" });
  if (view.state.doc.toString() !== expected) return { status: "unavailable", reason: "The source owner refused or transformed the restore; inspect current source before retrying" };
  return { status: "completed", value: { source_changed: true, saved: false, verified: false, historical_bytes_restored: true } };
}
function comparisonActions(): Extension {
  const number = { type: "number" as const, integer: true, minimum: 0, maximum: 100_000_000 };
  const preview = (view: EditorView) => view.state.field(removedPreview)?.removed;
  const inline = (view: EditorView) => removedMarks(view).length > 0;
  return [
    editorLanguageActions.of({ id: "comparison:markers", label: "Read comparison removed-line marker page", visible: inline,
      parameters: { first: number, count: { ...number, minimum: 1, maximum: 64 } }, enabled: () => true,
      execute: async (view, args) => {
        const marks = removedMarks(view), first = Number(args.first), count = Number(args.count);
        if (first > marks.length) return { status: "unavailable", reason: "Marker page is outside the current comparison" };
        return { status: "completed", value: { markers: marks.slice(first, first + count).map(mark => ({ position: mark.at, line: view.state.doc.lineAt(mark.line).number, removed_lines: splitLines(mark.text).length, can_restore: mark.canRestore })), total: marks.length, next: first + count < marks.length ? first + count : null, coordinate_unit: "UTF-16 code units", historical_text_disclosed: false } };
      } }),
    editorLanguageActions.of({ id: "comparison:open", label: "Open comparison removed-line preview", visible: inline,
      parameters: { position: number }, enabled: () => true, execute: async (view, args) => showRemoved(view, Number(args.position)) }),
    editorLanguageActions.of({ id: "comparison:read", label: "Read comparison removed-source bytes", visible: view => !!preview(view), revision: view => preview(view)?.id,
      parameters: { offset: number, limit: { ...number, minimum: 1, maximum: 8192 } }, enabled: view => !!preview(view),
      execute: async (view, args) => {
        const captured = preview(view); if (!captured) return { status: "unavailable", reason: "Open the current removed-line preview first" };
        const bytes = new TextEncoder().encode(captured.text), offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
        if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds the removed-source snapshot" };
        return { status: "completed", value: { bytes: [...bytes.slice(offset, end)], encoding: "UTF-8", span: [offset, end], next: end < bytes.length ? end : null, total_bytes: bytes.length, historical: true, edit_baseline: false, verified: false } };
      } }),
    editorLanguageActions.of({ id: "comparison:close", label: "Close comparison removed-line preview", visible: view => !!preview(view), parameters: {}, enabled: view => !!preview(view), execute: async view => closeRemoved(view) }),
    editorLanguageActions.of({ id: "comparison:restore", label: "Restore comparison removed-source bytes", visible: view => !!preview(view)?.canRestore, revision: view => preview(view)?.id,
      parameters: {}, enabled: view => !!preview(view)?.canRestore && !view.state.readOnly,
      execute: async (view, _args, _signal, actor) => { const captured = preview(view); return captured ? restoreRemoved(view, captured, actor) : { status: "unavailable", reason: "Open the current restorable preview first" }; } }),
  ];
}
