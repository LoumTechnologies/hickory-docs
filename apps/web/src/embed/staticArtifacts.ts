export async function artifactText(response: Response): Promise<string> {
  let bytes = await response.arrayBuffer();
  const signature = new Uint8Array(bytes, 0, Math.min(2, bytes.byteLength));
  if (signature[0] === 0x1f && signature[1] === 0x8b) {
    const stream = new Response(bytes).body!.pipeThrough(new DecompressionStream("gzip"));
    bytes = await new Response(stream).arrayBuffer();
  }
  return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
}
