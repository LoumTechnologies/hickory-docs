// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { AgentChanges } from "./AgentChanges";
import { acpApi, type AcpState, type AgentChange } from "../api/acp";
vi.mock("../api/acp", () => ({ acpApi: { edits: vi.fn() } }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
const change: AgentChange = { id: "change", name: "Untitled", path: null, oldText: "# Old 📝\nkeep\n", newText: "# New 📝\nkeep\n", editor: true, status: "pending" };
const state: AcpState = { backend: "codex", ready: true, edits: { mode: "review", changes: [change] } };
it("uses the Git diff renderer, waits for Accept, and applies before acknowledging", async () => {
  const apply = vi.fn(); const update = vi.fn();
  vi.mocked(acpApi.edits).mockImplementation(async () => { expect(apply).toHaveBeenCalledWith(change); return state; });
  const view = render(<AgentChanges doc="workspace" state={state} running apply={apply} update={update} />);
  expect(view.container.querySelector(".diff-line--del")?.textContent).toContain("# Old 📝");
  expect(view.container.querySelector(".diff-line--add")?.textContent).toContain("# New 📝");
  expect(view.container.querySelector(".diff-line--ctx")?.textContent).toContain("keep");
  expect(apply).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Accept change" }));
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { id: "change", accepted: true, error: undefined }));
});
it("rejects without changing the editor", async () => {
  const apply = vi.fn(); vi.mocked(acpApi.edits).mockResolvedValue(state);
  render(<AgentChanges doc="workspace" state={state} running apply={apply} update={() => {}} />);
  fireEvent.click(screen.getByRole("button", { name: "Reject change" }));
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { id: "change", accepted: false, error: undefined }));
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
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { id: "change", accepted: false, error: "The document changed." }));
  expect(screen.getByRole("alert").textContent).toBe("The document changed.");
});
it("changes the per-conversation policy and disables it during a turn", async () => {
  vi.mocked(acpApi.edits).mockResolvedValue(state);
  const view = render(<AgentChanges doc="workspace" state={state} running={false} update={() => {}} />);
  fireEvent.change(screen.getByRole("combobox", { name: "Document edits" }), { target: { value: "auto-accept" } });
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { mode: "auto-accept" }));
  view.rerender(<AgentChanges doc="workspace" state={state} running update={() => {}} />);
  expect((screen.getByRole("combobox", { name: "Document edits" }) as HTMLSelectElement).disabled).toBe(true);
});

it("returns newer editor text with a stale rejection so the agent can read again", async () => {
  const { AgentEditConflict } = await import("../lib/agentEdit");
  vi.mocked(acpApi.edits).mockResolvedValue(state);
  render(<AgentChanges doc="workspace" state={state} running apply={() => { throw new AgentEditConflict("Changed during review", "My new sentence"); }} update={() => {}} />);
  fireEvent.click(screen.getByRole("button", { name: "Accept change" }));
  await waitFor(() => expect(acpApi.edits).toHaveBeenCalledWith("workspace", { id: "change", accepted: false, error: "Changed during review", current_text: "My new sentence" }));
});
