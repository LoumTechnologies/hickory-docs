<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/terminalPriority.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `a0ff0974efd3674dcde380760789a465a201221c2f416dfd73929def8ccc3b4d`.

<book:entry path="apps/web/src/lib/terminalPriority.test.ts">
import { describe, expect, it } from "vitest";

import { pickTerminal } from "./terminalPriority";

describe("pickTerminal", () => {
  it("prefers visible pane text over everything", () => {
    expect(pickTerminal({ text: true, tab: true, tree: true, port: true })).toBe("text");
  });

  it("prefers the file's tab when its text is not on screen", () => {
    expect(pickTerminal({ tab: true, tree: true, port: true })).toBe("tab");
  });

  it("prefers a visible tree row over the divider port", () => {
    expect(pickTerminal({ tree: true, port: true })).toBe("tree");
  });

  it("falls through to the port when the row is not visible", () => {
    // A collapsed directory means the row is simply not rendered: the caller
    // reports tree: false, and the connection lands on the port.
    expect(pickTerminal({ tree: false, port: true })).toBe("port");
  });

  it("answers null when nothing on screen can host the connection", () => {
    expect(pickTerminal({})).toBeNull();
    expect(pickTerminal({ text: false, tab: false, tree: false, port: false })).toBeNull();
  });
});
</book:entry>

</book:doc>
