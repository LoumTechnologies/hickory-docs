<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/agentEdit.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Conversation edits use the client's review policy](../../../../../../guarantees/agent/conversation-edits-use-the-client-review-policy/contract.md)
- [A final changeset revision applies B to C](../../../../../../guarantees/agent/final-changeset-revisions-apply-b-to-c/contract.md)

Captured SHA-256: `f4a42ed2adee16fe588c4492c0406cfef3b621f0f79f8b23848ce5499f4ed3bd`.

<book:entry path="apps/web/src/lib/agentEdit.test.ts">
// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
import { expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { history, undo } from "@codemirror/commands";
import { applyAgentEdit } from "./agentEdit";
import type { AgentChange } from "../api/acp";
const change: AgentChange = { id: "edit", name: "Untitled", path: null, oldText: "# Note 📝\n\nKeep this\n", newText: "# Changed 📝\n\nKeep this\n", editor: true, status: "pending" };
it("edits the live note and supports normal editor undo", () => {
  const view = new EditorView({ state: EditorState.create({ doc: change.oldText, extensions: [history()] }), parent: document.body });
  try {
    applyAgentEdit(view, change);
    expect(view.state.doc.toString()).toBe(change.newText);
    expect(undo(view)).toBe(true);
    expect(view.state.doc.toString()).toBe(change.oldText);
  } finally { view.destroy(); }
});
it("preserves typing that arrived during review by refusing the stale proposal", () => {
  const view = new EditorView({ state: EditorState.create({ doc: change.oldText }), parent: document.body });
  try {
    view.dispatch({ changes: { from: view.state.doc.length, insert: "My new sentence\n" } });
    const typed = view.state.doc.toString();
    expect(() => applyAgentEdit(view, change)).toThrow("The document changed");
    expect(view.state.doc.toString()).toBe(typed);
  } finally { view.destroy(); }
});
it("refuses a closed editor", () => {
  const view = new EditorView({ state: EditorState.create({ doc: change.oldText }) });
  try { expect(() => applyAgentEdit(view, change)).toThrow("unavailable"); }
  finally { view.destroy(); }
});

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
</book:entry>

</book:doc>
