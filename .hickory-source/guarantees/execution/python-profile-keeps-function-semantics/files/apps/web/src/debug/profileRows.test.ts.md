<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/profileRows.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Python profiles keep function scope and measurement semantics](../../../../../contract.md)

Captured SHA-256: `4159a2506b66121f5d4b414e0ae66ac811add0e97df473f4cfaa445b89bc54de`.

<book:entry path="apps/web/src/debug/profileRows.test.ts">
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
</book:entry>

</book:doc>
