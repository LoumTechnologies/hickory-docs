// Guarantee: docs/guarantees/agent/explicit-context-updates.md
import { afterEach, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { UiActionContext } from "../lib/uiActionBridge";
import { UiActions } from "../lib/uiActions";
import { ContextUpdates } from "./ContextUpdates";
import { request } from "../api/client";
vi.mock("../api/client", () => ({ request: vi.fn() }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
it("discloses patches before selection and queues their exact observed revisions through the shared handler", async () => {
  const item = { reference: "old", path: "AGENTS.md", before: "HIDDEN-ORIGINAL-CONTEXT\n", after: "HIDDEN-CANDIDATE-CONTEXT\n", status: "changed_observed_passage", revision: "r2", eligible: true, role: "user", scope: "root", request: "r1" };
  const view = { items: [item], updates: [], coverage: "Confirmed comparison only" };
  vi.mocked(request).mockResolvedValue(view);
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><ContextUpdates doc="workspace" branch="a" /></UiActionContext.Provider>);
  await screen.findByText("AGENTS.md · changed_observed_passage");
  expect(registry.groups().groups).toEqual(["agent-context-disclosures"]);
  let n = 0;
  const invoke = async (parent: string, label: string, args = {}) => {
    const action = registry.observe(parent).actions.find(action => action.label === label)!;
    let result; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: args, request_id: String(++n) }); }); return result;
  };
  await invoke("agent-context-disclosures", "Set changed agent context visibility", { open: true });
  const parent = registry.groups().groups.find(group => group.startsWith("context-updates:"))!;
  expect(JSON.stringify(registry.observe(parent))).not.toContain("HIDDEN-ORIGINAL-CONTEXT");
  expect(JSON.stringify(registry.observe(parent))).not.toContain("HIDDEN-CANDIDATE-CONTEXT");
  expect(registry.observe(parent).actions[0].revision).toMatch(/^ui-revision:/);
  await waitFor(() => expect(registry.observe(parent).actions.find(action => action.label === "Refresh patches and delivery status")?.enabled).toBe(true));
  await invoke(parent, "Set context patch visibility: AGENTS.md", { open: true });
  const patch = registry.groups().groups.find(group => group.includes("/patch/"))!;
  await invoke(patch, "Select context update: AGENTS.md", { checked: true });
  vi.mocked(request).mockResolvedValue({ ...view, updates: [{ id: "u1", old: "old", state: "queued", request: null, source: { revision: "r2" } }] });
  expect(await invoke(parent, "Queue selected context updates")).toMatchObject({ status: "result", outcome: { value: { updates: [{ id: "u1", state: "queued" }] } } });
  expect(request).toHaveBeenLastCalledWith("POST", "/api/docs/workspace/agent/python/context", { branch: "a", context: { buffers: [] }, selected: ["old"], revisions: { old: "r2" } });
  await invoke("agent-context-disclosures", "Set changed agent context visibility", { open: false });
  expect(registry.observe(patch).actions).toEqual([]);
});
it("shows the candidate patch and queues explicitly selected unsaved bytes without claiming delivery", async () => {
  const item = { reference: "old", path: "AGENTS.md", before: "old\n", after: "new\n", status: "changed_observed_passage", revision: "r2", eligible: true, role: "user", scope: "root", request: "r1" };
  const view = { items: [item], updates: [], coverage: "Opaque context unavailable" };
  vi.mocked(request).mockResolvedValue(view);
  const getContext = vi.fn(async () => ({ buffers: [{ id: "unsaved", name: "Draft", path: null, content: "draft", focused: true }] }));
  render(<ContextUpdates doc="workspace" branch="a" getContext={getContext} />);
  await screen.findByText("AGENTS.md · changed_observed_passage");
  fireEvent.click(screen.getByLabelText("Select update"));
  vi.mocked(request).mockResolvedValue({ ...view, updates: [{ id: "u1", old: "old", state: "queued", request: null, source: { revision: "r2" } }] });
  fireEvent.click(screen.getByText("Queue selected updates"));
  await waitFor(() => expect(request).toHaveBeenLastCalledWith("POST", "/api/docs/workspace/agent/python/context", expect.objectContaining({ branch: "a", selected: ["old"], revisions: { old: "r2" }, context: expect.objectContaining({ buffers: expect.any(Array) }) })));
  expect(screen.queryByText("Included")).toBeNull();
  expect(document.querySelector(".diff-line--add")?.textContent).toContain("new");
});
// Guarantee: docs/guarantees/agent/explicit-context-updates.md
it("polls at its interval using the current capture callback without a rerender request flood", async () => {
  vi.useFakeTimers();
  vi.mocked(request).mockClear();
  vi.mocked(request).mockResolvedValue({ items: [], updates: [], coverage: "fixture" });
  const first = vi.fn(async () => ({ buffers: [] }));
  const latest = vi.fn(async () => ({ buffers: [{ name: "fresh", path: null, content: "new", focused: true }] }));
  const ui = render(<ContextUpdates doc="workspace" branch="a" getContext={first} />);
  try {
    await act(async () => {});
    expect(request).toHaveBeenCalledTimes(1);
    for (let i = 0; i < 10; i++) ui.rerender(<ContextUpdates doc="workspace" branch="a" getContext={latest} />);
    await act(async () => {});
    expect(request).toHaveBeenCalledTimes(1);
    await act(async () => { await vi.advanceTimersByTimeAsync(5000); });
    expect(request).toHaveBeenCalledTimes(2);
    expect(latest).toHaveBeenCalledTimes(1);
    expect(request).toHaveBeenLastCalledWith("POST", "/api/docs/workspace/agent/python/context", { branch: "a", context: await latest() });
  } finally { ui.unmount(); vi.useRealTimers(); }
});

// Guarantee: docs/guarantees/agent/explicit-context-updates.md
it("does not overlap slow captures or post a retired branch capture", async () => {
  vi.useFakeTimers();
  vi.mocked(request).mockClear();
  vi.mocked(request).mockResolvedValue({ items: [], updates: [], coverage: "fixture" });
  let resolve!: (context: { buffers: [] }) => void;
  const capture = vi.fn(() => new Promise<{ buffers: [] }>(done => { resolve = done; }));
  const ui = render(<ContextUpdates doc="workspace" branch="a" getContext={capture} />);
  try {
    await act(async () => { await vi.advanceTimersByTimeAsync(15000); });
    expect(capture).toHaveBeenCalledTimes(1);
    expect(request).not.toHaveBeenCalled();
    ui.rerender(<ContextUpdates doc="workspace" branch="b" getContext={async () => ({ buffers: [] })} />);
    await act(async () => { resolve({ buffers: [] }); });
    expect(request).toHaveBeenCalledTimes(1);
    expect(request).toHaveBeenCalledWith("POST", "/api/docs/workspace/agent/python/context", { branch: "b", context: { buffers: [] } });
  } finally { ui.unmount(); vi.useRealTimers(); }
});
