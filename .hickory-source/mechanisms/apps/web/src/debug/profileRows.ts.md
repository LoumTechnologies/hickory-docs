<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/profileRows.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1734df7f247e"></a>
## `apps/web/src/debug/profileRows.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-1734df7f247e-profileRows"></a>
### `profileRows`

<storycode:copy id="f-1734df7f247e-profileRows">
import type { RuntimeEvent } from "./runtimeObservations";
/** cProfile values are function aggregates; inclusive costs are never added. */
export function profileRows(events: RuntimeEvent[]) {
  const rows = new Map<string, { function: NonNullable<Extract<RuntimeEvent["observation"], { kind: "profile" }>["function"]>; self: number | null; cumulative: number | null; events: RuntimeEvent[] }>();
  for (const event of events) {
    const observation = event.observation;
    if (observation.kind !== "profile" || !observation.function) continue;
    const key = JSON.stringify([observation.function.file, observation.function.line, observation.function.name]);
    const row = rows.get(key) ?? { function: observation.function, self: null, cumulative: null, events: [] };
    if (observation.metric === "self_seconds") row.self = observation.value;
    if (observation.metric === "cumulative_seconds") row.cumulative = observation.value;
    row.events.push(event); rows.set(key, row);
  }
  return [...rows.values()].sort((a,b) => (b.self ?? 0) - (a.self ?? 0));
}
</storycode:copy>

### Assemble `apps/web/src/debug/profileRows.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/profileRows.ts"><storycode:paste select="#f-1734df7f247e-profileRows"/></storycode:file>

</storycode:doc>
