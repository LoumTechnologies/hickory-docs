// Guarantee: docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { request } from "../api/client";
import { retainDebugReading } from "../debug/readingCapture";
import type { ObservationReading } from "../debug/observations";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { NativeWorkflowTour } from "./NativeWorkflowTour";
vi.mock("../api/client", () => ({ request: vi.fn() }));
const owner = Symbol("test-captured-debugger");
afterEach(() => { cleanup(); vi.clearAllMocks(); localStorage.clear(); retainDebugReading(owner, null); });
it("creates a separate repository only on the explicit action and never sends a model request", async () => {
  vi.mocked(request).mockImplementation(async (method, path) => path === "/api/tour/practice" && method === "GET" ? { active: false } : { path: "/data/practice", opened: false });
  render(<NativeWorkflowTour />); fireEvent.click(screen.getByText("AI engineer tour"));
  await waitFor(() => expect(request).toHaveBeenCalledWith("GET", "/api/tour/practice"));
  expect(vi.mocked(request).mock.calls.filter(call => call[0] === "POST")).toHaveLength(0);
  fireEvent.click(screen.getByText("Create and open an isolated practice repository"));
  await screen.findByText("/data/practice");
  expect(vi.mocked(request).mock.calls.filter(call => call[0] === "POST").map(call => call[1])).toEqual(["/api/tour/practice"]);
});
function responses(confirmed = true, paused = false) {
  vi.mocked(request).mockImplementation(async (_method, path) => {
    if (path === "/api/tour/practice") return { active: true, id: "practice", path: "/data/practice", source: "saved source", source_hash: "a".repeat(64), seed_source_hash: "a".repeat(64), seed: "Transparent seed", agent_requests: confirmed ? [{ request: "request-actual", branch: "turn-actual", state: "provider_confirmed" }] : [], executions: [{ id: "failed-actual", status: "failed", source_hash: "a".repeat(64), error: "R1 failed: expected 100, got 160", blocks: [] }], debug_evidence: paused ? [{ run: "retained-run", stop: "retained-stop", evidence: ["bound", "paused"] }] : [] };
    if (path === "/api/reviews") return { events: [] };
    if (path.endsWith("/turns")) return { turns: [{ id: "turn-actual", status: "ok" }] };
    throw Error(`Unexpected request ${path}`);
  });
}
it("requires an actual delivered request rather than a model completion or saved step number", async () => {
  responses(false); localStorage.setItem("hickory.aiEngineerCoach.v2:practice", "5");
  render(<NativeWorkflowTour />); fireEvent.click(screen.getByText("AI engineer tour"));
  await screen.findByText("Delegate investigation and setup");
  expect((screen.getByRole("button", { name: "Continue" }) as HTMLButtonElement).disabled).toBe(true);
  expect(screen.getByText(/No model request is sent by the coach/)).toBeTruthy();
});
it("requires source-matching bound paused evidence and never accepts a marker in prose", async () => {
  responses(); localStorage.setItem("hickory.aiEngineerCoach.v2:practice", "1");
  render(<NativeWorkflowTour />); fireEvent.click(screen.getByText("AI engineer tour"));
  await screen.findByText("Interpret the counterexample and paused source");
  const next = screen.getByRole("button", { name: "Continue" }) as HTMLButtonElement;
  expect(next.disabled).toBe(true);
  const reading = { source: { id: "source", uri: "hick:///deadline-spec.md", revision: "b".repeat(64), bytes: [] }, scope: { run: "actual-run", stop: "actual-stop" }, annotations: [{ evidence: "paused", payload: { kind: "paused" } }, { evidence: "bound", payload: { kind: "breakpoint_bound" } }] } as unknown as ObservationReading;
  act(() => retainDebugReading(owner, reading)); expect(next.disabled).toBe(true);
  act(() => retainDebugReading(owner, { ...reading, source: { ...reading.source, revision: "a".repeat(64) } }));
  expect(next.disabled).toBe(false);
});
it("resumes from retained host observations while marking skipped steps as unestablished", async () => {
  responses(true, true); localStorage.setItem("hickory.aiEngineerCoach.v2:practice", "4");
  render(<NativeWorkflowTour />); fireEvent.click(screen.getByText("AI engineer tour"));
  await screen.findByText("Decide what the customer meant");
  fireEvent.click(screen.getByText("Skip this step"));
  await screen.findByText("Delegate correction and focused verification");
  expect(screen.getByText(/Skipped steps: 3; these are not passed/)).toBeTruthy();
  expect((screen.getByRole("button", { name: "Continue" }) as HTMLButtonElement).disabled).toBe(true);
});
it("does not count an agent comment as the human acceptance criterion", async () => {
  responses(true, true);
  const existing = vi.mocked(request).getMockImplementation()!;
  vi.mocked(request).mockImplementation(async (method, path, body) => path === "/api/reviews" ? { events: [{ id: "agent", thread: "agent-thread", action: "comment", actor: "agent-declared", target: { path: "deadline-spec.md", quote: "The deadline bounds the entire operation" } }] } : existing(method, path, body));
  localStorage.setItem("hickory.aiEngineerCoach.v2:practice", "4");
  render(<NativeWorkflowTour />); fireEvent.click(screen.getByText("AI engineer tour"));
  await screen.findByText("Decide what the customer meant");
  expect((screen.getByRole("button", { name: "Continue" }) as HTMLButtonElement).disabled).toBe(true);
});

it("requires coach disclosure, reads guidance explicitly and retires actions on dismiss", async () => {
  responses();
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><NativeWorkflowTour /></UiActionContext.Provider>);
  const parent = registry.groups().groups.find(group => group.startsWith("coach:"))!;
  expect(registry.observe(parent).actions.map(a => a.label)).toEqual(["AI engineer tour"]);
  const toggle = registry.observe(parent).actions[0];
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: toggle.handle, arguments: {}, request_id: "open" }); });
  const read = registry.observe(parent).actions.find(a => a.label === "Read current practice guidance bytes")!;
  expect(JSON.stringify(registry.observe(parent))).not.toContain("/data/practice");
  const reading = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: read.handle, arguments: { offset: 0, limit: 8192 }, request_id: "read" });
  if (reading.status !== "result" || reading.outcome.status !== "completed") throw Error("Expected reading");
  expect(new TextDecoder().decode(new Uint8Array((reading.outcome.value as { bytes: number[] }).bytes))).toContain("/data/practice");
  expect(registry.observe(parent).actions.some(a => a.label === "Compare reviewed published JSON")).toBe(false);
  const dismiss = registry.observe(parent).actions.find(a => a.label === "Dismiss practice guidance")!;
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: dismiss.handle, arguments: {}, request_id: "close" }); });
  expect(registry.observe(parent).actions.map(a => a.label)).toEqual(["AI engineer tour"]);
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: read.handle, arguments: { offset: 0, limit: 8192 }, request_id: "old" })).toMatchObject({ status: "unavailable" });
  expect(registry.groups().groups.some(g => g.startsWith("coach-popup:"))).toBe(false);
});

it("keeps private practice creation human-owned and reports the host execution setup request", async () => {
  vi.mocked(request).mockImplementation(async (method, path) => method === "GET" ? { active: false } : { created: false, requires_input: { request: "private-practice-environment", reason: "Choose the supported host executor explicitly" } });
  const registry = new UiActions({ workspace: "ordinary", window: "creation" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><NativeWorkflowTour /></UiActionContext.Provider>);
  fireEvent.click(screen.getByText("AI engineer tour"));
  await screen.findByRole("button", { name: "Create a private prediction practice" });
  const parent = registry.groups().groups.find(group => group.startsWith("coach:"))!;
  const command = registry.observe(parent).actions.find(action => action.label === "Create a private prediction practice")!;
  let result: unknown;
  await act(async () => { result = await registry.act(registry.scope, { kind: "agent", id: "predictor" }, { handle: command.handle, arguments: {}, request_id: "reset-history" }); });
  expect(result).toMatchObject({ outcome: { status: "requires_input", request: "create-private-practice" } });
  expect(vi.mocked(request).mock.calls.some(call => call[0] === "POST")).toBe(false);
  fireEvent.click(screen.getByRole("button", { name: "Create a private prediction practice" }));
  await screen.findByText("Choose the supported host executor explicitly");
  expect(vi.mocked(request).mock.calls.filter(call => call[0] === "POST").map(call => call[1])).toEqual(["/api/tour/private-practice"]);
});
