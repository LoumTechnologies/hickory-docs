<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/wordMotion.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Word Navigation Is The Person's Choice](../../../../../contract.md)

Captured SHA-256: `64cc97a15b88ff42c04b3f8ce813c25a4b1b0044dce556ff70ca7a55ea19aa72`.

<book:entry path="apps/web/src/lib/wordMotion.test.ts">
import { describe, expect, it } from "vitest";

import { loadWordMotion, saveWordMotion, WORD_MOTION_KEY } from "./wordMotion";

describe("word motion persistence", () => {
  it("defaults to whole words with nothing stored", () => {
    localStorage.removeItem(WORD_MOTION_KEY);
    expect(loadWordMotion()).toBe("word");
  });

  it("round-trips a saved choice through localStorage", () => {
    saveWordMotion("subword");
    expect(localStorage.getItem(WORD_MOTION_KEY)).toBe("subword");
    expect(loadWordMotion()).toBe("subword");
    saveWordMotion("word");
    expect(loadWordMotion()).toBe("word");
  });

  it("falls back to whole words on a value nothing ever wrote", () => {
    localStorage.setItem(WORD_MOTION_KEY, "camelhumps");
    expect(loadWordMotion()).toBe("word");
  });

  it("works without any storage at all", () => {
    expect(loadWordMotion(null)).toBe("word");
    expect(() => saveWordMotion("subword", null)).not.toThrow();
  });
});
</book:entry>

</book:doc>
