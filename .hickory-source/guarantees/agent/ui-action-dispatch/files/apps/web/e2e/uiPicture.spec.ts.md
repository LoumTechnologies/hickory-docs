<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/e2e/uiPicture.spec.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../contract.md)

Captured SHA-256: `6feb403048eac1960652a540e662b810e5a7e62777ccc1f7495fd05485027a14`.

<book:entry path="apps/web/e2e/uiPicture.spec.ts">
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
</book:entry>

</book:doc>
