<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/ObservationReading.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Observations read beside their captured source](../../../../../contract.md)

Captured SHA-256: `e1c290782a05cf71f286672580642aea065bc68a606febcd7407efcfdd9ba740`.

<book:entry path="apps/web/src/debug/ObservationReading.tsx">
import { useEffect, useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { readingProviderAnnotations } from "./providerObservations";
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
  const parent = `source-observations:${useId()}`;
  const revision = useUiRevision(() => [reading, historical, changed, expanded, pinned]);
  useUiCommands([
    { key: `${parent}:open`, command: { parent, subject: parent, label: "Set source observation visibility", parameters: { open: { type: "boolean" } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true, value: { open: expanded } }), execute: async args => { setExpanded(args.open as boolean); return { status: "completed", value: { open: args.open, context_selection_requested: true } }; } } },
    { key: `${parent}:pin`, command: { parent, subject: parent, label: "Set source observations pinned", parameters: { pinned: { type: "boolean" } }, prerequisites: ["Pinned evidence enters explicitly selected context; it remains captured/historical evidence"],
      read: () => ({ revision: revision(), enabled: true, value: { pinned } }), execute: async args => { setPinned(args.pinned as boolean); return { status: "completed", value: { pinned: args.pinned, context_selection_requested: true, provider_delivery_asserted: false } }; } } },
  ]);
  useUiReading("source observation disclosure", { expanded, pinned, scope: reading.scope, source_revision: reading.source.revision, historical, changed });
  return <><label><input type="checkbox" checked={pinned} onChange={event => setPinned(event.target.checked)} />Pin observations · {reading.scope.run} · {reading.scope.stop}</label><details data-observation-kind={reading.scope.stop === "runtime" ? "runtime" : "debugger"} className="debug-observations" open={expanded} onToggle={event => setExpanded(event.currentTarget.open)}>
    <summary>Source observations · {changed ? "source changed" : historical ? "historical capture" : reading.scope.stop === "runtime" ? "captured runtime" : "captured stop"}</summary>
    {expanded && <ObservationDetails reading={reading} historical={historical} changed={changed} />}
  </details></>;
}
function ObservationDetails({ reading, historical, changed }: { reading: Reading; historical: boolean; changed: boolean }) {
  const source = new TextDecoder().decode(new Uint8Array(reading.source.bytes));
  const providers = readingProviderAnnotations(reading);
  useUiReading("expanded source observations", {
    source_revision: reading.source.revision, scope: reading.scope, historical, changed,
    locations: reading.locations.map(location => ({ id: location.id, line: location.line, text: source.split("\n")[location.line],
      annotations: reading.annotations.filter(annotation => annotation.location === location.id) })),
    unmapped: reading.annotations.filter(annotation => annotation.location === null),
    providers: providers.map(annotation => ({ evidence: annotation.evidence, source_precision: annotation.source_precision,
      measurement_semantics: annotation.measurement_semantics ?? "not supplied", uncertainty: annotation.uncertainty ?? "not supplied" })),
    unsupported: reading.unsupported, scope_note: "Rendered captured evidence; not a current edit baseline, independent verification or proof of provider delivery",
  });
  return <>
    <p>Revision {reading.source.revision} · session {reading.scope.session ?? "unknown"} · run {reading.scope.run} · {reading.scope.stop === "runtime" ? "runtime observation batch; no debugger stop" : `stop ${reading.scope.stop}`}. Expanded evidence enters agent context.</p>
    {reading.locations.map(location => <div key={location.id} data-location={location.id}>
      <code>[{location.id}] {source.split("\n")[location.line]}</code>
      {reading.annotations.filter(a => a.location === location.id).map(a => <div key={a.evidence} data-evidence={a.evidence}>
        <strong>{a.payload.kind === "paused" ? "PAUSED HERE" : a.payload.kind.replaceAll("_", " ")}</strong>
        <code> {JSON.stringify(a.payload)}</code><small> · {a.evidence} · frame {a.scope.frame ?? "unknown"} · invocation {a.scope.invocation ?? "unknown"}</small>
      </div>)}
    </div>)}
    {reading.annotations.filter(a => a.location === null).map(a => <div key={a.evidence}>Unmapped · {a.evidence} · {JSON.stringify(a.payload)}</div>)}
    {providers.map(annotation => <small key={`provider-${annotation.evidence}`} data-provider-evidence={annotation.evidence}>
      {annotation.evidence} · source {annotation.source_precision} · measurement {annotation.measurement_semantics ?? "not supplied"} · uncertainty {annotation.uncertainty ?? "not supplied"}
    </small>)}
    <p>Unsupported: {reading.unsupported.join(", ")}</p>
  </>;
}
</book:entry>

</book:doc>
