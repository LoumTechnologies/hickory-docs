/** Matches hick_observe::reading::ProviderAnnotation, independently of debugger scope. */
export type EvidenceOrigin =
  | { kind: "runtime"; session: string | null; run: string; stop: string | null; frame: number | null; invocation: string | null }
  | { kind: "review"; repository: string; pull_request: number; thread: string; base_revision: string;
      head_revision: string; side: string; author: string; resolved: boolean; outdated: boolean };
export interface ProviderAnnotation {
  evidence: string; provider: string; provider_version: string; payload_schema: string;
  origin: EvidenceOrigin;
  location: { id: string; revision: string; line: number; location: { source: string; block?: string | null;
    bytes: { start: number; end: number } | null; precision: string } } | null;
  source_precision: string; measurement_semantics: string | null; uncertainty: string | null; payload: unknown;
}
export interface ProviderReading {
  schema_version: 1;
  source: { id: string; uri: string; revision: string | null; bytes: number[] | null };
  annotations: ProviderAnnotation[]; unsupported: string[];
}
const selected = new Map<symbol, { reading: ProviderReading; historical: boolean }>();
export function selectProviderReading(owner: symbol, reading: ProviderReading, include: boolean, historical: boolean) {
  if (include) selected.set(owner, { reading, historical }); else selected.delete(owner);
}
export function providerObservationContext(path: string, content: string) {
  const normalized = (uri: string) => uri.replace(/^hick:\/\/\//, "");
  const unique = new Map([...selected.values()].map(item => [JSON.stringify([item.reading.source.uri, item.reading.source.revision, item.reading.annotations.map(a => a.evidence)]), item]));
  return [...unique.values()].filter(item => normalized(item.reading.source.uri) === normalized(path)).map(({ reading, historical }) => ({
    state: reading.source.bytes === null ? "unavailable" : new TextDecoder().decode(new Uint8Array(reading.source.bytes)) !== content ? "changed" : historical ? "historical" : "captured",
    evidence: reading.annotations.map(item => item.evidence), reading: { schema_version: reading.schema_version,
      source: { id: reading.source.id, uri: reading.source.uri, revision: reading.source.revision },
      provider_annotations: reading.annotations, unsupported: reading.unsupported,
      excerpts: reading.annotations.flatMap(annotation => annotation.location ? [{ location: annotation.location,
        text: reading.source.bytes && annotation.location.location.bytes ? new TextDecoder().decode(new Uint8Array(reading.source.bytes.slice(annotation.location.location.bytes.start, annotation.location.location.bytes.end))) : null }] : []),
    },
  }));
}

/** Preserves provider-issued identities; never authorizes a debugger operation. */
export function readingProviderAnnotations(reading: import("./observations").ObservationReading): ProviderAnnotation[] {
  return reading.provider_annotations ?? reading.annotations.map(annotation => {
    const location = reading.locations.find(item => item.id === annotation.location) ?? null;
    return { evidence: annotation.evidence, provider: annotation.provider, provider_version: "unknown",
      payload_schema: annotation.payload_schema,
      origin: { kind: "runtime", session: annotation.scope.session ?? null, run: annotation.scope.run,
        stop: annotation.scope.stop === "runtime" ? null : annotation.scope.stop,
        frame: annotation.scope.frame, invocation: annotation.scope.invocation },
      location, source_precision: location?.location.precision ?? "unknown",
      measurement_semantics: null, uncertainty: null, payload: annotation.payload };
  });
}
