// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { request } from "../api/client";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { PrivatePredictionCoach, validatePrivatePredictionState, type PrivatePredictionState } from "./PrivatePredictionCoach";
vi.mock("../api/client", () => ({ request: vi.fn() }));
vi.mock("./ComparisonEditor", () => ({ CodeEditor: ({ item }: { item: { text: string } }) => <pre>{item.text}</pre> }));
afterEach(() => { cleanup(); vi.clearAllMocks(); localStorage.clear(); });
const initial: PrivatePredictionState = { document: "bound", source: "doc.md", source_hash: "a".repeat(64), current: true, question: { requirement_reference: "A single total budget" }, target: {}, environment_scope: "Host startup identities only", actual_values: "withheld", application_integration: "incomplete" };
function responses() {
  vi.mocked(request).mockImplementation(async (method, path) => {
    if (path.endsWith("/state")) return initial;
    if (path.endsWith("/tool")) return { ok: true, source: "captured source only" };
    if (path.endsWith("/evidence")) return { evidence: null };
    if (path.endsWith("/agent")) return { backend: "python", ready: true, edits: { mode: "review", changes: [] }, run: method === "POST" ? { id: "run", status: "running" } : null };
    throw Error(`Unexpected surface ${path}`);
  });
}
it("uses only quarantined readers and starts a provider request only after explicit delegation", async () => {
  responses(); render(<PrivatePredictionCoach initial={initial} />);
  await screen.findByText("captured source only");
  expect(vi.mocked(request).mock.calls.some(call => call[0] === "POST" && call[1].endsWith("/agent"))).toBe(false);
  expect(vi.mocked(request).mock.calls.every(call => call[1].startsWith("/api/prediction/"))).toBe(true);
  fireEvent.click(screen.getByRole("button", { name: "Delegate investigation" }));
  await waitFor(() => expect(vi.mocked(request).mock.calls.some(call => call[0] === "POST" && call[1] === "/api/prediction/agent")).toBe(true));
  expect(screen.getByText(/Current certification is unavailable/)).toBeTruthy();
});
it("requires human input for recursive provider delegation and preserves that request boundary", async () => {
  responses();
  const registry = new UiActions({ workspace: "private", window: "one" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><PrivatePredictionCoach initial={initial} /></UiActionContext.Provider>);
  await screen.findByText("captured source only");
  const command = registry.observe("private-prediction").actions.find(action => action.label === "Delegate private investigation")!;
  let result: unknown;
  await act(async () => { result = await registry.act(registry.scope, { id: "predictor", kind: "agent" }, { handle: command.handle, arguments: {}, request_id: "recursive" }); });
  expect(result).toMatchObject({ status: "result", outcome: { status: "requires_input" } });
  expect(vi.mocked(request).mock.calls.some(call => call[0] === "POST" && call[1].endsWith("/agent"))).toBe(false);
});

it("rejects malformed host metadata before rendering source-reading controls", () => {
  expect(() => validatePrivatePredictionState({ ...initial, source: { Unix: [100, 111, 99] } })).toThrow("Invalid private prediction startup response");
  expect(() => validatePrivatePredictionState({ ...initial, actual_values: "available" })).toThrow();
  expect(() => validatePrivatePredictionState({ ...initial, actual: "unexpected held-out value" })).toThrow();
  expect(() => validatePrivatePredictionState({ ...initial, retained_debugger: null })).toThrow("Invalid private prediction startup response");
  expect(() => validatePrivatePredictionState({ ...initial, retained_debugger: { phase: "complete", actual_values: "withheld", last: { summary: { current_certification: false }, dispatched: true, actual: "unexpected" } } })).toThrow();
});

it("retains consecutive provider/model edits while their separate intents are persisted before Send", async () => {
  responses();
  const waiting: (() => void)[] = [];
  const registry = new UiActions({ workspace: "private", window: "forms" }, () => true, receipt =>
    receipt.stage === "requested" && receipt.action.label.startsWith("Choose private investigation")
      ? new Promise<void>(resolve => { waiting.push(resolve); }) : Promise.resolve());
  render(<UiActionContext.Provider value={registry}><PrivatePredictionCoach initial={initial} /></UiActionContext.Provider>);
  await screen.findByText("captured source only");
  fireEvent.change(screen.getByRole("textbox", { name: "Provider" }), { target: { value: "openrouter" } });
  fireEvent.change(screen.getByRole("textbox", { name: "Model" }), { target: { value: "anthropic/claude-sonnet-4.5" } });
  await waitFor(() => expect(waiting).toHaveLength(2));
  expect((screen.getByRole("button", { name: "Delegate investigation" }) as HTMLButtonElement).disabled).toBe(true);
  await act(async () => { waiting.forEach(release => release()); });
  expect((screen.getByRole("textbox", { name: "Provider" }) as HTMLInputElement).value).toBe("openrouter");
  expect((screen.getByRole("textbox", { name: "Model" }) as HTMLInputElement).value).toBe("anthropic/claude-sonnet-4.5");
  fireEvent.click(screen.getByRole("button", { name: "Delegate investigation" }));
  await waitFor(() => expect(vi.mocked(request).mock.calls.find(call => call[0] === "POST" && call[1].endsWith("/agent"))?.[2]).toMatchObject({ provider: "openrouter", model: "anthropic/claude-sonnet-4.5" }));
});

it("requires a human to end and retains a real host lifecycle receipt before retiring the coach", async () => {
  responses();
  const ordinary = vi.mocked(request).getMockImplementation()!;
  vi.mocked(request).mockImplementation(async (method, path, body) => path.endsWith("/end")
    ? { ended: true, history_retained: true, root_still_private: true }
    : ordinary(method, path, body));
  const registry = new UiActions({ workspace: "private", window: "ending" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><PrivatePredictionCoach initial={initial} /></UiActionContext.Provider>);
  await screen.findByText("captured source only");
  const command = registry.observe("private-prediction").actions.find(action => action.label === "End private practice")!;
  let result: unknown;
  await act(async () => { result = await registry.act(registry.scope, { id: "predictor", kind: "agent" }, { handle: command.handle, arguments: {}, request_id: "end-own-quarantine" }); });
  expect(result).toMatchObject({ status: "result", outcome: { status: "requires_input" } });
  expect(vi.mocked(request).mock.calls.some(call => call[1].endsWith("/end"))).toBe(false);
  fireEvent.click(screen.getByRole("button", { name: "End private practice" }));
  await screen.findByRole("heading", { name: "Private practice ended" });
  expect(screen.queryByRole("button", { name: "Delegate investigation" })).toBeNull();
  expect(registry.observe("private-prediction").actions).toHaveLength(0);
});

it("shares agent sealing and human Continue through the host-bound retained controller", async () => {
  let current: PrivatePredictionState = { ...initial, retained_plan: { initial: { requirement_reference: "Initial allowance" }, next: { requirement_reference: "Future allowance" } }, retained_debugger: { phase: "not_started", actual_values: "withheld", last: null } };
  const calls: unknown[] = [];
  vi.mocked(request).mockImplementation(async (_method, path, body) => {
    if (path.endsWith("/state")) return current;
    if (path.endsWith("/tool")) return { ok: true, source: "captured source only" };
    if (path.endsWith("/evidence")) return { evidence: null };
    if (path.endsWith("/agent")) return { backend: "python", ready: true, edits: { mode: "review", changes: [] } };
    if (path.endsWith("/debugger")) {
      calls.push(body);
      const seal = (body as { name: string }).name === "seal_debugger_prediction";
      current = { ...current, retained_debugger: { phase: seal ? "sealed" : "paused_needs_prediction", actual_values: "withheld", last: null } };
      return { isError: false, structuredContent: seal ? { sealed: "initial", executed: false } : { dispatched: true, summary: { classification: "first_attempt_verified" } } };
    }
    throw Error(`Unexpected ${path}`);
  });
  const registry = new UiActions({ workspace: "private", window: "retained" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><PrivatePredictionCoach initial={current} /></UiActionContext.Provider>);
  await screen.findByText("captured source only");
  expect((screen.getByRole("button", { name: "Launch or continue private debugger" }) as HTMLButtonElement).disabled).toBe(true);
  fireEvent.change(screen.getByRole("textbox", { name: "Debugger predicate JSON" }), { target: { value: '{"kind":"equals","value":100}' } });
  fireEvent.change(screen.getByRole("textbox", { name: "Debugger prediction rationale" }), { target: { value: "Initial budget before launch" } });
  await waitFor(() => expect((screen.getByRole("button", { name: "Seal debugger prediction" }) as HTMLButtonElement).disabled).toBe(false));
  const seal = registry.observe("private-prediction").actions.find(action => action.label === "Seal debugger prediction")!;
  await act(async () => { await registry.act(registry.scope, { id: "predictor", kind: "agent" }, { handle: seal.handle, arguments: {}, request_id: "seal-initial" }); });
  expect(calls[0]).toEqual({ name: "seal_debugger_prediction", arguments: { expected_source_hash: initial.source_hash, predicted: { kind: "equals", value: 100 }, rationale: "Initial budget before launch" } });
  fireEvent.click(screen.getByRole("button", { name: "Launch or continue private debugger" }));
  await waitFor(() => expect(calls).toHaveLength(2));
  expect(calls[1]).toEqual({ name: "continue_debugger_prediction", arguments: {} });
  await screen.findByText(/Phase: paused_needs_prediction/);
});


it("dispatches an explicitly chosen private ACP adapter without inventing a passing result", async () => {
  responses(); render(<PrivatePredictionCoach initial={initial} />);
  await screen.findByText("captured source only");
  fireEvent.change(screen.getByRole("textbox", { name: "ACP adapter (optional)" }), { target: { value: "isolated-qa" } });
  await waitFor(() => expect((screen.getByRole("button", { name: "Delegate investigation" }) as HTMLButtonElement).disabled).toBe(false));
  fireEvent.click(screen.getByRole("button", { name: "Delegate investigation" }));
  await waitFor(() => expect(vi.mocked(request).mock.calls.find(call => call[0] === "POST" && call[1].endsWith("/agent"))?.[2]).toMatchObject({ acp_agent: "isolated-qa" }));
  expect(screen.getByText(/Current certification is unavailable/)).toBeTruthy();
});

it("requires End before native close and closes after End independently of retired receipts", async () => {
  responses();
  const ordinary = vi.mocked(request).getMockImplementation()!;
  vi.mocked(request).mockImplementation(async (method, path, body) => {
    if (path.endsWith("/end")) return { ended: true, history_retained: true, root_still_private: true };
    if (path === "/api/window/close") return { ok: true };
    return ordinary(method, path, body);
  });
  let retired = false;
  const registry = new UiActions({ workspace: "private", window: "close" }, () => true, async () => { if (retired) throw Error("retired receipt endpoint"); });
  render(<UiActionContext.Provider value={registry}><PrivatePredictionCoach initial={initial} /></UiActionContext.Provider>);
  await screen.findByText("captured source only");
  await act(async () => { window.dispatchEvent(new Event("hickory-window-close-request")); });
  expect(screen.getByText(/End private practice before closing/)).toBeTruthy();
  expect(vi.mocked(request).mock.calls.some(call => call[1] === "/api/window/close")).toBe(false);
  const command = registry.observe("private-window").actions[0];
  let result: unknown;
  await act(async () => { result = await registry.act(registry.scope, { kind: "agent", id: "predictor" }, { handle: command.handle, arguments: {}, request_id: "close-quarantine" }); });
  expect(result).toMatchObject({ outcome: { status: "requires_input" } });
  fireEvent.click(screen.getByRole("button", { name: "End private practice" }));
  await screen.findByRole("heading", { name: "Private practice ended" });
  retired = true;
  fireEvent.click(screen.getByRole("button", { name: "Close window" }));
  await waitFor(() => expect(request).toHaveBeenCalledWith("POST", "/api/window/close", {}));
});
