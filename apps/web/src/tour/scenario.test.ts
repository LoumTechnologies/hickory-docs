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
