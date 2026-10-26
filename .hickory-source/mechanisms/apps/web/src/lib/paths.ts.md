<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/paths.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6cce9cb2728c"></a>
## `apps/web/src/lib/paths.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-6cce9cb2728c-samePath"></a>
### `samePath`

Whether two paths name the same document.
The engine answers with the path it opened — absolute, because that is what
it has — and the app knows the document by the path it was listed under,
which is relative to the project. Comparing those with `===` says "no"
forever, and the symptom is silent: provenance stops matching, so ribbons
simply do not draw and nothing anywhere says why.
True when `candidate` and `known` name the same file, allowing one of them
to be absolute.
Suffix matching on whole segments, never on characters: `stats.md` must
not match `my-stats.md`, and `a/b.md` must not match `za/b.md`.

<storycode:copy id="f-6cce9cb2728c-samePath">
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
  if (candidate.startsWith("hickory-buffer://") || known.startsWith("hickory-buffer://")) return false;
  const a = normalise(candidate);
  const b = normalise(known);
  if (a === b) return true;
  return a.endsWith(`/${b}`) || b.endsWith(`/${a}`);
}

</storycode:copy>
<a id="f-6cce9cb2728c-normalise"></a>
### `normalise`

<storycode:copy id="f-6cce9cb2728c-normalise">
function normalise(path: string): string {
  return path.replace(/\\/g, "/").replace(/^\.\//, "");
}
</storycode:copy>

### Assemble `apps/web/src/lib/paths.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/paths.ts"><storycode:paste select="#f-6cce9cb2728c-samePath"/><storycode:paste select="#f-6cce9cb2728c-normalise"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-16b294833cd3"></a>
## `apps/web/src/lib/paths.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-16b294833cd3-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-16b294833cd3-expression_statement">
import { describe, expect, it } from "vitest";
import { samePath } from "./paths";

describe("is this provenance about this document", () => {
  it("matches an absolute engine path against a relative app path", () => {
    // The bug this exists for: the engine answers with the path it opened,
    // the app knows the path it listed, and `===` says no forever — silently,
    // because the only symptom is ribbons that never draw.
    expect(samePath("/home/me/project/stats.md", "stats.md")).toBe(true);
    expect(samePath("/home/me/project/docs/stats.md", "docs/stats.md")).toBe(true);
    expect(samePath("stats.md", "stats.md")).toBe(true);
  });

  it("matches on whole segments, never on characters", () => {
    expect(samePath("/home/me/my-stats.md", "stats.md")).toBe(false);
    expect(samePath("/home/me/za/b.md", "a/b.md")).toBe(false);
  });

  it("does not match a different document", () => {
    expect(samePath("/home/me/project/other.md", "stats.md")).toBe(false);
  });

  it("tolerates windows separators and a leading ./", () => {
    expect(samePath("C:\\projects\\demo\\stats.md", "stats.md")).toBe(true);
    expect(samePath("./stats.md", "stats.md")).toBe(true);
  });
  it("matches editor buffer identities exactly, without filesystem suffix matching", () => {
    expect(samePath("hickory-buffer://u1", "hickory-buffer://u1")).toBe(true);
    expect(samePath("hickory-buffer://u1", "u1")).toBe(false);
    expect(samePath("hickory-buffer://u1", "hickory-buffer://u2")).toBe(false);
  });

});
</storycode:copy>

### Assemble `apps/web/src/lib/paths.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/paths.test.ts"><storycode:paste select="#f-16b294833cd3-expression_statement"/></storycode:file>


</storycode:doc>
