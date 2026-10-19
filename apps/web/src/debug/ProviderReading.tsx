import { useEffect, useRef, useState } from "react";
import { selectProviderReading, type ProviderReading as Reading } from "./providerObservations";

/** General provider view; runtime and external reviews retain their own origin. */
export function ProviderReading({ reading, historical = false, changed = false }: { reading: Reading; historical?: boolean; changed?: boolean }) {
  const owner = useRef(Symbol("provider-reading"));
  const [expanded, setExpanded] = useState(false);
  const [pinned, setPinned] = useState(false);
  useEffect(() => {
    selectProviderReading(owner.current, reading, expanded || pinned, historical || changed);
    return () => selectProviderReading(owner.current, reading, false, historical);
  }, [reading, expanded, pinned, historical, changed]);
  const text = reading.source.bytes && new TextDecoder().decode(new Uint8Array(reading.source.bytes));
  return <section aria-label="Provider source observations">
    <label><input type="checkbox" checked={pinned} onChange={event => setPinned(event.target.checked)} />Pin provider evidence in next context</label>
    <details onToggle={event => setExpanded(event.currentTarget.open)}>
      <summary>Provider observations · {changed ? "source changed" : historical ? "historical" : "captured"}</summary>
      <p>{reading.source.uri} · revision {reading.source.revision ?? "unavailable"}. Expanded/pinned scope selects context; it is not a delivery receipt or action permission.</p>
      {reading.annotations.map(annotation => <article key={annotation.evidence} data-evidence={annotation.evidence}>
        {annotation.location && text && <code>[{annotation.location.id}] {text.split("\n")[annotation.location.line]}</code>}
        <p>{annotation.provider} · {annotation.evidence} · source precision {annotation.source_precision}</p>
        <pre>{JSON.stringify(annotation.origin, null, 2)}</pre><pre>{JSON.stringify(annotation.payload, null, 2)}</pre>
        <p>Measurement: {annotation.measurement_semantics ?? "not supplied"} · uncertainty {annotation.uncertainty ?? "not supplied"}</p>
      </article>)}
      <p>Unsupported: {reading.unsupported.join(", ") || "none declared"}</p>
    </details>
  </section>;
}
