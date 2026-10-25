<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/tour/browserPrediction.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Browser-local prediction practice uses the shared gate](../../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md)
- [Keep a prediction separate from the answer it is tested against](../../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md)

Captured SHA-256: `39e1af5efaf490de16b38e56fbe45f6d59a686428e7c7c2908e027f99cff89a5`.

<book:entry path="apps/web/src/tour/browserPrediction.test.ts">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
declare const require: (name: string) => { webcrypto: Crypto };
const { webcrypto } = require("node:crypto");
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { browserPredictionTransition, hickLangArtifactHash } from "../editor/hickLang";
import { browserRuntimeAssetHash, createBrowserDebugger } from "../debug/browser/transport";
import { parseBrowserInputs, savedBrowserPredictionInputs, sealBrowserPrediction, runBrowserPrediction } from "./browserPrediction";
vi.mock("../editor/hickLang", () => ({ loadHickLang: vi.fn(async () => {}), hickLangArtifactHash: vi.fn(), browserPredictionTransition: vi.fn() }));
vi.mock("../debug/browser/transport", () => ({ browserRuntimeAssetHash: vi.fn(), createBrowserDebugger: vi.fn() }));
beforeEach(() => { vi.stubGlobal("crypto", webcrypto); Object.defineProperty(navigator, "locks", { configurable: true, value: { request: async (_key: string, callback: () => unknown) => callback() } }); });
afterEach(() => { vi.unstubAllGlobals(); vi.clearAllMocks(); localStorage.clear(); });
it("accepts bounded integer questions and refuses injected/non-finite or ambiguous inputs", () => {
  expect(parseBrowserInputs("[70,65]", "90")).toEqual({ attempts: [70, 65], budget: 90 });
  for (const [attempts, budget] of [["[1,\"globalThis.secret\"]", "90"], ["[0.5]", "90"], ["[1]", "0"], ["[1]", "Infinity"], [JSON.stringify(Array(9).fill(1)), "90"]]) expect(() => parseBrowserInputs(attempts, budget)).toThrow();
});
it("restores question inputs without restoring foreign predictions or private fields", () => {
  localStorage.setItem("hickory.browserPredictionInputs.v1", JSON.stringify({ attempts: [70, 65], budget: 90 }));
  expect(savedBrowserPredictionInputs()).toEqual({ attempts: [70, 65], budget: 90 });
  localStorage.setItem("hickory.browserPredictionInputs.v1", JSON.stringify({ attempts: [70, 65], budget: 90, prediction: "private-held-out-value" }));
  expect(JSON.stringify(savedBrowserPredictionInputs())).not.toContain("private-held-out-value");
  expect(savedBrowserPredictionInputs()).toEqual({ attempts: [80, 80], budget: 100 });
});
it("refuses a changed runtime asset before admission or worker creation", async () => {
  vi.mocked(browserRuntimeAssetHash).mockResolvedValue("a".repeat(64)); vi.mocked(hickLangArtifactHash).mockResolvedValue("b".repeat(64));
  vi.mocked(browserPredictionTransition).mockReturnValue({ snapshot: '{"entries":[]}', value: { receipt: { id: "sealed", generation: 1 }, scenario: "scenario" }, error: null });
  const actor = { id: "predictor", kind: "agent" as const };
  const seal = await sealBrowserPrediction(false, { attempts: [70, 65], budget: 90 }, "90", "One budget", actor, () => true);
  vi.mocked(browserRuntimeAssetHash).mockResolvedValue("c".repeat(64));
  await expect(runBrowserPrediction(seal, { attempts: [70, 65], budget: 90 }, actor, new AbortController().signal, () => true)).rejects.toThrow("assets changed");
  expect(createBrowserDebugger).not.toHaveBeenCalled();
  expect(browserPredictionTransition).toHaveBeenCalledTimes(1); // Seal only; no admission.
});
</book:entry>

</book:doc>
