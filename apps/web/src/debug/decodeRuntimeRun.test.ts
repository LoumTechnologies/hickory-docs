// Guarantee: docs/guarantees/execution/python-profile-keeps-function-semantics.md
import { expect, it } from "vitest";
import record from "./fixtures/python-profile.json";
import { decodeRuntimeRun } from "./decodeRuntimeRun";
it("imports the actual selected native profile as captured data without executing it", async () => {
  const run = await decodeRuntimeRun(JSON.stringify(record));
  expect(run.id).toBe(record.id);
  expect(run.events).toEqual(record.events);
  expect(new TextDecoder().decode(new Uint8Array(run.sources[0].bytes))).toContain("subtotal");
});
it("refuses altered source bytes and malformed profile state rather than manufacturing a valid capture", async () => {
  const changed = structuredClone(record); changed.sources[0].bytes[0] ^= 1;
  await expect(decodeRuntimeRun(JSON.stringify(changed))).rejects.toThrow("source hash");
  const malformed = structuredClone(record); malformed.events[0].observation.value = "not a measurement" as never;
  await expect(decodeRuntimeRun(JSON.stringify(malformed))).rejects.toThrow("Invalid recorded runtime evidence");
  await expect(decodeRuntimeRun(JSON.stringify({ ...record, extra_actual: "unrecognized" }))).rejects.toThrow();
});
