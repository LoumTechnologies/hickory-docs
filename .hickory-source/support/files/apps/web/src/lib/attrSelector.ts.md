<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/attrSelector.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `decb689be39a6f33ad81d85ac0df8fa910d40caf91b738fe434e0fb6dbf72479`.

<book:entry path="apps/web/src/lib/attrSelector.ts">
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
</book:entry>

</book:doc>
