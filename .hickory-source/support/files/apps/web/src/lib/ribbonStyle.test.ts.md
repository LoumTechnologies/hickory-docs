<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/ribbonStyle.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `bfb402cdeaad032f75dfe2c7dd94a2a0c959042216929dbe103a72f2cc7e49a8`.

<book:entry path="apps/web/src/lib/ribbonStyle.test.ts">
import { describe, expect, it } from "vitest";

import { loadRibbonStyle, RIBBON_STYLE_KEY, saveRibbonStyle } from "./ribbonStyle";

describe("ribbon style persistence", () => {
  it("defaults to braces with nothing stored", () => {
    localStorage.removeItem(RIBBON_STYLE_KEY);
    expect(loadRibbonStyle()).toBe("braces");
  });

  it("round-trips a saved choice through localStorage", () => {
    saveRibbonStyle("braces");
    expect(localStorage.getItem(RIBBON_STYLE_KEY)).toBe("braces");
    expect(loadRibbonStyle()).toBe("braces");
    // "bands" stays a valid persisted value — only the default changed.
    saveRibbonStyle("bands");
    expect(loadRibbonStyle()).toBe("bands");
  });

  it("falls back to braces on a value nothing ever wrote", () => {
    localStorage.setItem(RIBBON_STYLE_KEY, "sparkles");
    expect(loadRibbonStyle()).toBe("braces");
  });

  it("works without any storage at all", () => {
    expect(loadRibbonStyle(null)).toBe("braces");
    expect(() => saveRibbonStyle("braces", null)).not.toThrow();
  });
});
</book:entry>

</book:doc>
