import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, it, expect, vi } from "vitest";
import { installMockHandler } from "../api/client";
import { CommandPathSettings, type CommandPathStatus } from "./CommandPathSettings";
const initial: CommandPathStatus = { available: true, installed: false, can_remove: false,
  command: "/home/me/.local/bin/hick", source: "/app/hick", conflict: null, message: "Install for your user." };
afterEach(() => { cleanup(); installMockHandler(null as never); });
// Guarantee: docs/guarantees/release/the-app-installs-its-command-for-the-user.md
it("installs and removes through the desktop API, showing the resulting status", async () => {
  const calls = vi.fn();
  installMockHandler(async (method, _path, body) => {
    if (method === "GET") return initial;
    calls(body);
    const installed = (body as { action: string }).action === "install";
    return { ...initial, installed, can_remove: installed, message: "Open a new terminal." };
  });
  render(<CommandPathSettings />);
  fireEvent.click(await screen.findByRole("button", { name: "Install hick command…" }));
  fireEvent.click(await screen.findByRole("button", { name: "Remove hick command" }));
  await waitFor(() => expect(calls.mock.calls).toEqual([[{ action: "install" }], [{ action: "remove" }]]));
  expect(await screen.findByText("Not installed")).toBeTruthy();
});
it("shows a conflict without offering to overwrite it", async () => {
  installMockHandler(async () => ({ ...initial, conflict: "/other/hick" }));
  render(<CommandPathSettings />);
  expect(await screen.findByText(/Another hick command/)).toBeTruthy();
  expect(screen.queryByRole("button", { name: "Install hick command…" })).toBeNull();
});
it("shows installation failures and allows retry", async () => {
  installMockHandler(async (method) => {
    if (method === "GET") return initial;
    throw new Error("Cannot write your shell profile; check its permissions.");
  });
  render(<CommandPathSettings />);
  fireEvent.click(await screen.findByRole("button", { name: "Install hick command…" }));
  expect((await screen.findByRole("alert")).textContent).toContain("Cannot write");
  expect((screen.getByRole("button", { name: "Install hick command…" }) as HTMLButtonElement).disabled).toBe(false);
});
it("hides the section for a headless engine", async () => {
  const read = vi.fn().mockRejectedValue(new Error("Not found"));
  installMockHandler(read);
  const { container } = render(<CommandPathSettings />);
  await waitFor(() => expect(read).toHaveBeenCalled());
  expect(container.textContent).toBe("");
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares registered install with the real host status and disables conflicting operations", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const calls = vi.fn();
  installMockHandler(async (method, _path, body) => {
    if (method === "GET") return initial;
    calls(body); return { ...initial, installed: true, can_remove: true, message: "Actual installed status" };
  });
  render(<UiActionContext.Provider value={registry}><CommandPathSettings /></UiActionContext.Provider>);
  await screen.findByRole("button", { name: "Install hick command…" });
  const parent = registry.groups().groups.find(g => g.startsWith("command-path:"))!;
  const install = registry.observe(parent).actions.find(a => a.label === "Install hick command")!;
  expect(JSON.stringify(registry.observe(parent))).not.toContain(initial.source);
  await act(async () => { expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: install.handle, arguments: {}, request_id: "install" })).toMatchObject({ outcome: { value: { installed: true, message: "Actual installed status" } } }); });
  expect(calls).toHaveBeenCalledExactlyOnceWith({ action: "install" });
  expect(registry.observe(parent).actions.find(a => a.label === "Install hick command")!.enabled).toBe(false);
  expect(registry.observe(parent).actions.find(a => a.label === "Remove hick command")!.enabled).toBe(true);
  const read = registry.observe(parent).actions.find(a => a.label === "Read hick command status bytes")!;
  const reading = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: read.handle, arguments: { offset: 0, limit: 8192 }, request_id: "read" });
  if (reading.status !== "result" || reading.outcome.status !== "completed") throw Error("Expected bytes");
  expect(new TextDecoder().decode(new Uint8Array((reading.outcome.value as { bytes: number[] }).bytes))).toContain(initial.command);
});
