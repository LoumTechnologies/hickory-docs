<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/providerObservations.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d4c4b5c7739b"></a>
## `apps/web/src/debug/providerObservations.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-d4c4b5c7739b-EvidenceOrigin"></a>
### `EvidenceOrigin`

Matches hick_observe::reading::ProviderAnnotation, independently of debugger scope.

<storycode:copy id="f-d4c4b5c7739b-EvidenceOrigin">
/** Matches hick_observe::reading::ProviderAnnotation, independently of debugger scope. */
export type EvidenceOrigin =
  | { kind: "runtime"; session: string | null; run: string; stop: string | null; frame: number | null; invocation: string | null }
  | { kind: "review"; repository: string; pull_request: number; thread: string; base_revision: string;
      head_revision: string; side: string; author: string; resolved: boolean; outdated: boolean };
</storycode:copy>
<a id="f-d4c4b5c7739b-ProviderAnnotation"></a>
### `ProviderAnnotation`

<storycode:copy id="f-d4c4b5c7739b-ProviderAnnotation">
export interface ProviderAnnotation {
  evidence: string; provider: string; provider_version: string; payload_schema: string;
  origin: EvidenceOrigin;
  location: { id: string; revision: string; line: number; location: { source: string; block?: string | null;
    bytes: { start: number; end: number } | null; precision: string } } | null;
  source_precision: string; measurement_semantics: string | null; uncertainty: string | null; payload: unknown;
}
</storycode:copy>
<a id="f-d4c4b5c7739b-ProviderReading"></a>
### `ProviderReading`

<storycode:copy id="f-d4c4b5c7739b-ProviderReading">
export interface ProviderReading {
  schema_version: 1;
  source: { id: string; uri: string; revision: string | null; bytes: number[] | null };
  annotations: ProviderAnnotation[]; unsupported: string[];
}
</storycode:copy>
<a id="f-d4c4b5c7739b-selected"></a>
### `selected`

<storycode:copy id="f-d4c4b5c7739b-selected">
const selected = new Map<symbol, { reading: ProviderReading; historical: boolean }>();
</storycode:copy>
<a id="f-d4c4b5c7739b-selectProviderReading"></a>
### `selectProviderReading`

<storycode:copy id="f-d4c4b5c7739b-selectProviderReading">
export function selectProviderReading(owner: symbol, reading: ProviderReading, include: boolean, historical: boolean) {
  if (include) selected.set(owner, { reading, historical }); else selected.delete(owner);
}
</storycode:copy>
<a id="f-d4c4b5c7739b-providerObservationContext"></a>
### `providerObservationContext`

<storycode:copy id="f-d4c4b5c7739b-providerObservationContext">
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

</storycode:copy>
<a id="f-d4c4b5c7739b-readingProviderAnnotations"></a>
### `readingProviderAnnotations`

Preserves provider-issued identities; never authorizes a debugger operation.

<storycode:copy id="f-d4c4b5c7739b-readingProviderAnnotations">
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
</storycode:copy>

### Assemble `apps/web/src/debug/providerObservations.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/providerObservations.ts"><storycode:paste select="#f-d4c4b5c7739b-EvidenceOrigin"/><storycode:paste select="#f-d4c4b5c7739b-ProviderAnnotation"/><storycode:paste select="#f-d4c4b5c7739b-ProviderReading"/><storycode:paste select="#f-d4c4b5c7739b-selected"/><storycode:paste select="#f-d4c4b5c7739b-selectProviderReading"/><storycode:paste select="#f-d4c4b5c7739b-providerObservationContext"/><storycode:paste select="#f-d4c4b5c7739b-readingProviderAnnotations"/></storycode:file>

</storycode:doc>
