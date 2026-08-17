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
