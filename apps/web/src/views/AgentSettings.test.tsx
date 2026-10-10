// Guarantee: docs/guarantees/agent/python-execution-has-two-boundaries.md
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { acpApi } from "../api/acp";
import { AgentSettings } from "./AgentSettings";
vi.mock("../api/acp", () => ({ acpApi: {
  catalogue: vi.fn().mockResolvedValue({ agents: [{ id: "python", name: "Hickory Python", command: "/old/venv/bin/python", args: [] }, { id: "codex", name: "Codex", command: "codex-acp", args: [] }] }),
  pythonSettings: vi.fn().mockResolvedValue({ python: "/old/venv/bin/python", available: true }),
  savePython: vi.fn(), save: vi.fn(),
} }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
it("checks and stores the Python runtime separately from ACP commands", async () => {
  vi.mocked(acpApi.savePython).mockResolvedValue({ python: "/new/venv/bin/python", available: true });
  render(<AgentSettings />);
  await waitFor(() => expect((screen.getByRole("textbox", { name: "Python executable" }) as HTMLInputElement).value).toBe("/old/venv/bin/python"));
  expect((screen.getByRole("textbox", { name: "Agent commands" }) as HTMLTextAreaElement).value).not.toContain('"python"');
  fireEvent.change(screen.getByRole("textbox", { name: "Python executable" }), { target: { value: "/new/venv/bin/python" } });
  fireEvent.click(screen.getByRole("button", { name: "Save Python runtime" }));
  await screen.findByRole("status");
  expect(acpApi.savePython).toHaveBeenCalledWith("/new/venv/bin/python");
  expect(acpApi.save).not.toHaveBeenCalled();
});
it("keeps a failed runtime check visible without claiming it was saved", async () => {
  vi.mocked(acpApi.savePython).mockRejectedValue(new Error("Confinement unavailable"));
  render(<AgentSettings />);
  fireEvent.click(screen.getByRole("button", { name: "Save Python runtime" }));
  expect((await screen.findByRole("alert")).textContent).toBe("Confinement unavailable");
  expect(screen.queryByRole("status")).toBeNull();
});
// Guarantee: docs/guarantees/agent/a-desktop-app-bundles-its-python-agent.md
it("names the runtime bundled with the app without pretending it was configured", async () => {
  const bundled = "/Applications/Hickory Docs.app/Contents/Resources/python-agent/bin/python";
  vi.mocked(acpApi.pythonSettings).mockResolvedValueOnce({ python: "", available: true, bundled });
  render(<AgentSettings />);
  await screen.findByText(new RegExp("bundled with this app"));
  const field = screen.getByRole("textbox", { name: "Python executable" }) as HTMLInputElement;
  expect(field.value).toBe("");
  expect(field.placeholder).toBe(bundled);
});
