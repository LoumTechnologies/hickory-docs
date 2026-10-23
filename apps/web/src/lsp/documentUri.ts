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
