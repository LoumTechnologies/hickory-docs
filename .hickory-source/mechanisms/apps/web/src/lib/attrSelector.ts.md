<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/attrSelector.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4f80578db388"></a>
## `apps/web/src/lib/attrSelector.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-4f80578db388-attrValue"></a>
### `attrValue`

One attribute-selector value, quoted and escaped.
Three places look an element up by a value that can hold anything a person
typed — a file path, a block id, a card key — and all three built the
selector by hand. Two got it right and one reached for `CSS.escape`, which
is the wrong tool twice over: it escapes an IDENTIFIER (`a\ b`), not a
quoted string, so it only works inside quotes by accident; and jsdom ships
no `CSS` object at all, so every card rail mounted in a test threw from
inside a measure callback — an unhandled error beside a green suite, which
meant the rail's own positioning was never once exercised there.
Inside a quoted attribute value exactly two characters need escaping, and
the backslash must go first or it re-escapes the quote's own backslash.
`value`, ready to drop into `[attr=…]` — quotes included.

<storycode:copy id="f-4f80578db388-attrValue">
// One attribute-selector value, quoted and escaped.
//
// Three places look an element up by a value that can hold anything a person
// typed — a file path, a block id, a card key — and all three built the
// selector by hand. Two got it right and one reached for `CSS.escape`, which
// is the wrong tool twice over: it escapes an IDENTIFIER (`a\ b`), not a
// quoted string, so it only works inside quotes by accident; and jsdom ships
// no `CSS` object at all, so every card rail mounted in a test threw from
// inside a measure callback — an unhandled error beside a green suite, which
// meant the rail's own positioning was never once exercised there.
//
// Inside a quoted attribute value exactly two characters need escaping, and
// the backslash must go first or it re-escapes the quote's own backslash.

/** `value`, ready to drop into `[attr=…]` — quotes included. */
export function attrValue(value: string): string {
  return `"${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}
</storycode:copy>

### Assemble `apps/web/src/lib/attrSelector.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/attrSelector.ts"><storycode:paste select="#f-4f80578db388-attrValue"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-54a27e3f7b4e"></a>
## `apps/web/src/lib/attrSelector.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-54a27e3f7b4e-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-54a27e3f7b4e-expression_statement">
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
</storycode:copy>

### Assemble `apps/web/src/lib/attrSelector.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/attrSelector.test.ts"><storycode:paste select="#f-54a27e3f7b4e-expression_statement"/></storycode:file>


</storycode:doc>
