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
