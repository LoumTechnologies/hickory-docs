<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/tabStyle.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `6bccbb80c57a30ba4679b2bef1ab8d3ec63e784d4b8483db1fee4d614b769708`.

<book:entry path="apps/web/src/lib/tabStyle.test.ts">
import { describe, expect, it } from "vitest";

import { loadTabStyle, saveTabStyle, TAB_STYLE_KEY } from "./tabStyle";

describe("tab style persistence", () => {
  it("defaults to top with nothing stored", () => {
    localStorage.removeItem(TAB_STYLE_KEY);
    expect(loadTabStyle()).toBe("top");
  });

  it("round-trips a saved choice through localStorage", () => {
    saveTabStyle("side");
    expect(localStorage.getItem(TAB_STYLE_KEY)).toBe("side");
    expect(loadTabStyle()).toBe("side");
    saveTabStyle("top");
    expect(loadTabStyle()).toBe("top");
  });

  it("falls back to top on a value nothing ever wrote", () => {
    localStorage.setItem(TAB_STYLE_KEY, "diagonal");
    expect(loadTabStyle()).toBe("top");
  });

  it("works without any storage at all", () => {
    expect(loadTabStyle(null)).toBe("top");
    expect(() => saveTabStyle("side", null)).not.toThrow();
  });
});
</book:entry>

</book:doc>
