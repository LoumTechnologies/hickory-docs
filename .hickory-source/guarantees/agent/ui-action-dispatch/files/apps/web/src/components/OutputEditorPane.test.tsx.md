<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/OutputEditorPane.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `462c3599120c6f216b9a32654df0ffa6d24fabc70d8730a07b916a60e2e2373a`.

<book:entry path="apps/web/src/components/OutputEditorPane.test.tsx">
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorView } from "@codemirror/view";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { OutputEditorPane } from "./OutputEditorPane";
afterEach(cleanup);
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
</book:entry>

</book:doc>
