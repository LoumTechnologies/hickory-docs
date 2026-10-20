// Protects docs/guarantees/agent/acp-agents-are-first-class.md.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, renderHook, screen, waitFor } from "@testing-library/react";
import { AcpControls, useAcp } from "./AcpControls";
import { acpApi, type AcpState } from "../api/acp";
vi.mock("../api/acp", () => ({ acpApi: { catalogue: vi.fn(), connect: vi.fn(), state: vi.fn(), authenticate: vi.fn(), configure: vi.fn(), permission: vi.fn(), install: vi.fn() } }));
afterEach(() => { cleanup(); vi.clearAllMocks(); vi.mocked(acpApi.state).mockReset(); });
function Surface({ running = null, settingsOpen = true }: { running?: string | null; settingsOpen?: boolean }) {
  const control = useAcp("doc", "codex", undefined, running);
  return <AcpControls doc="doc" backend="codex" running={!!running} control={control} settingsOpen={settingsOpen} />;
}
const ready: AcpState = { backend: "codex", ready: true, configOptions: [{ id: "model", name: "Model", type: "select", currentValue: "fast", options: [{ value: "fast", name: "Fast" }, { value: "careful", name: "Careful" }] }] };
describe("ACP controls", () => {
  it("ignores a sign-in result after switching to the built-in agent", async () => {
    vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [] });
    vi.mocked(acpApi.connect).mockResolvedValue({ backend: "codex", ready: false });
    const hook = renderHook(({ backend }) => useAcp("doc", backend, undefined, null), { initialProps: { backend: "codex" } });
    await waitFor(() => expect(hook.result.current.state?.backend).toBe("codex"));
    let finish!: (state: AcpState) => void;
    const pending = new Promise<AcpState>(resolve => { finish = resolve; });
    let operation!: ReturnType<typeof hook.result.current.act>;
    act(() => { operation = hook.result.current.act(() => pending); });
    vi.mocked(acpApi.state).mockResolvedValue({ backend: "python", ready: true });
    hook.rerender({ backend: "python" });
    await act(async () => { finish(ready); await operation; });
    await waitFor(() => expect(hook.result.current.state?.backend).toBe("python"));
    expect(hook.result.current.busy).toBe(false);
  });
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
    render(<Surface running="turn" settingsOpen={false} />);
    fireEvent.click(await screen.findByRole("button", { name: "Reject" }));
    expect(screen.queryByRole("combobox", { name: "Model" })).toBeNull();
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
  it("distinguishes an installed CLI from its missing ACP adapter", async () => {
    vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [{ id: "codex", name: "Codex", command: "codex-acp", args: [], available: false, cli_available: true, installable: true }] });
    vi.mocked(acpApi.connect).mockRejectedValue(new Error("Codex adapter is not installed"));
    render(<Surface />);
    await screen.findByText(/Codex CLI was found, but its ACP adapter is still needed/);
    expect(screen.getByRole("button", { name: "Install Codex adapter" })).toBeTruthy();
  });
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("retains an addressable permission request without allowing the agent to answer itself", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [] });
  vi.mocked(acpApi.state).mockResolvedValue({ ...ready, permissions: [{ id: "permission", toolCall: { title: "Write", rawInput: { private: "proposal λ" } }, options: [{ optionId: "yes", name: "Allow once", kind: "allow_once" }, { optionId: "no", name: "Reject", kind: "reject_once" }] }] });
  vi.mocked(acpApi.permission).mockResolvedValue({ answered: true });
  render(<UiActionContext.Provider value={registry}><Surface running="turn" settingsOpen={false} /></UiActionContext.Provider>);
  await screen.findByText("Allow once");
  const parent = registry.groups().groups.find(g => g.startsWith("adapter-permission:"))!;
  expect(JSON.stringify(registry.observe(parent))).not.toContain("proposal λ");
  const action = registry.observe(parent).actions.find(a => a.label === "Answer adapter permission")!;
  expect(await registry.act(registry.scope, { id: "acp:session", kind: "agent" }, { handle: action.handle, arguments: { option: "yes" }, request_id: "request-human" })).toMatchObject({ outcome: { status: "requires_input", request: "adapter-permission:doc:codex:permission" } });
  expect(acpApi.permission).not.toHaveBeenCalled();
  const reader = registry.observe(parent).actions.find(a => a.label === "Read adapter permission bytes")!;
  const reading = await registry.act(registry.scope, { id: "acp:session", kind: "agent" }, { handle: reader.handle, arguments: { offset: 0, limit: 8192 }, request_id: "read" });
  if (reading.status !== "result" || reading.outcome.status !== "completed") throw Error("Expected bytes");
  expect(new TextDecoder().decode(new Uint8Array((reading.outcome.value as { bytes: number[] }).bytes))).toContain("proposal λ");
  fireEvent.click(screen.getByText("Reject"));
  await waitFor(() => expect(acpApi.permission).toHaveBeenCalledWith("doc", "permission", "no"));
  await waitFor(() => expect(registry.observe(parent).actions).toEqual([]));
});
it("keeps a permission request visible when the host refuses its answer", async () => {
  vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [] });
  vi.mocked(acpApi.state).mockResolvedValue({ ...ready, permissions: [{ id: "permission", toolCall: { title: "Write" }, options: [{ optionId: "no", name: "Reject", kind: "reject_once" }] }] });
  vi.mocked(acpApi.permission).mockResolvedValue({ answered: false });
  render(<Surface running="turn" settingsOpen={false} />);
  fireEvent.click(await screen.findByText("Reject"));
  await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("host did not accept"));
  expect(screen.getByText("Reject")).toBeTruthy();
});
it("discloses completed adapter tool bytes only after opening its actual details", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [] });
  vi.mocked(acpApi.state).mockResolvedValue({ ...ready, tools: [{ toolCallId: "tool", title: "Read evidence", status: "completed", rawOutput: "HIDDEN OUTPUT λ" }] });
  render(<UiActionContext.Provider value={registry}><Surface running="turn" settingsOpen={false} /></UiActionContext.Provider>);
  await screen.findByText("Read evidence");
  expect(registry.groups().groups.some(g => g.startsWith("adapter-tool-reading:"))).toBe(false);
  const parent = registry.groups().groups.find(g => g.startsWith("adapter-tool:"))!;
  const open = registry.observe(parent).actions[0];
  await act(async () => { await registry.act(registry.scope, { id: "acp:session", kind: "agent" }, { handle: open.handle, arguments: { open: true }, request_id: "open" }); });
  expect(registry.groups().groups.some(g => g.startsWith("adapter-tool-reading:"))).toBe(true);
  expect(JSON.stringify(registry.observe(parent))).not.toContain("HIDDEN OUTPUT");
  const close = registry.observe(parent).actions[0];
  await act(async () => { await registry.act(registry.scope, { id: "acp:session", kind: "agent" }, { handle: close.handle, arguments: { open: false }, request_id: "close" }); });
  expect(registry.groups().groups.some(g => g.startsWith("adapter-tool-reading:"))).toBe(false);
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("configures an observed model through the real controller and refuses stale choices", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [] });
  vi.mocked(acpApi.connect).mockResolvedValue(ready);
  vi.mocked(acpApi.configure).mockResolvedValue({ ...ready, configOptions: [{ ...ready.configOptions![0], currentValue: "careful" }] });
  render(<UiActionContext.Provider value={registry}><Surface /></UiActionContext.Provider>);
  await screen.findByRole("combobox", { name: "Model" });
  const parent = registry.groups().groups.find(g => g.startsWith("adapter-config:"))!;
  const action = registry.observe(parent).actions[0];
  await act(async () => { expect(await registry.act(registry.scope, { id: "acp:session", kind: "agent" }, { handle: action.handle, arguments: { value: "careful" }, request_id: "model" })).toMatchObject({ outcome: { status: "completed", value: { backend: "codex", ready: true } } }); });
  expect(acpApi.configure).toHaveBeenCalledWith("doc", "model", "careful");
  expect((screen.getByLabelText("Model") as HTMLSelectElement).value).toBe("careful");
  expect(await registry.act(registry.scope, { id: "acp:session", kind: "agent" }, { handle: action.handle, arguments: { value: "fast" }, request_id: "stale" })).toMatchObject({ status: "stale" });
});
it("exposes adapter sign-in as a human input request without calling authentication", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [] });
  vi.mocked(acpApi.connect).mockResolvedValue({ backend: "codex", ready: false, authMethods: [{ id: "login", name: "Sign in" }] });
  render(<UiActionContext.Provider value={registry}><Surface /></UiActionContext.Provider>);
  await screen.findByText("Sign in");
  const parent = registry.groups().groups.find(g => g.startsWith("adapter-connection:"))!;
  const action = registry.observe(parent).actions.find(a => a.label === "Authenticate conversation adapter: Sign in")!;
  expect(await registry.act(registry.scope, { id: "acp:session", kind: "agent" }, { handle: action.handle, arguments: {}, request_id: "login" })).toMatchObject({ outcome: { status: "requires_input", request: "adapter-auth:doc:codex:login" } });
  expect(acpApi.authenticate).not.toHaveBeenCalled();
});

it("reports a failed configuration operation as an error result", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [] });
  vi.mocked(acpApi.connect).mockResolvedValue(ready);
  vi.mocked(acpApi.configure).mockRejectedValue(Error("adapter refused configuration"));
  render(<UiActionContext.Provider value={registry}><Surface /></UiActionContext.Provider>);
  await screen.findByRole("combobox", { name: "Model" });
  const parent = registry.groups().groups.find(g => g.startsWith("adapter-config:"))!;
  const action = registry.observe(parent).actions[0];
  await act(async () => { expect(await registry.act(registry.scope, { id: "acp:session", kind: "agent" }, { handle: action.handle, arguments: { value: "careful" }, request_id: "model" })).toMatchObject({ outcome: { status: "unavailable", reason: "adapter refused configuration" } }); });
  expect(screen.getByRole("alert").textContent).toContain("adapter refused configuration");
  expect((screen.getByLabelText("Model") as HTMLSelectElement).value).toBe("fast");
});
