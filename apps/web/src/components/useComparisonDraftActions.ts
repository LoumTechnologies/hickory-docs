import { useId } from "react";
import type { EditorView } from "@codemirror/view";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";

/** Edit the same CodeMirror draft as typing, with a live source-state guard. */
export function useComparisonDraftActions(view: EditorView | null, editable: boolean, source: string) {
  const parent = `comparison-draft:${useId()}`;
  const revision = useUiRevision(() => [source, editable, view?.dom.isConnected, view?.state.readOnly, view?.state.doc.toString(), view?.state.selection.toJSON()]);
  useUiCommands(view ? [{ key: parent, command: {
    parent, subject: parent, label: "Replace comparison draft range",
    parameters: { from: { type: "number", minimum: 0, maximum: 100000000, integer: true }, to: { type: "number", minimum: 0, maximum: 100000000, integer: true }, text: { type: "string", maxLength: 1048576 } }, prerequisites: [],
    read: () => ({ revision: revision(), enabled: editable && view.dom.isConnected && !view.state.readOnly, value: { source, length: view.state.doc.length, scope: "Visible draft only; no save, proposal approval or verification" } }),
    execute: async (args, actor) => {
      if (!editable || !view.dom.isConnected || view.state.readOnly) return { status: "unavailable", reason: "Comparison draft became read-only or closed" };
      const from = args.from as number, to = args.to as number;
      if (from > to || to > view.state.doc.length) return { status: "unavailable", reason: "Range outside current comparison draft" };
      view.dispatch({ changes: { from, to, insert: args.text as string }, userEvent: actor.kind === "agent" ? "input.uiAgentComparison" : "input" });
      return { status: "completed", value: { draft_changed: true, saved: false, approved: false, verified: false } };
    },
  } }] : []);
}
