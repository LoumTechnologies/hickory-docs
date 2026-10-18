import { expect, it } from "vitest";
import { scenarioSource } from "./scenario";
declare const require: (id: string) => { readFileSync(p: string, encoding: string): string };
declare const process: { cwd(): string };
// Guarantee: docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md
it("downloads the specification exercised by the native engine", () => {
  const fixture = require("node:fs").readFileSync(`${process.cwd()}/../../examples/ai-engineer-deadline.md`, "utf8");
  expect(scenarioSource(true)).toBe(fixture);
});
