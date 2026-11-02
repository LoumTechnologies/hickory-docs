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
