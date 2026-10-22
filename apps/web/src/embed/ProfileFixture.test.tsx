// Guarantees: agent/ui-action-dispatch.md; execution/python-profile-keeps-function-semantics.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import record from "../debug/fixtures/python-profile.json";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { ProfileFixture } from "./ProfileFixture";
vi.mock("./DocumentEmbed", () => ({ DocumentEmbed: ({ source }: { source: string }) => <pre data-testid="source">{source}</pre> }));
vi.mock("../debug/RuntimeRuns", () => ({ RuntimeHistory: ({ runs, onPin, onDelete }: { runs: { pinned: boolean }[]; onPin: (run: unknown) => Promise<void>; onDelete: (run: unknown) => Promise<void> }) => <><output data-testid="pinned">{String(runs[0].pinned)}</output><button onClick={() => void onPin(runs[0])}>Pin run</button><button onClick={() => void onDelete(runs[0])}>Remove run</button></> }));
afterEach(cleanup);
it("imports through the shared controller, retains the valid record after refusal, and applies local pin/remove", async () => {
  const registry = new UiActions({ workspace: "recorded", window: "profile" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><ProfileFixture /></UiActionContext.Provider>);
  const load = async (json: string) => {
    const target = registry.observe("profile-fixture").actions.find(action => action.label === "Import retained profile JSON")!;
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "parent", kind: "agent" }, { handle: target.handle, arguments: { json }, request_id: crypto.randomUUID() }); });
    return result;
  };
  expect(await load(JSON.stringify(record))).toMatchObject({ status: "result", outcome: { status: "completed", value: { loaded: true, executed: false, historical: true } } });
  const source = screen.getByTestId("source").textContent;
  const invalid = structuredClone(record); invalid.sources[0].bytes[0] ^= 1;
  expect(await load(JSON.stringify(invalid))).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  expect(screen.getByRole("alert").textContent).toContain("source hash");
  expect(screen.getByTestId("source").textContent).toBe(source);
  fireEvent.click(screen.getByRole("button", { name: "Pin run" }));
  expect(screen.getByTestId("pinned").textContent).toBe("true");
  fireEvent.click(screen.getByRole("button", { name: "Remove run" }));
  expect(screen.queryByTestId("source")).toBeNull();
});
