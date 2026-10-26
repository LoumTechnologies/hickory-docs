<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/languages.test.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b4a225d4fd39"></a>
## `apps/web/src/debug/languages.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b4a225d4fd39-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b4a225d4fd39-expression_statement">
import { describe, expect, it } from "vitest";
import { LANGUAGE_CAPABILITIES } from "../editor/generated/languages";
import { isDebuggable } from "./languages";

describe("plugin debug capabilities", () => {
  it("offers debugging for every native debug provider", () => {
    for (const [language, capabilities] of Object.entries(LANGUAGE_CAPABILITIES)) {
      expect(isDebuggable(language), language).toBe(capabilities.includes("Dap"));
    }
  });
  it("keeps React components and unknown files out of standalone debugging", () => {
    for (const language of ["typescriptreact", "javascriptreact", "unknown", null, undefined]) {
      expect(isDebuggable(language)).toBe(false);
    }
  });
});
</storycode:copy>

### Assemble `apps/web/src/debug/languages.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/languages.test.ts"><storycode:paste select="#f-b4a225d4fd39-expression_statement"/></storycode:file>

</storycode:doc>
