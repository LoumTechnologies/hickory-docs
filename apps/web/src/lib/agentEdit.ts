import type { EditorView } from "@codemirror/view";
import type { AgentChange } from "../api/acp";
import { computeEdits } from "./diff";

export class AgentEditConflict extends Error {
  constructor(message: string, public readonly currentText: string) { super(message); }
}

/** Apply to the identified buffer, never whichever editor currently has focus. */
export function applyAgentEdit(view: EditorView | null | undefined, change: AgentChange): void {
  if (!view || !view.dom.isConnected) throw new Error("The target editor is unavailable. Open it and ask the agent again.");
  if (view.state.readOnly) throw new Error("This editor is read-only. Open its editable source before applying a change.");
  if (view.state.doc.toString() !== change.oldText) throw new AgentEditConflict("The document changed after the agent read it. Nothing was applied; ask the agent to read it again.", view.state.doc.toString());
  view.dispatch({ changes: computeEdits(change.oldText, change.newText).map(e => ({ from: e.start, to: e.end, insert: e.text })) });
}
