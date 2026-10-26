<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/publicationAsset.test.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f80f5c932bce"></a>
## `apps/web/src/embed/publicationAsset.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-f80f5c932bce-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-f80f5c932bce-expression_statement">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { afterEach, expect, it, vi } from "vitest";
// @ts-expect-error Node test runtime only; production uses browser Web Crypto.
import { webcrypto } from "node:crypto";
import { verifiedPublicationAsset, type ViewerSnapshot } from "./PublishedSnapshot";
afterEach(() => { vi.unstubAllGlobals(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-f80f5c932bce-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-f80f5c932bce-expression_statement-2">
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
</storycode:copy>

### Assemble `apps/web/src/embed/publicationAsset.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/publicationAsset.test.ts"><storycode:paste select="#f-f80f5c932bce-expression_statement"/><storycode:paste select="#f-f80f5c932bce-expression_statement-2"/></storycode:file>

</storycode:doc>
