// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { installMockHandler } from "../api/client";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { GithubReviewThreads } from "./GithubReviewThreads";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
const capture = { schema_version: 1, source: { id: "actual-source", uri: "spec.md", revision: "actual-revision", bytes: [...new TextEncoder().encode("original\n")] }, annotations: [], unsupported: [], incomplete: true, unavailable: [{ thread: "thread", reason: "reviewed revision unavailable" }] };
function setup() {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  return { registry, actions, run };
}
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares typed read-only GitHub selection and actual partial/unavailable capture results", async () => {
  const calls = vi.fn();
  installMockHandler(async (method, path, body) => { calls(method, path, body); return capture; });
  const { registry, run } = setup();
  render(<UiActionContext.Provider value={registry}><GithubReviewThreads /></UiActionContext.Provider>);
  await run("Set GitHub review capture selection", { field: "repository", value: "owner/repo" });
  expect(await run("Set GitHub review capture selection", { field: "number", value: "1e4" })).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  await run("Set GitHub review capture selection", { field: "number", value: "7" });
  await run("Set GitHub review capture selection", { field: "document", value: "spec.md" });
  expect((screen.getByLabelText("Review repository") as HTMLInputElement).value).toBe("owner/repo");
  expect(await run("Read selected GitHub PR review evidence")).toMatchObject({ status: "result", outcome: { status: "completed", value: { source: { id: "actual-source", revision: "actual-revision" }, incomplete: true, unavailable: capture.unavailable } } });
  expect(calls).toHaveBeenCalledTimes(1); expect(calls.mock.calls[0][0]).toBe("GET");
  expect(calls.mock.calls[0][1]).toBe("/api/workspace/github/review-threads?repository=owner%2Frepo&number=7&document=spec.md");
  expect(screen.getByText(/Incomplete paginated capture/)).toBeTruthy();
  const reading = await run("Read GitHub review capture status bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  const status = JSON.parse(new TextDecoder().decode(new Uint8Array(reading.outcome.value.bytes)));
  expect(status.changed).toBe(false); // No focused editor is not a changed source.
});

it("refuses a late capture after its selection changes", async () => {
  let complete!: (value: typeof capture) => void;
  const requested = vi.fn();
  installMockHandler(async () => { requested(); return new Promise(done => { complete = done; }); });
  const { registry, actions, run } = setup();
  render(<UiActionContext.Provider value={registry}><GithubReviewThreads document="spec.md" source="original\n" /></UiActionContext.Provider>);
  await run("Set GitHub review capture selection", { field: "repository", value: "owner/repo" });
  await run("Set GitHub review capture selection", { field: "number", value: "7" });
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === "Read selected GitHub PR review evidence")!.handle, arguments: {}, request_id: "capture" }); });
  await waitFor(() => expect(requested).toHaveBeenCalledTimes(1));
  fireEvent.change(screen.getByLabelText("Review repository"), { target: { value: "other/repo" } });
  await act(async () => complete(capture));
  expect(await pending).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: expect.stringContaining("selection or source changed") } });
  expect(screen.queryByText(/No review comments returned/)).toBeNull();
  expect(screen.getByRole("alert").textContent).toContain("late capture was not attached");
});
