// Guarantee: docs/guarantees/agent/explicit-context-updates.md
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ContextUpdates } from "./ContextUpdates";
import { request } from "../api/client";
vi.mock("../api/client", () => ({ request: vi.fn() }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
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
