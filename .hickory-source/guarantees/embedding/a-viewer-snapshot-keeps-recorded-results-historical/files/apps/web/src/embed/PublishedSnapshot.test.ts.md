<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/embed/PublishedSnapshot.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A viewer snapshot keeps recorded results historical](../../../../../contract.md)

Captured SHA-256: `6f3dbe72cbbce0f370d67b942af3270ca2aa8dc3c6e8a24bf5a0d0ada62729b0`.

<book:entry path="apps/web/src/embed/PublishedSnapshot.test.ts">
import { afterEach, expect, it, vi } from "vitest";
import { decodeSnapshot } from "./PublishedSnapshot";
const digest = vi.fn(async (_algorithm: string, bytes: Uint8Array) => {
  const out = new Uint8Array(32); bytes.forEach((byte, i) => { out[i % 32] ^= byte; }); return out.buffer;
});
afterEach(() => vi.unstubAllGlobals());
// Guarantee: docs/guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical.md
it("rejects changed source and unsupported transcript events", async () => {
  vi.stubGlobal("crypto", { subtle: { digest } });
  const source = "original";
  const hash = Array.from(new Uint8Array(await digest("SHA-256", new TextEncoder().encode(source))), b => b.toString(16).padStart(2,"0")).join("");
  const snapshot = {format:"hickory-viewer-snapshot",version:1,path:"spec.md",source,source_hash:hash,captured_at_ms:0,execution:"recorded",cells:[]};
  expect((await decodeSnapshot(snapshot)).source).toBe(source);
  await expect(decodeSnapshot({...snapshot,source:"changed"})).rejects.toThrow("hash");
  await expect(decodeSnapshot({...snapshot,cells:[{kind:"exec",id:"a",container:"shell",command:"echo",span:[0,1],saved_output:{html:"untrusted"}}]})).rejects.toThrow("saved output");
  await expect(decodeSnapshot({...snapshot,cells:[{kind:"exec",id:"a",container:"shell",command:"echo",span:[0,1],transcript:[{kind:"injected",data:"x",t:0}]}]})).rejects.toThrow("transcript");
});
</book:entry>

</book:doc>
