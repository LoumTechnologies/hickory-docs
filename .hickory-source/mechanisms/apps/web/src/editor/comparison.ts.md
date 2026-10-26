<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/comparison.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ceadb3379aac"></a>
## `apps/web/src/editor/comparison.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A comparison keeps current code editable](../../../../../guarantees/editing/a-comparison-keeps-current-code-editable/contract.md).

<a id="f-ceadb3379aac-Change"></a>
### `Change`

<storycode:copy id="f-ceadb3379aac-Change">
import { editorLanguageActions } from "./languageActions";
import type { UiActor, UiOutcome } from "../lib/uiActions";
import { foldedRanges } from "@codemirror/language";
import { accessibleGutterActions } from "./gutterActions";
// Historical bytes are decorations, never part of saved text or LSP positions.
import { RangeSet, StateEffect, StateField, type Extension } from "@codemirror/state";
import { Decoration, EditorView, GutterMarker, gutter, keymap, showTooltip, type Tooltip, type DecorationSet } from "@codemirror/view";
import { matchedLines, splitLines } from "../lib/merge";

export interface Change { from: number; to: number; removed: string }
</storycode:copy>
<a id="f-ceadb3379aac-comparisonChanges"></a>
### `comparisonChanges`

<storycode:copy id="f-ceadb3379aac-comparisonChanges">
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
</storycode:copy>
<a id="f-ceadb3379aac-ComparisonState"></a>
### `ComparisonState`

<storycode:copy id="f-ceadb3379aac-ComparisonState">
export interface ComparisonState { base: string | null; editable: boolean; presentation?: "inline" | "old" | "new" }
</storycode:copy>
<a id="f-ceadb3379aac-setComparison"></a>
### `setComparison`

<storycode:copy id="f-ceadb3379aac-setComparison">
export const setComparison = StateEffect.define<ComparisonState>();
</storycode:copy>
<a id="f-ceadb3379aac-RemovedCapture"></a>
### `RemovedCapture`

<storycode:copy id="f-ceadb3379aac-RemovedCapture">
type RemovedCapture = { id: string; text: string; at: number; canRestore: boolean; source: EditorView["state"]["doc"]; base: string };
</storycode:copy>
<a id="f-ceadb3379aac-openRemoved"></a>
### `openRemoved`

<storycode:copy id="f-ceadb3379aac-openRemoved">
const openRemoved = StateEffect.define<RemovedCapture | null>();
</storycode:copy>
<a id="f-ceadb3379aac-Removed"></a>
### `Removed`

<storycode:copy id="f-ceadb3379aac-Removed">
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
</storycode:copy>
<a id="f-ceadb3379aac-removedPreview"></a>
### `removedPreview`

<storycode:copy id="f-ceadb3379aac-removedPreview">
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
</storycode:copy>
<a id="f-ceadb3379aac-decorate"></a>
### `decorate`

<storycode:copy id="f-ceadb3379aac-decorate">
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
</storycode:copy>
<a id="f-ceadb3379aac-comparisonField"></a>
### `comparisonField`

<storycode:copy id="f-ceadb3379aac-comparisonField">
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


</storycode:copy>
<a id="f-ceadb3379aac-removedMarks"></a>
### `removedMarks`

<storycode:copy id="f-ceadb3379aac-removedMarks">
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
</storycode:copy>
<a id="f-ceadb3379aac-showRemoved"></a>
### `showRemoved`

<storycode:copy id="f-ceadb3379aac-showRemoved">
function showRemoved(view: EditorView, at: number, expectedText?: string): UiOutcome {
  if (!view.dom.isConnected) return { status: "unavailable", reason: "Comparison editor was closed" };
  const mark = removedMarks(view).find(mark => mark.at === at), comparison = view.state.field(comparisonField).comparison;
  if (!mark || comparison.base === null || (expectedText !== undefined && mark.text !== expectedText)) return { status: "unavailable", reason: "The removed-line marker changed or is hidden; inspect current comparison controls" };
  view.dispatch({ effects: openRemoved.of({ ...mark, id: crypto.randomUUID(), source: view.state.doc, base: comparison.base }) });
  return { status: "completed", value: { preview_open: !!view.state.field(removedPreview), historical: true } };
}
</storycode:copy>
<a id="f-ceadb3379aac-closeRemoved"></a>
### `closeRemoved`

<storycode:copy id="f-ceadb3379aac-closeRemoved">
function closeRemoved(view: EditorView): UiOutcome {
  view.dispatch({ effects: openRemoved.of(null) });
  return { status: "completed", value: { preview_open: false } };
}
</storycode:copy>
<a id="f-ceadb3379aac-restoreRemoved"></a>
### `restoreRemoved`

<storycode:copy id="f-ceadb3379aac-restoreRemoved">
function restoreRemoved(view: EditorView, captured: RemovedCapture, actor?: UiActor): UiOutcome {
  const current = view.state.field(removedPreview)?.removed, comparison = view.state.field(comparisonField).comparison;
  if (!view.dom.isConnected || view.state.readOnly || !captured.canRestore || !comparison.editable || current !== captured || view.state.doc !== captured.source || comparison.base !== captured.base) return { status: "unavailable", reason: "Removed source changed, closed or is read-only; reopen the current preview before restoring" };
  const before = view.state.doc.toString(), expected = before.slice(0, captured.at) + captured.text + before.slice(captured.at);
  view.dispatch({ changes: { from: captured.at, insert: captured.text }, userEvent: actor?.kind === "agent" ? "input.uiAgentRestore" : "input.restore" });
  if (view.state.doc.toString() !== expected) return { status: "unavailable", reason: "The source owner refused or transformed the restore; inspect current source before retrying" };
  return { status: "completed", value: { source_changed: true, saved: false, verified: false, historical_bytes_restored: true } };
}
</storycode:copy>
<a id="f-ceadb3379aac-comparisonActions"></a>
### `comparisonActions`

<storycode:copy id="f-ceadb3379aac-comparisonActions">
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
</storycode:copy>

### Assemble `apps/web/src/editor/comparison.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/comparison.ts"><storycode:paste select="#f-ceadb3379aac-Change"/><storycode:paste select="#f-ceadb3379aac-comparisonChanges"/><storycode:paste select="#f-ceadb3379aac-ComparisonState"/><storycode:paste select="#f-ceadb3379aac-setComparison"/><storycode:paste select="#f-ceadb3379aac-RemovedCapture"/><storycode:paste select="#f-ceadb3379aac-openRemoved"/><storycode:paste select="#f-ceadb3379aac-Removed"/><storycode:paste select="#f-ceadb3379aac-removedPreview"/><storycode:paste select="#f-ceadb3379aac-decorate"/><storycode:paste select="#f-ceadb3379aac-comparisonField"/><storycode:paste select="#f-ceadb3379aac-removedMarks"/><storycode:paste select="#f-ceadb3379aac-showRemoved"/><storycode:paste select="#f-ceadb3379aac-closeRemoved"/><storycode:paste select="#f-ceadb3379aac-restoreRemoved"/><storycode:paste select="#f-ceadb3379aac-comparisonActions"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-a13b9a534b46"></a>
## `apps/web/src/editor/comparison.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A comparison keeps current code editable](../../../../../guarantees/editing/a-comparison-keeps-current-code-editable/contract.md), [A commit reads as a literate change](../../../../../guarantees/lenses/a-commit-reads-as-a-literate-change/contract.md).

<a id="f-a13b9a534b46-views"></a>
### `views`

<storycode:copy id="f-a13b9a534b46-views">
// @vitest-environment jsdom
// Guarantee: editing/a-comparison-keeps-current-code-editable.md
import { afterEach, describe, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { comparisonChanges, comparisonField, setComparison } from "./comparison";
const views: EditorView[] = [];
</storycode:copy>
<a id="f-a13b9a534b46-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-a13b9a534b46-expression_statement">
afterEach(() => { views.splice(0).forEach(view => view.destroy()); document.body.replaceChildren(); });
</storycode:copy>
<a id="f-a13b9a534b46-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-a13b9a534b46-expression_statement-2">
describe("editable comparison in the main buffer", () => {
  it("keeps deleted bytes out of current text and recomputes when editing", () => {
    const view = new EditorView({ state:EditorState.create({ doc:"same\nnew\n", extensions:[comparisonField] }), parent:document.body }); views.push(view);
    view.dispatch({ effects:setComparison.of({ base:"same\nold\n",editable:true }) });
    expect(view.state.doc.toString()).toBe("same\nnew\n");
    expect(document.querySelector(".comparison-removed")).toBeNull();
    expect(document.querySelector(".comparison-deletion")?.getAttribute("aria-label")).toBe("Show 1 removed line");
    expect(document.querySelector(".comparison-added")?.textContent).toBe("new");
    view.dispatch({ changes:{ from:5,to:8,insert:"edited" } });
    expect(view.state.doc.toString()).toBe("same\nedited\n");
    expect(document.querySelector(".comparison-added")?.textContent).toBe("edited");
    view.dispatch({effects:setComparison.of({base:null,editable:true})});
    expect(document.querySelector(".comparison-removed")).toBeNull();
  });
  it("restores only explicitly chosen removed lines and hides restoration for history", async () => {
    const view=new EditorView({state:EditorState.create({doc:"same\n",extensions:[comparisonField]}),parent:document.body}); views.push(view);
    view.dispatch({effects:setComparison.of({base:"same\ndeleted\n",editable:false})});
    expect(document.querySelector(".comparison-deletion")).not.toBeNull();
    view.dispatch({effects:setComparison.of({base:"same\ndeleted\n",editable:true})});
    const marker = document.querySelector(".comparison-deletion") as HTMLButtonElement;
    marker.click();
    const tooltip = view.state.field(comparisonField);
    expect(tooltip.comparison.editable).toBe(true);
    await vi.waitFor(() => expect(document.querySelector(".comparison-removed pre")?.textContent).toContain("deleted"));
    const restore = [...document.querySelectorAll(".comparison-removed button")].find(button => button.textContent === "Restore these lines") as HTMLButtonElement;
    restore.click();
    expect(view.state.doc.toString()).toBe("same\ndeleted\n");
    expect(document.querySelector(".comparison-removed")).toBeNull();
  });
  it("handles empty sides, missing final newlines, and UTF-16 coordinates",()=>{
    expect(comparisonChanges("", "é🙂")).toEqual([{from:0,to:3,removed:""}]);
    expect(comparisonChanges("é🙂", "")).toEqual([{from:0,to:0,removed:"é🙂"}]);
    expect(comparisonChanges("a\n", "a")).toEqual([{from:0,to:1,removed:"a\n"}]);
  });
});
</storycode:copy>

### Assemble `apps/web/src/editor/comparison.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/comparison.test.ts"><storycode:paste select="#f-a13b9a534b46-views"/><storycode:paste select="#f-a13b9a534b46-expression_statement"/><storycode:paste select="#f-a13b9a534b46-expression_statement-2"/></storycode:file>


</storycode:doc>
