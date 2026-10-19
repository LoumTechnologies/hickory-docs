import { useEffect, useRef, useState } from "react";
import { selectObservations, type ObservationReading as Reading } from "./observations";

/** Both the human view and selected agent context consume these exact evidence IDs. */
export function ObservationReading({ reading, historical, changed = false }: { reading: Reading; historical: boolean; changed?: boolean }) {
  const owner = useRef(Symbol("observation-reading"));
  const [expanded, setExpanded] = useState(false);
  const [pinned, setPinned] = useState(false);
  useEffect(() => {
    selectObservations(reading, expanded || pinned, historical || changed, owner.current);
    return () => selectObservations(reading, false, historical, owner.current);
  }, [reading, expanded, pinned, historical, changed]);
  const source = new TextDecoder().decode(new Uint8Array(reading.source.bytes));
  return <><label><input type="checkbox" checked={pinned} onChange={event => setPinned(event.target.checked)} />Pin observations · {reading.scope.run} · {reading.scope.stop}</label><details className="debug-observations" onToggle={event => setExpanded(event.currentTarget.open)}>
    <summary>Source observations · {changed ? "source changed" : historical ? "historical capture" : "captured stop"}</summary>
    <p>Revision {reading.source.revision} · session {reading.scope.session ?? "unknown"} · run {reading.scope.run} · stop {reading.scope.stop}. Expanded evidence enters agent context.</p>
    {reading.locations.map(location => <div key={location.id} data-location={location.id}>
      <code>[{location.id}] {source.split("\n")[location.line]}</code>
      {reading.annotations.filter(a => a.location === location.id).map(a => <div key={a.evidence} data-evidence={a.evidence}>
        <strong>{a.payload.kind === "paused" ? "PAUSED HERE" : a.payload.kind.replaceAll("_", " ")}</strong>
        <code> {JSON.stringify(a.payload)}</code><small> · {a.evidence} · frame {a.scope.frame ?? "unknown"} · invocation {a.scope.invocation ?? "unknown"}</small>
      </div>)}
    </div>)}
    {reading.annotations.filter(a => a.location === null).map(a => <div key={a.evidence}>Unmapped · {a.evidence} · {JSON.stringify(a.payload)}</div>)}
    <p>Unsupported: {reading.unsupported.join(", ")}</p>
  </details></>;
}
