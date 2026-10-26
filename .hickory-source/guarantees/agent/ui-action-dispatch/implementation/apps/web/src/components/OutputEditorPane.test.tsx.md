<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/OutputEditorPane.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0c1c6071248d"></a>
## `apps/web/src/components/OutputEditorPane.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-0c1c6071248d-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-0c1c6071248d-expression_statement">
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorView } from "@codemirror/view";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { OutputEditorPane } from "./OutputEditorPane";
afterEach(cleanup);
</storycode:copy>
<a id="f-0c1c6071248d-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-0c1c6071248d-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("keeps reading-only output immutable and sends agent edits through an explicit existing edit callback", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const file = { path: "generated.txt", language: "text", content: "before", provenance: [] };
  const local = vi.fn();
  const mount = (editable: boolean, source = file.content) => <UiActionContext.Provider value={registry}><OutputEditorPane file={{ ...file, content: source }} onLocalEdit={editable ? local : undefined} /></UiActionContext.Provider>;
  const ui = render(mount(false));
  await waitFor(() => expect(document.querySelector(".cm-content")).toBeTruthy());
  const view = EditorView.findFromDOM(document.querySelector<HTMLElement>(".cm-editor")!)!;
  const action = () => registry.groups().groups.flatMap(g => registry.observe(g).actions).find(a => a.label === "Replace output editor range")!;
  expect(view.state.readOnly).toBe(true); expect(action().enabled).toBe(false);
  act(() => { view.dispatch({ changes: { from: 0, to: 6, insert: "wrong" }, userEvent: "input" }); });
  expect(view.state.doc.toString()).toBe("before");
  ui.rerender(mount(true));
  expect(view.state.readOnly).toBe(false);
  let result: unknown;
  await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action().handle, arguments: { from: 0, to: 6, text: "changed" }, request_id: "edit" }); });
  expect(result).toMatchObject({ status: "result", outcome: { value: { requested_applied: true, approved: false, verified: false } } });
  expect(local).toHaveBeenCalledWith("changed");
  ui.rerender(mount(false, "new captured source"));
  expect(view.state.readOnly).toBe(true); expect(action().enabled).toBe(false);
  expect(view.state.doc.toString()).toBe("new captured source");
  expect(local).toHaveBeenCalledOnce();
});
</storycode:copy>

### Assemble `apps/web/src/components/OutputEditorPane.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/OutputEditorPane.test.tsx"><storycode:paste select="#f-0c1c6071248d-expression_statement"/><storycode:paste select="#f-0c1c6071248d-expression_statement-2"/></storycode:file>

</storycode:doc>
