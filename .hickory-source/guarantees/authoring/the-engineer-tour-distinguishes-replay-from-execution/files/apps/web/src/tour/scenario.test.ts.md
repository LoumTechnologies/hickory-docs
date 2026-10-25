<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/tour/scenario.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The engineer tour distinguishes replay from execution](../../../../../contract.md)

Captured SHA-256: `e4a97132a5bc6aba0aae1a68e5028c74ebae2333df1761c95dc745a68578b71d`.

<book:entry path="apps/web/src/tour/scenario.test.ts">
import { expect, it } from "vitest";
import { scenarioSource, reconciledCoachStep } from "./scenario";
declare const require: (id: string) => { readFileSync(p: string, encoding: string): string };
declare const process: { cwd(): string };
// Guarantee: docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md
it("downloads the specification exercised by the native engine", () => {
  const fixture = require("node:fs").readFileSync(`${process.cwd()}/../../examples/ai-engineer-deadline.md`, "utf8");
  expect(scenarioSource(true)).toBe(fixture);
});
it("reconciles saved guidance to the first missing actual evidence", () => {
  expect(reconciledCoachStep(4, [true, false, true, true, false, false])).toBe(1);
  expect(reconciledCoachStep(1, [true, true, true, true, true, true])).toBe(1);
  expect(reconciledCoachStep(99, [true, true])).toBe(0);
});
</book:entry>

</book:doc>
