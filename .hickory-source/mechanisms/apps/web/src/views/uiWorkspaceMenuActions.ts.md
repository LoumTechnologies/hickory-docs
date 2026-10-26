<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/uiWorkspaceMenuActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d4b2f02b516e"></a>
## `apps/web/src/views/uiWorkspaceMenuActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [New Document Creates Another Unnamed Buffer](../../../../../guarantees/authoring/new-document-is-an-act/contract.md), [Shared print requests do not attest physical output](../../../../../guarantees/authoring/print-requests-do-not-attest-physical-output/contract.md).

<a id="f-d4b2f02b516e-actions"></a>
### `actions`

<storycode:copy id="f-d4b2f02b516e-actions">
import { useId, useRef } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiActor, UiCommand, UiOutcome, UiValue } from "../lib/uiActions";
import type { MenuAction } from "../lib/menuBridge";
const actions = [
  ["save", "Save focused buffer"], ["save-as", "Save focused buffer as"], ["save-all", "Save all workspace buffers"],
  ["print", "Print focused buffer"], ["insert", "Open Insert menu"], ["edit-element", "Edit element at focused selection"],
] as const;
</storycode:copy>
<a id="f-d4b2f02b516e-useWorkspaceMenuActions"></a>
### `useWorkspaceMenuActions`

<storycode:copy id="f-d4b2f02b516e-useWorkspaceMenuActions">
export function useWorkspaceMenuActions(snapshot: () => { visible: boolean; guard: unknown; status: UiValue }, execute: (action: MenuAction, actor: UiActor, destination?: string) => Promise<UiOutcome>) {
  const parent = `workspace-menu:${useId()}`;
  const last = useRef<UiValue>(null);
  const revision = useUiRevision(() => [snapshot().guard, last.current]);
  const commands: { key: string; command: UiCommand }[] = actions.map(([action, label]) => ({ key: `${parent}:${action}`, command: { label, parent, subject: parent, parameters: {}, prerequisites: ["Current window, focused buffer and source/UI revision", ...(action === "print" ? ["Opens the same browser/OS print callback as the native menu. Printer selection is OS-owned; printed vs cancelled and physical output are not observable."] : [])], read: () => ({ revision: revision(), enabled: snapshot().visible }), execute: async (_args, actor) => {
    const outcome = await execute(action, actor);
    last.current = JSON.parse(JSON.stringify({ action, actor, outcome })); return outcome;
  } } }));
  commands.push({ key: `${parent}:destination`, command: { label: "Save unnamed buffer to specified path", parent, subject: parent,
    parameters: { path: { type: "string", maxLength: 4096 } }, prerequisites: ["Current focused unnamed buffer; destination must be a new .md file. Existing files are refused."],
    read: () => { const status = snapshot().status as { target?: { kind?: string } } | null;
      return { revision: revision(), enabled: snapshot().visible && status?.target?.kind === "untitled" }; },
    execute: async (args, actor) => {
      const outcome = await execute("save-as", actor, args.path as string);
      last.current = JSON.parse(JSON.stringify({ action: "save-to-path", actor, destination: args.path, outcome })); return outcome;
    },
  } });
  commands.push({ key: `${parent}:read`, command: { label: "Read workspace command status bytes", parent, subject: `${parent}:read`, parameters: { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, prerequisites: [], read: () => ({ revision: revision(), enabled: snapshot().visible }), execute: async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ current: snapshot().status, last: last.current }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside workspace command status" };
    return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  } } });
  const invoke = useUiCommands(commands);
  return async (action: MenuAction) => {
    if (actions.some(([id]) => id === action)) await invoke(`${parent}:${action}`);
    else await execute(action, { id: "local-user", kind: "human" });
  };
}
</storycode:copy>

### Assemble `apps/web/src/views/uiWorkspaceMenuActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/uiWorkspaceMenuActions.ts"><storycode:paste select="#f-d4b2f02b516e-actions"/><storycode:paste select="#f-d4b2f02b516e-useWorkspaceMenuActions"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-34e80d20ccfb"></a>
## `apps/web/src/views/uiWorkspaceMenuActions.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [New Document Creates Another Unnamed Buffer](../../../../../guarantees/authoring/new-document-is-an-act/contract.md).

<a id="f-34e80d20ccfb-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-34e80d20ccfb-expression_statement">
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useWorkspaceMenuActions } from "./uiWorkspaceMenuActions";
afterEach(cleanup);
</storycode:copy>
<a id="f-34e80d20ccfb-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-34e80d20ccfb-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("checks live focused source before dispatch even without a React render and shares human routing", async () => {
  const effects = vi.fn(async () => ({ status: "completed" as const, value: { handler_returned: true } }));
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let focus = "first", source = "PRIVATE INITIAL SOURCE";
  let human!: ReturnType<typeof useWorkspaceMenuActions>;
  function Harness() { human = useWorkspaceMenuActions(() => ({ visible: true, guard: [focus, source], status: { focus } }), effects); return null; }
  render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  const parent = registry.groups().groups[0];
  const old = registry.observe(parent).actions.find(a => a.label === "Save focused buffer")!;
  expect(JSON.stringify(registry.observe(parent))).not.toContain(source);
  focus = "second"; source = "DIFFERENT PRIVATE SOURCE";
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "stale" })).toMatchObject({ status: "stale" });
  expect(effects).not.toHaveBeenCalled();
  const current = registry.observe(parent).actions.find(a => a.label === "Save focused buffer")!;
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: current.handle, arguments: {}, request_id: "save" }); });
  expect(effects).toHaveBeenLastCalledWith("save", { id: "python:turn", kind: "agent" });
  await act(async () => { await human("save"); });
  expect(effects).toHaveBeenLastCalledWith("save", { id: "local-user", kind: "human" });
});
</storycode:copy>
<a id="f-34e80d20ccfb-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-34e80d20ccfb-expression_statement-3">
// Guarantee: docs/guarantees/authoring/new-document-is-an-act.md
it("binds explicit unnamed-save destinations to the current focused buffer", async () => {
  const effects = vi.fn(async () => ({ status: "completed" as const, value: { save_handler_returned: true } }));
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let kind = "untitled", id = "first", source = "captured source";
  function Harness() {
    useWorkspaceMenuActions(() => ({ visible: true, guard: [id, kind, source], status: { target: { id, kind } } }), effects);
    return null;
  }
  render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  const action = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions).find(action => action.label === "Save unnamed buffer to specified path")!;
  const stale = action().handle;
  id = "second"; source = "different source";
  await act(async () => { expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: stale, arguments: { path: "chosen.md" }, request_id: "stale-path" })).toMatchObject({ status: "stale" }); });
  expect(effects).not.toHaveBeenCalled();
  await act(async () => { expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: action().handle, arguments: { path: "chosen.md" }, request_id: "path" })).toMatchObject({ status: "result" }); });
  expect(effects).toHaveBeenCalledExactlyOnceWith("save-as", { id: "acp:fixture", kind: "agent" }, "chosen.md");
  kind = "document";
  await act(async () => { expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: action().handle, arguments: { path: "other.md" }, request_id: "named" })).toMatchObject({ status: "disabled" }); });
  expect(effects).toHaveBeenCalledTimes(1);
});
</storycode:copy>

### Assemble `apps/web/src/views/uiWorkspaceMenuActions.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/uiWorkspaceMenuActions.test.tsx"><storycode:paste select="#f-34e80d20ccfb-expression_statement"/><storycode:paste select="#f-34e80d20ccfb-expression_statement-2"/><storycode:paste select="#f-34e80d20ccfb-expression_statement-3"/></storycode:file>


</storycode:doc>
