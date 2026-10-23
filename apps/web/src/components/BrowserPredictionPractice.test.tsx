// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions, type UiResult } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { browserPredictionHistory, runBrowserPrediction, sealBrowserPrediction, type BrowserSeal } from "../tour/browserPrediction";
import { BrowserPredictionPractice } from "./BrowserPredictionPractice";
vi.mock("../tour/browserPrediction", () => ({
  savedBrowserPredictionInputs: () => ({ attempts: [80, 80], budget: 100 }),
  parseBrowserInputs: (attempts: string, budget: string) => ({ attempts: JSON.parse(attempts), budget: Number(budget) }),
  browserPredictionSource: (corrected: boolean) => corrected ? "corrected fixture" : "original fixture",
  sealBrowserPrediction: vi.fn(), browserPredictionHistory: vi.fn(), runBrowserPrediction: vi.fn(),
  recoverBrowserPrediction: vi.fn(), promoteBrowserPrediction: vi.fn(), inspectBrowserPrediction: vi.fn(),
}));
afterEach(() => { cleanup(); vi.clearAllMocks(); localStorage.clear(); });
it("does not adopt a late seal after source ownership changes away and back", async () => {
  let resolve!: (seal: BrowserSeal) => void;
  vi.mocked(sealBrowserPrediction).mockImplementation(() => new Promise<BrowserSeal>(done => { resolve = done; }));
  const registry = new UiActions({ workspace: "browser", window: "one" }, () => true, async () => {});
  const result = vi.fn(), invalidate = vi.fn();
  const view = (corrected: boolean) => <UiActionContext.Provider value={registry}><BrowserPredictionPractice corrected={corrected} onResult={result} onInvalidate={invalidate} /></UiActionContext.Provider>;
  const rendered = render(view(false));
  fireEvent.change(screen.getByLabelText("Prediction rationale"), { target: { value: "One total budget" } });
  const command = registry.observe("browser-prediction").actions.find(action => action.label === "Seal browser prediction before execution")!;
  let pending!: Promise<UiResult>;
  await act(async () => { pending = registry.act(registry.scope, { id: "predictor", kind: "agent" }, { handle: command.handle, arguments: {}, request_id: "seal-owner" }); await Promise.resolve(); });
  expect(sealBrowserPrediction).toHaveBeenCalledOnce();
  rendered.rerender(view(true)); rendered.rerender(view(false));
  let actual: UiResult | undefined;
  await act(async () => {
    resolve({ receipt: { id: "old", generation: 1 }, scenario: "fixture", identity: { source: "a".repeat(64), inputs: "b".repeat(64), dependencies: {}, environment: {} }, actor: { id: "predictor", kind: "agent" }, source: "original fixture" });
    actual = await pending;
  });
  expect(actual).toMatchObject({ outcome: { status: "unavailable" } });
  expect(browserPredictionHistory).not.toHaveBeenCalled();
  expect(runBrowserPrediction).not.toHaveBeenCalled();
  expect(result).not.toHaveBeenCalled();
  expect((screen.getByRole("button", { name: "Run sealed prediction" }) as HTMLButtonElement).disabled).toBe(true);
});
