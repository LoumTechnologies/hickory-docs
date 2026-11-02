<static:doc xmlns:static="http://www.hickorydocs.com/1.0">
# Read the same artifact from plain and content-decoding hosts

Test transport representations independently of source identity checks: gzip bytes, already-decoded responses, malformed streams and invalid UTF-8. A plain static server need not configure Content-Encoding for the exported files to work.

<static:copy id="static-apps-web-src-embed-staticArtifacts-test-ts-0">
import { expect, it } from "vitest";
import { artifactText } from "./staticArtifacts";

it("reads plain, gzip and already-decoded UTF-8 artifacts without changing bytes", async () => {
  const source = '{"source":"λ 😀"}\n';
  expect(await artifactText(new Response(source))).toBe(source);
  expect(await artifactText(new Response(Uint8Array.from(atob("H4sIAAAAAAAC/6tWKs4vLUpOVbJSOrdb4cP8GQ1KtVwA5w/L3RUAAAA="), char => char.charCodeAt(0))))).toBe(source);
  expect(await artifactText(new Response(source, { headers: { "Content-Encoding": "gzip" } }))).toBe(source);
});
it("refuses truncated gzip and invalid UTF-8", async () => {
  await expect(artifactText(new Response(new Uint8Array([0x1f, 0x8b, 0])))).rejects.toThrow();
  await expect(artifactText(new Response(new Uint8Array([0xff])))).rejects.toThrow();
});
</static:copy>

<static:file path="apps/web/src/embed/staticArtifacts.test.ts"><static:paste select="#static-apps-web-src-embed-staticArtifacts-test-ts-0"/></static:file>
</static:doc>
