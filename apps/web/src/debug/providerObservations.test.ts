// Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
import { expect, it } from "vitest";
import { providerObservationContext, selectProviderReading, type ProviderReading } from "./providerObservations";
it("selects historical review identity without fabricating runtime scope", () => {
  const owner = Symbol();
  const reading: ProviderReading = { schema_version: 1, source: { id: "doc", uri: "hick:///x.py", revision: "head", bytes: [120] }, unsupported: [],
    annotations: [{ evidence: "thread-1", provider: "github", provider_version: "unknown", payload_schema: "review.v1",
      origin: { kind: "review", repository: "org/repo", pull_request: 2, thread: "t1", base_revision: "base", head_revision: "head", side: "RIGHT", author: "reviewer", resolved: false, outdated: true },
      location: null, source_precision: "unknown", measurement_semantics: null, uncertainty: "outdated", payload: { body: "Check this" } }] };
  expect(providerObservationContext("x.py", "x")).toEqual([]);
  selectProviderReading(owner, reading, true, true);
  const context = providerObservationContext("x.py", "changed")[0];
  expect(context.state).toBe("changed"); expect(context.evidence).toEqual(["thread-1"]);
  expect(context.reading.provider_annotations[0].origin).toEqual(reading.annotations[0].origin);
  expect(context.reading).not.toHaveProperty("scope");
  selectProviderReading(owner, reading, false, false);
});
