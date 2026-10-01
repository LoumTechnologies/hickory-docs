// Protects docs/guarantees/editor-intelligence/a-project-environment-offers-its-own-sync.md
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { EnvironmentNotice, EnvironmentPanel } from "./EnvironmentPanel";
import type { EnvironmentFinding } from "../api/environments";
import type { Environments } from "./useEnvironments";

afterEach(cleanup);
const finding: EnvironmentFinding = {
  project: "/repo/analysis", manager: "uv", target: "host", profile: "manager defaults", revision: "one",
  state: "environment-missing", message: "Python dependencies are not installed.", details: "No .venv exists.",
  actions: [{ id: "sync", label: "Sync dependencies", argv: ["uv", "sync", "--locked"], effects: "Keeps uv.lock unchanged." }],
  install_url: null, interpreter: null, manager_choices: [],
};
const controller = (over: Partial<Environments> = {}): Environments => ({
  findings: [finding], running: {}, notices: [finding], pending: null, error: null,
  refresh: vi.fn(), act: vi.fn(), dismiss: vi.fn(), choose: vi.fn(), ...over,
});
describe("project environment affordances", () => {
  it("shows command and directory beside the sync action and submits that finding", () => {
    const environments = controller();
    render(<EnvironmentNotice environments={environments} />);
    expect(screen.getByText("uv sync --locked")).toBeTruthy();
    expect(screen.getByText("/repo/analysis · host")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Sync dependencies" }));
    expect(environments.act).toHaveBeenCalledWith(finding, "sync");
    fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
    expect(environments.dismiss).toHaveBeenCalledWith(finding);
  });
  it("disables a repair while its terminal is running", () => {
    render(<EnvironmentNotice environments={controller({ running: { [finding.project]: "session" } })} />);
    expect((screen.getByRole("button", { name: "Installing…" }) as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByRole("button", { name: "Show terminal" })).toBeTruthy();
  });
  it("clears the notice when the recheck succeeds and keeps readiness in the expandable panel", () => {
    const environments = controller({ findings: [{ ...finding, state: "ready", message: "Python environment is ready.", actions: [] }], notices: [] });
    const { container } = render(<><EnvironmentNotice environments={environments} /><EnvironmentPanel environments={environments} /></>);
    expect(screen.queryByRole("status")).toBeNull();
    expect(container.querySelector("details")?.open).toBe(false);
    fireEvent.click(screen.getByRole("button", { name: "Recheck" }));
    expect(environments.refresh).toHaveBeenCalledWith(true);
  });
});
