<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/agentEdit.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-71947c69b25c"></a>
## `apps/web/src/lib/agentEdit.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-71947c69b25c-AgentEditConflict"></a>
### `AgentEditConflict`

<storycode:copy id="f-71947c69b25c-AgentEditConflict">
import { isolateHistory } from "@codemirror/commands";
import type { EditorView } from "@codemirror/view";
import type { AgentChange } from "../api/acp";
import { computeEdits } from "./diff";

export class AgentEditConflict extends Error {
  constructor(message: string, public readonly currentText: string) { super(message); }
}

</storycode:copy>
<a id="f-71947c69b25c-applyAgentEdit"></a>
### `applyAgentEdit`

Apply to the identified buffer, never whichever editor currently has focus.

<storycode:copy id="f-71947c69b25c-applyAgentEdit">
/** Apply to the identified buffer, never whichever editor currently has focus. */
export function applyAgentEdit(view: EditorView | null | undefined, change: AgentChange): void {
  if (!view || !view.dom.isConnected) throw new Error("The target editor is unavailable. Open it and ask the agent again.");
  if (view.state.readOnly) throw new Error("This editor is read-only. Open its editable source before applying a change.");
  if (view.state.doc.toString() !== change.oldText) throw new AgentEditConflict("The document changed after the agent read it. Nothing was applied; ask the agent to read it again.", view.state.doc.toString());
  view.dispatch({ userEvent: "input.apply", annotations: isolateHistory.of("full"), changes: computeEdits(change.oldText, change.newText).map(e => ({ from: e.start, to: e.end, insert: e.text })) });
}
</storycode:copy>

### Assemble `apps/web/src/lib/agentEdit.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/agentEdit.ts"><storycode:paste select="#f-71947c69b25c-AgentEditConflict"/><storycode:paste select="#f-71947c69b25c-applyAgentEdit"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-ebba6809ee8c"></a>
## `apps/web/src/lib/agentEdit.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Conversation edits use the client's review policy](../../../../../guarantees/agent/conversation-edits-use-the-client-review-policy/contract.md), [A final changeset revision applies B to C](../../../../../guarantees/agent/final-changeset-revisions-apply-b-to-c/contract.md).

<a id="f-ebba6809ee8c-change"></a>
### `change`

<storycode:copy id="f-ebba6809ee8c-change">
// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
import { expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { history, undo } from "@codemirror/commands";
import { applyAgentEdit } from "./agentEdit";
import type { AgentChange } from "../api/acp";
const change: AgentChange = { id: "edit", name: "Untitled", path: null, oldText: "# Note 📝\n\nKeep this\n", newText: "# Changed 📝\n\nKeep this\n", editor: true, status: "pending" };
</storycode:copy>
<a id="f-ebba6809ee8c-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-ebba6809ee8c-expression_statement">
it("edits the live note and supports normal editor undo", () => {
  const view = new EditorView({ state: EditorState.create({ doc: change.oldText, extensions: [history()] }), parent: document.body });
  try {
    applyAgentEdit(view, change);
    expect(view.state.doc.toString()).toBe(change.newText);
    expect(undo(view)).toBe(true);
    expect(view.state.doc.toString()).toBe(change.oldText);
  } finally { view.destroy(); }
});
</storycode:copy>
<a id="f-ebba6809ee8c-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-ebba6809ee8c-expression_statement-2">
it("preserves typing that arrived during review by refusing the stale proposal", () => {
  const view = new EditorView({ state: EditorState.create({ doc: change.oldText }), parent: document.body });
  try {
    view.dispatch({ changes: { from: view.state.doc.length, insert: "My new sentence\n" } });
    const typed = view.state.doc.toString();
    expect(() => applyAgentEdit(view, change)).toThrow("The document changed");
    expect(view.state.doc.toString()).toBe(typed);
  } finally { view.destroy(); }
});
</storycode:copy>
<a id="f-ebba6809ee8c-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-ebba6809ee8c-expression_statement-3">
it("refuses a closed editor", () => {
  const view = new EditorView({ state: EditorState.create({ doc: change.oldText }) });
  try { expect(() => applyAgentEdit(view, change)).toThrow("unavailable"); }
  finally { view.destroy(); }
});

</storycode:copy>
<a id="f-ebba6809ee8c-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-ebba6809ee8c-expression_statement-4">
// Guarantee: docs/guarantees/agent/final-changeset-revisions-apply-b-to-c.md
it("undoes an applied revision separately from immediately preceding typing", () => {
  const view = new EditorView({state:EditorState.create({doc:"A",extensions:[history()]}),parent:document.body});
  try {
    view.dispatch({changes:{from:0,to:1,insert:"B"},userEvent:"input.type"});
    applyAgentEdit(view,{...change,oldText:"B",newText:"C"});
    expect(undo(view)).toBe(true);
    expect(view.state.doc.toString()).toBe("B");
    expect(undo(view)).toBe(true);
    expect(view.state.doc.toString()).toBe("A");
  } finally { view.destroy(); }
});
</storycode:copy>

### Assemble `apps/web/src/lib/agentEdit.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/agentEdit.test.ts"><storycode:paste select="#f-ebba6809ee8c-change"/><storycode:paste select="#f-ebba6809ee8c-expression_statement"/><storycode:paste select="#f-ebba6809ee8c-expression_statement-2"/><storycode:paste select="#f-ebba6809ee8c-expression_statement-3"/><storycode:paste select="#f-ebba6809ee8c-expression_statement-4"/></storycode:file>


</storycode:doc>
