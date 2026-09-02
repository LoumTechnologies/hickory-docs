import { describe, expect, it } from "vitest";

import { attrValue } from "./attrSelector";

describe("an attribute-selector value", () => {
  it("quotes an ordinary path", () => {
    expect(attrValue("service/Program.cs")).toBe('"service/Program.cs"');
  });

  it("escapes the two characters a quoted value can hold", () => {
    expect(attrValue('a"b')).toBe('"a\\"b"');
    expect(attrValue("a\\b")).toBe('"a\\\\b"');
    // Backslash first, or the quote's own escape gets escaped again.
    expect(attrValue('a\\"b')).toBe('"a\\\\\\"b"');
  });

  it("leaves alone everything an identifier escape would mangle", () => {
    // The spaces, dots and colons a path really carries stay literal — this
    // is a quoted string, not an identifier.
    expect(attrValue("my notes/2026-09-02 sync.hick")).toBe(
      '"my notes/2026-09-02 sync.hick"',
    );
  });

  it("finds the element it names, including one with a quote in its key", () => {
    const host = document.createElement("div");
    const plain = document.createElement("i");
    plain.dataset.card = "plain";
    const quoted = document.createElement("i");
    quoted.dataset.card = 'say "hi"';
    host.append(plain, quoted);
    document.body.appendChild(host);
    expect(host.querySelector(`[data-card=${attrValue("plain")}]`)).toBe(plain);
    expect(host.querySelector(`[data-card=${attrValue('say "hi"')}]`)).toBe(quoted);
    expect(host.querySelector(`[data-card=${attrValue("absent")}]`)).toBeNull();
    host.remove();
  });
});
