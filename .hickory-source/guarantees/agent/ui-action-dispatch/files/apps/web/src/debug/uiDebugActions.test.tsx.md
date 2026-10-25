<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/uiDebugActions.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `2cc59db61f40722666487d28ec64e0bac593e8af59f5e7372014b3f52f119da0`.

<book:entry path="apps/web/src/debug/uiDebugActions.test.tsx">
import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useDebugActions } from "./uiDebugActions";
import type { DebugSession } from "./useDebugger";

afterEach(cleanup);
function fixture() {
  const registry = new UiActions({ workspace: "practice", window: "window" }, () => true, async () => {});
  const debug = { status: "paused", message: null, program: "deadline.js", frames: [{ id: 7, name: "elapsed", line: 12 }], variables: [{ name: "total", value: "80" }], selectedFrame: 7, breakpoints: [{ line: 12, state: "bound" }], sourceBreakpoints: {}, watches: [], exceptionFilters: [], buildOutput: [], exitCode: null, lastValue: null,
    capabilities: { step_back: false, restart_frame: false, goto_targets: false, conditional_breakpoints: true, hit_conditional_breakpoints: false, log_points: false, exception_filters: [] },
    start: vi.fn(), stop: vi.fn(), step: vi.fn(), selectFrame: vi.fn(), toggleBreakpoint: vi.fn(), setBreakpointCondition: vi.fn(), addWatch: vi.fn(), removeWatch: vi.fn(), toggleExceptionFilter: vi.fn(), jumpTo: vi.fn(), runTo: vi.fn(), query: vi.fn().mockResolvedValue("80"),
    reading: { source: { id: "source", uri: "deadline.md", revision: "source-hash" }, scope: { run: "run-1", stop: "stop-1", frame: 7 }, locations: [], annotations: [{ evidence: "paused-1", payload: { kind: "paused" } }], unsupported: [] },
  } as unknown as DebugSession;
  let epoch = 1, sourceIdentity = "original source";
  const wrapper = ({ children }: { children: ReactNode }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider>;
  const hook = renderHook(() => useDebugActions("deadline.md", debug, () => epoch, true, sourceIdentity), { wrapper });
  const descriptor = (label: string) => registry.observe(registry.groups().groups[0]).actions.find(item => item.label === label)!;
  let requests = 0;
  const invoke = async (label: string, args = {}) => registry.act(registry.scope, { id: "python:debug", kind: "agent" }, { handle: descriptor(label).handle, arguments: args, request_id: String(++requests) });
  return { registry, debug, hook, descriptor, invoke, advance: () => { epoch++; }, changeSource: () => { sourceIdentity = "replacement source"; hook.rerender(); } };
}

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("uses the real debugger controller for stepping, frame selection and evaluation", async () => {
  const f = fixture();
  expect(f.descriptor("Read debugger state").value).toMatchObject({ status: "paused", scope: { stop: "stop-1" } });
  expect(f.descriptor("Read debugger state").value).not.toHaveProperty("variables");
  expect(await f.invoke("Read debugger state")).toMatchObject({ outcome: { value: { collections: { variables: { items: 1 } }, evidence: { scope: { stop: "stop-1" } } } } });
  expect(await f.invoke("Read debugger detail bytes", { collection: "variables", offset: 0, limit: 4 })).toMatchObject({ outcome: { value: { span: [0, 4], bytes: [91, 123, 34, 110], next: 4 } } });
  await act(async () => { expect(await f.invoke("Step over")).toMatchObject({ status: "result", outcome: { value: { dispatch: "requested" } } }); });
  expect(f.debug.step).toHaveBeenCalledWith("over");
  await f.invoke("Select stack frame", { frame: 7 }); expect(f.debug.selectFrame).toHaveBeenCalledWith(7);
  expect(await f.invoke("Select stack frame", { frame: 999 })).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  expect(await f.invoke("Evaluate expression in selected frame", { expression: "total" })).toMatchObject({ status: "result", outcome: { value: { value: "80", requested_scope: { stop: "stop-1" } } } });
});

it("rejects changed stops and unsupported reverse capabilities without dispatching", async () => {
  const f = fixture(); const previous = f.descriptor("Continue"); f.advance();
  expect(await f.registry.act(f.registry.scope, { id: "python:debug", kind: "agent" }, { handle: previous.handle, arguments: {}, request_id: "stale-stop" })).toMatchObject({ status: "stale" });
  f.hook.rerender();
  expect(await f.invoke("Reverse step")).toMatchObject({ status: "disabled" });
  expect(await f.invoke("Restart frame")).toMatchObject({ status: "disabled" });
  expect(await f.invoke("Toggle document breakpoint", { line: 1.5 })).toMatchObject({ status: "invalid" });
  expect(f.debug.step).not.toHaveBeenCalled(); expect(f.debug.toggleBreakpoint).not.toHaveBeenCalled();
});

it("validates breakpoint options against the actual adapter and bound target", async () => {
  const f = fixture();
  await f.invoke("Toggle document breakpoint", { line: 12 }); expect(f.debug.toggleBreakpoint).toHaveBeenCalledWith(12);
  const args = { line: 12, condition: "total > 0", hit_condition: "", log_message: "" };
  await f.invoke("Set breakpoint condition, hit count and log message", args);
  expect(f.debug.setBreakpointCondition).toHaveBeenCalledWith(12, { condition: "total > 0", hit_condition: "", log_message: "" });
  expect(await f.invoke("Set breakpoint condition, hit count and log message", { ...args, log_message: "secret" })).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  expect(f.debug.setBreakpointCondition).toHaveBeenCalledOnce();
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("retires source-bound handles even when debugger stop and controller epoch have not changed", async () => {
  const f = fixture(), original = f.descriptor("Continue");
  f.changeSource();
  expect(await f.registry.act(f.registry.scope, { id: "python:debug", kind: "agent" }, { handle: original.handle, arguments: {}, request_id: "changed-browser-source" })).toMatchObject({ status: "stale" });
  expect(f.debug.step).not.toHaveBeenCalled();
});
</book:entry>

</book:doc>
