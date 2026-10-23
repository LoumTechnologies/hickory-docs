// @vitest-environment jsdom
// Guarantee: docs/guarantees/execution/runtime-history-keeps-runs-and-occurrences-separate.md
import { afterEach, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { createMemoryStorage } from "../embed/storage";
import { runRepository, runtimeReading } from "./runtimeHistory";
import { RuntimeHistory, NativeRuntimeHistory } from "./RuntimeRuns";
import { observationContext } from "./observations";
import type { RuntimeRun } from "./runtimeObservations";
import { request } from "../api/client";
vi.mock("../api/client", () => ({ request: vi.fn() }));
afterEach(() => { cleanup(); vi.mocked(request).mockReset(); });
async function fixture(id = "run-one"): Promise<RuntimeRun> {
  const bytes = [...new TextEncoder().encode("# 🦀\nconsole.log('hello');\n")];
  const revision = [...new Uint8Array(await crypto.subtle.digest("SHA-256", new Uint8Array(bytes)))].map(byte => byte.toString(16).padStart(2, "0")).join("");
  return { schema_version: 1, id, started_unix_ms: Date.now(), status: "completed", pinned: false,
    sources: [{ id: "document", uri: "example.md", revision, bytes }], truncated: false, missing_artifacts: [], detail: null,
    events: [{ id: `${id}-E1`, provider: "browser-interpreter", block: "main.js", invocation: "call-1", parent_invocation: null,
      thread: null, task: null, time: null, emission: { source: "document", block: "main.js", bytes: { start: 7, end: 8 }, precision: "exact" }, callers: [],
      observation: { kind: "output", stream: "stdout", category: "log", bytes: [...new TextEncoder().encode("hello\n")] } }] };
}
it("retains immutable evidence, explicit pins and digest validation", async () => {
  const storage = createMemoryStorage(); const repository = runRepository(storage); const run = await fixture();
  await repository.save(run); await repository.save({ ...run, pinned: true }, true);
  expect((await repository.list())[0].pinned).toBe(true);
  await expect(repository.save({ ...run, events: [] })).rejects.toThrow("immutable");
  const corrupted = structuredClone(run); corrupted.sources[0].bytes[0] = 65;
  await expect(repository.save(corrupted)).rejects.toThrow("digest");
  await repository.remove(run.id); expect(await repository.list()).toEqual([]);
  await storage.write("runs/tampered.json", new TextEncoder().encode(JSON.stringify(corrupted)), null);
  await expect(repository.list()).rejects.toThrow("digest");
});
it("selects latest failure separately, preserves source positions, and supplies exact selected event IDs", async () => {
  const first = await fixture(); const failed = await fixture("run-two"); failed.started_unix_ms++; failed.status = "failed"; failed.truncated = true;
  const source = new TextDecoder().decode(new Uint8Array(first.sources[0].bytes));
  const host = document.createElement("div"); document.body.append(host);
  const view = new EditorView({ state: EditorState.create({ doc: source }), parent: host });
  const pin = vi.fn(async () => undefined);
  const rendered = render(<RuntimeHistory runs={[first, failed]} source={source} view={view} onPin={pin} onDelete={vi.fn()} />);
  expect(screen.getByText(/^Run run-two · failed/)).toBeTruthy();
  expect(view.state.doc.toString()).toBe(source);
  expect(host.querySelector(".runtime-inline")).toBeTruthy();
  fireEvent.change(screen.getByLabelText("Runtime run"), { target: { value: first.id } });
  expect(screen.getByText(/run-one · completed · historical/)).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Pin run" })); await waitFor(() => expect(pin).toHaveBeenCalledWith(first));
  const details = screen.getByText(/Source observations/).closest("details")!;
  details.open = true; fireEvent(details, new Event("toggle"));
  await waitFor(() => expect(observationContext("example.md", source)[0]?.evidence).toEqual([first.events[0].id]));
  fireEvent.change(screen.getByLabelText("Compare retained run"), { target: { value: failed.id } });
  expect(screen.getByLabelText("Separate run comparison")).toBeTruthy();
  rendered.rerender(<RuntimeHistory runs={[first, failed]} source={source + "edited"} view={view} onPin={pin} onDelete={vi.fn()} />);
  expect(screen.getByText(/source changed \(stale\)/)).toBeTruthy();
  expect(host.querySelector(".runtime-inline")).toBeNull();
  expect(runtimeReading(first).annotations[0].evidence).toBe(first.events[0].id);
  view.destroy(); host.remove();
});

// Guarantee: docs/guarantees/execution/python-observations-stay-confined.md
it("explicit native launch executes the saved path and refreshes retained evidence", async () => {
  const run = await fixture(); run.sources[0].uri = "example.md";
  let captured = false;
  vi.mocked(request).mockImplementation(async (method) => {
    if (method === "POST") { captured = true; return { run }; }
    return { runs: captured ? [{ state: "present", run }] : [] };
  });
  render(<NativeRuntimeHistory path="example.md" source="unsaved content" />);
  const details = screen.getByText("Runtime observations").closest("details")!;
  details.open = true; fireEvent(details, new Event("toggle"));
  await screen.findByRole("button", { name: "Run saved Python confined" });
  fireEvent.change(screen.getByLabelText(/Generated Python entry/), { target: { value: "main.py" } });
  fireEvent.click(screen.getByRole("button", { name: "Run saved Python confined" }));
  await waitFor(() => expect(request).toHaveBeenCalledWith("POST", "/api/runtime/python", { document: "example.md", program: "main.py", seconds: 10, profile: false }));
  await screen.findByText(/Run run-one · completed/);
  expect(screen.getByText(/Unsaved edits are not executed/)).toBeTruthy();
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("agent profiling invokes native capture and returns the saved-source receipt without certifying unsaved edits", async () => {
  const run = await fixture("profile-saved-source"); run.sources[0].uri = "example.md";
  let captured = false;
  vi.mocked(request).mockImplementation(async method => {
    if (method === "POST") { captured = true; return { run }; }
    return { runs: captured ? [{ state: "present", run }] : [] };
  });
  const registry = new UiActions({ workspace: "profile", window: "native" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><NativeRuntimeHistory path="example.md" source="unsaved source is different" /></UiActionContext.Provider>);
  const descriptor = (label: string) => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions).find(action => action.label === label)!;
  const actor = { id: "python:profile-loop", kind: "agent" as const };
  await act(async () => {
    const disclosure = descriptor("Runtime observations");
    await registry.act(registry.scope, actor, { handle: disclosure.handle, arguments: {}, request_id: "disclose" });
  });
  await screen.findByRole("button", { name: "Run saved Python confined" });
  await act(async () => {
    const capture = descriptor("Run saved Python confined");
    const result = await registry.act(registry.scope, actor, { handle: capture.handle, arguments: { program: "main.py", profile: true }, request_id: "profile" });
    expect(result).toMatchObject({ status: "result", outcome: { status: "completed", value: { run: run.id, sources: [{ revision: run.sources[0].revision }], interpretation: "Inspect actual correctness output and profile metrics; completion alone is not verification" } } });
  });
  expect(request).toHaveBeenCalledWith("POST", "/api/runtime/python", { document: "example.md", program: "main.py", seconds: 10, profile: true });
  await screen.findByText(/source changed \(stale\)/);
  expect(screen.getByText(/Unsaved edits are not executed/)).toBeTruthy();
});
