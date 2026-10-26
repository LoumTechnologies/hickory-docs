<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/tour/browserPrediction.test.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5eb82555b921"></a>
## `apps/web/src/tour/browserPrediction.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Browser-local prediction practice uses the shared gate](../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-5eb82555b921-ambient_declaration"></a>
### `ambient_declaration`

<storycode:copy id="f-5eb82555b921-ambient_declaration">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
declare const require: (name: string) => { webcrypto: Crypto };
</storycode:copy>
<a id="f-5eb82555b921-webcrypto"></a>
### `{ webcrypto }`

<storycode:copy id="f-5eb82555b921-webcrypto">
const { webcrypto } = require("node:crypto");
</storycode:copy>
<a id="f-5eb82555b921-import_statement"></a>
### `import_statement`

<storycode:copy id="f-5eb82555b921-import_statement">
import { afterEach, beforeEach, expect, it, vi } from "vitest";
</storycode:copy>
<a id="f-5eb82555b921-import_statement-2"></a>
### `import_statement`

<storycode:copy id="f-5eb82555b921-import_statement-2">
import { browserPredictionTransition, hickLangArtifactHash } from "../editor/hickLang";
</storycode:copy>
<a id="f-5eb82555b921-import_statement-3"></a>
### `import_statement`

<storycode:copy id="f-5eb82555b921-import_statement-3">
import { browserRuntimeAssetHash, createBrowserDebugger } from "../debug/browser/transport";
</storycode:copy>
<a id="f-5eb82555b921-import_statement-4"></a>
### `import_statement`

<storycode:copy id="f-5eb82555b921-import_statement-4">
import { parseBrowserInputs, savedBrowserPredictionInputs, sealBrowserPrediction, runBrowserPrediction } from "./browserPrediction";
</storycode:copy>
<a id="f-5eb82555b921-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-5eb82555b921-expression_statement">
vi.mock("../editor/hickLang", () => ({ loadHickLang: vi.fn(async () => {}), hickLangArtifactHash: vi.fn(), browserPredictionTransition: vi.fn() }));
</storycode:copy>
<a id="f-5eb82555b921-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-5eb82555b921-expression_statement-2">
vi.mock("../debug/browser/transport", () => ({ browserRuntimeAssetHash: vi.fn(), createBrowserDebugger: vi.fn() }));
</storycode:copy>
<a id="f-5eb82555b921-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-5eb82555b921-expression_statement-3">
beforeEach(() => { vi.stubGlobal("crypto", webcrypto); Object.defineProperty(navigator, "locks", { configurable: true, value: { request: async (_key: string, callback: () => unknown) => callback() } }); });
</storycode:copy>
<a id="f-5eb82555b921-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-5eb82555b921-expression_statement-4">
afterEach(() => { vi.unstubAllGlobals(); vi.clearAllMocks(); localStorage.clear(); });
</storycode:copy>
<a id="f-5eb82555b921-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-5eb82555b921-expression_statement-5">
it("accepts bounded integer questions and refuses injected/non-finite or ambiguous inputs", () => {
  expect(parseBrowserInputs("[70,65]", "90")).toEqual({ attempts: [70, 65], budget: 90 });
  for (const [attempts, budget] of [["[1,\"globalThis.secret\"]", "90"], ["[0.5]", "90"], ["[1]", "0"], ["[1]", "Infinity"], [JSON.stringify(Array(9).fill(1)), "90"]]) expect(() => parseBrowserInputs(attempts, budget)).toThrow();
});
</storycode:copy>
<a id="f-5eb82555b921-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-5eb82555b921-expression_statement-6">
it("restores question inputs without restoring foreign predictions or private fields", () => {
  localStorage.setItem("hickory.browserPredictionInputs.v1", JSON.stringify({ attempts: [70, 65], budget: 90 }));
  expect(savedBrowserPredictionInputs()).toEqual({ attempts: [70, 65], budget: 90 });
  localStorage.setItem("hickory.browserPredictionInputs.v1", JSON.stringify({ attempts: [70, 65], budget: 90, prediction: "private-held-out-value" }));
  expect(JSON.stringify(savedBrowserPredictionInputs())).not.toContain("private-held-out-value");
  expect(savedBrowserPredictionInputs()).toEqual({ attempts: [80, 80], budget: 100 });
});
</storycode:copy>
<a id="f-5eb82555b921-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-5eb82555b921-expression_statement-7">
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
</storycode:copy>

### Assemble `apps/web/src/tour/browserPrediction.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/tour/browserPrediction.test.ts"><storycode:paste select="#f-5eb82555b921-ambient_declaration"/><storycode:paste select="#f-5eb82555b921-webcrypto"/><storycode:paste select="#f-5eb82555b921-import_statement"/><storycode:paste select="#f-5eb82555b921-import_statement-2"/><storycode:paste select="#f-5eb82555b921-import_statement-3"/><storycode:paste select="#f-5eb82555b921-import_statement-4"/><storycode:paste select="#f-5eb82555b921-expression_statement"/><storycode:paste select="#f-5eb82555b921-expression_statement-2"/><storycode:paste select="#f-5eb82555b921-expression_statement-3"/><storycode:paste select="#f-5eb82555b921-expression_statement-4"/><storycode:paste select="#f-5eb82555b921-expression_statement-5"/><storycode:paste select="#f-5eb82555b921-expression_statement-6"/><storycode:paste select="#f-5eb82555b921-expression_statement-7"/></storycode:file>

</storycode:doc>
