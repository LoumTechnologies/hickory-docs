<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Compare provider evidence with available source text

For maintainers changing workspace services independently of the editor renderer.

A child version belongs to the bridge, while each window owns a different unsaved buffer. The bridge therefore wraps original provider data separately from an engine-owned source digest. Compare that digest with current bytes, never with coincidentally equal counters. Each diagnostic owns its own evidence: an unknown report cannot disable an unrelated current report. A weak association records verified source by diagnostic object identity; serialized provider fields cannot fabricate that client knowledge. Unknown reports stay readable without enabling positions. Hashing is asynchronous and its caller rejects obsolete completions; opening text does not wait for it. The same SHA-256 primitive also serves existing source review targets.

<ui:copy id="diagnostic-revision-browser-proof">
const diagnosticSources = new WeakMap<object, string>();
export function bindDiagnosticSource(diagnostic: object, source?: string): void {
  if (source === undefined) diagnosticSources.delete(diagnostic); else diagnosticSources.set(diagnostic, source);
}
export function diagnosticBoundSource(diagnostic: object): string | undefined { return diagnosticSources.get(diagnostic); }
export async function sourceDigest(source: string): Promise<string> {
  const bytes = new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(source)));
  return `sha256:${Array.from(bytes, byte => byte.toString(16).padStart(2, "0")).join("")}`;
}
export function hasDiagnosticRevision(diagnostic: { data?: unknown }): boolean {
  return !!diagnostic.data && typeof diagnostic.data === "object" && Object.hasOwn(diagnostic.data, "hickory");
}
export function diagnosticSourceDigest(diagnostic: { data?: unknown }): string | null {
  const data = diagnostic.data as { hickory?: { source_hash?: unknown } } | null | undefined;
  const value = data?.hickory?.source_hash;
  return typeof value === "string" && /^sha256:[a-f0-9]{64}$/i.test(value) ? value.toLowerCase() : null;
}
</ui:copy>

<ui:file path="apps/web/src/lsp/diagnosticRevision.ts"><ui:paste select="#diagnostic-revision-browser-proof"/></ui:file>
</ui:doc>
