// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
import { EditorState, StateEffect, Transaction } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { afterEach, expect, it } from "vitest";
import { structureOf } from "./wysiwyg";
import { replaceCapturedBlock } from "./replaceCapturedBlock";
const source = '<hick:diagram renderer="graph">\n{"nodes":[],"edges":[]}\n</hick:diagram>\n';
const views: EditorView[] = [];
afterEach(() => { for (const view of views.splice(0)) { const parent = view.dom.parentElement; view.destroy(); parent?.remove(); } });
function editor(readOnly = false) {
  const parent = document.body.appendChild(document.createElement("div"));
  const events: string[] = [];
  const view = new EditorView({ parent, state: EditorState.create({ doc: source, extensions: [EditorState.readOnly.of(readOnly), EditorView.updateListener.of(update => { for (const transaction of update.transactions) { const event = transaction.annotation(Transaction.userEvent); if (event) events.push(event); } })] }) });
  views.push(view);
  const block = structureOf(view.state).blocks.find(block => block.name === "diagram")!;
  return { view, events, slot: { at: block.from, text: view.state.doc.sliceString(block.contentFrom, block.contentTo) } };
}
it("writes only the captured block and retains the agent's transaction origin", () => {
  const { view, events, slot } = editor();
  expect(replaceCapturedBlock(view, slot, '\n{"nodes":[{"id":"agent"}],"edges":[]}', "input.uiAgentDiagram")).toBe(true);
  expect(view.state.doc.toString()).toContain('"id":"agent"');
  expect(events).toEqual(["input.uiAgentDiagram"]);
  expect(view.state.doc.toString()).toContain("</hick:diagram>");
});
it("refuses read-only, changed and disconnected captured blocks", () => {
  const locked = editor(true);
  expect(replaceCapturedBlock(locked.view, locked.slot, "rejected", "input.uiAgentDiagram")).toBe(false);
  expect(locked.view.state.doc.toString()).toBe(source);
  const { view, slot } = editor();
  view.dispatch({ changes: { from: source.indexOf('"nodes"'), insert: '"changed":true,' } });
  const changed = view.state.doc.toString();
  expect(replaceCapturedBlock(view, slot, "stale", "input.uiAgentDiagram")).toBe(false);
  expect(view.state.doc.toString()).toBe(changed);
  view.dom.remove();
  expect(replaceCapturedBlock(view, slot, "closed", "input.uiAgentDiagram")).toBe(false);
});

it("keeps a Markdown table replacement in one source edit with working Undo", async () => {
  const { history, undo } = await import("@codemirror/commands");
  const { renderableBlocks } = await import("./rendered");
  const initial = '# Prose stays\n\n| Name | Count |\n| --- | --- |\n| before | 1 |\n';
  const parent = document.body.appendChild(document.createElement("div"));
  const view = new EditorView({ parent, state: EditorState.create({ doc: initial, extensions: [history()] }) });
  views.push(view);
  const block = renderableBlocks(structureOf(view.state), initial).find(block => block.name === "markdown-table")!;
  const slot = { at: block.from, text: view.state.doc.sliceString(block.contentFrom, block.contentTo) };
  expect(replaceCapturedBlock(view, slot, '| Name | Count |\n| --- | --- |\n| after | 2 |', "input.uiAgentTable")).toBe(true);
  expect(view.state.doc.toString()).toContain("# Prose stays\n\n| Name | Count |");
  expect(view.state.doc.toString()).toContain("| after | 2 |");
  expect(undo(view)).toBe(true);
  expect(view.state.doc.toString()).toBe(initial);
});


it("returns actual filter refusal instead of acknowledging an unapplied source edit", () => {
  const { view, slot } = editor();
  view.dispatch({ effects: StateEffect.appendConfig.of(EditorState.transactionFilter.of(transaction => transaction.docChanged ? [] : transaction)) });
  expect(replaceCapturedBlock(view, slot, "rejected by owner", "input.uiAgentDiagram")).toBe(false);
  expect(view.state.doc.toString()).toBe(source);
});
