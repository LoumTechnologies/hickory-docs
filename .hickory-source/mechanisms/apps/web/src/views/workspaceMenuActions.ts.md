<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/workspaceMenuActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b6d5b22ec427"></a>
## `apps/web/src/views/workspaceMenuActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [New Document Creates Another Unnamed Buffer](../../../../../guarantees/authoring/new-document-is-an-act/contract.md), [Shared print requests do not attest physical output](../../../../../guarantees/authoring/print-requests-do-not-attest-physical-output/contract.md).

<a id="f-b6d5b22ec427-Session"></a>
### `Session`

<storycode:copy id="f-b6d5b22ec427-Session">
import type { MenuAction } from "../lib/menuBridge";
import { insertTarget } from "../lib/menuBridge";
import type { UiActor, UiOutcome } from "../lib/uiActions";

type Session = { id: string | null; save(): Promise<boolean>; saveAs?(): void };
</storycode:copy>
<a id="f-b6d5b22ec427-MenuScope"></a>
### `MenuScope`

<storycode:copy id="f-b6d5b22ec427-MenuScope">
export type MenuScope = {
  target: { id: string | null; kind: string | null };
  untitled: string[]; untitledNeedingDestination?: string[]; unnamedEmpty?: boolean; sessions: Session[]; session?: Session; working?: { save(): Promise<boolean> };
};
</storycode:copy>
<a id="f-b6d5b22ec427-MenuHandlers"></a>
### `MenuHandlers`

<storycode:copy id="f-b6d5b22ec427-MenuHandlers">
export type MenuHandlers = {
  formatFirst(): Promise<void>;
  saveUntitled(saveAs: boolean, id?: string, destination?: string): Promise<boolean>;
  flush(): void; print(): boolean | void; insert(id: string | null): void; editElement(): void;
};

</storycode:copy>
<a id="f-b6d5b22ec427-runWorkspaceMenu"></a>
### `runWorkspaceMenu`

Capture the target before awaiting formatting/dialogs; never re-read focus.

<storycode:copy id="f-b6d5b22ec427-runWorkspaceMenu">
/** Capture the target before awaiting formatting/dialogs; never re-read focus. */
export async function runWorkspaceMenu(action: MenuAction, actor: UiActor, scope: MenuScope, handlers: MenuHandlers, destination?: string): Promise<UiOutcome> {
  scope = { ...scope, target: { ...scope.target }, untitled: [...scope.untitled], untitledNeedingDestination: scope.untitledNeedingDestination ? [...scope.untitledNeedingDestination] : undefined, sessions: scope.sessions.map(s => ({ ...s })), session: scope.session ? { ...scope.session } : undefined, working: scope.working ? { ...scope.working } : undefined };
  if (action === "insert" || insertTarget(action) !== null) { handlers.insert(insertTarget(action)); return { status: "completed", value: { insert_panel_requested: true, source_written: false } }; }
  if (action === "edit-element") { handlers.editElement(); return { status: "completed", value: { element_editor_requested: true, source_written: false } }; }
  if (action === "print") {
    if (handlers.print() === false) return { status: "unavailable", reason: "Open a document or file to print; no print callback was submitted" };
    return { status: "completed", value: { print_callback_returned: true, physical_output: "not observed", dialog_outcome: "This handler cannot distinguish printed, cancelled or an ignored browser request" } };
  }
  if (action === "save-all") {
    if (actor.kind === "agent" && (scope.untitledNeedingDestination ?? scope.untitled).length) return { status: "requires_input", request: `workspace-save-all:${scope.untitled.join(":")}`, reason: "Name each buffer with Save unnamed buffer to specified path, or use native Save dialogs; no partial Save all was submitted" };
    await handlers.formatFirst();
    for (const id of scope.untitled) if (!await handlers.saveUntitled(false, id)) return { status: "completed", value: { unnamed_save_completed: false, target: id, scope: "Cancelled, unavailable or failed; inspect actual UI result" } };
    const documents = [];
    for (const session of scope.sessions) documents.push({ id: session.id, unchanged_after_save: await session.save() });
    handlers.flush();
    return { status: "completed", value: { documents, self_saving_panes: "Flush requested; completion not established here", verification: "not requested" } };
  }
  if (action !== "save" && action !== "save-as") return { status: "unavailable", reason: "No scoped document handler for this menu action" };
  if (scope.target.kind === "untitled") {
    if (actor.kind === "agent" && scope.unnamedEmpty) return { status: "completed", value: { target: scope.target.id, empty_buffer_no_write: true } };
    if (actor.kind === "agent" && destination === undefined) return { status: "requires_input", request: `workspace-save:${scope.target.id}`, reason: "Choose a destination with Save unnamed buffer to specified path, or use the actual native Save dialog" };
    await handlers.formatFirst();
    const returned = await handlers.saveUntitled(action === "save-as", scope.target.id ?? undefined, destination);
    return { status: "completed", value: { target: scope.target.id, save_handler_returned: returned, path: "Inspect actual resulting named buffer; empty/cancelled buffers may create no file" } };
  }
  if (destination !== undefined) return { status: "unavailable", reason: "An explicit destination belongs to an unnamed buffer. Observe the current focused buffer again." };
  if (action === "save" && scope.working) {
    await handlers.formatFirst();
    return { status: "completed", value: { target: scope.target.id, unchanged_after_save: await scope.working.save(), verification: "not requested" } };
  }
  if (!scope.session) return { status: "unavailable", reason: "Open a saveable document or file" };
  if (action === "save") {
    await handlers.formatFirst();
    const unchanged = await scope.session.save(); handlers.flush();
    return { status: "completed", value: { document: scope.session.id, unchanged_after_save: unchanged, other_self_saving_panes: "Flush requested; unconfirmed", verification: "not requested" } };
  }
  if (!scope.session.saveAs) return { status: "unavailable", reason: "This buffer does not expose Save a copy" };
  scope.session.saveAs();
  return { status: "completed", value: { document: scope.session.id, copy_prompt_requested: true, file_created: "not established; complete and inspect the actual copy operation" } };
}
</storycode:copy>

### Assemble `apps/web/src/views/workspaceMenuActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/workspaceMenuActions.ts"><storycode:paste select="#f-b6d5b22ec427-Session"/><storycode:paste select="#f-b6d5b22ec427-MenuScope"/><storycode:paste select="#f-b6d5b22ec427-MenuHandlers"/><storycode:paste select="#f-b6d5b22ec427-runWorkspaceMenu"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-c4f5a89c7b3e"></a>
## `apps/web/src/views/workspaceMenuActions.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [New Document Creates Another Unnamed Buffer](../../../../../guarantees/authoring/new-document-is-an-act/contract.md), [Shared print requests do not attest physical output](../../../../../guarantees/authoring/print-requests-do-not-attest-physical-output/contract.md).

<a id="f-c4f5a89c7b3e-human"></a>
### `human`

<storycode:copy id="f-c4f5a89c7b3e-human">
import { expect, it, vi } from "vitest";
import { runWorkspaceMenu, type MenuHandlers, type MenuScope } from "./workspaceMenuActions";
const human = { id: "local-user", kind: "human" as const }, agent = { id: "python:turn", kind: "agent" as const };
</storycode:copy>
<a id="f-c4f5a89c7b3e-setup"></a>
### `setup`

<storycode:copy id="f-c4f5a89c7b3e-setup">
function setup() {
  const scope: MenuScope = { target: { id: "first", kind: "untitled" }, untitled: ["first"], sessions: [] };
  const handlers: MenuHandlers = { formatFirst: vi.fn(async () => {}), saveUntitled: vi.fn(async () => true), flush: vi.fn(), print: vi.fn(), insert: vi.fn(), editElement: vi.fn() };
  return { scope, handlers };
}
</storycode:copy>
<a id="f-c4f5a89c7b3e-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c4f5a89c7b3e-expression_statement">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("retains the originating unnamed target across async formatting and does not choose new focus", async () => {
  const { scope, handlers } = setup();
  let finish!: () => void;
  handlers.formatFirst = () => new Promise(resolve => { finish = resolve; });
  const pending = runWorkspaceMenu("save", human, scope, handlers);
  // The caller can replace its current scope while formatting is pending.
  scope.target = { id: "second", kind: "untitled" };
  scope.untitled.push("second");
  finish(); await pending;
  expect(handlers.saveUntitled).toHaveBeenCalledExactlyOnceWith(false, "first", undefined);
});
</storycode:copy>
<a id="f-c4f5a89c7b3e-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-c4f5a89c7b3e-expression_statement-2">
it("does not open native Save dialogs or perform partial saves without destinations", async () => {
  const { scope, handlers } = setup();
  expect(await runWorkspaceMenu("save", agent, scope, handlers)).toMatchObject({ status: "requires_input", request: "workspace-save:first" });
  expect(await runWorkspaceMenu("save-all", agent, scope, handlers)).toMatchObject({ status: "requires_input" });
  expect(handlers.formatFirst).not.toHaveBeenCalled(); expect(handlers.saveUntitled).not.toHaveBeenCalled(); expect(handlers.print).not.toHaveBeenCalled();
});
</storycode:copy>
<a id="f-c4f5a89c7b3e-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-c4f5a89c7b3e-expression_statement-3">
it("returns real document save acknowledgement and distinguishes copy/flush intent", async () => {
  const { scope, handlers } = setup();
  const save = vi.fn(async () => false), saveAs = vi.fn();
  scope.target = { id: "doc-tab", kind: "document" }; scope.untitled = [];
  scope.session = { id: "doc", save, saveAs }; scope.sessions = [scope.session];
  expect(await runWorkspaceMenu("save", agent, scope, handlers)).toMatchObject({ status: "completed", value: { document: "doc", unchanged_after_save: false, verification: "not requested" } });
  expect(save).toHaveBeenCalledOnce();
  expect(await runWorkspaceMenu("save-as", agent, scope, handlers)).toMatchObject({ status: "completed", value: { copy_prompt_requested: true, file_created: expect.stringContaining("not established") } });
  expect(saveAs).toHaveBeenCalledOnce();
  expect(await runWorkspaceMenu("save-all", agent, scope, handlers)).toMatchObject({ status: "completed", value: { documents: [{ id: "doc", unchanged_after_save: false }], self_saving_panes: expect.stringContaining("not established") } });
});
</storycode:copy>
<a id="f-c4f5a89c7b3e-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-c4f5a89c7b3e-expression_statement-4">
it("shares existing Insert/edit handlers and writes no source just by opening them", async () => {
  const { scope, handlers } = setup();
  expect(await runWorkspaceMenu("insert:table", agent, scope, handlers)).toMatchObject({ value: { insert_panel_requested: true, source_written: false } });
  expect(handlers.insert).toHaveBeenCalledWith("table");
  await runWorkspaceMenu("edit-element", human, scope, handlers); expect(handlers.editElement).toHaveBeenCalledOnce();
});

</storycode:copy>
<a id="f-c4f5a89c7b3e-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-c4f5a89c7b3e-expression_statement-5">
it("does not request a destination or write for a known empty unnamed buffer with formatting off", async () => {
  const { scope, handlers } = setup(); scope.unnamedEmpty = true;
  expect(await runWorkspaceMenu("save", agent, scope, handlers)).toMatchObject({ value: { empty_buffer_no_write: true } });
  expect(handlers.saveUntitled).not.toHaveBeenCalled(); expect(handlers.formatFirst).not.toHaveBeenCalled();
});

</storycode:copy>
<a id="f-c4f5a89c7b3e-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-c4f5a89c7b3e-expression_statement-6">
// Guarantee: docs/guarantees/authoring/new-document-is-an-act.md
it("uses an agent-specified destination through the same captured unnamed-save handler", async () => {
  const { scope, handlers } = setup();
  expect(await runWorkspaceMenu("save-as", agent, scope, handlers, "notes/Chosen.md")).toMatchObject({ status: "completed", value: { target: "first", save_handler_returned: true } });
  expect(handlers.saveUntitled).toHaveBeenCalledExactlyOnceWith(true, "first", "notes/Chosen.md");
  scope.target = { id: "named", kind: "document" };
  expect(await runWorkspaceMenu("save-as", agent, scope, handlers, "other.md")).toMatchObject({ status: "unavailable" });
  expect(handlers.saveUntitled).toHaveBeenCalledTimes(1);
});

</storycode:copy>
<a id="f-c4f5a89c7b3e-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-c4f5a89c7b3e-expression_statement-7">
// Guarantee: docs/guarantees/authoring/print-requests-do-not-attest-physical-output.md
it("shares the actual print callback and reports only its observable return", async () => {
  const { scope, handlers } = setup();
  for (const actor of [human, agent]) expect(await runWorkspaceMenu("print", actor, scope, handlers)).toMatchObject({ status: "completed", value: { print_callback_returned: true, physical_output: "not observed" } });
  expect(handlers.print).toHaveBeenCalledTimes(2);
  handlers.print = vi.fn(() => false);
  expect(await runWorkspaceMenu("print", agent, scope, handlers)).toMatchObject({ status: "unavailable" });
  handlers.print = () => { throw Error("platform printing refused"); };
  await expect(runWorkspaceMenu("print", agent, scope, handlers)).rejects.toThrow("platform printing refused");
});
</storycode:copy>

### Assemble `apps/web/src/views/workspaceMenuActions.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/workspaceMenuActions.test.ts"><storycode:paste select="#f-c4f5a89c7b3e-human"/><storycode:paste select="#f-c4f5a89c7b3e-setup"/><storycode:paste select="#f-c4f5a89c7b3e-expression_statement"/><storycode:paste select="#f-c4f5a89c7b3e-expression_statement-2"/><storycode:paste select="#f-c4f5a89c7b3e-expression_statement-3"/><storycode:paste select="#f-c4f5a89c7b3e-expression_statement-4"/><storycode:paste select="#f-c4f5a89c7b3e-expression_statement-5"/><storycode:paste select="#f-c4f5a89c7b3e-expression_statement-6"/><storycode:paste select="#f-c4f5a89c7b3e-expression_statement-7"/></storycode:file>


</storycode:doc>
