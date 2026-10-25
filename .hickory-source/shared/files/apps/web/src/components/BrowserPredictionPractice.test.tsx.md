<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/BrowserPredictionPractice.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Browser-local prediction practice uses the shared gate](../../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md)
- [Keep a prediction separate from the answer it is tested against](../../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md)

Captured SHA-256: `c233a6ce449d886e336c364e177b668002614fcc6a9af56e9e0a563aea924432`.

<book:entry path="apps/web/src/components/BrowserPredictionPractice.test.tsx">
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
</book:entry>

</book:doc>
