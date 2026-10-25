<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/windowTitle.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `8c1270eff77f55916e46385d2cc5f92e671fde26f5849ef8dd6d16fd233233bc`.

<book:entry path="apps/web/src/lib/windowTitle.test.ts">
import { describe, expect, it } from "vitest";

import { windowTitle } from "./windowTitle";

describe("window title precedence", () => {
  it("uses the custom override verbatim when one is set", () => {
    expect(
      windowTitle({ custom: "My Lab Notebook", folder: "notes", file: "a.md" }),
    ).toBe("My Lab Notebook");
  });

  it("falls back to the project folder name when there is no override", () => {
    expect(windowTitle({ custom: null, folder: "notes", file: "a.md" })).toBe(
      "notes — Hickory Docs",
    );
  });

  it("reduces a folder path to its last segment", () => {
    expect(windowTitle({ folder: "/home/mock/notebook", file: "a.md" })).toBe(
      "notebook — Hickory Docs",
    );
    expect(windowTitle({ folder: "C:\\Users\\me\\notes" })).toBe("notes — Hickory Docs");
  });

  it("falls back to the focused file name when there is no folder either", () => {
    expect(windowTitle({ custom: "", folder: null, file: "weave-demo.md" })).toBe(
      "weave-demo.md — Hickory Docs",
    );
  });

  it("treats a blank override as unset, not as an empty title", () => {
    expect(windowTitle({ custom: "   ", folder: "notes" })).toBe("notes — Hickory Docs");
  });

  it("is just the app when nothing at all is known", () => {
    expect(windowTitle({})).toBe("Hickory Docs");
  });
});
</book:entry>

</book:doc>
