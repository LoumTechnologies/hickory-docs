<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lsp/documentUri.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-81eed3045737"></a>
## `apps/web/src/lsp/documentUri.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-81eed3045737-documentUri"></a>
### `documentUri`

A filesystem path is not already a URI: encode literal %, #, ? and spaces.

<storycode:copy id="f-81eed3045737-documentUri">
/** A filesystem path is not already a URI: encode literal %, #, ? and spaces. */
export function documentUri(path: string, projectContext = "new"): string {
  if (path.startsWith("hick-project:///")) return path;
  const encoded = path.replace(/^\/+/, "").split("/").map(encodeURIComponent).join("/");
  return `hick:///${encoded}${projectContext === "new" ? "" : `?hick-project=${encodeURIComponent(projectContext)}`}`;
}

</storycode:copy>
<a id="f-81eed3045737-sameDocumentUri"></a>
### `sameDocumentUri`

Compare serialized URI identities without decoding path separators or
conflating a filename containing literal '%20' with one containing a space.

<storycode:copy id="f-81eed3045737-sameDocumentUri">
/** Compare serialized URI identities without decoding path separators or
 * conflating a filename containing literal '%20' with one containing a space. */
export function sameDocumentUri(left: string, right: string): boolean {
  if (left === right) return true;
  try { return new URL(left).href === new URL(right).href; }
  catch { return false; }
}
</storycode:copy>

### Assemble `apps/web/src/lsp/documentUri.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/documentUri.ts"><storycode:paste select="#f-81eed3045737-documentUri"/><storycode:paste select="#f-81eed3045737-sameDocumentUri"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-0653978670b7"></a>
## `apps/web/src/lsp/documentUri.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-0653978670b7-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-0653978670b7-expression_statement">
import { expect, it } from "vitest";
import { documentUri, sameDocumentUri } from "./documentUri";
it("encodes filesystem paths once, retaining literal percent, hash and question mark names", () => {
  expect(documentUri("/Application Support/main.cpp")).toBe("hick:///Application%20Support/main.cpp");
  expect(documentUri("notes/a%20b#c?.cpp", "old")).toBe("hick:///notes/a%2520b%23c%3F.cpp?hick-project=old");
  expect(documentUri("hick-project:///capture/library.cpp")).toBe("hick-project:///capture/library.cpp");
});
</storycode:copy>
<a id="f-0653978670b7-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-0653978670b7-expression_statement-2">
it("matches serialized space spellings without collapsing distinct literal filenames or revisions", () => {
  expect(sameDocumentUri("hick:///Application Support/main.cpp", "hick:///Application%20Support/main.cpp")).toBe(true);
  expect(sameDocumentUri(documentUri("a%20b.cpp"), documentUri("a b.cpp"))).toBe(false);
  expect(sameDocumentUri(documentUri("a.cpp", "old"), documentUri("a.cpp"))).toBe(false);
});
</storycode:copy>

### Assemble `apps/web/src/lsp/documentUri.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/documentUri.test.ts"><storycode:paste select="#f-0653978670b7-expression_statement"/><storycode:paste select="#f-0653978670b7-expression_statement-2"/></storycode:file>


</storycode:doc>
