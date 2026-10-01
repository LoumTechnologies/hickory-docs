// Whether two paths name the same document.
//
// The engine answers with the path it opened — absolute, because that is what
// it has — and the app knows the document by the path it was listed under,
// which is relative to the project. Comparing those with `===` says "no"
// forever, and the symptom is silent: provenance stops matching, so ribbons
// simply do not draw and nothing anywhere says why.

/**
 * True when `candidate` and `known` name the same file, allowing one of them
 * to be absolute.
 *
 * Suffix matching on whole segments, never on characters: `stats.md` must
 * not match `my-stats.md`, and `a/b.md` must not match `za/b.md`.
 */
export function samePath(candidate: string, known: string): boolean {
  if (candidate === known) return true;
  const a = normalise(candidate);
  const b = normalise(known);
  if (a === b) return true;
  return a.endsWith(`/${b}`) || b.endsWith(`/${a}`);
}

function normalise(path: string): string {
  return path.replace(/\\/g, "/").replace(/^\.\//, "");
}
