<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/RuntimeLogView.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `53aa7e06245178d8bac9d0d0a80f343f3776c7d4a197dbda25677d2e32b4b195`.

<book:entry path="apps/web/src/debug/RuntimeLogView.test.tsx">
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { RuntimeLog } from "./RuntimeLogView";
import type { RuntimeRun } from "./runtimeObservations";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import * as reviews from "../lib/reviews";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
function fixture(): RuntimeRun {
  return { schema_version: 1, id: "one", started_unix_ms: 1, status: "completed", pinned: false, truncated: false, missing_artifacts: [], detail: null,
    sources: [{ id: "first", uri: "first.md", revision: "a".repeat(64), bytes: [...new TextEncoder().encode("PRIVATE first\nsecond\n")] },
      { id: "second", uri: "second.md", revision: "b".repeat(64), bytes: [...new TextEncoder().encode("other source\n")] }],
    events: ["stdout", "stderr"].map((stream, index) => ({ id: `event-${index}`, provider: "fixture", block: "main", invocation: "call", parent_invocation: null,
      thread: null, task: null, time: null, emission: { source: "first", block: "main", bytes: { start: 0, end: 7 }, precision: "exact" }, callers: [],
      observation: { kind: "output", stream, category: "write", bytes: [...new TextEncoder().encode(`${stream} output\n`)] } })) };
}
function tools() {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id),
  }); }); return result; };
  return { registry, actions, run };
}
it("shares only expanded runtime relationships, selectors and exact captured review requests", async () => {
  const { registry, actions, run } = tools();
  const review = vi.spyOn(reviews, "openReview").mockImplementation(() => {});
  const captured = fixture();
  render(<UiActionContext.Provider value={registry}><RuntimeLog run={captured} /></UiActionContext.Provider>);
  expect(actions().some(action => action.label === "Read runtime log relationship metadata bytes")).toBe(false);
  expect(JSON.stringify(actions())).not.toContain("PRIVATE");
  await run("Set runtime log visibility", { open: true });
  const reading = await run("Read runtime log relationship metadata bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  const text = new TextDecoder().decode(Uint8Array.from(reading.outcome.value.bytes));
  expect(text).toContain("event-0"); expect(text).not.toContain("PRIVATE");
  const old = actions().find(action => action.label === "Go to runtime emitting/throwing location: event-0")!.handle;
  fireEvent.change(screen.getByLabelText("Runtime log stream"), { target: { value: JSON.stringify(["stderr", "write"]) } });
  expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: old, arguments: {}, request_id: "other-stream" })).not.toMatchObject({ status: "result" });
  await run("Select runtime captured source", { index: 1 });
  expect((screen.getByLabelText("Captured source") as HTMLSelectElement).value).toBe("second");
  await run("Challenge runtime relationship: event-1");
  expect(review.mock.calls[0][0]).toMatchObject({ kind: "runtime-record", relationship: { source_revision: captured.sources[0].revision } });
  expect(review.mock.calls[0][0].source_hash).toBe(await reviews.sha256(JSON.stringify(captured)));
  await run("Set runtime relationship connector style", { style: "bands" });
  expect((screen.getByLabelText("Execution connector style") as HTMLSelectElement).value).toBe("bands");
  await run("Go to runtime emitting/throwing location: event-1");
  expect((screen.getByLabelText("Captured source") as HTMLSelectElement).value).toBe("first");
  expect(await run("Challenge runtime relationship: event-1")).toMatchObject({ status: "result", outcome: { value: { review_requested: true, review_completed: false } } });
  expect(review).toHaveBeenCalledWith(expect.objectContaining({ source_hash: captured.sources[0].revision, relationship: expect.objectContaining({ run: "one", event: expect.objectContaining({ id: "event-1" }) }) }));
  await run("Set runtime log visibility", { open: false });
  expect(actions().some(action => action.label === "Select runtime captured source")).toBe(false);
  expect(actions().some(action => action.label === "Read runtime log relationship metadata bytes")).toBe(false);
});
it("handles missing captured source and refuses a late review hash after capture replacement", async () => {
  const { registry, actions, run } = tools();
  const review = vi.spyOn(reviews, "openReview").mockImplementation(() => {});
  let finish!: (hash: string) => void;
  vi.spyOn(reviews, "sha256").mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  const captured = { ...fixture(), sources: [] };
  const panel = (value: RuntimeRun) => <UiActionContext.Provider value={registry}><RuntimeLog run={value} /></UiActionContext.Provider>;
  const view = render(panel(captured));
  await run("Set runtime log visibility", { open: true });
  expect(screen.getByText("No captured source retained for this run.")).toBeTruthy();
  expect(actions().some(action => action.label === "Select runtime captured source")).toBe(false);
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === "Challenge runtime relationship: event-0")!.handle, arguments: {}, request_id: "late",
  }); });
  view.rerender(panel({ ...captured, id: "replacement" }));
  await act(async () => { finish("c".repeat(64)); expect(await pending).toMatchObject({ status: "result", outcome: { status: "unavailable" } }); });
  expect(review).not.toHaveBeenCalled();
  await act(async () => { pending = registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === "Challenge runtime relationship: event-0")!.handle, arguments: {}, request_id: "cancelled-hash",
  }); });
  expect(registry.cancel(registry.scope, { id: "python:fixture", kind: "agent" }, "cancelled-hash")).toBe(true);
  await act(async () => { finish("e".repeat(64)); expect(await pending).toMatchObject({ status: "result", outcome: { status: "unavailable" } }); });
  expect(review).not.toHaveBeenCalled();
  await act(async () => { pending = registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === "Challenge runtime relationship: event-0")!.handle, arguments: {}, request_id: "closed-disclosure",
  }); });
  expect(await run("Set runtime log visibility", { open: false })).toMatchObject({ status: "result", outcome: { value: { open: false } } });
  expect(await run("Set runtime log visibility", { open: true })).toMatchObject({ status: "result", outcome: { value: { open: true } } });
  await act(async () => { finish("d".repeat(64)); expect(await pending).toMatchObject({ status: "result", outcome: { status: "unavailable" } }); });
  expect(review).not.toHaveBeenCalled();
});
</book:entry>

</book:doc>
