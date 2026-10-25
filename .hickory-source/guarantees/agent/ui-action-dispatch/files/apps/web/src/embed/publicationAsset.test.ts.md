<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/embed/publicationAsset.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `e7c362872c37519201106d628d0a783ce961e8c72ba637b2305bb51c697e2878`.

<book:entry path="apps/web/src/embed/publicationAsset.test.ts">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { afterEach, expect, it, vi } from "vitest";
// @ts-expect-error Node test runtime only; production uses browser Web Crypto.
import { webcrypto } from "node:crypto";
import { verifiedPublicationAsset, type ViewerSnapshot } from "./PublishedSnapshot";
afterEach(() => { vi.unstubAllGlobals(); vi.restoreAllMocks(); });
it("verifies selected media bytes and rejects tampering and encoded escapes", async () => {
  vi.stubGlobal("crypto", webcrypto);
  const bytes = new TextEncoder().encode("reviewed test media bytes");
  const hash = [...new Uint8Array(await webcrypto.subtle.digest("SHA-256", bytes))].map(b => b.toString(16).padStart(2, "0")).join("");
  const fetcher = vi.fn(async () => new Response(bytes, { headers: { "Content-Type": "video/webm" } }));
  vi.stubGlobal("fetch", fetcher);
  const create = vi.fn(() => "blob:verified-test-bytes");
  vi.stubGlobal("URL", class extends URL { static createObjectURL = create; });
  const assets = Object.fromEntries(["footage/run.webm", "%2e%2e/escape.webm", "https://example.test/media.webm", "video.webm?query", "..%2fescape.webm"].map(key => [key, hash]));
  const snapshot = { verification: { assets } } as unknown as ViewerSnapshot;
  const base = new URL("/publications/selected/", location.href).href;
  expect(await verifiedPublicationAsset(snapshot, base, "footage/run.webm")).toBe("blob:verified-test-bytes");
  expect(fetcher.mock.calls).toHaveLength(1); expect(create).toHaveBeenCalledOnce();
  for (const key of Object.keys(assets).slice(1)) await expect(verifiedPublicationAsset(snapshot, base, key)).rejects.toThrow("not selected");
  expect(fetcher).toHaveBeenCalledOnce();
  fetcher.mockImplementation(async () => new Response("changed bytes"));
  await expect(verifiedPublicationAsset(snapshot, base, "footage/run.webm")).rejects.toThrow("hash mismatch");
  expect(create).toHaveBeenCalledOnce();
});
</book:entry>

</book:doc>
