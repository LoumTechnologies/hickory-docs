// Guarantee: docs/guarantees/execution/python-profile-keeps-function-semantics.md
import { expect, it } from "vitest";
import captured from "./fixtures/python-profile.json";
import { profileRows } from "./profileRows";
import { runtimeReading } from "./runtimeHistory";
import type { RuntimeRun } from "./runtimeObservations";
it("parses actual cProfile function aggregates without summing inclusive times", () => {
  const run = captured as RuntimeRun;
  const rows = profileRows(run.events), subtotal = rows.find(row => row.function.name === "subtotal")!;
  expect(subtotal.function.total_calls).toBe(500); expect(subtotal.function.primitive_calls).toBe(500);
  const recursive = rows.find(row => row.function.name === "recursive")!;
  expect(recursive.function.total_calls).toBe(10); expect(recursive.function.primitive_calls).toBe(1);
  const measured = subtotal.events.find(event => event.observation.kind === "profile" && event.observation.metric === "self_seconds")!.observation;
  expect(measured.kind).toBe("profile");
  if (measured.kind === "profile") expect(subtotal.self).toBe(measured.value);
  expect(subtotal.events).toHaveLength(2);
  const reading = runtimeReading(run);
  expect(reading.provider_annotations!.every(a => a.provider_version === "3.9.6")).toBe(true);
  expect(reading.provider_annotations!.every(a => a.origin.kind === "runtime" && a.origin.stop === null)).toBe(true);
  expect(rows.some(row => row.events.every(event => event.emission === null))).toBe(true);
});
