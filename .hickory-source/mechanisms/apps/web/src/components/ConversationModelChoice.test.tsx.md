<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Use Codex subscription inference through Hickory’s built-in agent

For maintainers tracing subscription authentication, confinement and accounting.

## Verify subscription controls with account fixtures

These UI fixtures establish selection, reuse of existing sign-in, recovery actions and human-only authentication. They never call Codex or a provider. Live browser callbacks and real account eligibility remain separate verification.

<storycode:copy id="subscription-1efc15db-0">
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { useState } from "react";
import { ConversationModelChoice } from "./ConversationModelChoice";
import { request } from "../api/client";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
vi.mock("../api/client", () => ({ request: vi.fn() }));
afterEach(() => { cleanup(); vi.resetAllMocks(); });
const providers = [{ id: "openai", label: "OpenAI" }, { id: "grok-subscription", label: "Grok — subscription" }, { id: "codex-subscription", label: "Codex — ChatGPT subscription" }];
function Picker() {
  const [provider, setProvider] = useState("openai");
  return <ConversationModelChoice scope="fixture" provider={provider} model="" disabled={false} providers={providers} defaultModel="" onProvider={setProvider} onModel={() => {}} />;
}
it("selects subscription separately, shows signed-out state and starts only explicit sign-in", async () => {
  vi.mocked(request).mockResolvedValue({ ready: false });
  render(<Picker />);
  expect(request).not.toHaveBeenCalled();
  fireEvent.change(screen.getByRole("combobox", { name: "Provider" }), { target: { value: "codex-subscription" } });
  await waitFor(() => expect(request).toHaveBeenCalledWith("GET", "/api/agents/codex-subscription"));
  expect(request).not.toHaveBeenCalledWith("POST", expect.anything());
  expect(screen.getByText(/No API fallback or automatic credit purchases/)).toBeTruthy();
  vi.mocked(request).mockResolvedValueOnce({ ready: false, auth_url: "https://auth.openai.com/fixture" });
  fireEvent.click(screen.getByRole("button", { name: "Sign in with ChatGPT" }));
  expect((await screen.findByRole("link", { name: "Continue ChatGPT sign-in" })).getAttribute("href")).toBe("https://auth.openai.com/fixture");
  vi.mocked(request).mockResolvedValueOnce({ ready: true });
  fireEvent.click(screen.getByRole("button", { name: "Check sign-in" }));
  expect(await screen.findByText("ChatGPT sign-in ready")).toBeTruthy();
  expect(screen.queryByRole("button", { name: "Sign in with ChatGPT" })).toBeNull();
});
it("reuses existing ChatGPT sign-in without starting another login", async () => {
  vi.mocked(request).mockResolvedValue({ ready: true });
  render(<Picker />);
  fireEvent.change(screen.getByRole("combobox", { name: "Provider" }), { target: { value: "codex-subscription" } });
  expect(await screen.findByText("ChatGPT sign-in ready")).toBeTruthy();
  expect(request).toHaveBeenCalledTimes(1);
});
it("can repair expired authentication even when cached status still reports ready", async () => {
  vi.mocked(request).mockResolvedValue({ ready:true });
  render(<Picker />);
  fireEvent.change(screen.getByRole("combobox", { name:"Provider" }), { target:{ value:"codex-subscription" } });
  const retry = await screen.findByRole("button", { name:"Sign in again with ChatGPT" });
  vi.mocked(request).mockResolvedValueOnce({ ready:false, auth_url:"https://auth.openai.com/fixture" });
  fireEvent.click(retry);
  expect(await screen.findByRole("link", { name:"Continue ChatGPT sign-in" })).toBeTruthy();
  expect(request).toHaveBeenCalledWith("POST", "/api/agents/codex-subscription");
});
it("retains actionable network and authentication failure states", async () => {
  vi.mocked(request).mockRejectedValueOnce(new Error("Network unavailable"));
  render(<Picker />);
  fireEvent.change(screen.getByRole("combobox", { name: "Provider" }), { target: { value: "codex-subscription" } });
  expect((await screen.findByRole("alert")).textContent).toContain("Check your connection and Codex installation");
  vi.mocked(request).mockResolvedValueOnce({ ready: false, error: "ChatGPT sign-in did not complete" });
  fireEvent.click(screen.getByRole("button", { name: "Check sign-in" }));
  await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("ChatGPT sign-in did not complete"));
  expect(screen.getByRole("button", { name: "Sign in with ChatGPT" })).toBeTruthy();
});
it("requires a human to start authentication through an agent UI action", async () => {
  vi.mocked(request).mockResolvedValue({ ready: false });
  const registry = new UiActions({ workspace:"fixture", window:"subscription-test" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><Picker /></UiActionContext.Provider>);
  fireEvent.change(screen.getByRole("combobox", { name: "Provider" }), { target: { value: "codex-subscription" } });
  await waitFor(() => expect(request).toHaveBeenCalledTimes(1));
  const action = registry.groups().groups.flatMap(group => registry.observe(group,64).actions).find(a => a.label === "Sign in to Codex with ChatGPT")!;
  await act(async () => {
    expect(await registry.act(registry.scope, { id:"python:fixture", kind:"agent" }, { handle:action.handle, arguments:{}, request_id:"login" })).toMatchObject({ outcome:{ status:"requires_input" } });
  });
  expect(request).toHaveBeenCalledTimes(1);
});
it("starts Grok native sign-in only explicitly and clears state when changing subscriptions", async () => {
  vi.mocked(request).mockResolvedValue({ ready: false });
  render(<Picker />);
  const picker = screen.getByRole("combobox", { name:"Provider" });
  fireEvent.change(picker, { target:{ value:"grok-subscription" } });
  await waitFor(() => expect(request).toHaveBeenCalledWith("GET", "/api/agents/grok-subscription"));
  expect(request).not.toHaveBeenCalledWith("POST", expect.anything());
  vi.mocked(request).mockResolvedValueOnce({ ready:false, login_pending:true });
  fireEvent.click(screen.getByRole("button", { name:"Sign in with Grok" }));
  expect(await screen.findByText(/Complete Grok sign-in in your browser/)).toBeTruthy();
  expect(request).toHaveBeenCalledWith("POST", "/api/agents/grok-subscription");
  vi.mocked(request).mockResolvedValueOnce({ ready:true });
  fireEvent.click(screen.getByRole("button", { name:"Check sign-in" }));
  expect(await screen.findByText("Grok sign-in ready")).toBeTruthy();
  vi.mocked(request).mockRejectedValueOnce(new Error("Network unavailable"));
  fireEvent.change(picker, { target:{ value:"codex-subscription" } });
  expect(screen.queryByText("Grok sign-in ready")).toBeNull();
  expect((await screen.findByRole("alert")).textContent).toContain("Codex installation");
});
it("requires a human to start Grok authentication through the UI action surface", async () => {
  vi.mocked(request).mockResolvedValue({ ready:false });
  const registry = new UiActions({ workspace:"fixture", window:"grok-test" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><Picker /></UiActionContext.Provider>);
  fireEvent.change(screen.getByRole("combobox", { name:"Provider" }), { target:{ value:"grok-subscription" } });
  await waitFor(() => expect(request).toHaveBeenCalledTimes(1));
  const action = registry.groups().groups.flatMap(group => registry.observe(group,64).actions).find(a => a.label === "Sign in to Grok with Grok")!;
  await act(async () => {
    expect(await registry.act(registry.scope, { id:"python:fixture",kind:"agent" }, { handle:action.handle,arguments:{},request_id:"login" })).toMatchObject({ outcome:{status:"requires_input"} });
  });
  expect(request).toHaveBeenCalledTimes(1);
});
</storycode:copy>

<storycode:file path="apps/web/src/components/ConversationModelChoice.test.tsx"><storycode:paste select="#subscription-1efc15db-0"/></storycode:file>
</storycode:doc>
