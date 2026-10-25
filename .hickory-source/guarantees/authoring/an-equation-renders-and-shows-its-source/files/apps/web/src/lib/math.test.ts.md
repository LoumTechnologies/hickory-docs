<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/math.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [An Equation Renders And Shows Its Source](../../../../../contract.md)

Captured SHA-256: `2d06f1e736db8b3d3856f8e5a7f885004fade8594e2e93bc0c3c7452ff5f05e9`.

<book:entry path="apps/web/src/lib/math.test.ts">
import { describe, expect, it } from "vitest";
import { mathSpans } from "./math";

const found = (src: string) =>
  mathSpans(src).map((s) => ({ text: src.slice(s.from, s.to), display: s.display }));

describe("finding LaTeX in prose", () => {
  it("finds display maths on its own lines", () => {
    const src = "before\n$$\na^2 + b^2 = c^2\n$$\nafter\n";
    const spans = mathSpans(src);
    expect(spans).toHaveLength(1);
    expect(spans[0].display).toBe(true);
    expect(spans[0].source.trim()).toBe("a^2 + b^2 = c^2");
  });

  it("finds inline maths inside a sentence", () => {
    expect(found("the value $x^2$ is squared")).toEqual([
      { text: "$x^2$", display: false },
    ]);
  });

  it("reads a `$` pair inside display maths as part of the display block", () => {
    const spans = mathSpans("$$ a $ b $$");
    expect(spans).toHaveLength(1);
    expect(spans[0].display).toBe(true);
  });
});

describe("not turning money and shell variables into equations", () => {
  it("leaves a pair of prices alone", () => {
    // "$5 and $6" is the classic false positive: two dollars with tight
    // content between them looks exactly like inline maths.
    expect(found("it cost $5 and $6 in total")).toEqual([]);
  });

  it("leaves `$ 5` alone, because maths never opens on a space", () => {
    expect(found("costs $ 5 today $")).toEqual([]);
  });

  it("leaves a trailing space before the closer alone", () => {
    expect(found("$x + y $")).toEqual([]);
  });

  it("does not let an unclosed `$` swallow the rest of the document", () => {
    expect(found("a lone $ sign\nand a later $ one\n")).toEqual([]);
  });

  it("respects an escaped delimiter", () => {
    expect(found("costs \\$5 and \\$6")).toEqual([]);
  });

  it("skips maths inside a verbatim range", () => {
    // `$PATH` in a shell cell is a variable, and the cell's payload is
    // verbatim by definition.
    const src = "echo $PATH$HOME done";
    expect(mathSpans(src, [[0, src.length]])).toEqual([]);
  });
});
</book:entry>

</book:doc>
