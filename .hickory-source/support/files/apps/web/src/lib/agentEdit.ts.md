<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/agentEdit.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `171c401926d18decb5a86c367d38f512ac70509d1161be131130d3fa02d8bf25`.

<book:entry path="apps/web/src/lib/agentEdit.ts">
import { isolateHistory } from "@codemirror/commands";
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
  view.dispatch({ userEvent: "input.apply", annotations: isolateHistory.of("full"), changes: computeEdits(change.oldText, change.newText).map(e => ({ from: e.start, to: e.end, insert: e.text })) });
}
</book:entry>

</book:doc>
