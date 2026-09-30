// Protects docs/guarantees/agent/acp-agents-are-first-class.md.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { AcpControls, useAcp } from "./AcpControls";
import { acpApi, type AcpState } from "../api/acp";
vi.mock("../api/acp", () => ({ acpApi: { catalogue: vi.fn(), connect: vi.fn(), state: vi.fn(), authenticate: vi.fn(), configure: vi.fn(), permission: vi.fn(), install: vi.fn() } }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
function Surface({ running = null }: { running?: string | null }) {
  const control = useAcp("doc", "codex", undefined, running);
  return <AcpControls doc="doc" backend="codex" running={!!running} control={control} />;
}
const ready: AcpState = { backend: "codex", ready: true, configOptions: [{ id: "model", name: "Model", type: "select", currentValue: "fast", options: [{ value: "fast", name: "Fast" }, { value: "careful", name: "Careful" }] }] };
describe("ACP controls", () => {
  it("renders the adapter's model choices and persists a change", async () => {
    vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [] });
    vi.mocked(acpApi.connect).mockResolvedValue(ready);
    vi.mocked(acpApi.configure).mockResolvedValue({ ...ready, configOptions: [{ ...ready.configOptions![0], currentValue: "careful" }] });
    render(<Surface />);
    await screen.findByRole("combobox", { name: "Model" });
    fireEvent.change(screen.getByRole("combobox", { name: "Model" }), { target: { value: "careful" } });
    await waitFor(() => expect(acpApi.configure).toHaveBeenCalledWith("doc", "model", "careful"));
  });
  it("offers adapter sign-in without asking for Hickory's API key", async () => {
    vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [] });
    vi.mocked(acpApi.connect).mockResolvedValue({ backend: "codex", ready: false, authMethods: [{ id: "chat-gpt", name: "Sign in with ChatGPT" }] });
    vi.mocked(acpApi.authenticate).mockResolvedValue(ready);
    render(<Surface />);
    fireEvent.click(await screen.findByRole("button", { name: "Sign in with ChatGPT" }));
    await waitFor(() => expect(acpApi.authenticate).toHaveBeenCalledWith("doc", "chat-gpt"));
    await screen.findByRole("combobox", { name: "Model" });
  });
  it("shows a pending permission with the adapter's exact choices", async () => {
    vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [] });
    vi.mocked(acpApi.state).mockResolvedValue({ ...ready, permissions: [{ id: "permission", toolCall: { title: "Write the document", rawInput: { path: "notes.md" } }, options: [{ optionId: "yes", name: "Allow once", kind: "allow_once" }, { optionId: "no", name: "Reject", kind: "reject_once" }] }] });
    vi.mocked(acpApi.permission).mockResolvedValue({ answered: true });
    render(<Surface running="turn" />);
    fireEvent.click(await screen.findByRole("button", { name: "Reject" }));
    await waitFor(() => expect(acpApi.permission).toHaveBeenCalledWith("doc", "permission", "no"));
  });
  it("offers installation for a missing catalogued adapter", async () => {
    vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [{ id: "codex", name: "Codex", command: "codex-acp", args: [], available: false, installable: true }] });
    vi.mocked(acpApi.connect).mockRejectedValue(new Error("Codex adapter is not installed"));
    vi.mocked(acpApi.install).mockResolvedValue({ agents: [{ id: "codex", name: "Codex", command: "codex-acp", args: [], available: true }] });
    render(<Surface />);
    fireEvent.click(await screen.findByRole("button", { name: "Install Codex adapter" }));
    await waitFor(() => expect(acpApi.install).toHaveBeenCalledWith("codex"));
  });
});
