<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/e2e/uiPicture.spec.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-bc3d0c24b142"></a>
## `apps/web/e2e/uiPicture.spec.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../contract.md).

<a id="f-bc3d0c24b142-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-bc3d0c24b142-expression_statement">
import { test, expect } from "@playwright/test";
import { createHash } from "node:crypto";
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
// Real browser raster/dispatch, no real model/provider or native release claim.
test("registered image reading captures actual displayed PNG pixels and digest", async ({ page }) => {
  const unexpected: string[] = [];
  await page.route(url => url.pathname.startsWith("/api/"), route => { unexpected.push(route.request().url()); return route.abort(); });
  const pageFailure = new Promise<never>((_resolve, reject) => page.on("pageerror", reject));
  await Promise.race([(async () => { await page.goto("/embed.html?picture"); await expect(page.getByAltText("red-pixel.png")).toBeVisible(); })(), pageFailure]);
  await page.getByRole("button", { name: "Read fixture image as agent" }).click();
  await expect(page.getByLabel("Image capture receipt")).toContainText('"status": "completed"');
  const result = JSON.parse(await page.getByLabel("Image capture receipt").innerText());
  const value = result.outcome.value;
  expect(value.resource.kind).toBe("inline_image"); expect(value.src).toBeUndefined(); expect(value.resource.location).toBeNull();
  expect(value.width).toBe(1); expect(value.height).toBe(1);
  const bytes = Buffer.from(value.ui_image.data, "base64");
  expect(value.ui_image.digest).toBe(createHash("sha256").update(bytes).digest("hex"));
  expect(value.ui_image.byte_length).toBe(bytes.length);
  const pixel = await page.evaluate(async (data: string) => {
    const image = new Image(); image.src = `data:image/png;base64,${data}`; await image.decode();
    const canvas = document.createElement("canvas"); canvas.width = 1; canvas.height = 1;
    const context = canvas.getContext("2d")!; context.drawImage(image, 0, 0);
    return Array.from(context.getImageData(0, 0, 1, 1).data);
  }, value.ui_image.data);
  expect(pixel).toEqual([255, 0, 0, 255]);
  expect(unexpected).toEqual([]);
});
</storycode:copy>

### Assemble `apps/web/e2e/uiPicture.spec.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/e2e/uiPicture.spec.ts"><storycode:paste select="#f-bc3d0c24b142-expression_statement"/></storycode:file>

</storycode:doc>
