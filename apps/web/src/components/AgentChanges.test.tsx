import { installMockHandler } from "../api/client";
// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { AgentChanges } from "./AgentChanges";
import { acpApi, type AcpState, type AgentChange } from "../api/acp";
vi.mock("../api/acp", () => ({ acpApi: { edits: vi.fn() } }));
installMockHandler(async () => ({}));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
const change: AgentChange = { id: "change", name: "Untitled", path: null, oldText: "# Old 📝\nkeep\n", newText: "# New 📝\nkeep\n", editor: true, status: "pending" };
const state: AcpState = { backend: "codex", ready: true, edits: { mode: "review", changes: [change] } };
it("uses the document comparison editor, waits for Accept, and applies before acknowledging", async () => {
  const apply = vi.fn(); const update = vi.fn();
  vi.mocked(acpApi.edits).mockImplementation(async () => { expect(apply).toHaveBeenCalledWith(change); return state; });
  const view = render(<AgentChanges doc="workspace" state={state} running apply={apply} update={update} />);
  await waitFor(() => expect(view.container.querySelector(".comparison-deleted")?.textContent).toContain("# Old 📝"));
  expect(view.container.querySelector(".comparison-added")?.textContent).toContain("New 📝");
  expect(view.container.querySelector(".cm-content")?.textContent).toContain("keep");
  expect(apply).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Accept change" }));
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { id: "change", accepted: true, error: undefined }, "codex"));
});
it("rejects without changing the editor", async () => {
  const apply = vi.fn(); vi.mocked(acpApi.edits).mockResolvedValue(state);
  render(<AgentChanges doc="workspace" state={state} running apply={apply} update={() => {}} />);
  fireEvent.click(screen.getByRole("button", { name: "Reject change" }));
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { id: "change", accepted: false, error: undefined }, "codex"));
  expect(apply).not.toHaveBeenCalled();
});
it("auto-accepts the same change without a user gesture", async () => {
  const apply = vi.fn(); vi.mocked(acpApi.edits).mockResolvedValue(state);
  render(<AgentChanges doc="workspace" state={{ ...state, edits: { mode: "auto-accept", changes: [{ ...change, status: "applying" }] } }} running apply={apply} update={() => {}} />);
  await waitFor(() => expect(apply).toHaveBeenCalledTimes(1));
  expect(screen.queryByRole("button", { name: "Accept change" })).toBeNull();
});
it("reports a stale buffer to the agent as a failed edit", async () => {
  vi.mocked(acpApi.edits).mockResolvedValue(state);
  render(<AgentChanges doc="workspace" state={state} running apply={() => { throw new Error("The document changed."); }} update={() => {}} />);
  fireEvent.click(screen.getByRole("button", { name: "Accept change" }));
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { id: "change", accepted: false, error: "The document changed." }, "codex"));
  expect(screen.getAllByRole("alert")[0].textContent).toBe("The document changed.");
});
it("changes the per-conversation policy and disables it during a turn", async () => {
  vi.mocked(acpApi.edits).mockResolvedValue(state);
  const view = render(<AgentChanges doc="workspace" state={state} running={false} update={() => {}} />);
  fireEvent.change(screen.getByRole("combobox", { name: "Document edits" }), { target: { value: "auto-accept" } });
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { mode: "auto-accept" }, "codex"));
  view.rerender(<AgentChanges doc="workspace" state={state} running update={() => {}} />);
  expect((screen.getByRole("combobox", { name: "Document edits" }) as HTMLSelectElement).disabled).toBe(true);
});

it("returns newer editor text with a stale rejection so the agent can read again", async () => {
  const { AgentEditConflict } = await import("../lib/agentEdit");
  vi.mocked(acpApi.edits).mockResolvedValue(state);
  render(<AgentChanges doc="workspace" state={state} running apply={() => { throw new AgentEditConflict("Changed during review", "My new sentence"); }} update={() => {}} />);
  fireEvent.click(screen.getByRole("button", { name: "Accept change" }));
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { id: "change", accepted: false, error: "Changed during review", current_text: "My new sentence" }, "codex"));
});

// Guarantee: docs/guarantees/agent/python-execution-has-two-boundaries.md
it("requires a human gesture for Python proposals and labels checks unrun", async () => {
  const pythonState: AcpState = { backend: "python", ready: true, edits: { mode: "review", changes: [{ ...change, editor: false }] } };
  const apply = vi.fn(); vi.mocked(acpApi.edits).mockResolvedValue(pythonState);
  render(<AgentChanges doc="workspace" state={pythonState} running={false} apply={apply} update={() => {}} />);
  expect((screen.getByRole("combobox", { name: "Document edits" }) as HTMLSelectElement).disabled).toBe(true);
  expect(screen.queryByRole("option", { name: "Auto-accept" })).toBeNull();
  expect(screen.getByText(/Checks remain unrun/)).toBeTruthy();
  expect(acpApi.edits).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Accept change" }));
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { id: "change", accepted: true, error: undefined }, "python"));
  expect(apply).not.toHaveBeenCalled(); // Rust applies approved file edits.
});

// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
it("accepts revised New bytes directly from the Agent pane", async () => {
  const { EditorView } = await import("@codemirror/view");
  const { act } = await import("@testing-library/react");
  const apply = vi.fn();
  vi.mocked(acpApi.edits).mockResolvedValue(state);
  const view = render(<AgentChanges doc="workspace" state={state} running apply={apply} update={() => {}} />);
  await waitFor(() => expect(view.container.querySelector('[aria-label="New · proposed change"]')).toBeTruthy());
  const host = view.container.querySelector<HTMLElement>('[aria-label="New · proposed change"]')!;
  const editor = EditorView.findFromDOM(host)!;
  act(() => editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: "My revised proposal\n" } }));
  fireEvent.click(screen.getByRole("button", { name: "Accept change" }));
  await waitFor(() => expect(apply).toHaveBeenCalledWith({ ...change, newText: "My revised proposal\n" }));
  expect(acpApi.edits).toHaveBeenCalledWith("workspace", { id: "change", accepted: true, error: undefined, new_text: "My revised proposal\n" }, "codex");
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("discloses exact proposals but requires the human decision and cannot self-enable auto-accept", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const { act } = await import("@testing-library/react");
  const { useState } = await import("react");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const apply = vi.fn();
  vi.mocked(acpApi.edits).mockResolvedValue({ ...state, edits: { mode: "review", changes: [{ ...change, status: "accepted" }] } });
  function Harness() { const [current, setCurrent] = useState(state); return <AgentChanges doc="workspace" state={current} running={false} apply={apply} update={setCurrent} />; }
  render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  const parent = registry.groups().groups.find(g => g.startsWith("agent-changes:"))!;
  expect(JSON.stringify(registry.observe(parent))).not.toContain("# New");
  const reader = registry.observe(parent).actions.find(a => a.label === "Read conversation edit review bytes")!;
  const reading = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: reader.handle, arguments: { offset: 0, limit: 8192 }, request_id: "read" });
  if (reading.status !== "result" || reading.outcome.status !== "completed") throw Error("Expected reading");
  expect(new TextDecoder().decode(new Uint8Array((reading.outcome.value as { bytes: number[] }).bytes))).toContain("# New 📝");
  const decision = registry.observe(parent).actions.find(a => a.label === "Decide conversation change")!;
  await act(async () => { expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: decision.handle, arguments: { id: "change", accepted: true }, request_id: "ask-human" })).toMatchObject({ outcome: { status: "requires_input", request: "edit-review:workspace:codex:change" } }); });
  const mode = registry.observe(parent).actions.find(a => a.label === "Set conversation edit review mode")!;
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: mode.handle, arguments: { mode: "auto-accept" }, request_id: "no-self-approve" })).toMatchObject({ outcome: { status: "requires_input" } });
  expect(apply).not.toHaveBeenCalled(); expect(acpApi.edits).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Accept change" }));
  await waitFor(() => expect(apply).toHaveBeenCalledWith(change));
  await waitFor(() => expect(screen.queryByRole("button", { name: "Accept change" })).toBeNull());
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: decision.handle, arguments: { id: "change", accepted: true }, request_id: "old-proposal" })).toMatchObject({ status: "stale" });
});

it("uses the host-selected private review transport without calling ordinary review APIs", async () => {
  const submitDecision = vi.fn().mockResolvedValue({ ...state, backend: "python" });
  render(<AgentChanges doc="private:bound" state={{ ...state, backend: "python", edits: { mode: "review", changes: [{ ...change, editor: false }] } }} running submitDecision={submitDecision} update={() => {}} />);
  fireEvent.click(screen.getByRole("button", { name: "Reject change" }));
  await waitFor(() => expect(submitDecision).toHaveBeenCalledWith({ id: "change", accepted: false, error: undefined }));
  expect(acpApi.edits).not.toHaveBeenCalled();
});
