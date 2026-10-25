<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/profileRows.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `7b6b2ed304f906cf434df9a661a5ee59a6da8e1fb14c67fe3251ebebd955cd31`.

<book:entry path="apps/web/src/debug/profileRows.ts">
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
</book:entry>

</book:doc>
