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
