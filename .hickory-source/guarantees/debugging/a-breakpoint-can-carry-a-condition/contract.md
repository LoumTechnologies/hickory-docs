<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Breakpoint Can Carry A Condition

For: Authors debugging generated code while navigating its document source.

Given a breakpoint in a document or a plain file, when it is right-clicked or
alt-clicked in the gutter, then a panel opens on that line offering three
things — a **condition**, a **hit count**, and a **log message** — and what is
typed reaches the adapter on the next `setBreakpoints`, or on the next run if
nothing is running.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/a-breakpoint-can-carry-a-condition.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-e431ad05f82f"></a>
## `apps/web/src/debug/breakpointForm.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Breakpoint Can Carry A Condition](contract.md).

<a id="f-e431ad05f82f-view"></a>
### `view`

<storycode:copy id="f-e431ad05f82f-view">
// Guarantee: docs/guarantees/debugging/a-breakpoint-can-carry-a-condition.md
import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "../editor/uiEditorActions";
import { debugEditor, setBreakpointMarks } from "./cmDebug";
let view: EditorView | undefined;
</storycode:copy>
<a id="f-e431ad05f82f-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-e431ad05f82f-expression_statement">
afterEach(() => { cleanup(); const host = view?.dom.parentElement; view?.destroy(); host?.remove(); });
</storycode:copy>
<a id="f-e431ad05f82f-setup"></a>
### `setup`

<storycode:copy id="f-e431ad05f82f-setup">
async function setup() {
  const toggle = vi.fn(), set = vi.fn(), host = document.body.appendChild(document.createElement("div"));
  const current = new EditorView({ parent: host, state: EditorState.create({ doc: "value = 1\nprint(value)\n", extensions: debugEditor({ language: "python", onToggleBreakpoint: toggle, onSetBreakpointCondition: set }) }) }); view = current;
  const registry = new UiActions({ workspace: "p", window: "breakpoint" }, () => true, async () => {});
  function Bridge() { useEditorActions(current, "breakpoint"); return null; }
  render(<UiActionContext.Provider value={registry}><Bridge /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args: Record<string, UiValue> = {}) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "acp:condition", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return reply; };
  return { current, registry, actions, run, set, toggle };
}
</storycode:copy>
<a id="f-e431ad05f82f-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-e431ad05f82f-expression_statement-2">
it("shares actual drafts and native/agent apply while allowing the new breakpoint's adapter acknowledgement", async () => {
  const { current, actions, run, set, toggle } = await setup();
  expect(actions().some(action => action.label === "Read gutter breakpoint editor bytes")).toBe(false);
  await run("Open gutter breakpoint condition editor", { line: 1 }); expect(toggle).toHaveBeenCalledWith(0);
  const input = current.dom.querySelector<HTMLInputElement>(".cm-bp-editor input")!;
  await run("Set gutter breakpoint editor field", { field: "condition", text: "  value > 2  " }); expect(input.value).toBe("  value > 2  ");
  await act(async () => current.dispatch({ effects: setBreakpointMarks.of([{ line: 0, state: "bound", conditional: false }]) }));
  expect(input.isConnected).toBe(true);
  const reply = await run("Read gutter breakpoint editor bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(JSON.parse(new TextDecoder().decode(new Uint8Array(reply.outcome.value.bytes)))).toMatchObject({ line: 1, fields: { condition: "  value > 2  " } });
  expect(await run("Apply gutter breakpoint editor")).toMatchObject({ outcome: { value: { callback_requested: true, adapter_acceptance_unknown: true } } });
  expect(set).toHaveBeenCalledWith(0, { condition: "value > 2", hit_condition: "", log_message: "" });
  await run("Open gutter breakpoint condition editor", { line: 2 });
  const native = current.dom.querySelector<HTMLInputElement>(".cm-bp-editor input")!;
  await act(async () => { fireEvent.input(native, { target: { value: "native condition" } }); fireEvent.keyDown(native, { key: "Enter" }); });
  expect(set).toHaveBeenLastCalledWith(1, { condition: "native condition", hit_condition: "", log_message: "" });
  expect(current.state.doc.toString()).toBe("value = 1\nprint(value)\n");
});
</storycode:copy>
<a id="f-e431ad05f82f-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-e431ad05f82f-expression_statement-3">
it("preserves another focused field and refuses stale native/agent application after source or target replacement", async () => {
  const { current, registry, actions, run, set } = await setup();
  const external = document.body.appendChild(document.createElement("input")); external.focus();
  expect(await run("Open gutter breakpoint condition editor", { line: 1 })).toMatchObject({ outcome: { status: "requires_input" } });
  expect(document.activeElement).toBe(external); external.remove();
  await run("Open gutter breakpoint condition editor", { line: 1 });
  const native = current.dom.querySelector<HTMLInputElement>(".cm-bp-editor input")!;
  const old = actions().find(action => action.label === "Apply gutter breakpoint editor")!;
  await act(async () => current.dispatch({ changes: { from: 0, insert: "# shifted\n" } }));
  expect(current.dom.querySelector(".cm-bp-editor")).toBeNull();
  await act(async () => fireEvent.keyDown(native, { key: "Enter" })); expect(set).not.toHaveBeenCalled();
  expect(await registry.act(registry.scope, { id: "acp:condition", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "stale" })).not.toMatchObject({ outcome: { status: "completed" } });
  await act(async () => current.dispatch({ effects: setBreakpointMarks.of([{ line: 1, state: "bound", conditional: false }]) }));
  await run("Open gutter breakpoint condition editor", { line: 2 });
  await act(async () => current.dispatch({ effects: setBreakpointMarks.of([]) }));
  expect(current.dom.querySelector(".cm-bp-editor")).toBeNull();
  await run("Open gutter breakpoint condition editor", { line: 2 });
  const latest = current.dom.querySelector<HTMLInputElement>(".cm-bp-editor input")!;
  await act(async () => { fireEvent.input(latest, { target: { value: "blur condition" } }); latest.blur(); });
  await waitFor(() => expect(set).toHaveBeenCalledWith(1, { condition: "blur condition", hit_condition: "", log_message: "" }));
});
</storycode:copy>

### Assemble `apps/web/src/debug/breakpointForm.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/breakpointForm.test.tsx"><storycode:paste select="#f-e431ad05f82f-view"/><storycode:paste select="#f-e431ad05f82f-expression_statement"/><storycode:paste select="#f-e431ad05f82f-setup"/><storycode:paste select="#f-e431ad05f82f-expression_statement-2"/><storycode:paste select="#f-e431ad05f82f-expression_statement-3"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/debug/client.ts](../../../mechanisms/apps/web/src/debug/client.ts.md#output-9c2c3f79dace)
- [apps/web/src/debug/cmDebug.ts](../../../mechanisms/apps/web/src/debug/cmDebug.ts.md#output-9df55a31763b)
- [apps/web/src/debug/useDebugger.ts](../../../mechanisms/apps/web/src/debug/useDebugger.ts.md#output-597e991b795b)
- [crates/hick-dap/src/session.rs](../../../mechanisms/crates/hick-dap/src/session.rs.md#output-3c41c22e0551)

### Tests and fixtures

- [apps/web/src/debug/breakpointForm.test.tsx](contract.md#output-e431ad05f82f)
- [apps/web/src/debug/cmDebug.test.ts](../../../mechanisms/apps/web/src/debug/cmDebug.ts.md#output-8d12c7278102)
- [apps/web/src/debug/useDebugger.test.ts](../../../mechanisms/apps/web/src/debug/useDebugger.ts.md#output-101aa3d58173)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
