// Guarantee: docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { request } from "../api/client";
import type { RuntimeRun } from "../debug/runtimeObservations";
import type { ObservationReading } from "../debug/observations";
vi.mock("../api/client", () => ({ request: vi.fn() }));
vi.mock("../debug/RuntimeRuns", () => ({ NativeRuntimeHistory: () => <p>Normal runtime controls</p> }));
vi.mock("../embed/DebuggableDocument", () => ({ DebuggableDocument: ({ source, onRuntime, onObservation }: { source: string; onRuntime: (r: RuntimeRun) => void; onObservation: (r: ObservationReading) => void }) => {
  function failure(message: string) { onRuntime({ schema_version: 1, id: "actual-run", status: "failed", sources: [{ id: "document", uri: "deadline-spec.md", revision: "a".repeat(64), bytes: [...new TextEncoder().encode(source)] }], events: [{ id: "event", observation: { kind: "exception", message, type_name: "Error", handling: "unhandled", throwing: null, catching: null } }] } as RuntimeRun); }
  return <><button onClick={() => failure("Missing runtime")}>Observe unrelated failure</button><button onClick={() => failure("R1 failed: expected 100, got 160")}>Observe exact counterexample</button><button onClick={() => onObservation({ source: { id: "document", uri: "deadline-spec.md", revision: "a".repeat(64), bytes: [...new TextEncoder().encode(source)] }, scope: { run: "actual-run", stop: "actual-stop" }, annotations: [{ payload: { kind: "paused" } }, { payload: { kind: "breakpoint_bound" } }] } as ObservationReading)}>Observe bound paused marker</button></>;
} }));
import { NativeWorkflowTour } from "./NativeWorkflowTour";
afterEach(() => { cleanup(); vi.clearAllMocks(); localStorage.clear(); });
it("creates an isolated repository only on the explicit action and never sends a model request", async () => {
  vi.mocked(request).mockImplementation(async (method, path) => path === "/api/tour/practice" && method === "GET" ? { active: false } : { path: "/data/practice", opened: false });
  render(<NativeWorkflowTour />);
  fireEvent.click(screen.getByText("AI engineer tour"));
  await waitFor(() => expect(request).toHaveBeenCalledWith("GET", "/api/tour/practice"));
  expect(vi.mocked(request).mock.calls.filter(c => c[0] === "POST")).toHaveLength(0);
  fireEvent.click(screen.getByText("Create and open an isolated practice repository"));
  await screen.findByText("/data/practice");
  expect(vi.mocked(request).mock.calls.filter(c => c[0] === "POST").map(c => c[1])).toEqual(["/api/tour/practice"]);
});
it("requires the precise counterexample and an actual bound stop instead of any failed action", async () => {
  localStorage.setItem("hickory.aiEngineerTour.native:practice", "2");
  vi.mocked(request).mockImplementation(async (_method, path) => {
    if (path === "/api/tour/practice") return { active: true, id: "practice", path: "/data/practice", source: "captured source", source_hash: "a".repeat(64), doc_id: "doc", meeting: "WEBVTT", seed: "Recorded fixture" };
    if (path === "/api/reviews") return { events: [] };
    if (path === "/api/runtime/runs") return { runs: [] };
    if (path.endsWith("/turns")) return { turns: [] };
    return {};
  });
  render(<NativeWorkflowTour />); fireEvent.click(screen.getByText("AI engineer tour"));
  await screen.findByText("Observe the real counterexample");
  const next = screen.getByRole("button", { name: "Continue" }) as HTMLButtonElement;
  expect(next.disabled).toBe(true);
  fireEvent.click(screen.getByText("Observe unrelated failure")); expect(next.disabled).toBe(true);
  fireEvent.click(screen.getByText("Observe exact counterexample")); expect(next.disabled).toBe(true);
  fireEvent.click(screen.getByText("Observe bound paused marker")); expect(next.disabled).toBe(false);
});
