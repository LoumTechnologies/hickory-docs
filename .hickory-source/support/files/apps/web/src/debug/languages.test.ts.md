<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/languages.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `cfac466a01495a9257c6aaae795eab273ed3f8599886591099e51ef8172c108d`.

<book:entry path="apps/web/src/debug/languages.test.ts">
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
</book:entry>

</book:doc>
