<static:doc xmlns:static="http://www.hickorydocs.com/1.0">
# Read compressed static artifacts before checking their identities

Bulk semantic data repeats names, paths and coordinates. Store document and model payloads as deterministic gzip files while keeping the index readable JSON. Inspect the response bytes so both a plain static server and a host that already decoded Content-Encoding work; hashes continue to identify the original uncompressed text.

<static:copy id="static-apps-web-src-embed-staticArtifacts-ts-0">
export async function artifactText(response: Response): Promise<string> {
  let bytes = await response.arrayBuffer();
  const signature = new Uint8Array(bytes, 0, Math.min(2, bytes.byteLength));
  if (signature[0] === 0x1f && signature[1] === 0x8b) {
    const stream = new Response(bytes).body!.pipeThrough(new DecompressionStream("gzip"));
    bytes = await new Response(stream).arrayBuffer();
  }
  return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
}
</static:copy>

<static:file path="apps/web/src/embed/staticArtifacts.ts"><static:paste select="#static-apps-web-src-embed-staticArtifacts-ts-0"/></static:file>
</static:doc>
