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
