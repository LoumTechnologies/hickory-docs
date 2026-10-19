import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import type { ReactNode } from "react";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "./uiEditorActions";
afterEach(cleanup);
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("reads bounded Unicode lines and uses real editor selection without editing source", async () => {
  const host = document.createElement("div"); document.body.append(host);
  const source = "# λ😀\nsecond line\nthird line";
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: source }) });
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  const wrapper = ({ children }: { children: ReactNode }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider>;
  const hook = renderHook(() => useEditorActions(view, "deadline.md"), { wrapper });
  const descriptor = (label: string) => registry.observe(registry.groups().groups[0]).actions.find(item => item.label === label)!;
  const actor = { id: "python:read", kind: "agent" as const };
  const read = descriptor("Read editor lines");
  expect(await registry.act(registry.scope, actor, { handle: read.handle, arguments: { first: 1, count: 1 }, request_id: "read" })).toMatchObject({ status: "result", outcome: { value: { text: "# λ😀", first_line: 1, last_line: 1, next_line: 2, edit_baseline: false } } });
  const select = descriptor("Select editor range");
  await act(async () => { await registry.act(registry.scope, actor, { handle: select.handle, arguments: { anchor: 0, head: 5 }, request_id: "select" }); });
  expect(view.state.selection.main.head).toBe(5);
  expect(view.state.doc.toString()).toBe(source);
  expect(await registry.act(registry.scope, actor, { handle: read.handle, arguments: { first: 1, count: 1 }, request_id: "old-state" })).toMatchObject({ status: "stale" });
  hook.unmount(); view.destroy(); host.remove();
});
