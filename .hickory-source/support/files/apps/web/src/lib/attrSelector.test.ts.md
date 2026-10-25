<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/attrSelector.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `cc8b71df058f8537bf69a572d3145a22979e8f317b27744a61851b9fee5eb7a9`.

<book:entry path="apps/web/src/lib/attrSelector.test.ts">
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
    expect(attrValue("my notes/2026-09-02 sync.md")).toBe(
      '"my notes/2026-09-02 sync.md"',
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
</book:entry>

</book:doc>
