/** A filesystem path is not already a URI: encode literal %, #, ? and spaces. */
/** Construct a captured URI from a serialized capture root and a raw authored path.
 * Existing captured document URIs are already encoded and must remain verbatim. */
export function capturedDocumentUri(root:string,path:string):string {
  if(!/^hick-project:\/\/\/[^/?#]+\/$/.test(root))throw Error("Invalid captured source root");
  if(path.startsWith("hick-project:///"))return path;
  return root+path.replace(/^\/+/,"").split("/").map(encodeURIComponent).join("/");
}
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
