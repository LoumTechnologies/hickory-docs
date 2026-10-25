<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/uiEditorActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `68c7483975fd9685ba3bd8e075d8f3b2bbba77522f465c2dfbc8bd3c367a377d`.

<book:entry path="apps/web/src/editor/uiEditorActions.ts">
import { useEditorLanguageActions } from "./useEditorLanguageActions";
import { EditorSelection, EditorState } from "@codemirror/state";
import { useId } from "react";
import { foldAll, foldCode, foldedRanges, unfoldAll, unfoldCode } from "@codemirror/language";
import type { EditorView } from "@codemirror/view";
import { useUiCommands } from "../lib/uiActionBridge";
import type { UiCommand, UiField, UiValue } from "../lib/uiActions";
const identities = new WeakMap<object, string>();
function identity(state: object) { let id = identities.get(state); if (!id) { id = crypto.randomUUID(); identities.set(state, id); } return id; }

/** Source and editor state are explicitly paged; scrolling never reads them. */
export function useEditorActions(view: EditorView | null, subject: string) {
  useEditorLanguageActions(view, subject);
  const instance = useId(); const parent = `editor:${subject}:${instance}`;
  const commands: { key: string; command: UiCommand }[] = [];
  const number = { type: "number" as const, minimum: 0, maximum: 100_000_000, integer: true };
  function add(id: string, label: string, parameters: Record<string, UiField>, execute: (args: Record<string, UiValue>, view: EditorView) => UiValue | Promise<UiValue>) {
    commands.push({ key: `${parent}/${id}`, command: {
      label, subject: parent, parent, parameters, prerequisites: ["Current editor state; source ranges use UTF-16, byte offsets use UTF-8, line numbers are one-based and page indices are zero-based"],
      read: () => ({ revision: view ? identity(view.state) : "unavailable", enabled: !!view?.dom.isConnected, reason: !view?.dom.isConnected ? "Editor was closed" : undefined,
        value: view ? { subject, length: view.state.doc.length, lines: view.state.doc.lines, selection: { anchor: view.state.selection.main.anchor, head: view.state.selection.main.head }, selection_count: view.state.selection.ranges.length, main_index: view.state.selection.mainIndex, multiple_selections: view.state.facet(EditorState.allowMultipleSelections), read_only: view.state.readOnly, edit_baseline: false } : null }),
      execute: async args => {
        if (!view?.dom.isConnected) return { status: "unavailable", reason: "Editor was closed" };
        return { status: "completed", value: await execute(args, view) };
      },
    } });
  }
  add("read", "Read editor lines", { first: { ...number, minimum: 1 }, count: { ...number, minimum: 1, maximum: 200 } }, (args, editor) => {
    const first = Number(args.first); if (first > editor.state.doc.lines) throw Error("Line is outside this editor");
    const last = Math.min(editor.state.doc.lines, first + Number(args.count) - 1);
    const start = editor.state.doc.line(first).from, end = editor.state.doc.line(last).to;
    const text = editor.state.sliceDoc(start, end);
    if (new TextEncoder().encode(text).length > 8192) throw Error("Line page exceeds 8192 UTF-8 bytes; use Read editor UTF-8 bytes for bounded source reading");
    return { editor: subject, revision: identity(editor.state), coordinate_unit: "UTF-16 code units", first_line: first, last_line: last, span: { start, end }, text, total_lines: editor.state.doc.lines, next_line: last < editor.state.doc.lines ? last + 1 : null, edit_baseline: false };
  });
  add("bytes", "Read editor UTF-8 bytes", { offset: { ...number, maximum: 1000000000 }, limit: { ...number, minimum: 1, maximum: 8192 } }, (args, editor) => {
    const bytes = new TextEncoder().encode(editor.state.doc.toString()), offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    if (offset > bytes.length) throw Error("Offset outside current editor source");
    return { editor: subject, revision: identity(editor.state), encoding: "UTF-8", bytes: Array.from(bytes.slice(offset, end)), offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null, edit_baseline: false, coordinate_unit: "UTF-8 bytes for reading only; editor writes and selections use UTF-16 code units" };
  });
  add("selections", "Read editor selection page", { first: number, count: { ...number, minimum: 1, maximum: 64 } }, (args, editor) => {
    const first = args.first as number, count = args.count as number, ranges = editor.state.selection.ranges;
    if (first > ranges.length) throw Error("Selection page outside current editor");
    const selected = ranges.slice(first, first + count).map((r, index) => ({ index: first + index, anchor: r.anchor, head: r.head, from: r.from, to: r.to }));
    return { editor: subject, revision: identity(editor.state), ranges: selected, total: ranges.length, main_index: editor.state.selection.mainIndex, next: first + selected.length < ranges.length ? first + selected.length : null, coordinate_unit: "UTF-16 code units", edit_baseline: false };
  });
  add("primary-selection", "Make editor range primary", { index: number }, (args, editor) => {
    const index = args.index as number;
    if (index >= editor.state.selection.ranges.length) throw Error("Range outside current editor selection");
    editor.dispatch({ selection: EditorSelection.create([...editor.state.selection.ranges], index), scrollIntoView: true }); editor.focus();
    return { main_index: editor.state.selection.mainIndex, selection_count: editor.state.selection.ranges.length, source_written: false };
  });
  add("select-many", "Select editor ranges", { anchors: { type: "array", items: number, maxItems: 64 }, heads: { type: "array", items: number, maxItems: 64 }, main: { ...number, maximum: 63 } }, (args, editor) => {
    const anchors = args.anchors as number[], heads = args.heads as number[], main = args.main as number;
    if (!anchors.length || anchors.length !== heads.length || main >= anchors.length || anchors.some(n => n > editor.state.doc.length) || heads.some(n => n > editor.state.doc.length)) throw Error("Selection arrays must match and lie inside current source");
    if (anchors.length > 1 && !editor.state.facet(EditorState.allowMultipleSelections)) throw Error("This editor does not enable multiple selections");
    editor.dispatch({ selection: EditorSelection.create(anchors.map((anchor, i) => EditorSelection.range(anchor, heads[i])), main), scrollIntoView: true }); editor.focus();
    return { selection_count: editor.state.selection.ranges.length, main_index: editor.state.selection.mainIndex, scope: "Actual normalized CodeMirror selections; overlapping ranges may merge" };
  });
  add("select", "Select editor range", { anchor: number, head: number }, (args, editor) => {
    const anchor = Number(args.anchor), head = Number(args.head);
    if (anchor > editor.state.doc.length || head > editor.state.doc.length) throw Error("Selection is outside this editor revision");
    editor.dispatch({ selection: { anchor, head }, scrollIntoView: true }); editor.focus();
    return { selection: { anchor, head }, coordinate_unit: "UTF-16 code units" };
  });
  const foldPage = (editor: EditorView, first: number, count: number) => {
    let total = 0; const folds: { from: number; to: number }[] = [];
    foldedRanges(editor.state).between(0, editor.state.doc.length, (from, to) => { if (total >= first && total < first + count) folds.push({ from, to }); total++; });
    if (first > total) throw Error("Fold page outside current editor");
    return { folds, total, next: first + folds.length < total ? first + folds.length : null, coordinate_unit: "UTF-16 code units", revision: identity(editor.state) };
  };
  add("folds", "Read editor folds", {}, (_args, editor) => {
    const page = foldPage(editor, 0, 64);
    if (page.next !== null) throw Error("More than 64 folds; use Read editor fold page");
    return page;
  });
  add("fold-page", "Read editor fold page", { first: number, count: { ...number, minimum: 1, maximum: 64 } }, (args, editor) => foldPage(editor, args.first as number, args.count as number));
  add("fold-all", "Expand or collapse all editor sections", { expanded: { type: "boolean" } }, (args, editor) => {
    const changed = (args.expanded ? unfoldAll : foldAll)(editor);
    return { changed, expanded: args.expanded, source_written: false, scope: "Actual CodeMirror fold command result" };
  });
  add("fold", "Expand or collapse editor section", { line: { ...number, minimum: 1 }, expanded: { type: "boolean" } }, (args, editor) => {
    const line = Number(args.line); if (line > editor.state.doc.lines) throw Error("Line is outside this editor");
    editor.dispatch({ selection: { anchor: editor.state.doc.line(line).from } });
    const changed = (args.expanded ? unfoldCode : foldCode)(editor);
    return { changed, line, expanded: Boolean(args.expanded), meaning: "Actual CodeMirror fold command result; no source was edited" };
  });
  useUiCommands(view ? commands : []);
}
</book:entry>

</book:doc>
