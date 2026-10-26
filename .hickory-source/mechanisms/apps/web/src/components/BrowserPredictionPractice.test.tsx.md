<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/BrowserPredictionPractice.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-32e2b9b729a8"></a>
## `apps/web/src/components/BrowserPredictionPractice.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Browser-local prediction practice uses the shared gate](../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-32e2b9b729a8-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-32e2b9b729a8-expression_statement">
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
</storycode:copy>
<a id="f-32e2b9b729a8-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-32e2b9b729a8-expression_statement-2">
afterEach(() => { cleanup(); vi.clearAllMocks(); localStorage.clear(); });
</storycode:copy>
<a id="f-32e2b9b729a8-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-32e2b9b729a8-expression_statement-3">
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
</storycode:copy>

### Assemble `apps/web/src/components/BrowserPredictionPractice.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/BrowserPredictionPractice.test.tsx"><storycode:paste select="#f-32e2b9b729a8-expression_statement"/><storycode:paste select="#f-32e2b9b729a8-expression_statement-2"/><storycode:paste select="#f-32e2b9b729a8-expression_statement-3"/></storycode:file>

</storycode:doc>
