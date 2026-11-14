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
