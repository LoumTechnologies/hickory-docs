import { expect, it, vi } from "vitest";
import { runWorkspaceMenu, type MenuHandlers, type MenuScope } from "./workspaceMenuActions";
const human = { id: "local-user", kind: "human" as const }, agent = { id: "python:turn", kind: "agent" as const };
function setup() {
  const scope: MenuScope = { target: { id: "first", kind: "untitled" }, untitled: ["first"], sessions: [] };
  const handlers: MenuHandlers = { formatFirst: vi.fn(async () => {}), saveUntitled: vi.fn(async () => true), flush: vi.fn(), print: vi.fn(), insert: vi.fn(), editElement: vi.fn() };
  return { scope, handlers };
}
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
  expect(handlers.saveUntitled).toHaveBeenCalledExactlyOnceWith(false, "first");
});
it("does not open OS print/save dialogs or perform partial writes when an agent needs human input", async () => {
  const { scope, handlers } = setup();
  expect(await runWorkspaceMenu("save", agent, scope, handlers)).toMatchObject({ status: "requires_input", request: "workspace-save:first" });
  expect(await runWorkspaceMenu("save-all", agent, scope, handlers)).toMatchObject({ status: "requires_input" });
  expect(await runWorkspaceMenu("print", agent, scope, handlers)).toMatchObject({ status: "requires_input" });
  expect(handlers.formatFirst).not.toHaveBeenCalled(); expect(handlers.saveUntitled).not.toHaveBeenCalled(); expect(handlers.print).not.toHaveBeenCalled();
});
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
it("shares existing Insert/edit handlers and writes no source just by opening them", async () => {
  const { scope, handlers } = setup();
  expect(await runWorkspaceMenu("insert:table", agent, scope, handlers)).toMatchObject({ value: { insert_panel_requested: true, source_written: false } });
  expect(handlers.insert).toHaveBeenCalledWith("table");
  await runWorkspaceMenu("edit-element", human, scope, handlers); expect(handlers.editElement).toHaveBeenCalledOnce();
});

it("does not request a destination or write for a known empty unnamed buffer with formatting off", async () => {
  const { scope, handlers } = setup(); scope.unnamedEmpty = true;
  expect(await runWorkspaceMenu("save", agent, scope, handlers)).toMatchObject({ value: { empty_buffer_no_write: true } });
  expect(handlers.saveUntitled).not.toHaveBeenCalled(); expect(handlers.formatFirst).not.toHaveBeenCalled();
});
