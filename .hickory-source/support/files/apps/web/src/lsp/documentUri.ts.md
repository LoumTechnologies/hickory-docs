<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lsp/documentUri.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `110163d07c9a9cdc808475b09f597ee2516144b14461e67d5f26a2910772d803`.

<book:entry path="apps/web/src/lsp/documentUri.ts">
/** A filesystem path is not already a URI: encode literal %, #, ? and spaces. */
export function documentUri(path: string, projectContext = "new"): string {
  if (path.startsWith("hick-project:///")) return path;
  const encoded = path.replace(/^\/+/, "").split("/").map(encodeURIComponent).join("/");
  return `hick:///${encoded}${projectContext === "new" ? "" : `?hick-project=${encodeURIComponent(projectContext)}`}`;
}

/** Compare serialized URI identities without decoding path separators or
 * conflating a filename containing literal '%20' with one containing a space. */
export function sameDocumentUri(left: string, right: string): boolean {
  if (left === right) return true;
  try { return new URL(left).href === new URL(right).href; }
  catch { return false; }
}
</book:entry>

</book:doc>
