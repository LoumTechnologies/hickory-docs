import { api } from "../api/client";
import { useDired } from "./useDired";
import { FileActionPrompts } from "./fileMenuActions";
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { TreePrompt } from "./TreePrompt";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("reads/changes the visible draft, shares real submit and retires a replacement prompt owner", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const submit = vi.fn().mockResolvedValue(false), cancel = vi.fn();
  const ui = (owner: string) => <UiActionContext.Provider value={registry}><TreePrompt owner={owner} subject="file-operations" label="Copy source to folder" verb="Copy" initial="first" onSubmit={submit} onCancel={cancel} /></UiActionContext.Provider>;
  const rendered = render(ui("one"));
  const action = (label: string) => registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === label)!;
  let id = 0;
  const run = async (label: string, args = {}) => { let result!: Awaited<ReturnType<typeof registry.act>>; await act(async () => { result = await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: action(label).handle, arguments: args, request_id: String(++id) }); }); return result; };
  fireEvent.change(screen.getByRole("textbox"), { target: { value: "human draft" } });
  const read = await run("Read visible file prompt bytes", { offset: 0, limit: 8192 }) as unknown as { outcome: { value: { bytes: number[] } } };
  expect(JSON.parse(new TextDecoder().decode(new Uint8Array(read.outcome.value.bytes))).draft).toBe("human draft");
  await run("Edit visible file prompt: Copy", { value: "agent draft" });
  expect(screen.getByRole("textbox")).toHaveProperty("value", "agent draft");
  expect(await run("Submit visible file prompt: Copy")).toMatchObject({ outcome: { status: "unavailable" } });
  expect(submit).toHaveBeenLastCalledWith("agent draft");
  const old = action("Submit visible file prompt: Copy"); rendered.rerender(ui("two"));
  expect(screen.getByRole("textbox")).toHaveProperty("value", "first");
  expect(await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "old" })).toMatchObject({ status: "unavailable" });
  submit.mockResolvedValue(true);
  await act(async () => fireEvent.submit(screen.getByRole("dialog")));
  expect(submit).toHaveBeenLastCalledWith("first");
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("requires explicit confirmation and refuses Escape/duplicate submit during the actual callback", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let release!: (result: boolean) => void;
  const submit = vi.fn(() => new Promise<boolean>(resolve => { release = resolve; })), cancel = vi.fn();
  render(<UiActionContext.Provider value={registry}><TreePrompt owner="delete-one" label="Delete source?" verb="Delete" onSubmit={submit} onCancel={cancel} /></UiActionContext.Provider>);
  const action = () => registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === "Submit visible file prompt: Delete")!;
  const run = (request_id: string, confirm: boolean) => registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: action().handle, arguments: { confirm }, request_id });
  expect(await run("unconfirmed", false)).toMatchObject({ outcome: { status: "unavailable" } });
  expect(submit).not.toHaveBeenCalled();
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = run("confirmed", true); });
  fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
  expect(cancel).not.toHaveBeenCalled(); expect(submit).toHaveBeenCalledTimes(1);
  expect(await run("duplicate", true)).toMatchObject({ status: "disabled" });
  await act(async () => { release(false); expect(await pending).toMatchObject({ outcome: { status: "unavailable" } }); });
  expect(submit).toHaveBeenCalledWith(null);
  expect(screen.getByRole("button", { name: "Delete" })).toHaveProperty("disabled", false);
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares the human draft with parent file tools and paints an explicit agent destination during dispatch", async () => {
  let release!: () => void;
  const fileOp = vi.spyOn(api, "fileOp").mockImplementation(() => new Promise(resolve => { release = () => resolve({} as never); }));
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const notice = vi.fn(); let latest!: ReturnType<typeof useDired>;
  function Harness() { latest = useDired(notice); return <FileActionPrompts dired={latest} />; }
  render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  await act(async () => latest.run({ kind: "copy", paths: ["source.md"] }));
  const action = (label: string) => registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === label)!;
  const old = action("Submit file prompt: Copy");
  fireEvent.change(screen.getByRole("textbox"), { target: { value: "human destination" } });
  expect(latest.promptDraft).toBe("human destination");
  expect(await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: old.handle, arguments: { value: "old-destination" }, request_id: "stale-draft" })).toMatchObject({ status: "stale" });
  expect(fileOp).not.toHaveBeenCalled();
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: action("Submit file prompt: Copy").handle, arguments: { value: "agent destination" }, request_id: "submit" }); });
  expect(screen.getByRole("textbox")).toHaveProperty("value", "agent destination");
  expect(screen.getByRole("textbox")).toHaveProperty("disabled", true);
  expect(fileOp).toHaveBeenCalledWith({ op: "copy", path: "source.md", to: "agent destination" });
  await act(async () => { release(); expect(await pending).toMatchObject({ outcome: { status: "completed" } }); });
  expect(screen.queryByRole("dialog")).toBeNull();
});
