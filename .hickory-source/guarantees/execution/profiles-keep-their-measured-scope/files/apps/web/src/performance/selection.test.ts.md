<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/performance/selection.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Profiles keep their measured scope](../../../../../contract.md)

Captured SHA-256: `593c7ac9ff6d8c5fd7bf3e5afec04df906a5409d51109e22d2a10e9333de30c3`.

<book:entry path="apps/web/src/performance/selection.test.ts">
import { expect, it } from "vitest";
import { profileReading } from "./selection";
import type { PerformanceRun } from "./types";
// Guarantee: docs/guarantees/execution/profiles-keep-their-measured-scope.md
it("anchors function aggregates in literate source and retains scope, units and unknown calls", () => {
  const source = "# App\nfunction hot() {}\n";
  const run = { id: "profile", source_state: "current", sources: { "app.md": source }, identity: { collector: "V8", version: "1", source_hashes: { "app.md": "hash" } }, labels: ["manual"], mappings: { "app.ts": { 1: { path: "app.md", line: 2 } } }, report: { limitations: ["sampling"], threads: [{ name: "main", unit: "seconds", aggregation: "sampled", measured_weight: 0.1, interval_start: 0, interval_end: 0.1, rows: [{ frame: { name: "hot", file: "app.ts", line: 1 }, self_weight: 0.02, inclusive_weight: 0.1, samples: 20, total_calls: null, primitive_calls: null }] }] } } as unknown as PerformanceRun;
  const reading = profileReading(run, 0, "app.md")!;
  expect(reading.locations[0].line).toBe(1);
  expect(reading.annotations[0].payload).toMatchObject({ kind: "profile", unit: "seconds", row: { samples: 20, total_calls: null }, labels: ["manual"] });
  expect(new TextDecoder().decode(new Uint8Array(reading.source.bytes))).toBe(source);
  expect(profileReading(run, 0, "other.md")).toBeNull();
});
</book:entry>

</book:doc>
