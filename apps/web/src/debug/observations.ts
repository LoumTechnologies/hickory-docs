import { readingProviderAnnotations, type ProviderAnnotation } from "./providerObservations";
/** Tool-produced evidence, never parsed from source text or display labels. */
export interface ObservationScope { session?: string | null; run: string; stop: string; frame: number | null; invocation: string | null }
export interface ObservationAnnotation {
  evidence: string; provider: string; payload_schema: string; location: string | null;
  scope: ObservationScope; payload: { kind: string; [key: string]: unknown };
}
export interface ObservationReading {
  provider_annotations?: ProviderAnnotation[];
  source_aliases?: string[];
  schema_version: number;
  source: { id: string; uri: string; revision: string; bytes: number[] };
  scope: ObservationScope;
  locations: { id: string; revision: string; line: number; location: { source: string; bytes: { start: number; end: number } | null; precision: string } }[];
  annotations: ObservationAnnotation[]; unsupported: string[];
}
export interface SelectedObservations { reading: ObservationReading; state: "captured" | "historical" | "changed"; evidence: string[] }
// Explicit expanded scope, independent of captured event volume or visible pixels.
const selected = new Map<symbol | string, SelectedObservations>();
export function selectObservations(reading: ObservationReading, expanded: boolean, historical: boolean, owner?: symbol) {
  const key = owner ?? reading.scope.run;
  if (!expanded) { selected.delete(key); return; }
  selected.set(key, { reading, state: historical ? "historical" : "captured", evidence: reading.annotations.map(a => a.evidence) });
}
export function selectedObservations(path: string, content: string): SelectedObservations[] {
  const unique = new Map([...selected.values()].map(s => [JSON.stringify([s.reading.source.id, s.reading.source.revision, s.reading.scope.run, s.reading.scope.stop, s.reading.scope.frame, s.evidence]), s]));
  const normalized = (uri: string) => uri.replace(/^hick:\/\/\//, "");
  return [...unique.values()].filter(s => [s.reading.source.uri, ...(s.reading.source_aliases ?? [])].some(uri => normalized(uri) === normalized(path))).map(s => ({ ...s,
    state: new TextDecoder().decode(new Uint8Array(s.reading.source.bytes)) === content ? s.state : "changed" }));
}

/** Same source excerpts, coordinates and evidence as the expanded view, without a byte-array dump. */
export function observationContext(path: string, content: string) {
  return selectedObservations(path, content).map(selected => {
    const { reading } = selected;
    return { state: selected.state, evidence: selected.evidence, reading: {
      schema_version: reading.schema_version,
      source: { id: reading.source.id, uri: reading.source.uri, revision: reading.source.revision },
      scope: reading.scope, annotations: reading.annotations, provider_annotations: readingProviderAnnotations(reading).filter(item => selected.evidence.includes(item.evidence)), unsupported: reading.unsupported,
      excerpts: reading.locations.map(location => ({ location, text: location.location.bytes ?
        new TextDecoder().decode(new Uint8Array(reading.source.bytes.slice(location.location.bytes.start, location.location.bytes.end))) : null })),
    } };
  });
}

export function observationSourceChanged(reading: ObservationReading | null | undefined, source: string | undefined): boolean {
  return !!reading && source !== undefined && new TextDecoder().decode(new Uint8Array(reading.source.bytes)) !== source;
}
