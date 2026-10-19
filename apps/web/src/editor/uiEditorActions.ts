import { useId } from "react";
import { foldCode, foldedRanges, unfoldCode } from "@codemirror/language";
import type { EditorView } from "@codemirror/view";
import { useUiCommands } from "../lib/uiActionBridge";
import type { UiCommand, UiField, UiValue } from "../lib/uiActions";
const identities = new WeakMap<object, string>();
function identity(state: object) { let id = identities.get(state); if (!id) { id = crypto.randomUUID(); identities.set(state, id); } return id; }

/** Source is disclosed by explicit line ranges; scrolling never reads it. */
export function useEditorActions(view: EditorView | null, subject: string) {
  const instance = useId(); const parent = `editor:${subject}:${instance}`;
  const commands: { key: string; command: UiCommand }[] = [];
  const number = { type: "number" as const, minimum: 0, maximum: 100_000_000, integer: true };
  function add(id: string, label: string, parameters: Record<string, UiField>, execute: (args: Record<string, UiValue>, view: EditorView) => UiValue | Promise<UiValue>) {
    commands.push({ key: `${parent}/${id}`, command: {
      label, subject: parent, parent, parameters, prerequisites: ["Current editor state; coordinates are UTF-16 code units unless labelled as lines"],
      read: () => ({ revision: view ? identity(view.state) : "unavailable", enabled: !!view?.dom.isConnected, reason: !view?.dom.isConnected ? "Editor was closed" : undefined,
        value: view ? { subject, length: view.state.doc.length, lines: view.state.doc.lines, selection: { anchor: view.state.selection.main.anchor, head: view.state.selection.main.head }, read_only: view.state.readOnly, edit_baseline: false } : null }),
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
    return { editor: subject, revision: identity(editor.state), coordinate_unit: "UTF-16 code units", first_line: first, last_line: last, span: { start, end }, text: editor.state.sliceDoc(start, end), total_lines: editor.state.doc.lines, next_line: last < editor.state.doc.lines ? last + 1 : null, edit_baseline: false };
  });
  add("select", "Select editor range", { anchor: number, head: number }, (args, editor) => {
    const anchor = Number(args.anchor), head = Number(args.head);
    if (anchor > editor.state.doc.length || head > editor.state.doc.length) throw Error("Selection is outside this editor revision");
    editor.dispatch({ selection: { anchor, head }, scrollIntoView: true }); editor.focus();
    return { selection: { anchor, head }, coordinate_unit: "UTF-16 code units" };
  });
  add("folds", "Read editor folds", {}, (_args, editor) => {
    const folds: { from: number; to: number }[] = []; foldedRanges(editor.state).between(0, editor.state.doc.length, (from, to) => { folds.push({ from, to }); });
    return { folds, coordinate_unit: "UTF-16 code units", revision: identity(editor.state) };
  });
  add("fold", "Expand or collapse editor section", { line: { ...number, minimum: 1 }, expanded: { type: "boolean" } }, (args, editor) => {
    const line = Number(args.line); if (line > editor.state.doc.lines) throw Error("Line is outside this editor");
    editor.dispatch({ selection: { anchor: editor.state.doc.line(line).from } });
    const changed = (args.expanded ? unfoldCode : foldCode)(editor);
    return { changed, line, expanded: Boolean(args.expanded), meaning: "Actual CodeMirror fold command result; no source was edited" };
  });
  useUiCommands(view ? commands : []);
}
