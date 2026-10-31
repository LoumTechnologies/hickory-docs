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
  await screen.findByRole("button", { name: "Changed agent context (1)" });
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
  fireEvent.click(await screen.findByRole("button", { name: "Changed agent context (1)" }));
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
    expect(screen.queryByRole("button", { name: /Changed agent context/ })).toBeNull();
    expect(document.querySelector(".agent-context-gutter")).toBeNull();
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

// Guarantee: docs/guarantees/agent/explicit-context-updates.md
it("does not submit a manual context capture after its originating branch retires", async () => {
  const view = { items: [{ reference: "old", path: "note.md", before: "before", after: "after", status: "changed", revision: "r2", eligible: true }], updates: [], coverage: "current fixture" };
  vi.mocked(request).mockResolvedValue(view);
  let release!: (value: { buffers: [] }) => void;
  const capture = vi.fn().mockResolvedValue({ buffers: [] }).mockImplementationOnce(async () => ({ buffers: [] })).mockImplementationOnce(() => new Promise(resolve => { release = resolve; }));
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const ui = (branch: string) => <UiActionContext.Provider value={registry}><ContextUpdates doc="workspace" branch={branch} getContext={capture} /></UiActionContext.Provider>;
  const rendered = render(ui("a"));
  await waitFor(() => expect(request).toHaveBeenCalledTimes(1));
  const open = registry.observe("agent-context-disclosures").actions[0];
  await act(async () => { await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: open.handle, arguments: { open: true }, request_id: "open" }); });
  await waitFor(() => expect(release).toBeTypeOf("function"));
  rendered.rerender(ui("b"));
  await waitFor(() => expect(vi.mocked(request).mock.calls.some(call => (call[2] as { branch: string }).branch === "b")).toBe(true));
  const calls = vi.mocked(request).mock.calls.length;
  await act(async () => { release({ buffers: [] }); });
  expect(request).toHaveBeenCalledTimes(calls);
  expect(vi.mocked(request).mock.calls.filter(call => (call[2] as { branch: string }).branch === "a")).toHaveLength(1);
});

// Guarantee: docs/guarantees/agent/explicit-context-updates.md
it("keeps a newer explicit queue result when an older background refresh completes late", async () => {
  vi.useFakeTimers();
  const item = { reference: "old", path: "note.md", before: "before", after: "after", status: "changed_observed_passage", revision: "r2", eligible: true, role: "user", scope: "root", request: "r1" };
  const initial = { items: [item], updates: [], coverage: "initial candidate" };
  let background = false, release!: (value: typeof initial) => void;
  vi.mocked(request).mockImplementation(async (_method, _path, body) => {
    if ((body as { selected?: string[] }).selected?.length) return { ...initial, coverage: "queued candidate", updates: [{ id: "u1", old: "old", state: "queued", request: null, source: { revision: "r2" } }] };
    if (background) return new Promise(resolve => { release = resolve; });
    return initial;
  });
  const rendered = render(<ContextUpdates doc="workspace" branch="a" />);
  try {
    await act(async () => {});
    await act(async () => { fireEvent.click(screen.getByRole("button", { name: "Changed agent context (1)" })); });
    const patch = document.querySelector<HTMLDetailsElement>(".agent-context-panel details")!;
    await act(async () => { patch.open = true; fireEvent(patch, new Event("toggle")); });
    fireEvent.click(screen.getByLabelText("Select update"));
    await act(async () => { await vi.advanceTimersByTimeAsync(0); });
    background = true;
    await act(async () => { await vi.advanceTimersByTimeAsync(5000); });
    expect(release).toBeTypeOf("function");
    await act(async () => fireEvent.click(screen.getByText("Queue selected updates")));
    expect(screen.getByText("queued candidate")).toBeTruthy();
    await act(async () => { release({ ...initial, coverage: "stale background" }); });
    expect(screen.getByText("queued candidate")).toBeTruthy();
    expect(screen.queryByText("stale background")).toBeNull();
  } finally { rendered.unmount(); vi.useRealTimers(); }
});
