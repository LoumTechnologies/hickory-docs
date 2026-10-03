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
  expect(screen.queryByRole("button")).toBeNull();
});
it("shows installation failures and allows retry", async () => {
  installMockHandler(async (method) => {
    if (method === "GET") return initial;
    throw new Error("Cannot write your shell profile; check its permissions.");
  });
  render(<CommandPathSettings />);
  fireEvent.click(await screen.findByRole("button"));
  expect((await screen.findByRole("alert")).textContent).toContain("Cannot write");
  expect((screen.getByRole("button") as HTMLButtonElement).disabled).toBe(false);
});
it("hides the section for a headless engine", async () => {
  const read = vi.fn().mockRejectedValue(new Error("Not found"));
  installMockHandler(read);
  const { container } = render(<CommandPathSettings />);
  await waitFor(() => expect(read).toHaveBeenCalled());
  expect(container.textContent).toBe("");
});
