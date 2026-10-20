import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { AnchorBar } from "./AnchorBar";
import { MonitorDock } from "./MonitorDock";
import type { TerminalSession } from "../api/types";
afterEach(cleanup);
function tools() {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args: Record<string, UiValue> = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id),
  }); }); return result; };
  return { registry, actions, run };
}
it("shares recording-strip status and actions, with owner changes and inactive controls retired", async () => {
  const { registry, actions, run } = tools();
  const resume = vi.fn().mockResolvedValue(true), stop = vi.fn().mockResolvedValue(false);
  const anchor = { doc: "one", container: "py", suspended: "Recording paused", foreign: null, recorded: 2 };
  const panel = (current: typeof anchor | null) => <UiActionContext.Provider value={registry}><AnchorBar anchor={current} onResume={resume} onUnanchor={stop} /></UiActionContext.Provider>;
  const view = render(panel(anchor));
  const reading = await run("Read terminal recording strip bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(new TextDecoder().decode(Uint8Array.from(reading.outcome.value.bytes))).toContain("Recording paused");
  expect(await run("Resume terminal recording")).toMatchObject({ status: "result", outcome: { value: { host_request_completed: true } } });
  expect(resume).toHaveBeenCalledTimes(1);
  const old = actions().find(action => action.label === "Stop terminal writing to document")!.handle;
  view.rerender(panel({ ...anchor, doc: "two" }));
  expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: old, arguments: {}, request_id: "stale" })).not.toMatchObject({ status: "result" });
  await act(async () => { fireEvent.click(screen.getByRole("button", { name: "Stop writing" })); });
  expect(stop).toHaveBeenCalledTimes(1);
  expect(screen.getByRole("alert").textContent).toContain("refused");
  view.rerender(panel(null));
  expect(actions().some(action => action.label === "Resume terminal recording")).toBe(false);
  expect(actions().some(action => action.label === "Stop terminal writing to document")).toBe(false);
});
it("shares monitor navigation and reports actual close refusal without touching a replaced session", async () => {
  const { registry, actions, run } = tools();
  const open = vi.fn(), close = vi.fn().mockResolvedValue(false);
  const session = { id: "first", title: "dev server", cwd: "/fixture", monitor: true, state: "working", since_ms: 0, branch: null, dirty: false, preview: "Listening", prompt: null, exit_code: null } as TerminalSession;
  const panel = (current: TerminalSession[]) => <UiActionContext.Provider value={registry}><MonitorDock sessions={current} onOpen={open} onClose={close} /></UiActionContext.Provider>;
  const view = render(panel([session]));
  await act(async () => { fireEvent.click(screen.getByRole("button", { name: "dev server" })); });
  expect(open).toHaveBeenCalledExactlyOnceWith("first");
  const old = actions().find(action => action.label === "Stop monitor terminal: dev server")!.handle;
  view.rerender(panel([{ ...session, id: "second" }]));
  expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: old, arguments: {}, request_id: "replaced" })).not.toMatchObject({ status: "result" });
  expect(close).not.toHaveBeenCalled();
  expect(await run("Stop monitor terminal: dev server")).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  expect(close).toHaveBeenCalledExactlyOnceWith("second");
  view.rerender(panel([]));
  expect(actions().some(action => action.label === "Stop monitor terminal: dev server")).toBe(false);
});
